use clap::{Parser, Subcommand};
use pgen::{fd, gen};
use std::{error::Error, path::PathBuf};

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate project from template file
    Gen {
        /// Path where project will be generated
        root: PathBuf,

        /// Path to template that will be used to generate project
        #[arg(long)]
        template: PathBuf,

        /// Path to file defining variables in template
        #[arg(long)]
        definitions: PathBuf,
    },

    /// Generate template file from directory
    Fd {
        /// Directory from which template will be generated
        directory: PathBuf,

        /// Path where template will be written
        #[arg(short, long)]
        output: PathBuf,

        /// Overwrite output if it already exists
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Gen {
            root,
            template,
            definitions,
        }) => Ok(gen(root, template, definitions)?),
        Some(Commands::Fd {
            directory,
            output,
            force,
        }) => Ok(fd(directory, output, *force)?),
        None => Ok(()),
    }
}
