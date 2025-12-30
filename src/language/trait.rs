use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::AocError;

pub trait Ext {
    fn extension(&self) -> &str;
}

pub trait Runner: Ext {
    fn execute(&self, args: RunningArgs) -> Result<duct::Expression, AocError>;
}

pub trait Compile: Ext {
    fn compile(&self, args: RunningArgs) -> Result<duct::Expression, AocError>;
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Common {
    pub files: HashMap<String, PathBuf>,
    pub day_folder: PathBuf,
    pub day: i32,
    pub root_folder: PathBuf,
    pub input_file: PathBuf,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct RunningArgs {
    pub arguments: Vec<String>,
    pub release: bool,
    pub common: Common,
    pub runner: Option<String>,
}
