use clap::{Parser, Subcommand};
use mv7mute_core::{Command, run};

#[derive(Parser)]
#[command(name = "mv7mute", about = "Toggle mute on the Shure MV7", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Clone, Copy, Subcommand)]
enum CliCommand {
    /// Toggle mute state (default)
    Toggle,
    /// Mute the microphone
    On,
    /// Unmute the microphone
    Off,
    /// Print current mute state
    Status,
    /// Lock the device
    Lock,
    /// Unlock the device
    Unlock,
    /// Print version information
    Version,
}

impl From<CliCommand> for Command {
    fn from(cmd: CliCommand) -> Self {
        match cmd {
            CliCommand::Toggle => Command::Toggle,
            CliCommand::On => Command::On,
            CliCommand::Off => Command::Off,
            CliCommand::Status => Command::Status,
            CliCommand::Lock => Command::Lock,
            CliCommand::Unlock => Command::Unlock,
            CliCommand::Version => Command::Version,
        }
    }
}

fn main() {
    match run(Cli::parse()
        .command
        .map(Command::from)
        .unwrap_or(Command::Toggle))
    {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}
