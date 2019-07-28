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
use std::path::{Path, PathBuf};
use std::io;
use std::convert::From;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Mutex;
use std::fs;

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

/// An error occurred while calling an external Command to do the actual processing.
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
    // Check for required tools
    check()?;
    
    // Find chapters in input-file:
    let result = FFPROBE::execute(&input_file)?;

    // Setup directory
    if !Path::new(&output_folder).exists() {
        fs::create_dir_all(&output_folder)?;
    }

    // Setup progress-bar for CLI.
    let progress = ProgressBar::new(result.chapters.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .progress_chars("=>-")
        .template("{prefix} [{wide_bar}] {pos}/{len} (took {elapsed})")
    );
    progress.enable_steady_tick(1000);
    let progress = Mutex::new(progress);

    // Start paralell processing via Rayon iterators:
    result.chapters.par_iter().take(3).map(|chapter| {
        ffmpeg::FfmpegOptions {
            activation_bytes: &activation_bytes,
            start: &chapter.start, end: &chapter.end,
            title: &chapter.tags["title"],
            track_nr: chapter.track_nr + 1,
            input_file: PathBuf::from(&input_file),
            output_folder: PathBuf::from(&output_folder)
        }
    }).filter(|option| {
        let exists = option.output_exists();
        if exists {
            let progress = progress.lock().unwrap();
            progress.inc(1);
            progress.println(&format!(
                " Skipping: Chapter {}, file \"{}\" already exists.", 
                option.track_nr, option.output_file()
            ));
        }
        !exists
    }).map(|option| {
        {
            let progress = progress.lock().unwrap();
            progress.println(&format!(
                " Transcoding: Chapter {} starting", option.track_nr
            ));   
        }
        let result = FFMPEG::execute(&option);
        {
            let progress = progress.lock().unwrap();
            progress.inc(1);
            match &result {
                Ok(_) => progress.println(format!(" Done: Chapter {}", option.track_nr)),
                Err(e) => progress.println(format!(" Error: Chapter {} errored: {}", option.track_nr, e))
            }
        }
        result
    }).reduce(|| Ok(()), |acc, result| {
        // reduce to an overall success-state. Individual failures are printed previously.
        match &acc {
            Err(_) => acc,
            Ok(_) => result
        }
    })?; // Using ? converts the io::Error to a CliError

    progress.lock().unwrap().finish_with_message("All done");

    Ok(())
}