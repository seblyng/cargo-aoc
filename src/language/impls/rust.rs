use crate::language::{Language, RunningArgs};
use duct::cmd;
pub struct Rust;

impl Language for Rust {
    fn extension(&self) -> &'static str {
        "rs"
    }
    fn execute(&self, args: RunningArgs) -> duct::Expression {
        let input = args.common.input_file.display().to_string();

        let forwarded = args.arguments.iter().map(|s| s.as_str());
        let command_args = std::iter::once("run")
            .chain(forwarded)
            .chain(["--color", "always"])
            .chain(std::iter::once("--release").filter(|_| args.release))
            .chain(std::iter::once(input.as_str()));

        cmd("cargo", command_args).dir(args.common.day_folder)
    }
}
