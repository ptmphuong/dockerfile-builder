This library provides a convenient way to programmatically generate Dockerfiles using Rust.

Dockerfiles instructions can be generated using structured and type-safe interfaces, or they can be added flexibly in raw form.

Example:

Quickstart:

```rust
use dockerfile_builder::DockerFile;
use dockerfile_builder::instruction::{Run, Arg};

let dockerfile = Dockerfile::default()
    .push(Run::from("echo $HOME"))
    .push(Expose::from("80/tcp"))
    .push_any("# Just adding a comment");

let expected = expect![[r#"
    RUN echo $HOME
    EXPOSE 80/tcp
    # Just adding a comment"#]];
expected.assert_eq(&dockerfile.to_string());`
```

Dockerfile instructions can be created from a string or with instruction builders.
Instruction builders provide structured and type-safe interfaces to build instructions.

```rust
use dockerfile_builder::DockerFile;
use dockerfile_builder::{instruction::Expose, instruction_builder::ExposeBuilder};

let expose = Expose::from("80/tcp");

let expose_from_builder = ExposeBuilder::builder()
    .port(80)
    .proto("tcp")
    .build()
    .unwrap();

assert_eq!(expose, expose_from_builder);

let dockerfile = Dockerfile::default()
    .push(expose_from_builder);

let expected = expect!["EXPOSE 80/tcp"];
expected.assert_eq(&dockerfile.to_string());
```


