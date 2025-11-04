use std::path::PathBuf;

pub trait Language {
    fn extension(&self) -> &'static str;
    fn execute(&self, args: RunningArgs) -> duct::Expression;
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Common {
    pub file: PathBuf,
    pub day_folder: PathBuf,
    pub root_folder: PathBuf,
    pub input_file: PathBuf,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct RunningArgs {
    pub arguments: Vec<String>,
    pub release: bool,
    pub common: Common,
}
