use std::fmt::{Display, Formatter};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    IllegalOpcode {  opcode: u16 },
    StackOverflow,
    StackUnderflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalOpcode { opcode } => write!(f, "illegal opcode: {}", opcode),
            Self::StackOverflow => write!(f, "stack overflow"),
            Self::StackUnderflow => write!(f, "stack underflow"),
        }
    }
}
