use std::sync::LazyLock;

use super::Language;

pub struct Register {
    langs: Vec<Box<dyn Language + Sync + Send>>,
}

impl Register {
    pub fn new() -> Self {
        Self { langs: Vec::new() }
    }
    pub fn register<L: Language + Sync + Send + 'static>(&mut self, lang: L) {
        self.langs.push(Box::new(lang));
    }
    pub fn by_extension<'a>(&'a self, ext: &str) -> Option<&'a (dyn Language + Sync + Send)> {
        self.langs
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
