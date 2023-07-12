//! Dockerfile Instruction definitions
//!
//! See [`Instruction`]

use dockerfile_derive::InstructionInit;

/// Dockerfile Instructions
///
/// There are 2 ways to build an Instruction:
/// * from string literals, or 
/// * using the [`instruction_builder`] interface.
///
/// [`instruction_builder`]: crate::instruction_builder
///
/// ```
/// use dockerfile_builder::instruction::FROM;
/// use dockerfile_builder::instruction_builder::FromBuilder;
///
/// let from = FROM::from("cargo-chef AS chef");
///
/// let from_by_builder = FromBuilder::builder()
///     .image("cargo-chef")
///     .name("chef")
///     .build()
///     .unwrap();
/// 
/// assert_eq!(from, from_by_builder);
/// ```
//#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(Debug, InstructionInit, Clone, Eq, PartialEq)]
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
    WORKDIR(WORKDIR),
    ARG(ARG),
    ONBUILD(ONBUILD),
    STOPSIGNAL(STOPSIGNAL),
    HEALTHCHECK(HEALTHCHECK),
    SHELL(SHELL),
    ANY(String),
}

impl<T> std::convert::From<T> for Instruction where T: Into<String> {
    fn from(instruction: T) -> Self {
        Instruction::ANY(instruction.into())
    }
}

