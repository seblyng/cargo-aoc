use std::sync::LazyLock;

use crate::language::Compile;

use super::Language;
use super::impls::*;

pub struct Register {
    langs: Vec<Box<dyn Language + Sync + Send>>,
    compilers: Vec<Box<dyn Compile + Sync + Send>>,
}

impl Register {
    pub fn new() -> Self {
        Self {
            langs: Vec::new(),
            compilers: Vec::new(),
        }
    }
    pub fn register<L: Language + Sync + Send + 'static>(&mut self, lang: L) {
        self.langs.push(Box::new(lang));
    }
    pub fn register_compiler<C: Compile + Sync + Send + 'static>(&mut self, c: C) {
        self.compilers.push(Box::new(c));
    }
    pub fn by_extension<'a>(&'a self, ext: &str) -> Option<&'a (dyn Language + Sync + Send)> {
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
    // Add impls here
    r.register(Rust);
    r.register(Python);

    r.register_compiler(Rust);
    r.register_compiler(Python);
    r
});
