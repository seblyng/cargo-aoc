use std::path::PathBuf;

pub trait Ext {
    fn extension(&self) -> &str;
}

pub trait Language: Ext {
    fn execute(&self, args: RunningArgs) -> duct::Expression;
}

pub trait Compile: Ext {
    fn compile(&self, args: RunningArgs) -> std::io::Result<duct::Expression>;
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Common {
    pub file: PathBuf,
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
}
