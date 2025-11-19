use thiserror::Error;

#[derive(Error, Debug)]
pub enum AocError {
    #[error("Could not find AOC_TOKEN to download input or submit")]
    TokenError(#[from] dotenv::Error),

    #[error("Issue with the token: {0}")]
    InvalidTokenError(String),

    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("download error: {0}")]
    DownloadError(String),

    #[error("error from adventofcode.com: {0}")]
    ApiError(String),

    #[cfg(feature = "submit")]
    #[error("Error on sanitizing answer")]
    SanitizeHtml,

    #[error("Error on getting answer from task")]
    ParseStdout,

    #[error("Day must be between 1 and 25")]
    InvalidRunDay,

    #[cfg(feature = "submit")]
    #[error("Can only submit task 1 or 2")]
    InvalidSubmitTask,

    #[error("Year must be between 2015 ..= current year")]
    InvalidYear,
    #[error("Its not yet december for this year's puzzles!")]
    InvalidMonth,

    #[error("Error parsing to number")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Error on getting argument")]
    ArgMatches,

    #[error("Error on parsing to utf-8")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("stdio error {0}")]
    StdIoErr(#[from] std::io::Error),

    #[cfg(feature = "bench")]
    #[error("argument error {0}")]
    ArgError(String),

    #[error("Setup for year already exists")]
    SetupExists,

    #[error("Unsupported language with extension {0}")]
    UnsupportedLanguage(String),

    #[error("Error when replacing template string")]
    TemplateError(String),
}
