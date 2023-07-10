use dockerfile_derive::InstructionInit;

//#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(Debug, InstructionInit, Eq, PartialEq)]
pub enum Instruction {
    FROM(FROM),
    ENV(ENV),
    RUN(RUN),
    CMD(CMD),
    LABEL(LABEL),
    EXPOSE(EXPOSE),
    ADD(ADD),
    COPY(COPY),
    ENTRYPOINT(ENTRYPOINT),
    VOLUME(VOLUME),
    USER(USER),
    ARG(ARG),
    ANY(String),
}

impl<T> std::convert::From<T> for Instruction where T: Into<String> {
    fn from(instruction: T) -> Self {
        Instruction::ANY(instruction.into())
    }
}

