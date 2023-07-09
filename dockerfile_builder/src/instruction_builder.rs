//! # Type-safe interfaces for building Instructions
//!
//!
//! This module provides the definition of Instruction Builders and their fields.
//!
//!
//! ## Usage
//!
//! All build and setter methods for Instruction Builders are automatically generated and follow the same format.
//!
//! For example: 
//!
//! `ExposeBuilder` is the builder struct for `Expose`.
//!
//! ```rust
//! pub struct ExposeBuilder {
//!     pub port: u16,
//!     pub protocol: Option<String>,
//! }
//! ```
//!
//! `Expose` can be constructed as follow:
//!
//! ```rust
//! use dockerfile_builder::instruction_builder::ExposeBuilder;
//! let expose = ExposeBuilder::builder()
//!     .port(80)
//!     .protocol("tcp")
//!     .build()
//!     .unwrap();
//! ```
//! 
//! Note that:
//! * The setter method names are identical to the fields names. 
//! * For fields with `Option<...>` type: The argument type is the inner type of `Option`. It is
//! optional to set these fields.
//! * Once all fields are set as desired, use `build()` to build the Instruction. `build()` returns
//! `Result<InstructionBuilder, std::err::Err>` to safely handle errors.
//!
//!
//! For fields with `Vec<...>` type, it is also possible to set each element of the Vec.
//!
//! For example: 
//!
//! `RunBuilder` is the builder struct for `Run`.
//!
//! ```rust
//! pub struct RunBuilder {
//!     pub commands: Vec<String>,
//! }
//! ```
//!
//! `Run` can be constructed as follow:
//! ```rust
//! use dockerfile_builder::instruction_builder::RunBuilder;
//! let run = RunBuilder::builder()
//!     .command("source $HOME/.bashrc")
//!     .command("echo $HOME")
//!     .build()
//!     .unwrap();
//! ```
//!

use crate::instruction::{FROM, RUN, CMD, ENV, ARG, EXPOSE, LABEL};
use dockerfile_derive::InstructionBuilder;

/// Builder struct for `FROM` instruction
/// * `FROM [--platform=<platform>] <image> [AS <name>]`
/// or 
/// * `FROM [--platform=<platform>] <image>[:<tag>] [AS <name>]`
/// or 
/// * `FROM [--platform=<platform>] <image>[@<digest>] [AS <name>]`
///
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = FROM, 
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


/// Builder struct for `ENV` instruction
/// * `ENV <key>=<value>`
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ENV, 
    value_method = value,
)]
pub struct EnvBuilder {
    pub key: String,
    pub value: String,
}

impl EnvBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}={}",
            self.key, self.value
        ))
    }
}


/// Builder struct for `RUN` instruction (shell form)
/// 
/// RunBuilder constructs the shell form for [`RUN`] by default.
/// * `RUN <command>`
///
/// To construct the exec form, use [`RunExecBuilder`]
///
/// [RUN]: dockerfile_builder::instruction::RUN
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = RUN, 
    value_method = value,
)]
pub struct RunBuilder {
    #[instruction_builder(each = command)]
    pub commands: Vec<String>,
}

impl RunBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!("{}", self.commands.join(" && \\ \n")))
    }
}


/// Builder struct for `RUN` instruction (exec form)
/// 
/// RunBuilder constructs the exec form for [`RUN`].
/// * `RUN ["executable", "param1", "param2"]`
///
/// To construct the shell form, use [`RunBuilder`]
///
/// [RUN]: dockerfile_builder::instruction::RUN
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = RUN, 
    value_method = value,
)]
pub struct RunExecBuilder {
    #[instruction_builder(each = command)]
    pub commands: Vec<String>,
}

impl RunExecBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(r#"["{}"]"#, self.commands.join(r#"",""#)))
    }
}


/// Builder struct for `CMD` instruction (shell form)
/// 
/// CmdBuilder constructs the shell form for [`CMD`] by default.
/// * `CMD command param1 param2`
///
/// To construct the exec form, use [`CmdExecBuilder`]
///
/// [CMD]: dockerfile_builder::instruction::CMD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = CMD, 
    value_method = value,
)]
pub struct CmdBuilder {
    #[instruction_builder(each = command)]
    pub commands: Vec<String>,
}

impl CmdBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!("{}", self.commands.join(" ")))
    }
}


/// Builder struct for `CMD` instruction (exec form)
/// 
/// CmdBuilder constructs the exec form for [`CMD`].
/// * `CMD ["executable", "param1", "param2"]`
///
/// To construct the shell form, use [`CmdBuilder`]
///
/// [CMD]: dockerfile_builder::instruction::CMD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = CMD, 
    value_method = value,
)]
pub struct CmdExecBuilder {
    #[instruction_builder(each = command)]
    pub commands: Vec<String>,
}

impl CmdExecBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(r#"["{}"]"#, self.commands.join(r#"",""#)))
    }
}


/// Builder struct for `LABEL` instruction
/// * `LABEL <key>=<value>`
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = LABEL, 
    value_method = value,
)]
pub struct LabelBuilder {
    pub key: String,
    pub value: String,
}

impl LabelBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}={}",
            self.key, self.value
        ))
    }
}


/// Builder struct for `ARG` instruction
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ARG,
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

/// Builder struct for `EXPOSE` instruction
/// * `EXPOSE <port>`
/// or
/// * `EXPOSE <port>/<protocol>`
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = EXPOSE,
    value_method = value,
)]
pub struct ExposeBuilder {
    pub port: u16,
    pub protocol: Option<String>,
}

impl ExposeBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}", 
            self.port, 
            self.protocol.clone().map(|p| format!("/{}", p)).unwrap_or_default()
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

    #[test]
    fn env() {
        let env = EnvBuilder::builder()
            .key("foo").value("bar")
            .build().unwrap();
        let expected = expect!["ENV foo=bar"];
        expected.assert_eq(&env.to_string());
    }

    #[test]
    fn run() {
        let commands = vec!["source $HOME/.bashrc".to_string(), "echo $HOME".to_string()];

        let run_shell_form = RunBuilder::builder().commands(commands.clone()).build().unwrap();
        let expected = expect![[r#"
            RUN source $HOME/.bashrc && \ 
            echo $HOME"#]];
        expected.assert_eq(&run_shell_form.to_string());

        let run_exec_form = RunExecBuilder::builder().commands(commands).build().unwrap();
        let expected = expect![[r#"RUN ["source $HOME/.bashrc","echo $HOME"]"#]];
        expected.assert_eq(&run_exec_form.to_string());
    }

    #[test]
    fn run_each() {
        let run_shell_form = RunBuilder::builder()
            .command("source $HOME/.bashrc")
            .command("echo $HOME")
            .build()
            .unwrap();
        let expected = expect![[r#"
            RUN source $HOME/.bashrc && \ 
            echo $HOME"#]];
        expected.assert_eq(&run_shell_form.to_string());

        let run_exec_form = RunExecBuilder::builder()
            .command("source $HOME/.bashrc")
            .command("echo $HOME")
            .build()
            .unwrap();
        let expected = expect![[r#"RUN ["source $HOME/.bashrc","echo $HOME"]"#]];
        expected.assert_eq(&run_exec_form.to_string());
    }

    #[test]
    fn cmd() {
        let commands = vec![r#"echo "This is a test.""#.to_string(), "|".to_string(), "wc -".to_string()];
        let cmd_shell_form = CmdBuilder::builder().commands(commands).build().unwrap();
        let expected = expect![[r#"CMD echo "This is a test." | wc -"#]];
        expected.assert_eq(&cmd_shell_form.to_string());

        let commands = vec!["/usr/bin/wc".to_string(),"--help".to_string()];
        let cmd_exec_form = CmdExecBuilder::builder().commands(commands).build().unwrap();
        let expected = expect![[r#"CMD ["/usr/bin/wc","--help"]"#]];
        expected.assert_eq(&cmd_exec_form.to_string());
    }

    #[test]
    fn label() {
        let label = LabelBuilder::builder()
            .key("version").value(r#""1.0""#)
            .build().unwrap();
        let expected = expect![[r#"LABEL version="1.0""#]];
        expected.assert_eq(&label.to_string());
    }

    #[test]
    fn expose() {
        let expose = ExposeBuilder::builder()
            .port(80)
            .build().unwrap();
        let expected = expect![[r#"EXPOSE 80"#]];
        expected.assert_eq(&expose.to_string());

        let expose = ExposeBuilder::builder()
            .port(80).protocol("udp")
            .build().unwrap();
        let expected = expect![[r#"EXPOSE 80/udp"#]];
        expected.assert_eq(&expose.to_string());
    }
}
