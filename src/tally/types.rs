use std::path::PathBuf;

use duct::Expression;

use crate::util::AocInfo;

#[derive(Debug)]
pub struct DiscoveredDay {
    pub day: usize,
    pub folder: PathBuf,
}

#[derive(Debug)]
pub struct CompiledDay {
    pub day: usize,
    pub expr: Expression,
}

#[derive(Debug)]
pub struct VerifiedDay {
    pub day: usize,
    pub expr: Expression,
    pub info: AocInfo,
}

#[derive(Debug)]
pub struct RunDayResult {
    pub day: usize,
    pub info: AocInfo,
    pub run: RunRes,
}

#[derive(Debug)]
pub struct DayError {
    pub day: usize,
    pub info: AocInfo,
    pub error: ErrorTypes,
}

#[derive(Debug)]
pub enum ErrorTypes {
    MissingDay,
    InputDownload,
    Compiler(String),
    Runtime(String),
    MissingImplementation,
    MissingExtension,
    GetAnswers,
    Unsupported(String),
}

impl std::fmt::Display for ErrorTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDay => write!(f, "Missing day"),
            Self::InputDownload => write!(f, "Could not download input"),
            Self::Compiler(s) => write!(f, "Compiler error: {}", s),
            Self::Runtime(s) => write!(f, "Runtime error: {}", s),
            Self::MissingExtension => write!(f, "Missing extension"),
            Self::MissingImplementation => write!(f, "Missing implementation"),
            Self::GetAnswers => write!(f, "Error getting answers"),
            Self::Unsupported(ext) => write!(f, "Unsuppored lang: {}", ext),
        }
    }
}

#[derive(Debug)]
pub struct Answer {
    pub value: Option<String>,
    pub time: Option<usize>,
}
#[derive(Debug)]
pub struct RunRes {
    pub p1: Answer,
    pub p2: Answer,
}
