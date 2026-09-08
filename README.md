# 📖 lexeme

Lexeme is a command-line tool for searching through dictionaries across 400+ languages.

## Instalation

To install the CLI tool, run:

```
cargo install lexeme
```

Afterwards, to download a language:

```
lexeme lang add <CODE>
```

So for adding english for example, run `lexeme lang add en`.

## Usage

Search a regex against every installed language:

```
lexeme '^un.*ing$'
```

The pattern is a [Rust regex](https://docs.rs/regex/latest/regex/#syntax) and is matched anywhere in the word, so anchor it with `^` and `$` when you want the whole thing.

### Options

| Option                    | Description                                                                            |
| ------------------------- | -------------------------------------------------------------------------------------- |
| `-l, --languages <CODE>`  | Only search these comma-separated languages (default: all installed)                   |
| `-s, --syllables <RANGE>` | Keep words with this many syllables (excludes obscure entries without syllable counts) |
| `--length <RANGE>`        | Keep words with this many characters                                                   |
| `-w, --words-only <BOOL>` | Exclude entries containing non-alphanumeric characters (default: true)                 |

A `<RANGE>` is either a single number or a Rust-style range like `5`, `3..6` (3 to 6 exclusive), `3..=6` (3 to 6 exclusive), `..6` (up to 6 exclusive), `3..` (from 3 onwards).

```
lexeme -l en,de '^haus'        search only English and German
lexeme -s 3 '^pro'             three-syllable words beginning "pro"
lexeme --length 5..=7 '^z'     words of 5 to 7 characters
```
