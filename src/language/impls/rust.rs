use crate::language::{Language, RunningArgs};
use duct::cmd;
pub struct Rust;

impl Language for Rust {
    fn extension(&self) -> &'static str {
        "rs"
    }
    fn execute(&self, args: RunningArgs) -> duct::Expression {
        let input = args.common.input_file.display().to_string();

        let args = args.arguments.iter().map(|s| s.as_str());
        let args = std::iter::once("run")
            .chain(args)
            .chain(["--color", "always"])
            .chain(std::iter::once(input.as_str()));

        cmd("cargo", args)
    }
}
