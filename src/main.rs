mod mv7;

use clap::{Parser, Subcommand};
use mv7::Mv7;

#[derive(Parser)]
#[command(name = "mv7mute", about = "Toggle mute on the Shure MV7")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
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
}

fn main() {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Toggle);

    let mut mv7 = Mv7::open().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    mv7.init().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    match command {
        Command::Toggle => {
            let muted = mv7.toggle().unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            mv7.restore_lock()
                .unwrap_or_else(|e| eprintln!("Warning: {e}"));
            println!("{}", if muted { "muted" } else { "unmuted" });
        }
        Command::On => {
            mv7.set_mute(true).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            mv7.restore_lock()
                .unwrap_or_else(|e| eprintln!("Warning: {e}"));
            println!("muted");
        }
        Command::Off => {
            mv7.set_mute(false).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            mv7.restore_lock()
                .unwrap_or_else(|e| eprintln!("Warning: {e}"));
            println!("unmuted");
        }
        Command::Status => {
            let muted = mv7.get_mute().unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            let locked = mv7.was_locked();
            mv7.restore_lock()
                .unwrap_or_else(|e| eprintln!("Warning: {e}"));
            println!("{}", if muted { "muted" } else { "unmuted" });
            println!("{}", if locked { "locked" } else { "unlocked" });
        }
        Command::Lock => {
            mv7.set_lock(true).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            println!("locked");
        }
        Command::Unlock => {
            mv7.set_lock(false).unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            println!("unlocked");
        }
    }
}
