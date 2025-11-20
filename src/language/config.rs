use std::{collections::HashMap, marker::PhantomData, path::Path};

use duct::{Expression, cmd};
use regex::{Captures, Regex};
use serde::Deserialize;

use crate::{
    error::AocError,
    language::{Runner, RunningArgs, r#trait::Ext},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub toolchain: HashMap<String, Toolchain<Raw>>,
}

impl Config {
    pub fn from_file(p: &Path) -> Result<Self, AocError> {
        let s = std::fs::read_to_string(p)?;
        let res = toml::from_str(&s)?;

        Ok(res)
    }
    pub fn runners(&self) -> Vec<impl Runner + 'static> {
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

impl Runner for Toolchain<RunState> {
    fn execute(&self, args: super::RunningArgs) -> Result<duct::Expression, AocError> {
        run_command(&self.run, self, &args, true)
    }
}

fn transform_command(s: &str) -> (String, Vec<String>) {
    if cfg!(windows) {
        ("cmd".to_owned(), vec!["/c".to_string(), s.to_owned()])
    } else {
        ("sh".to_owned(), vec!["-c".to_string(), s.to_owned()])
    }
}

fn run_command<T>(
    command: &str,
    t: &Toolchain<T>,
    args: &RunningArgs,
    include_input: bool,
) -> Result<Expression, AocError> {
    let input = args.common.input_file.display().to_string();

    let run = expand_templates(command, args)?;

    let (program, mut vec) = transform_command(&run);
    if include_input {
        vec.push(input);
    }

    let mut cmd = cmd(program, vec);
    if let Some(dir) = &t.dir {
        let dir = expand_templates(dir, args)?;
        cmd = cmd.dir(dir);
    }

    Ok(cmd)
}

impl super::r#trait::Compile for Toolchain<CompileState> {
    fn compile(&self, args: super::RunningArgs) -> Result<duct::Expression, AocError> {
        let compile = self.compile.as_ref().unwrap();
        if let Some(build) = &compile.build {
            let expr = run_command(build, self, &args, false)?;
            let out = expr.stderr_to_stdout().stdout_capture().unchecked().run()?;
            if !out.status.success() {
                let err = std::str::from_utf8(&out.stdout).unwrap();
                let err_line = err.lines().find(|line| line.starts_with("error: "));
                let io = std::io::Error::other(err_line.unwrap_or(err));
                return Err(AocError::StdIoErr(io));
            }
        }

        run_command(&compile.execute, self, &args, true)
    }
}

fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}

pub fn expand_templates(input: &str, args: &RunningArgs) -> Result<String, AocError> {
    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let forwarded = args.arguments.join(" ");

    let r#fn = |caps: &regex::Captures| {
        let raw = &caps[1];

        let mut parts = raw.splitn(2, ':');
        let first = parts.next().unwrap();
        let second = parts.next();

        let (prefix, key) = match second {
            Some(key) => (first, key),
            None => ("", first),
        };

        let s = match key {
            "day" => &args.common.day_folder,
            "file" => &args.common.file,
            "args" => return Ok(forwarded.clone()),
            _ => return Err(AocError::TemplateError(format!("template: {}", key))),
        };

        match prefix {
            "" => Ok(abs(s)),
            "rel" => Ok(rel(s, args)),
            "name" => Ok(name(s)),
            _ => Err(AocError::TemplateError(format!("prefix: {}", prefix))),
        }
    };

    replace_all(&re, input, r#fn)
}

fn abs(p: &Path) -> String {
    p.display().to_string()
}
fn name(p: &Path) -> String {
    p.file_name().unwrap().to_str().unwrap().to_string()
}
fn rel(p: &Path, args: &RunningArgs) -> String {
    p.strip_prefix(&args.common.root_folder)
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
