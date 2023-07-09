This library provides a convenient way to programmatically generate Dockerfiles using Rust.

Dockerfiles instructions can be generated using structured and type-safe interfaces, or they can be added flexibly in raw form.

This library is actively developed and will be published to crates.io once features are complete.

# Quickstart

```toml
# Cargo.toml
[dependencies]
dockerfile_builder = { git = "https://github.com/ptmphuong/dockerfile-builder" }
```

```rust
// src/main.rs 
use dockerfile_builder::Dockerfile;
use dockerfile_builder::instruction::{RUN, EXPOSE};

fn main() {
    let dockerfile = Dockerfile::default()
        .push(RUN::from("echo $HOME"))
        .push(EXPOSE::from("80/tcp"))
        .push_any("# Just adding a comment");
    
    let expected = r#"RUN echo $HOME
EXPOSE 80/tcp
# Just adding a comment"#;

    assert_eq!(
        dockerfile.to_string(),
        expected
    );
}
```

# Type-safe support

Dockerfile instructions can be created from a string or with instruction builders.
Instruction builders provide structured and type-safe interfaces to build instructions.

```rust
// src/main.rs 
use dockerfile_builder::Dockerfile;
use dockerfile_builder::instruction::EXPOSE;
use dockerfile_builder::instruction_builder::ExposeBuilder;

fn main() {
    let expose = EXPOSE::from("80/tcp");
    
    let expose_from_builder = ExposeBuilder::builder()
        .port(80)
        .protocol("tcp")
        .build()
        .unwrap();
    
    assert_eq!(expose, expose_from_builder);
    
    let dockerfile = Dockerfile::default()
        .push(expose_from_builder);
      
    assert_eq!(
        dockerfile.to_string(), 
        "EXPOSE 80/tcp"
    );
}
```


