use std::io::{BufWriter, Write};

use clap::{Args, Parser};
use lexeme::{
    dict::{self, installed_languages},
    style::{self, Tone, new_pager, plural},
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
    #[arg(long, global = true, default_value = "false")]
    no_pager: bool,
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
    #[clap(subcommand)]
    Lang(Languages),
}

#[derive(Debug, Parser)]
pub enum Languages {
    Add { language: String },
    List,
    Remove { language: String },
}

fn main() {
    let args = Cli::parse();
    let result = match args.command {
        Some(Command::Lang(Languages::Add { language })) => add_language(&language),
        Some(Command::Lang(Languages::List)) => list_languages(!args.no_pager),
        Some(Command::Lang(Languages::Remove { language })) => remove_language(&language),
        None => run(args.query, !args.no_pager),
    };

    if let Err(err) = &result {
        style::error(err);
    }
}

pub fn run(query: Query, pager: bool) -> lexeme::Result<()> {
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

    let pager = new_pager().filter(|_| pager);
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
            if query.words_only && word.chars().any(|c| !c.is_alphanumeric()) {
                continue;
            }
            if let Some(syllables) = query.syllables
                && (entry.syllables == 0 || !syllables.contains(entry.syllables))
            {
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
    drop(pager);
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

pub fn add_language(language: &str) -> lexeme::Result<()> {
    let connecting = style::spinner(format!("resolving {language}"));
    let response = dict::resolve(language)?;
    connecting.finish_and_clear();

    let total = response.content_length();
    let download_bar = style::progress_bar(total, format!("downloading {language}"));
    let mut reader = download_bar.wrap_read(response);

    dict::download(language, &mut reader)?;

    download_bar.finish_and_clear();
    Ok(())
}

fn list_languages(pager: bool) -> lexeme::Result<()> {
    style::status(Tone::Accent, "list", "installed languages:");
    let pager = new_pager().filter(|_| pager);
    let languages = dict::installed_languages()?;
    println!();
    for language in languages {
        println!("• {}", language);
    }
    drop(pager);
    Ok(())
}

pub fn remove_language(language: &str) -> lexeme::Result<()> {
    dict::remove(language)?;
    style::success(format!("removed {language} language"));
    Ok(())
}
