extern crate regex;

mod ffmpeg;
use ffmpeg::FFMPEG;
mod ffprobe;
use ffprobe::FFPROBE;
use std::fmt;
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

/// Describes a CLI tool wrapped for usage in the program.
pub trait CliTool<O, T, E> {

    /// Check that this tool is installed and available on the system. If so, this
    ///  function returns the `Some(String)` where the String contains the installed
    ///  version of the tool.
    fn is_installed() -> Option<String>;

    /// Executes this program with the given options `O` and returns a result of either
    ///  `Ok(T)` in case of success or `Err(E)` in case of failure.
    /// 
    /// # Panics
    /// 
    /// This function panics if not all necessary or incompatible options are passed.
    fn execute(options: O) -> Result<T, E>;
}

#[derive(Debug, Clone)]
pub enum CliError {
    FfmpegNotFound,
    FfprobeNotFound,
    InvalidVersion(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CliError::FfmpegNotFound => writeln!(f, "CliError: Couldn't find ffmpeg in path"),
            CliError::FfprobeNotFound => writeln!(f, "CliError: Couldn't find ffprobe in path"),
            CliError::InvalidVersion(version) => writeln!(f, "CliError: Version {} not supported", version)
        }
    }
}

impl Error for CliError {}

/// Checks if all external tools are available and with the required version.
fn check() -> Result<(), CliError> {
    let has_ffprobe = FFPROBE::is_installed();
    let has_ffmpeg = FFMPEG::is_installed();

    if let Some(version) = has_ffprobe {
        println!("ffprobe v. {} found", version);
    } else {
        return Err(CliError::FfprobeNotFound);
    }
    if let Some(version) = has_ffmpeg {
        println!("ffmpeg v. {} found", version);
        return Ok(());
    } else {
        return Err(CliError::FfmpegNotFound);
    }
}

pub fn run(input_file: String, output_folder: String, activation_bytes: String) -> i32 {
    if let Err(e) = check() {
        eprint!("{}", e);
        return 1;
    }

    let result = match FFPROBE::execute(&input_file) {
        Ok(result) => result,
        Err(e) => {
            eprint!("{}", e);
            return 2;
        }
    };

    for chapter in result.chapters.iter() {
        println!("Chapter {} from {} to {}", chapter.track_nr, chapter.start, chapter.end);
    }

    let options: Vec<ffmpeg::FfmpegOptions> = result.chapters.iter().map(|chapter| {
        ffmpeg::FfmpegOptions {
            activation_bytes: &activation_bytes,
            start: &chapter.start, end: &chapter.end,
            title: &chapter.tags["title"],
            track_nr: chapter.track_nr + 1,
            input_file: PathBuf::from(&input_file),
            output_folder: PathBuf::from(&output_folder) // todo make sure this exists and is empty!
        }
    }).collect();
    for option in options {
        let start = Instant::now();
        match FFMPEG::execute(&option) {
            Ok(_) => println!("Chapter {} done in {}s", option.track_nr, start.elapsed().as_secs()),
            Err(e) => {
                eprintln!("{}", e);
                return 3;
            }
        }
    }

    0
}