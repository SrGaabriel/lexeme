use clap::{Args, Parser};
use lexeme::{dict, style, util::Range};

#[derive(Debug, Parser)]
#[clap(name = "lexeme", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub query: Query,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Args, Debug)]
pub struct Query {
    #[clap(short, long)]
    pub languages: Vec<String>,
    #[clap(short, long)]
    pub regex: Option<String>,
    #[clap(short, long)]
    pub syllables: Option<Range<u32>>,
}

#[derive(Debug, Parser)]
pub enum Command {
    Install { language: String },
}

fn main() -> lexeme::Result<()> {
    let args = Cli::parse();
    let result = match args.command {
        Some(Command::Install { language }) => dict::install(language),
        None => {
            
            Ok(())
        }
    };

    if let Err(err) = &result {
        style::error(err);
    }
    result
}

pub struct Engine {
    languages 
}

impl Engine {
    
}