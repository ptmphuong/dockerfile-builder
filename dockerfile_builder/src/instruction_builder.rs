//! # Type-safe interfaces for building Instructions
//!
//!
//! This module provides the definition of Instruction Builders and their fields, following the
//! [Dockerfile reference].
//!
//! [Dockerfile reference]: https://docs.docker.com/engine/reference/builder/
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
//! # use dockerfile_builder::instruction_builder::ExposeBuilder;
//! let expose = ExposeBuilder::builder()
//!     .port(80)
//!     .protocol("tcp")
//!     .build()
//!     .unwrap();
//! ```
//!
//! Note that:
//! * The setter method names are identical to the fields names.
//! * For fields with `Option<inner_type>` type: The argument type is the inner_type. It is
//! optional to set these fields.
//! * Use `build()` to complete building the instruction. `build()` returns
//! `Result<InstructionBuilder, std::err::Err>` to safely handle errors.
//!
//!
//! For fields with `Vec<_>` or `Option<Vec<_>>` type, it is possible to set each element of the Vec.
//!
//! For example:
//!
//! `RunBuilder` is the builder struct for `Run`.
//!
//! ```
//! pub struct RunBuilder {
//!     pub commands: Vec<String>,
//! }
//! ```
//!
//! `Run` can be constructed as follow:
//! ```
//! # use dockerfile_builder::instruction_builder::RunBuilder;
//! let run = RunBuilder::builder()
//!     .command("source $HOME/.bashrc")
//!     .command("echo $HOME")
//!     .build()
//!     .unwrap();
//! assert_eq!(
//!     run.to_string(),
//!     r#"RUN source $HOME/.bashrc && \
//!echo $HOME"#,
//! );
//! ```
//!

use crate::instruction::{
    Instruction, ADD, ARG, CMD, COPY, ENTRYPOINT, ENV, EXPOSE, FROM, HEALTHCHECK, LABEL, ONBUILD,
    RUN, SHELL, STOPSIGNAL, USER, VOLUME, WORKDIR,
};
use dockerfile_derive::InstructionBuilder;
use eyre::Result;

/// Builder struct for [`FROM`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#from):
/// * `FROM [--platform=<platform>] <image> [AS <name>]`
/// or
/// * `FROM [--platform=<platform>] <image>[:<tag>] [AS <name>]`
/// or
/// * `FROM [--platform=<platform>] <image>[@<digest>] [AS <name>]`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::FromBuilder;
/// // Build FROM with image and name
/// let from = FromBuilder::builder()
///     .image("cargo-chef")
///     .name("chef")
///     .build()
///     .unwrap();
/// assert_eq!(from.to_string(), "FROM cargo-chef AS chef");
///
/// // Build FROM with image, name, and tag
/// let from = FromBuilder::builder()
///     .image("cargo-chef")
///     .tag("latest")
///     .name("chef")
///     .build()
///     .unwrap();
/// assert_eq!(from.to_string(), "FROM cargo-chef:latest AS chef");
/// ```
///
/// [FROM]: dockerfile_builder::instruction::FROM
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
        } else { self.digest.as_ref().map(|d| format!("@{}", d)) };

        Ok(format!(
            "{}{}{}{}",
            self.platform
                .as_ref()
                .map(|s| format!("--platform={} ", s))
                .unwrap_or_default(),
            &self.image,
            tag_or_digest
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            self.name
                .as_ref()
                .map(|s| format!(" AS {}", s))
                .unwrap_or_default(),
        ))
    }
}

/// Builder struct for [`ENV`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#env):
/// * `ENV <key>=<value>`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::EnvBuilder;
/// let env = EnvBuilder::builder()
///     .key("foo")
///     .value("bar")
///     .build()
///     .unwrap();
/// assert_eq!(env.to_string(), "ENV foo=bar");
/// ```
///
/// [ENV]: dockerfile_builder::instruction::ENV
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
        Ok(format!("{}={}", self.key, self.value))
    }
}

/// Builder struct for [`RUN`] instruction (shell form)
///
/// `RunBuilder` constructs the shell form for [`RUN`] by default.
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#run):
/// * `RUN command`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::RunBuilder;
/// // build RUN with a single command
/// let run = RunBuilder::builder()
///     .command("source $HOME/.bashrc")
///     .build().unwrap();
/// assert_eq!(run.to_string(), "RUN source $HOME/.bashrc");
///
/// // build RUN with multiple commands, commands are separated by `&& \`
/// let run = RunBuilder::builder()
///     .command("source $HOME/.bashrc")
///     .command("echo $HOME")
///     .build().unwrap();
/// assert_eq!(
///     run.to_string(),
///     r#"RUN source $HOME/.bashrc && \
///echo $HOME"#,
/// );
///
/// // build RUN with multiple commands using a Vec
/// let run = RunBuilder::builder()
///     .commands(vec!["source $HOME/.bashrc", "echo $HOME"])
///     .build().unwrap();
/// assert_eq!(
///     run.to_string(),
///     r#"RUN source $HOME/.bashrc && \
///echo $HOME"#,
/// );
/// ```
///
/// To construct the exec form of `RUN`, use [`RunExecBuilder`]
///
/// [RUN]: dockerfile_builder::instruction::RUN
// TODO: Flag options for RUN
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
        Ok(self.commands.join(" && \\\n").to_string())
    }
}

/// Builder struct for [`RUN`] instruction (exec form)
///
/// RunBuilder constructs the exec form for [`RUN`].
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#run):
/// * `RUN ["executable", "param1", "param2"]`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::RunExecBuilder;
/// // build RUN by adding multiple params
/// let run = RunExecBuilder::builder()
///     .executable("mybin.exe")
///     .param("-f")
///     .param("-c")
///     .build().unwrap();
/// assert_eq!(run.to_string(), r#"RUN ["mybin.exe", "-f", "-c"]"#);
///
/// // build RUN with multiple params using a vec
/// let run = RunExecBuilder::builder()
///     .executable("mybin.exe")
///     .params(vec!["-f", "-c"])
///     .build().unwrap();
/// assert_eq!(run.to_string(), r#"RUN ["mybin.exe", "-f", "-c"]"#);
/// ```
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
    pub executable: String,
    #[instruction_builder(each = param)]
    pub params: Option<Vec<String>>,
}

impl RunExecBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            r#"["{}", "{}"]"#,
            self.executable,
            self.params.clone().unwrap_or_default().join(r#"", ""#)
        ))
    }
}

/// Builder struct for [`CMD`] instruction (shell form)
///
/// CmdBuilder constructs the shell form for [`CMD`] by default.
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#cmd):
/// * `CMD command param1 param2`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::CmdBuilder;
/// // build CMD by adding multiple params
/// let cmd = CmdBuilder::builder()
///     .command(r#"echo "this is a test""#)
///     .param("| wc")
///     .param("-l")
///     .build().unwrap();
/// assert_eq!(cmd.to_string(), r#"CMD echo "this is a test" | wc -l"#);
///
/// // build CMD with multiple params using a vec    
/// let cmd = CmdBuilder::builder()
///     .command(r#"echo "this is a test""#)
///     .params(vec!["| wc", "-l"])
///     .build().unwrap();
/// assert_eq!(cmd.to_string(), r#"CMD echo "this is a test" | wc -l"#);
/// ```
///
/// To construct the exec form or CMD in combination with ENTRYPOINT, use [`CmdExecBuilder`]
///
/// [CMD]: dockerfile_builder::instruction::CMD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = CMD,
    value_method = value,
)]
pub struct CmdBuilder {
    pub command: String,
    #[instruction_builder(each = param)]
    pub params: Option<Vec<String>>,
}

impl CmdBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{} {}",
            self.command,
            self.params.clone().unwrap_or_default().join(" "),
        ))
    }
}

/// Builder struct for [`CMD`] instruction (exec form)
///
/// CmdBuilder constructs the exec form for [`CMD`].
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#cmd):
/// * `CMD ["executable", "param1", "param2"]`
/// OR
/// * `CMD ["param1","param2"]` (as default parameters to ENTRYPOINT)
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::CmdExecBuilder;
/// // build CMD with a single param
/// let cmd = CmdExecBuilder::builder()
///     .executable("/usr/bin/wc")
///     .param("--help")
///     .build().unwrap();
/// assert_eq!(cmd.to_string(), r#"CMD ["/usr/bin/wc", "--help"]"#);
///
/// // build CMD for ENTRYPOINT
/// let cmd = CmdExecBuilder::builder()
///     .param("-l")
///     .param("8000")
///     .build().unwrap();
/// assert_eq!(cmd.to_string(), r#"CMD ["-l", "8000"]"#);
/// ```
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
    pub executable: Option<String>,
    #[instruction_builder(each = param)]
    pub params: Option<Vec<String>>,
}

impl CmdExecBuilder {
    fn value(&self) -> Result<String, String> {
        if self.executable.is_none() && self.params.is_none() {
            return Err("CMD cannot be empty".to_string());
        }
        Ok(format!(
            r#"[{}"{}"]"#,
            self.executable
                .as_ref()
                .map(|e| format!(r#""{}", "#, e))
                .unwrap_or_default(),
            self.params.clone().unwrap_or_default().join(r#"", ""#),
        ))
    }
}

/// Builder struct for [`LABEL`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#label):
/// * `LABEL <key>=<value>`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::LabelBuilder;
/// let label = LabelBuilder::builder()
///     .key("foo")
///     .value("bar")
///     .build()
///     .unwrap();
/// assert_eq!(label.to_string(), "LABEL foo=bar");
/// ```
///
/// [LABEL]: dockerfile_builder::instruction::LABEL
///
// TODO: The official format is
// * `LABEL <key>=<value> <key>=<value> <key>=<value> ...`
// Use `each` to support the multiple format.
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
        Ok(format!("{}={}", self.key, self.value))
    }
}

/// Builder struct for [`EXPOSE`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#expose):
/// * `EXPOSE <port>`
/// or
/// * `EXPOSE <port>/<protocol>`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::ExposeBuilder;
/// let expose = ExposeBuilder::builder()
///     .port(80)
///     .protocol("udp")
///     .build()
///     .unwrap();
/// assert_eq!(expose.to_string(), "EXPOSE 80/udp");
/// ```
///
/// [EXPOSE]: dockerfile_builder::instruction::EXPOSE
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
            self.protocol
                .as_ref()
                .map(|p| format!("/{}", p))
                .unwrap_or_default()
        ))
    }
}

/// Builder struct for [`ADD`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#add):
/// * `ADD [--chown=<chown>] [--chmod=<chmod>] <src>... <dest>`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::AddBuilder;
/// let add = AddBuilder::builder()
///     .chown("myuser:mygroup")
///     .chmod(655)
///     .src("hom*")
///     .dest("/mydir/")
///     .build().unwrap();
/// assert_eq!(add.to_string(), "ADD --chown=myuser:mygroup --chmod=655 hom* /mydir/");
/// ```
///
/// [ADD]: dockerfile_builder::instruction::ADD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ADD,
    value_method = value,
)]
pub struct AddBuilder {
    pub src: String,
    pub dest: String,
    pub chown: Option<String>,
    pub chmod: Option<u16>,
}

impl AddBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}{} {}",
            self.chown
                .as_ref()
                .map(|c| format!("--chown={} ", c))
                .unwrap_or_default(),
            self.chmod
                .as_ref()
                .map(|c| format!("--chmod={} ", c))
                .unwrap_or_default(),
            self.src,
            self.dest,
        ))
    }
}

/// Builder struct for [`ADD`] instruction (http src)
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#add):
/// * `ADD --checksum=<checksum> <src> <dest>`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::AddHttpBuilder;
/// let add = AddHttpBuilder::builder()
///     .checksum("sha256::123")
///     .src("http://example.com/foobar")
///     .dest("/")
///     .build().unwrap();
/// assert_eq!(add.to_string(), "ADD --checksum=sha256::123 http://example.com/foobar /");
/// ```
///
/// [ADD]: dockerfile_builder::instruction::ADD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ADD,
    value_method = value,
)]
pub struct AddHttpBuilder {
    pub src: String,
    pub dest: String,
    pub checksum: Option<String>,
}

impl AddHttpBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{} {}",
            self.checksum
                .as_ref()
                .map(|c| format!("--checksum={} ", c))
                .unwrap_or_default(),
            self.src,
            self.dest,
        ))
    }
}

/// Builder struct for [`ADD`] instruction (git repository)
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#add):
/// * `ADD [--keep-git-dir=<boolean>] <git ref> <dir>`
///
/// [ADD]: dockerfile_builder::instruction::ADD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ADD,
    value_method = value,
)]
pub struct AddGitBuilder {
    pub git_ref: String,
    pub dir: String,
    pub keep_git_dir: Option<bool>,
}

impl AddGitBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{} {}",
            self.keep_git_dir
                .as_ref()
                .map(|c| format!("--keep-git-dir={} ", c))
                .unwrap_or_default(),
            self.git_ref,
            self.dir,
        ))
    }
}

/// Builder struct for [`COPY`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#copy):
/// * `COPY [--chown=<chown>] [--chmod=<chmod>] [--from=<from>] [--link] <src>... <dest>`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::CopyBuilder;
/// let copy = CopyBuilder::builder()
///     .chown("55:mygroup")
///     .chmod(644)
///     .src("files*")
///     .dest("/somedir/")
///     .build().unwrap();
/// assert_eq!(copy.to_string(), "COPY --chown=55:mygroup --chmod=644 files* /somedir/");
/// ```
///
/// [COPY]: dockerfile_builder::instruction::COPY
// TODO: Add flag [--from=]
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = COPY,
    value_method = value,
)]
pub struct CopyBuilder {
    pub src: String,
    pub dest: String,
    pub chown: Option<String>,
    pub chmod: Option<u16>,
    pub link: Option<bool>,
    pub from: Option<String>,
}

impl CopyBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}{}{}{} {}",
            self.chown
                .as_ref()
                .map(|c| format!("--chown={} ", c))
                .unwrap_or_default(),
            self.chmod
                .as_ref()
                .map(|c| format!("--chmod={} ", c))
                .unwrap_or_default(),
            self.link
                .as_ref()
                .map(|c| match c {
                    true => "--link ".to_string(),
                    false => "".to_string(),
                })
                .unwrap_or_default(),
            self.from
                .as_ref()
                .map(|c| format!("--chmod={} ", c))
                .unwrap_or_default(),
            self.src,
            self.dest,
        ))
    }
}

/// Builder struct for [`ENTRYPOINT`] instruction (shell form)
///
/// EntrypointBuilder constructs the shell form for [`ENTRYPOINT`] by default.
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#entrypoint):
/// * `ENTRYPOINT command param1 param2`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::EntrypointBuilder;
/// // build ENTRYPOINT with params
/// let entrypoint = EntrypointBuilder::builder()
///     .command("some command")
///     .param("-f")
///     .param("-c")
///     .build().unwrap();
/// assert_eq!(entrypoint.to_string(), "ENTRYPOINT some command -f -c");
///
/// // build ENTRYPOINT with a param vec
/// let entrypoint = EntrypointBuilder::builder()
///     .command("some command")
///     .params(vec!["-f", "-c"])
///     .build().unwrap();
/// assert_eq!(entrypoint.to_string(), "ENTRYPOINT some command -f -c");
/// ```
///
/// To construct the exec form, use [`EntrypointExecBuilder`]
///
/// [ENTRYPOINT]: dockerfile_builder::instruction::ENTRYPOINT
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ENTRYPOINT,
    value_method = value,
)]
pub struct EntrypointBuilder {
    pub command: String,
    #[instruction_builder(each = param)]
    pub params: Option<Vec<String>>,
}

impl EntrypointBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}",
            self.command,
            self.params
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|p| format!(" {}", p))
                .collect::<Vec<String>>()
                .join(""),
        ))
    }
}

/// Builder struct for [`ENTRYPOINT`] instruction (exec form)
///
/// EntrypointExecBuilder constructs the exec form for [`ENTRYPOINT`].
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#entrypoint):
/// * `ENTRYPOINT ["executable", "param1", "param2"]`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::EntrypointExecBuilder;
/// // build ENTRYPOINT with params
/// let entrypoint = EntrypointExecBuilder::builder()
///     .executable("/usr/sbin/apache2ctl")
///     .param("-D")
///     .param("FOREGROUND")
///     .build().unwrap();
/// assert_eq!(entrypoint.to_string(), r#"ENTRYPOINT ["/usr/sbin/apache2ctl", "-D", "FOREGROUND"]"#);
///
/// // build ENTRYPOINT with a param vec
/// let entrypoint = EntrypointExecBuilder::builder()
///     .executable("/usr/sbin/apache2ctl")
///     .params(vec!["-D", "FOREGROUND"])
///     .build().unwrap();
/// assert_eq!(entrypoint.to_string(), r#"ENTRYPOINT ["/usr/sbin/apache2ctl", "-D", "FOREGROUND"]"#);
/// ```
///
/// To construct the shell form, use [`EntrypointBuilder`]
///
/// [ENTRYPOINT]: dockerfile_builder::instruction::ENTRYPOINT
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ENTRYPOINT,
    value_method = value,
)]
pub struct EntrypointExecBuilder {
    pub executable: String,
    #[instruction_builder(each = param)]
    pub params: Vec<String>,
}

impl EntrypointExecBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            r#"["{}", "{}"]"#,
            self.executable,
            self.params.join(r#"", ""#),
        ))
    }
}

/// Builder struct for [`VOLUME`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#volume):
/// * `VOLUME <path>...`
///
/// [VOLUME]: dockerfile_builder::instruction::VOLUME
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = VOLUME,
    value_method = value,
)]
pub struct VolumeBuilder {
    #[instruction_builder(each = path)]
    pub paths: Vec<String>,
}

impl VolumeBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(self.paths.join(" ").to_string())
    }
}

/// Builder struct for [`USER`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#user):
/// * `USER <user>`
/// or
/// * `USER <user>:<group>`
/// [USER]: dockerfile_builder::instruction::USER
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = USER,
    value_method = value,
)]
pub struct UserBuilder {
    pub user: String,
    pub group: Option<String>,
}

impl UserBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}",
            self.user,
            self.group
                .as_ref()
                .map(|g| format!(":{}", g))
                .unwrap_or_default()
        ))
    }
}

/// Builder struct for [`WORKDIR`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#workdir):
/// * `WORKDIR <path>`
///
/// [WORKDIR]: dockerfile_builder::instruction::WORKDIR
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = WORKDIR,
    value_method = value,
)]
pub struct WorkdirBuilder {
    pub path: String,
}

impl WorkdirBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(self.path.to_string())
    }
}

/// Builder struct for [`ARG`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#arg):
/// * `ARG <name>[=<value>]`
///
/// [ARG]: dockerfile_builder::instruction::ARG
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

/// Builder struct for [`ONBUILD`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#onbuild):
/// * `ONBUILD <INSTRUCTION>`
///
/// [ONBUILD]: dockerfile_builder::instruction::ONBUILD
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = ONBUILD,
    value_method = value,
)]
pub struct OnbuildBuilder {
    pub instruction: Instruction,
}

impl OnbuildBuilder {
    fn value(&self) -> Result<String, String> {
        match &self.instruction {
            Instruction::ONBUILD(_) => {
                Err("Chaining ONBUILD instructions using ONBUILD ONBUILD isn’t allowed".to_string())
            }
            Instruction::FROM(_) => {
                Err("ONBUILD instruction may not trigger FROM instruction".to_string())
            }
            ins => Ok(ins.to_string()),
        }
    }
}

/// Builder struct for [`STOPSIGNAL`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#stopsignal):
/// * `STOPSIGNAL <signal>`
///
/// [STOPSIGNAL]: dockerfile_builder::instruction::STOPSIGNAL
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = STOPSIGNAL,
    value_method = value,
)]
pub struct StopsignalBuilder {
    pub signal: String,
}

impl StopsignalBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(self.signal.to_string())
    }
}

/// Builder struct for [`HEALTHCHECK`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#healthcheck):
/// * `HEALTHCHECK [--interval=DURATION] [--timeout=DURATION]
///                [--start-period=DURATION] [--retries=N] CMD <command>`
///
/// [HEALTHCHECK]: dockerfile_builder::instruction::HEALTHCHECK
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = HEALTHCHECK,
    value_method = value,
)]
pub struct HealthcheckBuilder {
    pub cmd: CMD,
    pub interval: Option<i32>,
    pub timeout: Option<i32>,
    pub start_period: Option<i32>,
    pub retries: Option<i32>,
}

impl HealthcheckBuilder {
    fn value(&self) -> Result<String, String> {
        Ok(format!(
            "{}{}{}{}{}",
            self.interval
                .as_ref()
                .map(|i| format!("--interal={} ", i))
                .unwrap_or_default(),
            self.timeout
                .as_ref()
                .map(|t| format!("--timeout={} ", t))
                .unwrap_or_default(),
            self.start_period
                .as_ref()
                .map(|s| format!("--start-period={} ", s))
                .unwrap_or_default(),
            self.retries
                .as_ref()
                .map(|r| format!("--retries={} ", r))
                .unwrap_or_default(),
            self.cmd,
        ))
    }
}

/// Builder struct for [`SHELL`] instruction
///
/// Format according to [Dockerfile
/// reference](https://docs.docker.com/engine/reference/builder/#shell):
/// * `SHELL ["executable", "params"]`
///
/// Example:
/// ```
/// # use dockerfile_builder::instruction_builder::ShellBuilder;
/// // build SHELL with params
/// let shell = ShellBuilder::builder()
///     .executable("cmd")
///     .param("/S")
///     .param("/C")
///     .build().unwrap();
/// assert_eq!(shell.to_string(), r#"SHELL ["cmd", "/S", "/C"]"#);
///
/// // build SHELL with a param vec
/// let shell = ShellBuilder::builder()
///     .executable("cmd")
///     .params(vec!["/S", "/C"])
///     .build().unwrap();
/// assert_eq!(shell.to_string(), r#"SHELL ["cmd", "/S", "/C"]"#);
/// ```
///
/// [SHELL]: dockerfile_builder::instruction::SHELL
#[derive(Debug, InstructionBuilder)]
#[instruction_builder(
    instruction_name = SHELL,
    value_method = value,
)]
pub struct ShellBuilder {
    pub executable: String,
    #[instruction_builder(each = param)]
    pub params: Option<Vec<String>>,
}

impl ShellBuilder {
    fn value(&self) -> Result<String, String> {
        let params = match self.params.clone() {
            Some(param_vec) => {
                format!(r#", "{}""#, param_vec.join(r#"", ""#))
            }
            None => String::new(),
        };
        Ok(format!(r#"["{}"{}]"#, self.executable, params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn from() {
        let from = FromBuilder::builder().image("cargo-chef").build().unwrap();
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
        let from = FromBuilder::builder().build();
        match from {
            Ok(_) => panic!("Required field is not set. Expect test to fail"),
            Err(e) => assert_eq!(
                e.to_string(),
                "image is required for FromBuilder".to_string(),
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
    fn expose() {
        let expose = ExposeBuilder::builder().port(80).build().unwrap();
        let expected = expect!["EXPOSE 80"];
        expected.assert_eq(&expose.to_string());
    }

    #[test]
    fn add() {
        let add = AddBuilder::builder()
            .src("hom*")
            .dest("/mydir/")
            .build()
            .unwrap();
        let expected = expect!["ADD hom* /mydir/"];
        expected.assert_eq(&add.to_string());
    }

    #[test]
    fn add_http() {
        let add = AddHttpBuilder::builder()
            .checksum("sha256::123")
            .src("http://example.com/foobar")
            .dest("/")
            .build()
            .unwrap();
        let expected = expect!["ADD --checksum=sha256::123 http://example.com/foobar /"];
        expected.assert_eq(&add.to_string());
    }

    #[test]
    fn add_git() {
        let add = AddGitBuilder::builder()
            .keep_git_dir(true)
            .git_ref("https://github.com/moby/buildkit.git#v0.10.1")
            .dir("/buildkit")
            .build()
            .unwrap();
        let expected = expect![
            "ADD --keep-git-dir=true https://github.com/moby/buildkit.git#v0.10.1 /buildkit"
        ];
        expected.assert_eq(&add.to_string());
    }

    #[test]
    fn copy() {
        let copy = CopyBuilder::builder()
            .link(true)
            .src("foo/")
            .dest("bar/")
            .build()
            .unwrap();
        let expected = expect!["COPY --link foo/ bar/"];
        expected.assert_eq(&copy.to_string());
    }

    #[test]
    fn volume() {
        let volume = VolumeBuilder::builder()
            .path("/myvol1")
            .path("/myvol2")
            .build()
            .unwrap();
        let expected = expect!["VOLUME /myvol1 /myvol2"];
        expected.assert_eq(&volume.to_string());
    }

    #[test]
    fn user() {
        let user = UserBuilder::builder().user("myuser").build().unwrap();
        let expected = expect!["USER myuser"];
        expected.assert_eq(&user.to_string());

        let user = UserBuilder::builder()
            .user("myuser")
            .group("mygroup")
            .build()
            .unwrap();
        let expected = expect!["USER myuser:mygroup"];
        expected.assert_eq(&user.to_string());
    }

    #[test]
    fn workdir() {
        let workdir = WorkdirBuilder::builder()
            .path("/path/to/workdir")
            .build()
            .unwrap();
        let expected = expect!["WORKDIR /path/to/workdir"];
        expected.assert_eq(&workdir.to_string());
    }

    #[test]
    fn arg() {
        let arg = ArgBuilder::builder().name("user1").build().unwrap();
        let expected = expect!["ARG user1"];
        expected.assert_eq(&arg.to_string());

        let arg = ArgBuilder::builder()
            .name("user1")
            .value("someuser")
            .build()
            .unwrap();
        let expected = expect!["ARG user1=someuser"];
        expected.assert_eq(&arg.to_string());
    }

    #[test]
    fn onbuild() {
        let onbuild = OnbuildBuilder::builder()
            .instruction(Instruction::ADD(ADD::from(". /app/src")))
            .build()
            .unwrap();
        let expected = expect!["ONBUILD ADD . /app/src"];
        expected.assert_eq(&onbuild.to_string());
    }

    #[test]
    fn onbuild_err() {
        let onbuild = OnbuildBuilder::builder()
            .instruction(Instruction::ONBUILD(ONBUILD::from("RUN somecommand")))
            .build();
        match onbuild {
            Ok(_) => panic!("Chaining Onbuild instructions. Expect test to fail"),
            Err(e) => assert_eq!(
                e.to_string(),
                "Chaining ONBUILD instructions using ONBUILD ONBUILD isn’t allowed".to_string(),
            ),
        }

        let onbuild = OnbuildBuilder::builder()
            .instruction(Instruction::FROM(FROM::from("someimage")))
            .build();
        match onbuild {
            Ok(_) => {
                panic!("ONBUILD instruction may not trigger FROM instruction. Expect test to fail")
            }
            Err(e) => assert_eq!(
                e.to_string(),
                "ONBUILD instruction may not trigger FROM instruction".to_string(),
            ),
        }
    }

    #[test]
    fn stopsignal() {
        let stopsignal = StopsignalBuilder::builder()
            .signal("SIGKILL")
            .build()
            .unwrap();
        let expected = expect!["STOPSIGNAL SIGKILL"];
        expected.assert_eq(&stopsignal.to_string());
    }

    #[test]
    fn healthcheck() {
        let healthcheck = HealthcheckBuilder::builder()
            .cmd(CMD::from("curl -f http://localhost/"))
            .build()
            .unwrap();
        let expected = expect!["HEALTHCHECK CMD curl -f http://localhost/"];
        expected.assert_eq(&healthcheck.to_string());

        let healthcheck = HealthcheckBuilder::builder()
            .cmd(CMD::from("curl -f http://localhost/"))
            .interval(15)
            .timeout(200)
            .start_period(5)
            .retries(5)
            .build()
            .unwrap();
        let expected = expect!["HEALTHCHECK --interal=15 --timeout=200 --start-period=5 --retries=5 CMD curl -f http://localhost/"];
        expected.assert_eq(&healthcheck.to_string());
    }
}
