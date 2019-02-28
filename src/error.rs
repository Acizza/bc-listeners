use failure;

macro_rules! impl_error_conversion {
    ($err_name:ident, $($from_ty:ty => $to_ty:ident,)+) => {
        $(
        impl From<$from_ty> for $err_name {
            fn from(f: $from_ty) -> $err_name {
                $err_name::$to_ty(f)
            }
        }
        )+
    };
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "config error")]
    Config(#[cause] ConfigError),

    #[fail(display = "feed error")]
    Feed(#[cause] FeedError),

    #[fail(display = "statistics error")]
    Statistics(#[cause] StatisticsError),
}

impl_error_conversion!(Error,
    ConfigError => Config,
    FeedError => Feed,
    StatisticsError => Statistics,
);

#[derive(Fail, Debug)]
pub enum FeedError {
    #[fail(display = "HTTP error")]
    Reqwest(#[cause] ::reqwest::Error),

    #[fail(display = "failed to parse top feeds")]
    ParseTopFeeds(#[cause] ScrapeError),

    #[fail(display = "failed to parse state ({}) feeds", _1)]
    ParseStateFeeds(#[cause] ScrapeError, String),

    #[fail(display = "failed to create notification")]
    FailedToCreateNotification,
}

impl_error_conversion!(FeedError,
    ::reqwest::Error => Reqwest,
);

type ElementName = &'static str;

#[derive(Fail, Debug)]
pub enum ScrapeError {
    #[fail(display = "unable to find element that contains {} information", _0)]
    NoElement(ElementName),

    #[fail(display = "unable to parse {} information", _1)]
    FailedIntParse(#[cause] ::std::num::ParseIntError, ElementName),

    #[fail(display = "no feeds found")]
    NoneFound,
}

#[derive(Fail, Debug)]
pub enum StatisticsError {
    #[fail(display = "CSV error")]
    CSV(#[cause] ::csv::Error),

    #[fail(display = "io error")]
    Io(#[cause] ::std::io::Error),

    #[fail(display = "failed to parse integer")]
    ParseIntError(#[cause] ::std::num::ParseIntError),

    #[fail(display = "failed to parse float")]
    ParseFloatError(#[cause] ::std::num::ParseFloatError),

    #[fail(display = "CSV file contains record with too few rows")]
    TooFewRows,
}

impl_error_conversion!(StatisticsError,
    ::csv::Error => CSV,
    ::std::io::Error => Io,
);

#[derive(Fail, Debug)]
pub enum ConfigError {
    #[fail(display = "io error")]
    Io(#[cause] ::std::io::Error),

    #[fail(display = "error parsing YAML")]
    YAMLScan(#[cause] ::yaml_rust::ScanError),
}

impl_error_conversion!(ConfigError,
    ::std::io::Error => Io,
    ::yaml_rust::ScanError => YAMLScan,
);

fn build_err_msg(err: &failure::Error) -> String {
    let mut msg = format!("error: {}\n", err);

    for cause in err.iter_chain().skip(1) {
        msg.push_str(&format!("caused by: {}\n", cause));
    }

    msg
}

fn print_with_backtrace(msg: &str, err: &failure::Error) {
    eprintln!("{}", msg);
    eprintln!("{}", err.backtrace());
}

/// Displays the provided error with a notification and by writing it to the terminal
pub fn display(err: &failure::Error) {
    use notify_rust::Notification;

    let msg = build_err_msg(err);
    print_with_backtrace(&msg, err);

    let notif = Notification::new()
        .summary("Broadcastify Update Error")
        .body(&msg)
        .show();

    match notif {
        Ok(_) => (),
        Err(_) => {
            eprintln!("failed to create error notification");
        }
    }
}
