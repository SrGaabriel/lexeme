use std::io::{BufWriter, Write};

use clap::{Args, Parser};
use lexeme::{
    dict::{self, installed_languages},
    style::{self, Tone, plural},
    util::LenRange,
};
use regex::Regex;

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
    pub regex: Option<String>,
    #[clap(short, long)]
    pub languages: Option<Vec<String>>,
    #[clap(short, long)]
    pub syllables: Option<LenRange>,
    #[clap(long)]
    pub length: Option<LenRange>,
    #[clap(short, long, default_value = "true")]
    pub words_only: bool,
}

#[derive(Debug, Parser)]
pub enum Command {
    Install { language: String },
}

fn main() -> lexeme::Result<()> {
    let args = Cli::parse();
    let result = match args.command {
        Some(Command::Install { language }) => dict::install(language),
        None => run(args.query),
    };

    if let Err(err) = &result {
        style::error(err);
    }
    result
}

pub fn run(query: Query) -> lexeme::Result<()> {
    let languages = match query.languages {
        Some(languages) => languages,
        None => installed_languages()?,
    };
    let regex = query.regex.as_deref().map(Regex::new).transpose()?;

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let mut found = 0usize;
    let mut words_total = 0u64;
    let mut last = String::new();

    for language in &languages {
        let mut reader = dict::read(language)?;
        words_total += reader.meta().entries;
        while let Some(entry) = reader.next_entry()? {
            let word = entry.word;
            if let Some(length) = query.length
                && !length.contains(word.chars().count() as u8)
            {
                continue;
            }
            if query.words_only && !word.chars().all(char::is_alphanumeric) {
                continue;
            }
            if let Some(regex) = &regex
                && !regex.is_match(word)
            {
                continue;
            }
            if word == last {
                continue;
            }
            last.clear();
            last.push_str(word);
            writeln!(out, "{word}")?;
            found += 1;
        }
    }
    out.flush()?;
    drop(out);

    println!();
    style::status_meta(
        Tone::Success,
        "end",
        format!("found {found} words"),
        format!(
            "{} {}",
            plural(languages.len(), "language"),
            plural(words_total as usize, "word")
        ),
    );
    Ok(())
}
