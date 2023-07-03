use dockerfile_derive::InstructionInit;

//#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(Debug, InstructionInit, Eq, PartialEq)]
pub enum Instruction {
    From(From),
    Run(Run),
    Arg(Arg),
    Expose(Expose),
    Any(String),
}

impl<T> std::convert::From<T> for Instruction where T: Into<String> {
    fn from(instruction: T) -> Self {
        Instruction::Any(instruction.into())
    }
}

