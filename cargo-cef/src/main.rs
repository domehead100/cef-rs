use crate::Commands::{Build, Open, Run};
use clap::{Parser, Subcommand};
// use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};

// If the value of a given argument is intended to dictate the type and number of arguments that come
// after that argument, that is a subcommand.  So that's when you would make an enum that implements
// the subcommand trait, and each enum value would reference a struct that implements the arg trait
// with the fields that are expected after that subcommand.

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// TODO: add a "dev" command that will build a js application with vite and launch it
#[derive(Subcommand, Debug)]
enum Commands {
    /// Build a target from the examples folder, default to cefsimple example if no target is provided
    Build(CommandArgs),
    /// Build and run a target from the examples folder, default to cefsimple if no target is provided
    Run(CommandArgs),
    /// Open a target from the examples folder, default to cefsimple if no target is provided
    Open(CommandArgs),
}

#[derive(Parser, Debug)]
struct CommandArgs {
    #[arg(default_value = "cefsimple")]
    target: Option<String>,
    #[arg(short, long)]
    release: bool,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    other_args: Vec<String>,
}

// fn run_command(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
//     let status = Command::new("cargo")
//         .args(args)
//         .stdout(Stdio::inherit())
//         .stderr(Stdio::inherit())
//         .status()?;

//     if !status.success() {
//         std::process::exit(1);
//     }
//     Ok(())
// }

// build:
// cargo run --bin bundle_cefsimple [--release]

// run:
// target/debug/cefsimple.app/Contents/MacOS/cefsimple [--release]

// open:
// open target/debug/cefsimple.app

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // let line = match cli.command {
    //     Build(args) => [
    //         "build",
    //         "-p",
    //         &args.target.unwrap_or("bundle_cefsimple".into()),
    //     ],
    //     Run(args) => ["run", "-p", &args.target.unwrap_or("cefsimple".into())],
    //     Open(args) => ["build", "-p", &args.target.unwrap_or("cefsimple".into())],
    // };

    // run_command(&line)?;

    match cli.command {
        Build(args) => {
            let exe = if cfg!(target_os = "macos") {
                "bundle_cefsimple"
            } else {
                "cefsimple"
            };

            println!("cargo:info=Building {}", exe);

            let status = Command::new("cargo")
                .args(["build", "-p", &args.target.unwrap_or(exe.into())])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;

            if !status.success() {
                std::process::exit(1);
            }
        }
        Run(_args) => {
            let status = Command::new("target/debug/cefsimple.app/Contents/MacOS/cefsimple")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;

            if !status.success() {
                std::process::exit(1);
            }
        }
        Open(_args) => {
            let _cmd = Command::new("open")
                .args(["target/debug/cefsimple.app"])
                .spawn()?;
        }
    }

    Ok(())
}
