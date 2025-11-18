use std::{collections::HashMap, marker::PhantomData, path::Path};

use duct::{Expression, cmd};
use serde::Deserialize;

use crate::language::{Language, RunningArgs, r#trait::Ext};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub toolchain: HashMap<String, Toolchain<Raw>>,
}

impl Config {
    pub fn runners(&self) -> Vec<impl Language + 'static> {
        let mut vec = Vec::new();

        for value in self.toolchain.values() {
            vec.push(value.runner());
        }

        vec
    }
    pub fn compilers(&self) -> Vec<impl super::r#trait::Compile + 'static> {
        let mut vec = Vec::new();

        for value in self.toolchain.values() {
            // vec.push(value.compiler());
            if let Some(c) = value.compiler() {
                vec.push(c);
            }
        }

        vec
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Toolchain<State> {
    run: String,
    ext: String,
    dir: Option<String>,
    compile: Option<Compile>,
    #[serde(skip)]
    _phantom: PhantomData<State>,
}

impl Toolchain<Raw> {
    fn runner(&self) -> Toolchain<RunState> {
        Toolchain {
            run: self.run.clone(),
            ext: self.ext.clone(),
            dir: self.dir.clone(),
            compile: self.compile.clone(),
            _phantom: PhantomData,
        }
    }

    fn compiler(&self) -> Option<Toolchain<CompileState>> {
        self.compile.as_ref().map(|_| Toolchain {
            run: self.run.clone(),
            ext: self.ext.clone(),
            dir: self.dir.clone(),
            compile: self.compile.clone(),
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Compile {
    build: Option<String>,
    execute: String,
}

#[derive(Debug)]
pub struct Raw;
#[derive(Debug)]
struct RunState;
#[derive(Debug)]
struct CompileState;

/*
    Template options are:
    file => the main file (relative to the root)
    day => the (file)name of the day folder
    root => year folder
    args => forwarded arguments
*/

impl Ext for Toolchain<RunState> {
    fn extension(&self) -> &str {
        &self.ext
    }
}

impl Ext for Toolchain<CompileState> {
    fn extension(&self) -> &str {
        &self.ext
    }
}

impl Language for Toolchain<RunState> {
    fn execute(&self, args: super::RunningArgs) -> duct::Expression {
        let input = args.common.input_file.display().to_string();

        let forwarded = args.arguments.join(" ");

        let day = args
            .common
            .day_folder
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let file = args
            .common
            .file
            .strip_prefix(&args.common.root_folder)
            .unwrap();

        let file = file.display().to_string();

        let run = self
            .run
            .replace("{DAY}", &day)
            .replace("{day}", &day)
            .replace("{FILE}", &file)
            .replace("{file}", &file)
            .replace("{ARGS}", &forwarded)
            .replace("{args}", &forwarded)
            .replace("{root}", &args.common.root_folder.display().to_string())
            .replace("{root}", &args.common.root_folder.display().to_string());

        let (program, _args) = run.split_once(" ").unwrap();
        let mut _args = _args.split_whitespace().collect::<Vec<_>>();
        _args.push(&input);
        let mut cmd = cmd(program, _args);
        if let Some(dir) = &self.dir {
            let dir = dir.replace("{DAY}", &day).replace("{day}", &day);
            cmd = cmd.dir(dir);
        }

        cmd
    }
}

fn run_command<T>(
    command: &str,
    t: &Toolchain<T>,
    args: &RunningArgs,
    include_input: bool,
) -> Expression {
    let input = args.common.input_file.display().to_string();

    let forwarded = args.arguments.join(" ");

    let day = args
        .common
        .day_folder
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let file = args
        .common
        .file
        .strip_prefix(&args.common.root_folder)
        .unwrap();

    let file = file.display().to_string();

    let run = command
        .replace("{DAY}", &day)
        .replace("{day}", &day)
        .replace("{FILE}", &file)
        .replace("{file}", &file)
        .replace("{ARGS}", &forwarded)
        .replace("{args}", &forwarded)
        .replace("{root}", &args.common.root_folder.display().to_string())
        .replace("{root}", &args.common.root_folder.display().to_string());

    let (program, _args) = match run.split_once(" ") {
        Some((p, a)) => (p, a),
        _ => (run.as_str(), ""),
    };
    let mut _args = _args.split_whitespace().collect::<Vec<_>>();
    if include_input {
        _args.push(&input);
    }

    dbg!(&program, &_args);
    let mut cmd = cmd(program, _args);
    if let Some(dir) = &t.dir {
        let dir = dir.replace("{DAY}", &day).replace("{day}", &day);
        dbg!(&dir);
        cmd = cmd.dir(dir);
    }
    dbg!(&cmd);

    cmd
}

impl super::r#trait::Compile for Toolchain<CompileState> {
    fn compile(&self, args: super::RunningArgs) -> std::io::Result<duct::Expression> {
        let compile = self.compile.as_ref().unwrap();
        println!("???");
        if let Some(build) = &compile.build {
            let expr = run_command(&build, self, &args, false);
            let out = expr
                .stderr_to_stdout()
                .stdout_capture()
                .unchecked()
                .run()
                .unwrap();
            dbg!("outut = ", &out);
            if !out.status.success() {
                let err = std::str::from_utf8(&out.stdout).unwrap();
                let err_line = err.lines().find(|line| line.starts_with("error: "));
                return Err(std::io::Error::other(err_line.unwrap_or(err)));
            }
        }

        Ok(run_command(&compile.execute, self, &args, true))
    }
}

fn template_replce(s: &str, args: &RunningArgs) -> String {
    let rel = |p: &Path| {
        p.strip_prefix(&args.common.root_folder)
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    };

    let abs = |p: &Path| p.display().to_string();

    let c = &args.common;

    let mut ret = s.to_string();
    for (template, replace) in [
        ("day", abs(&c.day_folder)),
        ("rel:day", rel(&c.day_folder)),
        ("file", abs(&c.file)),
        ("rel:file", abs(&c.file)),
    ] {
        c
    }

    todo!()
}
