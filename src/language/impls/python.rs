use crate::language::{Language, RunningArgs, r#trait::Compile};
use duct::cmd;
pub struct Python;

impl Language for Python {
    fn extension(&self) -> &'static str {
        "py"
    }
    fn execute(&self, args: RunningArgs) -> duct::Expression {
        let input = args.common.input_file.display().to_string();

        let forwarded = args.arguments.iter().map(|s| s.as_str());

        let command_args = std::iter::once("main.py")
            .chain(forwarded)
            .chain(std::iter::once(input.as_str()));

        cmd("python3", command_args).dir(args.common.day_folder)
    }
}

impl Compile for Python {
    fn compile(&self, args: RunningArgs) -> std::io::Result<duct::Expression> {
        Ok(self.execute(args))
    }
}
