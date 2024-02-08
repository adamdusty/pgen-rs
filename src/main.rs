use clap::{Parser, Subcommand};
use pgen::cmd::gen::GenError::*;
use std::{error::Error, fs::remove_dir_all, io::ErrorKind, path::PathBuf};

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

fn generate(
    root: &PathBuf,
    template: &PathBuf,
    definitions: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    if root.exists() {
        return Err(Box::new(std::io::Error::from(
            std::io::ErrorKind::AlreadyExists,
        )));
    }

    match pgen::cmd::generate(root, template, definitions) {
        Ok(_) => {}
        Err(e) => {
            match &e {
                InvalidRenderedTemplateError => eprintln!("Rendered template is invalid."),
                IoError(e) => eprintln!("IoError: {}", e),
                SerializationError(e) => eprintln!("SerializationError: {}", e),
                WriteError => {
                    eprintln!("Error writing project. Removing root {:#?}", root);
                    remove_dir_all(root).expect("Error removing root");
                }
            }

            return Err(Box::new(e));
        }
    };

    return Ok(());
}

fn from_directory(
    directory: &PathBuf,
    output: &PathBuf,
    force: &bool,
) -> Result<(), Box<dyn Error>> {
    if output.exists() && !force {
        return Err(Box::new(std::io::Error::from(ErrorKind::AlreadyExists)));
    }

    return match pgen::cmd::fd(&directory, &output) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Gen {
            root,
            template,
            definitions,
        }) => generate(root, template, definitions),
        Some(Commands::Fd {
            directory,
            output,
            force,
        }) => from_directory(directory, output, force),
        None => Ok(()),
    }
}
