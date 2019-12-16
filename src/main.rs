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
    itertools::Itertools,
    std::process::{exit, Command},
    structopt::StructOpt,
};

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short, long, help = "start with an empty environment")]
    ignore_environment: bool,
    #[structopt(help = "", parse(try_from_str = "Self::parse_variable"))]
    variables: Vec<(String, String)>,
    #[structopt(
        help = "the command to run and optionally its arguments",
        raw(true),
    )]
    command_and_args: Vec<String>,
}

impl Cli {
    fn parse_variable(s: &str) -> Result<(String, String), String> {
        s.split('=')
            .map(ToOwned::to_owned)
            .collect_tuple()
            .ok_or_else(|| format!("{:?} is not a valid key-value pair", s))
    }
}

#[allow(missing_docs)]
pub fn main() {
    let Cli {
        ignore_environment,
        variables,
        mut command_and_args,
    } = Cli::from_args();

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
            exit(102);
        }
    };

    let _keep_child_stdin_intact = child.stdin.take();
    if let Err(e) = child.wait() {
        eprintln!("fatal: unable to wait for command to complete: {}", e);
        exit(103);
    }
}
