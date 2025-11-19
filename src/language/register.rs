use std::sync::LazyLock;

use crate::language::Compile;

use super::Runner;

pub struct Register {
    langs: Vec<Box<dyn Runner + Sync + Send>>,
    compilers: Vec<Box<dyn Compile + Sync + Send>>,
}

impl Register {
    pub fn new() -> Self {
        Self {
            langs: Vec::new(),
            compilers: Vec::new(),
        }
    }
    pub fn register<L: Runner + Sync + Send + 'static>(&mut self, lang: L) {
        self.langs.push(Box::new(lang));
    }
    pub fn register_compiler<C: Compile + Sync + Send + 'static>(&mut self, c: C) {
        self.compilers.push(Box::new(c));
    }
    pub fn by_extension<'a>(&'a self, ext: &str) -> Option<&'a (dyn Runner + Sync + Send)> {
        self.langs
            .iter()
            .find(|c| c.extension() == ext)
            .map(|b| &**b)
    }
    pub fn compiler_by_extension<'a>(
        &'a self,
        ext: &str,
    ) -> Option<&'a (dyn Compile + Sync + Send)> {
        self.compilers
            .iter()
            .find(|c| c.extension() == ext)
            .map(|b| &**b)
    }
}

pub static REGISTER: LazyLock<Register> = LazyLock::new(|| {
    let mut r = Register::new();

    let langs = include_str!("../../languages.toml");
    let config: super::dynamic::Config = toml::from_str(langs).unwrap();

    for runner in config.runners() {
        r.register(runner);
    }

    for compiler in config.compilers() {
        r.register_compiler(compiler);
    }

    r
});
