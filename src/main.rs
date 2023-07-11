//! These docs are really for `envr` maintainers. If you're curious how to use the built binary,
//! refer to `envr --help` for more details.

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(warn(
    bare_trait_objects,
    clippy::cargo,
    clippy::pedantic,
    elided_lifetimes_in_paths,
    missing_copy_implementations,
    single_use_lifetimes,
))))]
#![warn(
    bare_trait_objects,
    clippy::cargo,
    clippy::pedantic,
    elided_lifetimes_in_paths,
    missing_copy_implementations,
    missing_docs,
    single_use_lifetimes,
    unused_extern_crates
)]

use {
    clap::Parser,
    std::{
        ffi::OsString,
        process::{exit, Command},
    },
};

#[derive(Debug, Parser)]
#[clap(about, author)]
struct Cli {
    /// Do not inherit environment variables from the parent process.
    #[clap(short, long)]
    ignore_environment: bool,
    #[clap(value_parser = Self::parse_variable)]
    variables: Vec<(String, String)>,
    /// The command to run and its arguments, if any.
    #[clap(raw(true))]
    command_and_args: Vec<OsString>,
}

impl Cli {
    fn parse_variable(s: &str) -> Result<(String, String), String> {
        let mut split_iter = s.splitn(2, '=');
        let key = split_iter.next().unwrap();
        split_iter
            .next()
            .ok_or_else(|| format!("{:?} is not a equals-sign-separated key-value pair", s))
            .map(|value| (key.to_owned(), value.to_owned()))
    }
}

#[allow(missing_docs)]
pub fn main() {
    let Cli {
        ignore_environment,
        variables,
        mut command_and_args,
    } = Cli::parse();

    if command_and_args.is_empty() {
        eprintln!("fatal: command not specified");
        exit(101);
    }

    let mut command = Command::new(command_and_args.remove(0));
    command.args(command_and_args);
    if ignore_environment {
        command.env_clear();
    }
    command.envs(variables);

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("fatal: command {:?} failed to spawn: {}", command, e);
            exit(102); // TODO: document error codes
        }
    };

    let _keep_child_stdin_intact = child.stdin.take();
    if let Err(e) = child.wait() {
        eprintln!("fatal: unable to wait for command to complete: {}", e);
        exit(103); // TODO: document error codes
    }
}

#[test]
fn variable_parsing() {
    assert_eq!(
        Cli::parse_variable("RUST_LOG=module=trace"),
        Ok(("RUST_LOG".to_owned(), "module=trace".to_owned())),
    );
}
