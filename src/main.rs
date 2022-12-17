mod error;
mod ui;

use std::collections::HashSet;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use error::{exit, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub const ALPHABETS: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

const DEFAULT_WORDS: &[u8] = include_bytes!("../data/words.json");
const DEFAULT_ALLOWED_GUESSES: &[u8] = include_bytes!("../data/allowed_guesses.json");

const ABOUT: &str = "wordle-cli (wrdl) is a terminal-based game of Wordle.";

const USAGE: &str = "[OPTIONS]";

const OPTIONS: &str = "
    -a, --allowed-guesses [path]    Specify path to allowed guesses file, leave blank to unset
    -h, --help                      Print help information
    -r, --reset                     Set the next word pointer to the beginning
    -V, --version                   Print version information
    -w, --words [path]              Specify path to allowed words file, leave blank to unset";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Data {
    #[serde(default)]
    index: usize,
    #[serde(default)]
    words_path: Option<PathBuf>,
    #[serde(default)]
    allowed_guesses_path: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct Spot {
    letter: char,
    status: LetterStatus,
}

impl Default for Spot {
    fn default() -> Self {
        Self {
            letter: '\0',
            status: LetterStatus::NotInWord,
        }
    }
}

impl Spot {
    fn correct(letter: char) -> Self {
        Self {
            letter,
            status: LetterStatus::Correct,
        }
    }

    fn incorrect(letter: char) -> Self {
        Self {
            letter,
            status: LetterStatus::Incorrect,
        }
    }

    fn not_in_word(letter: char) -> Self {
        Self {
            letter,
            status: LetterStatus::NotInWord,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
enum LetterStatus {
    Correct,
    Incorrect,
    NotInWord,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GuessResult {
    word: String,
    guesses: Vec<Vec<Spot>>,
    duration: Duration,
}

/// Parses json data as a deserializable object.
fn parse_words_data<T: DeserializeOwned>(words_data: &[u8]) -> Result<T> {
    serde_json::from_slice(words_data).map_err(|e| e.into())
}

/// Returns the path to the persistent data file for the app.
fn get_data_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("WORDLE_CLI_DATA") {
        Ok(PathBuf::from(&path))
    } else {
        dirs_next::data_dir()
            .map(|d| d.join("wordle-cli/data.json"))
            .ok_or_else(|| "unable to retrieve home directory path".into())
    }
}

/// Loads file at the given path into a Deserializable object,
/// returning error if it does not exist.
fn load_file<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> Result<T> {
    serde_json::from_str(&fs::read_to_string(path)?).map_err(|e| e.into())
}

/// Updates (or creates) the data file at the given path with the provided data.
fn update_or_create_data<P: AsRef<Path>>(data: Data, path: P) -> Result<Data> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    };

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    serde_json::to_writer_pretty(file, &data)?;

    Ok(data)
}

/// Reads the next argument and checks if it's a valid path.
fn get_and_verify_path(mut args: env::Args) -> Result<Option<PathBuf>> {
    if let Some(p) = args.next() {
        let path = PathBuf::from(p);
        if path.exists() {
            Ok(Some(path.canonicalize()?))
        } else {
            Err("path does not exist".into())
        }
    } else {
        Ok(None)
    }
}

/// Prints the app version.
fn print_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

/// Prints the help text.
fn print_help() -> Result<()> {
    let bufwtr = BufferWriter::stdout(ColorChoice::Auto);
    let mut buffer = bufwtr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;

    let bin_name = env!("CARGO_BIN_NAME");
    write!(&mut buffer, "{bin_name}")?;
    buffer.reset()?;

    writeln!(
        &mut buffer,
        " {}\n{}\n\n{ABOUT}\n",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    )?;

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    writeln!(&mut buffer, "USAGE:")?;
    buffer.reset()?;

    writeln!(&mut buffer, "    {bin_name} {USAGE}\n")?;

    buffer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    write!(&mut buffer, "OPTIONS:")?;
    buffer.reset()?;

    writeln!(&mut buffer, "{OPTIONS}")?;

    bufwtr.print(&buffer)?;

    Ok(())
}

/// Runs the app.
fn run() -> Result<()> {
    let data_path = get_data_path()?;
    let mut data =
        load_file(&data_path).or_else(|_| update_or_create_data(Data::default(), &data_path))?;

    let mut args = env::args();
    if let Some(arg) = args.nth(1) {
        match arg.as_str() {
            "-w" | "--words" => data.words_path = get_and_verify_path(args)?,
            "-a" | "--allowed-guesses" => data.allowed_guesses_path = get_and_verify_path(args)?,
            "-r" | "--reset" => data.index = 0,
            "-V" | "--version" => print_version(),
            "-h" | "--help" => print_help()?,
            _ => return Err("invalid argument".into()),
        }
        update_or_create_data(data, data_path)?;
        return Ok(());
    };

    let words: Vec<String> = if let Some(ref path) = data.words_path {
        load_file(path)
    } else {
        parse_words_data(DEFAULT_WORDS)
    }?;

    let word = words
        .get(data.index)
        .ok_or("all available words have been used")?
        .to_ascii_uppercase();

    let mut allowed_guesses: HashSet<String> = if let Some(ref path) = data.allowed_guesses_path {
        load_file(path)
    } else {
        parse_words_data(DEFAULT_ALLOWED_GUESSES)
    }?;
    allowed_guesses.extend(words.into_iter());

    ui::main(
        word,
        allowed_guesses
            .iter()
            .map(|w| w.to_ascii_uppercase())
            .collect(),
        data.index,
    )?;

    data.index += 1;
    update_or_create_data(data, data_path)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        exit(e, 1);
    }
}
