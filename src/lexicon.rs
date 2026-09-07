use std::io::{self, Write};

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

#[derive(Debug, Clone, Copy)]
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
    if line.is_empty() || line.starts_with('#') {
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
