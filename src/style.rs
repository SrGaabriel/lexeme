#![allow(dead_code)]

use std::borrow::Cow;
use std::fmt::{self, Display};
use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
pub use owo_colors::Style;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,
    #[default]
    Normal,
    Verbose,
}

static VERBOSITY: AtomicU8 = AtomicU8::new(1);

static COLOR: AtomicU8 = AtomicU8::new(0);

pub fn init(color: ColorMode, verbosity: Verbosity) {
    VERBOSITY.store(verbosity as u8, Ordering::Relaxed);

    let enabled = match color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => detect_color(),
    };
    COLOR.store(if enabled { 2 } else { 1 }, Ordering::Relaxed);
}

fn detect_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}

pub fn verbosity() -> Verbosity {
    match VERBOSITY.load(Ordering::Relaxed) {
        0 => Verbosity::Quiet,
        2 => Verbosity::Verbose,
        _ => Verbosity::Normal,
    }
}

pub fn is_quiet() -> bool {
    verbosity() == Verbosity::Quiet
}

pub fn is_verbose() -> bool {
    verbosity() == Verbosity::Verbose
}

pub fn color_enabled() -> bool {
    match COLOR.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let enabled = detect_color();
            COLOR.store(if enabled { 2 } else { 1 }, Ordering::Relaxed);
            enabled
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Tone {
    Progress,
    Success,
    Note,
    Accent,
    Error,
    Warning,
}

impl Tone {
    pub fn style(self) -> Style {
        match self {
            Tone::Progress => Style::new().bold().cyan(),
            Tone::Success => Style::new().bold().green(),
            Tone::Note => Style::new().dimmed(),
            Tone::Accent => Style::new().cyan(),
            Tone::Error => Style::new().bold().red(),
            Tone::Warning => Style::new().bold().yellow(),
        }
    }
}

pub fn styled(text: impl Display, style: Style) -> String {
    Painter::auto().paint(text, style)
}

#[derive(Clone, Copy, Debug)]
pub struct Painter {
    enabled: bool,
}

impl Painter {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn auto() -> Self {
        Self::new(color_enabled())
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn paint(self, text: impl Display, style: Style) -> String {
        if self.enabled {
            text.style(style).to_string()
        } else {
            text.to_string()
        }
    }

    pub fn tone(self, text: impl Display, tone: Tone) -> String {
        self.paint(text, tone.style())
    }

    pub fn dim(self, text: impl Display) -> String {
        self.paint(text, Style::new().dimmed())
    }

    pub fn bold(self, text: impl Display) -> String {
        self.paint(text, Style::new().bold())
    }
}

pub fn paint(text: impl Display, tone: Tone) -> String {
    styled(text, tone.style())
}

pub fn dim(text: impl Display) -> String {
    styled(text, Style::new().dimmed())
}

pub fn bold(text: impl Display) -> String {
    styled(text, Style::new().bold())
}

pub fn format_status(tone: Tone, verb: &str, subject: impl Display) -> String {
    match tone {
        Tone::Error => styled(format!("{verb} {subject}"), Style::new().red()),
        _ => format!("{} {}", paint(verb, tone), dim(subject)),
    }
}

pub fn format_status_with_meta(
    tone: Tone,
    verb: &str,
    subject: impl Display,
    meta: impl Display,
) -> String {
    match tone {
        Tone::Error => styled(format!("{verb} {subject} {meta}"), Style::new().red()),
        _ => format!("{} {} {}", paint(verb, tone), dim(subject), dim(meta)),
    }
}

pub fn status(tone: Tone, verb: &str, subject: impl Display) {
    if is_quiet() {
        return;
    }
    println!("{}", format_status(tone, verb, subject));
    let _ = std::io::stdout().flush();
}

pub fn status_meta(tone: Tone, verb: &str, subject: impl Display, meta: impl Display) {
    if is_quiet() {
        return;
    }
    println!("{}", format_status_with_meta(tone, verb, subject, meta));
    let _ = std::io::stdout().flush();
}

pub fn success(msg: impl Display) {
    println!("{} {msg}", paint("success:", Tone::Success));
}

pub fn error(msg: impl Display) {
    eprintln!("{} {msg}", paint("error:", Tone::Error));
}

pub fn warning(msg: impl Display) {
    eprintln!("{} {msg}", paint("warning:", Tone::Warning));
}

pub fn spinner(message: impl Into<Cow<'static, str>>) -> ProgressBar {
    if is_quiet() || !std::io::stderr().is_terminal() {
        return ProgressBar::hidden();
    }
    let template = if color_enabled() {
        "{spinner:.cyan.bold} {msg}"
    } else {
        "{spinner} {msg}"
    };
    let style = ProgressStyle::with_template(template)
        .expect("spinner template is valid")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
    let bar = ProgressBar::new_spinner().with_style(style);
    bar.set_message(message);
    bar.enable_steady_tick(Duration::from_millis(80));
    bar
}

pub fn progress_bar(total: Option<u64>, message: impl Into<Cow<'static, str>>) -> ProgressBar {
    if is_quiet() || !std::io::stderr().is_terminal() {
        return ProgressBar::hidden();
    }
    let color = color_enabled();
    let bar = match total {
        Some(total) => {
            let template = if color {
                "{msg} {bar:28.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec} eta {eta}"
            } else {
                "{msg} {bar:28} {bytes}/{total_bytes} {bytes_per_sec} eta {eta}"
            };
            let style = ProgressStyle::with_template(template)
                .expect("progress template is valid")
                .progress_chars("━╸─");
            ProgressBar::new(total).with_style(style)
        }
        None => {
            let template = if color {
                "{spinner:.cyan.bold} {msg} {bytes} {bytes_per_sec}"
            } else {
                "{spinner} {msg} {bytes} {bytes_per_sec}"
            };
            let style = ProgressStyle::with_template(template)
                .expect("progress template is valid")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
            ProgressBar::new_spinner().with_style(style)
        }
    };
    bar.set_message(message);
    bar.enable_steady_tick(Duration::from_millis(80));
    bar
}

pub struct Hyperlink<'a> {
    url: String,
    text: &'a str,
}

impl<'a> Hyperlink<'a> {
    pub fn for_path(path: &'a std::path::Path) -> Hyperlink<'a> {
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let url = format!("file://{}", abs.display());
        Hyperlink {
            url,
            text: path_str(path),
        }
    }
}

impl Display for Hyperlink<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if color_enabled() && supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout) {
            write!(f, "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", self.url, self.text)
        } else {
            write!(f, "{}", self.text)
        }
    }
}

fn path_str(path: &std::path::Path) -> &str {
    path.to_str().unwrap_or("<non-utf8 path>")
}
