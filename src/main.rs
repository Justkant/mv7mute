use clap::Parser;
use mv7mute::{run, Command};

#[derive(Parser)]
#[command(name = "mv7mute", about = "Toggle mute on the Shure MV7")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() {
    match run(Cli::parse().command.unwrap_or(Command::Toggle)) {
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
