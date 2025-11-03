use std::sync::LazyLock;

use super::Compiler;

pub struct Register {
    compilers: Vec<Box<dyn Compiler + Sync + Send>>,
}

impl Register {
    pub fn new() -> Self {
        Self {
            compilers: Vec::new(),
        }
    }
    pub fn register<C: Compiler + Sync + Send + 'static>(&mut self, compiler: C) {
        self.compilers.push(Box::new(compiler));
    }
    pub fn by_extension<'a>(&'a self, ext: &str) -> Option<&'a (dyn Compiler + Sync + Send)> {
        self.compilers
            .iter()
            .find(|c| c.extension() == ext)
            .map(|b| &**b)
    }
}

pub static REGISTER: LazyLock<Register> = LazyLock::new(|| {
    let mut r = Register::new();
    // Add impls here
    r.register(super::impls::Rust);
    r
});
