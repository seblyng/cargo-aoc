use crate::language::{
    Language, RunningArgs,
    r#trait::{Compile, Ext},
};
use duct::cmd;
pub struct Rust;

impl Ext for Rust {
    fn extension(&self) -> &'static str {
        "rs"
    }
}

impl Language for Rust {
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

impl Compile for Rust {
    fn compile(&self, args: RunningArgs) -> std::io::Result<duct::Expression> {
        let command_args =
            std::iter::once("build").chain(std::iter::once("--release").filter(|_| args.release));
        let out = cmd("cargo", command_args)
            .dir(args.common.day_folder)
            .stderr_to_stdout()
            .stdout_capture()
            .unchecked()
            .run()?;

        if !out.status.success() {
            let err = std::str::from_utf8(&out.stdout).unwrap();
            let err_line = err.lines().find(|line| line.starts_with("error: "));
            return Err(std::io::Error::other(err_line.unwrap_or(err)));
        }

        let mode = if args.release { "release" } else { "debug" };

        let bin = format!("day_{:02}", args.common.day);
        let target = args
            .common
            .root_folder
            .join(&bin)
            .join("target")
            .join(mode)
            .join(&bin);

        Ok(cmd!(target, args.common.input_file))
    }
}
