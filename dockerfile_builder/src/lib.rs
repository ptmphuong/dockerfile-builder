//! This library provides a convenient way to programmatically generate Dockerfiles using Rust.
//!
//! Dockerfiles instructions can be generated using structured and type-safe interfaces, or they can be added flexibly in raw form.
//!
//! # Usage
//!
//! ## Build Dockerfile
//!
//! [Dockerfile] contains a list of [Instructions].
//!
//! [Dockerfile]: Dockerfile
//! [Instructions]: instruction
//!
//!```rust
//!use dockerfile_builder::Dockerfile;
//!use dockerfile_builder::instruction::{RUN, EXPOSE};
//!
//!let dockerfile = Dockerfile::default()
//!    .push(RUN::from("echo $HOME"))
//!    .push(EXPOSE::from("80/tcp"))
//!    .push_any("Just adding a comment");
//!    
//!let expected = r"RUN echo $HOME
//!EXPOSE 80/tcp
//!Just adding a comment";
//!
//!assert_eq!(
//!    dockerfile.to_string(),
//!    expected
//!);
//!```
//!
//! ## Dockerfile Instructions
//!
//! [Instruction] can be created from String or from [Instruction Builder].
//!
//! Instruction Builders provide structured and type-safe interfaces to build instructions.
//!
//! [Instruction]: instruction::Instruction
//! [Instruction Builder]: instruction_builder
//!
//! ```rust
//!use dockerfile_builder::Dockerfile;
//!use dockerfile_builder::instruction::EXPOSE;
//!use dockerfile_builder::instruction_builder::ExposeBuilder;
//!
//!let expose = EXPOSE::from("80/tcp");
//!
//!let expose_from_builder = ExposeBuilder::builder()
//!    .port(80)
//!    .protocol("tcp")
//!    .build()
//!    .unwrap();
//!
//!assert_eq!(expose, expose_from_builder);
//!
//!let dockerfile = Dockerfile::default()
//!    .push(expose_from_builder);
//!  
//!assert_eq!(
//!    dockerfile.to_string(), 
//!    "EXPOSE 80/tcp"
//!);
//!
//! ```


use std::fmt::{Display, self};

use instruction::Instruction;

pub mod instruction;
pub mod instruction_builder;

#[derive(Debug, Default)]
pub struct Dockerfile {
    instructions: Vec<Instruction>,
}

impl Dockerfile {
    /// Adds an [`Instruction`] to the end of the Dockerfile
    ///
    /// [Instruction]: instruction::Instruction
    pub fn push<T: Into<Instruction>>(mut self, instruction: T) -> Self {
        self.instructions.push(instruction.into());
        self
    }

    /// Adds any raw string to the end of the Dockerfile
    pub fn push_any<T: Into<String>>(mut self, instruction: T) -> Self {
        self.instructions.push(Instruction::ANY(instruction.into()));
        self
    }

    /// Appends multiple ['Instruction']s to the end of the Dockerfile
    ///
    /// [Instruction]: instruction::Instruction
    pub fn append<T: Into<Instruction>>(mut self, instructions: Vec<T>) -> Self {
        for i in instructions {
            self.instructions.push(i.into());
        }
        self
    }

    /// Appends multiple raw strings to the end of the Dockerfile
    pub fn append_any<T: Into<String>>(mut self, instructions: Vec<T>) -> Self {
        for i in instructions {
            self.instructions.push(Instruction::ANY(i.into()));
        }
        self
    }

    /// Retrieves the vec of `Instruction`s from Dockerfile
    ///
    /// [Instruction]: instruction::Instruction
    pub fn into_inner(self) -> Vec<Instruction> {
        self.instructions
    }
}

impl Display for Dockerfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instructions = 
            self.instructions
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>();
        write!(f, "{}", instructions.join("\n"))
    }
}


#[cfg(test)]
mod tests {
    use crate::{instruction::{FROM, RUN, EXPOSE}, instruction_builder::ExposeBuilder};
    use expect_test::expect;
    use super::*;

    #[test]
    fn quick_start() {
        let dockerfile = Dockerfile::default()
            .push(RUN::from("echo $HOME"))
            .push(EXPOSE::from("80/tcp"))
            .push_any("# Just adding a comment");

        let expected = expect![[r#"
            RUN echo $HOME
            EXPOSE 80/tcp
            # Just adding a comment"#]];
        expected.assert_eq(&dockerfile.to_string());
    }

    #[test]
    fn build_dockerfile() {
        // 2 ways of constructing Instruction.

        // Directly from String/&str
        let expose = EXPOSE::from("80/tcp");

        // Use a builder
        let expose_from_builder = ExposeBuilder::builder()
            .port(80)
            .protocol("tcp")
            .build()
            .unwrap();

        assert_eq!(expose, expose_from_builder);
        
        let dockerfile = Dockerfile::default()
            .push(expose_from_builder);

        let expected = expect!["EXPOSE 80/tcp"];
        expected.assert_eq(&dockerfile.to_string());
    }

    #[test]
    fn append_instructions() {
        let comments = vec![
            "# syntax=docker/dockerfile:1",
            "# escape=`",
        ];
        let instruction_vec = vec![
            Instruction::FROM(FROM::from("cargo-chef AS chef")),
            Instruction::RUN(RUN::from("cargo run")),
        ];

        let dockerfile = Dockerfile::default()
            .append_any(comments)
            .append(instruction_vec);

        let expected = expect![[r#"
            # syntax=docker/dockerfile:1
            # escape=`
            FROM cargo-chef AS chef
            RUN cargo run"#]];
        expected.assert_eq(&dockerfile.to_string());
    }

}
