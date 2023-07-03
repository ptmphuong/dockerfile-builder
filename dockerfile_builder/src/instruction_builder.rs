use crate::instruction::{Arg, Run, Expose};
use dockerfile_derive::InstructionBuilder;

#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = Run, 
    value_method = value,
)]
pub struct RunBuilder {
    pub commands: Vec<String>,
}

impl RunBuilder {
    fn value(&self) -> String {
        format!(r#"["{}"]"#, self.commands.join(r#"",""#))
    }
}

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
    fn value(&self) -> String {
        match &self.value {
            Some(value) => format!("{}={}", self.name, value),
            None => self.name.to_string(),
        }
    }
}

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
    fn value(&self) -> String {
        format!(
            "{}{}", 
            self.port, 
            self.proto.clone().map(|p| format!("/{}", p)).unwrap_or_default()
        )
    }
}

// TODO: add unit tests for InsBuilder value()
