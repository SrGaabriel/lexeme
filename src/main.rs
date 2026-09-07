use clap::Parser;
use lexeme::util::Range;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short, long, default_value = "en")]
    pub language: String,
    #[clap(short, long)]
    pub regex: Option<String>,
    #[clap(short, long)]
    pub syllables: Option<Range<u32>>,
}

fn main() {
    let args = Cli::parse();
    println!("args: {:?}", args);
}
