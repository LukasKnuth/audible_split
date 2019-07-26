extern crate regex;
extern crate rayon;
use rayon::prelude::*;
#[macro_use] extern crate log;

mod ffmpeg;
use ffmpeg::FFMPEG;
mod ffprobe;
use ffprobe::FFPROBE;
use std::fmt;
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;
use std::io;
use std::convert::From;

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
    IOError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CliError::FfmpegNotFound => writeln!(f, "CliError: Couldn't find ffmpeg in path"),
            CliError::FfprobeNotFound => writeln!(f, "CliError: Couldn't find ffprobe in path"),
            CliError::InvalidVersion(version) => writeln!(f, "CliError: Version {} not supported", version),
            CliError::IOError(e) => writeln!(f, "IO Error: {}", e),
        }
    }
}

impl Error for CliError {}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        // io::Error doesn't implement Clone, so we can't store it directly.
        let dump = format!("{}", error);
        CliError::IOError(dump)
    }
}

type CliResult = Result<(), CliError>;

/// Checks if all external tools are available and with the required version.
fn check() -> CliResult {
    let has_ffprobe = FFPROBE::is_installed();
    let has_ffmpeg = FFMPEG::is_installed();

    if let Some(version) = has_ffprobe {
        info!("ffprobe version {} found", version);
    } else {
        return Err(CliError::FfprobeNotFound);
    }
    if let Some(version) = has_ffmpeg {
        info!("ffmpeg version {} found", version);
        return Ok(());
    } else {
        return Err(CliError::FfmpegNotFound);
    }
}

/// Run the actual tool to transcode all chapters from the given `input_file` into individual
///  MP3 files in the `output_folder`. Use the given `activation_bytes` to decrypt the Audbile
///  AAX file.
pub fn run(input_file: String, output_folder: String, activation_bytes: String) -> CliResult {
    check()?;
    
    let result = FFPROBE::execute(&input_file)?;

    result.chapters.par_iter().map(|chapter| {
        ffmpeg::FfmpegOptions {
            activation_bytes: &activation_bytes,
            start: &chapter.start, end: &chapter.end,
            title: &chapter.tags["title"],
            track_nr: chapter.track_nr + 1,
            input_file: PathBuf::from(&input_file),
            output_folder: PathBuf::from(&output_folder) // todo make sure this exists and is empty!
        }
    }).filter(|option| {
        let exists = option.output_exists();
        if exists {
            warn!("Chapter {}, file \"{}\" already exists. Skipping...", 
                option.track_nr, option.output_file()
            );
        }
        !exists
    }).map(|option| {
        info!("Chapter {} starting transcoding", option.track_nr);
        let start = Instant::now();
        let result = FFMPEG::execute(&option);
        match &result {
            Ok(_) => info!("Chapter {} done (took {}s)", option.track_nr, start.elapsed().as_secs()),
            Err(e) => error!("Chapter {} errored: {}", option.track_nr, e)
        }
        result
    }).reduce(|| Ok(()), |acc, result| {
        // reduce an overall success-state. Individual failures are printed previously.
        match &acc {
            Err(_) => acc,
            Ok(_) => result
        }
    })?; // Using ? converts the io::Error to a CliError

    Ok(())
}