use std::io::{BufWriter, Write};

use clap::{ArgAction, Args, Parser, Subcommand};
use lexeme::{
    Error,
    dict::{self, installed_languages},
    style::{self, Tone, new_pager, plural},
    util::LenRange,
};
use regex::Regex;

const EXAMPLES: &str = "\
Examples:
  lexeme '^un.*ing$'             words beginning \"un\" and ending \"ing\"
  lexeme -l en,de '^haus'        search only English and German
  lexeme -s 3 '^pro'             three-syllable words beginning \"pro\"
  lexeme --length 5..=7 '^z'     words of 5 to 7 characters
  lexeme -w false '\\p{Emoji}'   include symbol and emoji entries

  lexeme lang add fi            install the Finnish dictionary
  lexeme lang list              show what is installed";

#[derive(Debug, Parser)]
#[command(name = "lexeme", version, after_help = EXAMPLES)]
pub struct Cli {
    #[command(flatten)]
    pub query: Query,

    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, global = true)]
    no_pager: bool,
}

#[derive(Args, Debug)]
pub struct Query {
    #[arg(value_name = "REGEX")]
    pub regex: String,

    #[arg(short, long, value_name = "CODE", value_delimiter = ',')]
    pub languages: Option<Vec<String>>,

    #[arg(short, long, value_name = "RANGE", verbatim_doc_comment)]
    pub syllables: Option<LenRange>,

    #[arg(long, value_name = "RANGE", verbatim_doc_comment)]
    pub length: Option<LenRange>,

    #[arg(
        short,
        long,
        value_name = "BOOL",
        default_value_t = true,
        action = ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
    )]
    pub words_only: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Lang(Languages),
}

#[derive(Debug, Subcommand)]
pub enum Languages {
    Add {
        #[arg(value_name = "CODE")]
        language: String,
    },
    List,
    Remove {
        #[arg(value_name = "CODE")]
        language: String,
    },
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
    if languages.is_empty() {
        return Err(Error::NoLanguagesInstalled);
    }
    let regex = Regex::new(&query.regex)?;

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

            if !regex.is_match(word) {
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
        println!("• {language}");
    }
    drop(pager);
    Ok(())
}

pub fn remove_language(language: &str) -> lexeme::Result<()> {
    dict::remove(language)?;
    style::success(format!("removed {language} language"));
    Ok(())
}
