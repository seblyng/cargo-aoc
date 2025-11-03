mod impls;
use impls::*;

mod r#trait;

pub use r#trait::{Common, Compiler, RunningArgs};
mod register;
pub use register::{REGISTER, Register};
