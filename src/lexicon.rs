use std::io::{self, BufRead, Write};

use crate::{Error, Result};

pub const MAGIC: &str = "#lexeme-lexicon";
pub const VERSION: u32 = 1;

pub const FORM_SEP: char = '\u{1f}';
pub const TAG_SEP: char = ':';

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub lang: String,
    pub name: String,
    pub source: String,
    pub generated: String,
    pub license: String,
    pub entries: u64,
    pub forms: u64,
}

impl Meta {
    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "{MAGIC}\tv{VERSION}")?;
        writeln!(w, "#lang\t{}", self.lang)?;
        writeln!(w, "#name\t{}", self.name)?;
        writeln!(w, "#source\t{}", self.source)?;
        writeln!(w, "#generated\t{}", self.generated)?;
        writeln!(w, "#license\t{}", self.license)?;
        writeln!(w, "#entries\t{}", self.entries)?;
        writeln!(w, "#forms\t{}", self.forms)?;
        Ok(())
    }

    pub fn apply_line(&mut self, line: &str) {
        let Some(rest) = line.strip_prefix('#') else {
            return;
        };
        let Some((key, value)) = rest.split_once('\t') else {
            return;
        };
        match key {
            "lang" => self.lang = value.to_owned(),
            "name" => self.name = value.to_owned(),
            "source" => self.source = value.to_owned(),
            "generated" => self.generated = value.to_owned(),
            "license" => self.license = value.to_owned(),
            "entries" => self.entries = value.parse().unwrap_or(0),
            "forms" => self.forms = value.parse().unwrap_or(0),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Form<'a> {
    pub surface: &'a str,
    pub tags: &'a str,
}

impl<'a> Form<'a> {
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.split(',').any(|t| t == tag)
    }

    pub fn tags(&self) -> impl Iterator<Item = &'a str> {
        self.tags.split(',').filter(|t| !t.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct Entry<'a> {
    pub word: &'a str,
    pub pos: &'a str,
    pub syllables: u8,
    pub ipa: &'a str,
    forms: &'a str,
}

impl<'a> Entry<'a> {
    pub fn forms(&self) -> impl Iterator<Item = Form<'a>> {
        self.forms
            .split(FORM_SEP)
            .filter(|f| !f.is_empty())
            .map(|f| match f.split_once(TAG_SEP) {
                Some((tags, surface)) => Form { surface, tags },
                None => Form {
                    surface: f,
                    tags: "",
                },
            })
    }

    pub fn surfaces(&self) -> impl Iterator<Item = &'a str> {
        std::iter::once(self.word).chain(self.forms().map(|f| f.surface))
    }
}

pub fn parse_entry(line: &str) -> Option<Entry<'_>> {
    if line.is_empty() {
        return None;
    }
    let mut cols = line.split('\t');
    let word = cols.next()?;
    let pos = cols.next()?;
    let syllables = cols.next()?.parse().ok()?;
    let ipa = cols.next()?;
    let forms = cols.next().unwrap_or("");
    if word.is_empty() {
        return None;
    }
    Some(Entry {
        word,
        pos,
        syllables,
        ipa,
        forms,
    })
}

pub struct Reader<R> {
    inner: R,
    meta: Meta,
    line: String,
    pending: bool,
    line_number: u64,
}

impl<R: BufRead> Reader<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let mut meta = Meta::default();
        let mut line = String::new();
        let mut line_number = 0u64;
        let mut version = None;
        let mut pending = false;

        while inner.fill_buf()?.first() == Some(&b'#') {
            line.clear();
            if inner.read_line(&mut line)? == 0 {
                break;
            }
            line_number += 1;
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if let Some(rest) = trimmed.strip_prefix(MAGIC) {
                version = Some(parse_version(rest)?);
            } else if is_header_line(trimmed) {
                meta.apply_line(trimmed);
            } else {
                pending = true;
                break;
            }
        }

        let version =
            version.ok_or_else(|| io::Error::other("not a lexicon: missing magic header"))?;
        if version != VERSION {
            return Err(Error::LexiconVersionMismatch(VERSION, version));
        }

        Ok(Self {
            inner,
            meta,
            line,
            pending,
            line_number,
        })
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    pub fn line_number(&self) -> u64 {
        self.line_number
    }

    pub fn next_entry(&mut self) -> io::Result<Option<Entry<'_>>> {
        let len = if std::mem::take(&mut self.pending) {
            self.line.trim_end_matches(['\r', '\n']).len()
        } else {
            loop {
                self.line.clear();
                if self.inner.read_line(&mut self.line)? == 0 {
                    return Ok(None);
                }
                self.line_number += 1;
                let len = self.line.trim_end_matches(['\r', '\n']).len();
                if len > 0 {
                    break len;
                }
            }
        };

        let line_number = self.line_number;
        parse_entry(&self.line[..len])
            .map(Some)
            .ok_or_else(move || io::Error::other(format!("malformed entry on line {line_number}")))
    }
}

fn is_header_line(line: &str) -> bool {
    line.bytes().filter(|b| *b == b'\t').count() == 1
}

fn parse_version(rest: &str) -> io::Result<u32> {
    rest.trim()
        .strip_prefix('v')
        .and_then(|digits| digits.parse().ok())
        .ok_or_else(|| io::Error::other(format!("unrecognized lexicon magic: {MAGIC}{rest}")))
}

pub fn write_entry<W: Write>(
    w: &mut W,
    word: &str,
    pos: &str,
    syllables: u8,
    ipa: &str,
    forms: &[(String, String)],
) -> io::Result<()> {
    write!(w, "{word}\t{pos}\t{syllables}\t{ipa}\t")?;
    for (i, (surface, tags)) in forms.iter().enumerate() {
        if i > 0 {
            write!(w, "{FORM_SEP}")?;
        }
        write!(w, "{tags}{TAG_SEP}{surface}")?;
    }
    writeln!(w)
}
