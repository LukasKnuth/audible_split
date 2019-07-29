extern crate regex;
extern crate rayon;
use rayon::prelude::*;
#[macro_use] extern crate log;
extern crate strfmt;
use strfmt::strfmt;

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
use std::collections::HashMap;
use regex::Regex;

/// Default format for output filenames.
pub const DEFAULT_FORMAT: &'static str = "{title}_{track_nr}.mp3";

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

pub struct RunParameters {
    pub input_file: String,
    pub output_folder: String,
    pub activation_bytes: String,
    pub quality: u8,
    pub output_format: String,
}
impl RunParameters {
    fn output_folder(&self) -> PathBuf {
        PathBuf::from(&self.output_folder)
    }

    fn input_file(&self) -> PathBuf {
        PathBuf::from(&self.input_file)
    }

    fn output_file(&self, title: &str, chapter_nr: u32, chapter_title: &str) -> PathBuf {
        let format = if self.output_format.ends_with(".mp3") {
            self.output_format.clone()
        } else {
            format!("{}.mp3", self.output_format)
        };

        let track = chapter_nr.to_string();
        let map: HashMap<String, &str> = [
            ("title".to_string(), title),
            ("track_nr".to_string(), &track),
            ("track_title".to_string(), chapter_title),
        ].iter().cloned().collect();

        let file_name = if let Ok(formated) = strfmt(&format, &map) {
            formated
        } else {
            warn!("Invalid --format option, falling back to default!");
            strfmt(DEFAULT_FORMAT, &map).unwrap()
        };
        let regex = Regex::new(r"[^0-9A-z.\-]").unwrap();
        let sanatized = regex.replace_all(&file_name, "_");
        self.output_folder().join(sanatized.to_string())
    }
}

/// Run the actual tool to transcode all chapters from the given `input_file` into individual
///  MP3 files in the `output_folder`. Use the given `activation_bytes` to decrypt the Audbile
///  AAX file.
pub fn run(params: RunParameters) -> CliResult {
    // Check for required tools
    check()?;
    
    // Find chapters in input-file:
    let result = FFPROBE::execute(&params.input_file)?;

    // Setup directory
    if !Path::new(&params.output_folder).exists() {
        fs::create_dir_all(&params.output_folder)?;
    }
    info!("Writing output to {}", params.output_folder);

    // Setup progress-bar for CLI.
    let progress = ProgressBar::new(result.chapters.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .progress_chars("=>-")
        .template("{prefix} [{wide_bar}] {pos}/{len} (took {elapsed})")
    );
    progress.enable_steady_tick(1000);
    let progress = Mutex::new(progress);

    // Start paralell processing via Rayon iterators:
    result.chapters.par_iter().map(|chapter| {
        let track_nr = chapter.track_nr + 1;
        let book_title = &result.format.tags["title"];
        ffmpeg::FfmpegOptions {
            activation_bytes: &params.activation_bytes,
            start: &chapter.start, end: &chapter.end,
            title: &chapter.tags["title"],
            track_nr,
            quality: params.quality,
            input_file: params.input_file(),
            output_file: params.output_file(book_title, track_nr, &chapter.tags["title"])
        }
    }).filter(|option| {
        let exists = option.output_file.exists();
        if exists {
            let progress = progress.lock().unwrap();
            progress.inc(1);
            progress.println(&format!(
                " Skipping: Chapter {}, file \"{}\" already exists.", 
                option.track_nr, option.output_file.to_str().unwrap()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn format_output_file() {
        let out_folder = Path::new("nested").join("out").join("folder");

        let params = RunParameters {
            input_file: String::from("test.aax"),
            output_folder: out_folder.to_str().unwrap().to_string(),
            activation_bytes: String::from("abcd1234"),
            quality: 6,
            output_format: String::from("{title}_{track_nr}.mp4"), // will add .mp3 anyways!
        };

        let out_file = params.output_file("Test Book", 13, "Chapter 13");
        assert_eq!(
            out_file.to_str().unwrap(), out_folder.join("Test_Book_13.mp4.mp3").to_str().unwrap()
        );

        let params = RunParameters {
            output_format: String::from("{track_nr} {track_title} {title}"),
            ..params
        };
        let out_file = params.output_file("Game of Thrones", 121, "Bran the broken");
        assert_eq!(
            out_file.to_str().unwrap(), 
            out_folder.join("121_Bran_the_broken_Game_of_Thrones.mp3").to_str().unwrap()
        );
    }

    /// When given an invalid format parameter, fallback to the default one.
    #[test]
    fn format_output_fallback() {
        let out_folder = Path::new("path");

        let params = RunParameters {
            input_file: String::from("test.aax"),
            output_folder: out_folder.to_str().unwrap().to_string(),
            activation_bytes: String::from("abcd1234"),
            quality: 6,
            output_format: String::from("{broken}"), // not valid!
        };

        let out_file = params.output_file("Test Book", 13, "Chapter 13");
        assert_eq!(
            out_file.to_str().unwrap(), out_folder.join("Test_Book_13.mp3").to_str().unwrap()
        );
    }
}