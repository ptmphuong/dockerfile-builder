//! # Type-safe interfaces for building Instructions
//!
//!
//! This module provides the definition of Instruction Builders and their fields.
//!
//!
//! ## Usage
//!
//! All build and setter methods are automatically generated following these formats: 
//! * The setter method names are identical to the fields names. 
//! * The argument types of the setter methods are the same as the field types unless the
//! field type is `Option<...>`. 
//! * If the field type is `Option<...>`, the argument type is the inner type of `Option`.
//! * Once all fields are set as desired, use `build()` to build the Instruction. `build()` returns
//! `Result<InstructionBuilder, std::err::Err>` to safely handle errors.
//!
//! ## Example 
//!
//! `ExposeBuilder` is the builder struct for `Expose`.
//!
//! ```rust
//! pub struct ExposeBuilder {
//!     pub port: u16,
//!     pub proto: Option<String>,
//! }
//! ```
//!
//! `Expose` can be constructed as follow:
//!
//! ```rust
//! use dockerfile_builder::instruction_builder::ExposeBuilder;
//! let expose = ExposeBuilder::builder()
//!     .port(80)
//!     .proto("tcp")
//!     .build()
//!     .unwrap();
//! ```

use crate::instruction::{From, Arg, Run, Expose};
use dockerfile_derive::InstructionBuilder;

/// Builder struct for `From` instruction
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = From, 
    value_method = value,
)]
pub struct FromBuilder {
    pub image: String,
    pub name: Option<String>,
    pub tag: Option<String>,
    pub digest: Option<String>,
    pub platform: Option<String>,
}

impl FromBuilder {
    fn value(&self) -> Result<String, String> {
        if self.tag.is_some() && self.digest.is_some() {
            return Err("Dockerfile image can only have tag OR digest".to_string());
        }

        let tag_or_digest = if let Some(t) = &self.tag {
            Some(format!(":{}", t))
        } else if let Some(d) = &self.digest {
            Some(format!("@{}", d))
        } else {
            None
        };

        Ok(
        format!(
            "{}{}{}{}", 
            self.platform.as_ref().map(|s| format!("--platform={} ", s)).unwrap_or_default(),
            &self.image,
            tag_or_digest.as_ref().map(|s| format!("{}", s)).unwrap_or_default(),
            self.name.as_ref().map(|s| format!(" AS {}", s)).unwrap_or_default(),
        )
        )
    }
}

/// Builder struct for `Run` instruction
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = Run, 
    value_method = value,
)]
pub struct RunBuilder {
    pub commands: Vec<String>,
}

impl RunBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(
            format!(r#"["{}"]"#, self.commands.join(r#"",""#))
        )
    }
}

/// Builder struct for `Arg` instruction
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = Arg,
    value_method = value,
)]
pub struct ArgBuilder {
    pub name: String,
    pub value: Option<String>,
}

impl ArgBuilder {
    fn value(&self) -> Result<String, String> {
        let value = match &self.value {
            Some(value) => format!("{}={}", self.name, value),
            None => self.name.to_string(),
        };
        Ok(value)
    }
}

/// Builder struct for `Expose` instruction
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = Expose,
    value_method = value,
)]
pub struct ExposeBuilder {
    pub port: u16,
    pub proto: Option<String>,
}

impl ExposeBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}", 
            self.port, 
            self.proto.clone().map(|p| format!("/{}", p)).unwrap_or_default()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn from() {
        let from = FromBuilder::builder()
            .image("cargo-chef")
            .build()
            .unwrap();
        let expected = expect!["FROM cargo-chef"];
        expected.assert_eq(&from.to_string());

        let from = FromBuilder::builder()
            .image("cargo-chef")
            .platform("linux/arm64")
            .build()
            .unwrap();
        let expected = expect!["FROM --platform=linux/arm64 cargo-chef"];
        expected.assert_eq(&from.to_string());

        let from = FromBuilder::builder()
            .image("cargo-chef")
            .name("chef")
            .build()
            .unwrap();
        let expected = expect!["FROM cargo-chef AS chef"];
        expected.assert_eq(&from.to_string());

        let from = FromBuilder::builder()
            .image("cargo-chef")
            .name("chef")
            .tag("latest")
            .build()
            .unwrap();
        let expected = expect!["FROM cargo-chef:latest AS chef"];
        expected.assert_eq(&from.to_string());

        let from = FromBuilder::builder()
            .image("cargo-chef")
            .name("chef")
            .digest("sha256")
            .build()
            .unwrap();
        let expected = expect!["FROM cargo-chef@sha256 AS chef"];
        expected.assert_eq(&from.to_string());
    }

    #[test]
    fn from_err() {
        let from = FromBuilder::builder()
            .build();
        match from {
            Ok(_) => panic!("Required field is not set. Expect test to fail"),
            Err(e) => assert_eq!(
                e.to_string(),
                "image is not set for FromBuilder".to_string(),
            ),
        }

        let from = FromBuilder::builder()
            .image("cargo-chef")
            .tag("t")
            .digest("d")
            .build();
        match from {
            Ok(_) => panic!("Both tag and digest are set. Expect test to fail"),
            Err(e) => assert_eq!(
                e.to_string(),
                "Dockerfile image can only have tag OR digest".to_string(),
            ),
        }
    }
}
