use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(long)]
    pub mount: Option<String>,

    #[command(subcommand)]
    pub subcommand: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Workspace {
        #[command(subcommand)]
        subcommand: Option<WorkspaceSubcommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceSubcommands {
    Create { name: String },
    List,
}
