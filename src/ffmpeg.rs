//! This wraps the `ffmpeg` CLI program to run it and parse it's output.

use crate::CliTool;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use regex::Regex;

const BIN_NAME: &str = "ffmpeg";

#[derive(Debug)]
pub struct FfmpegOptions<'a> {
    pub activation_bytes: &'a str,
    pub start: &'a str,
    pub end: &'a str,
    pub title: &'a str,
    pub track_nr: u32,
    pub input_file: PathBuf,
    pub output_folder: PathBuf,
}

impl<'a> FfmpegOptions<'a> {

    /// Creates a new `PathBuf` for the output of this file.
    fn output_path(&self) -> PathBuf {
        let file_name = format!("out_chapter_{}.mp3", self.track_nr);
        self.output_folder.join(&file_name)
    }

    pub fn output_file(&self) -> String {
        // Can't return ref because PathBuf is owned by this fn. Alternative without copy?
        String::from(self.output_path().to_str().unwrap())
    }

    pub fn output_exists(&self) -> bool {
        self.output_path().exists()
    }
}

pub struct FFMPEG;

impl<'a> CliTool<&FfmpegOptions<'a>, (), io::Error> for FFMPEG {
    
    fn is_installed() -> Option<String> {
        let mut command = Command::new(BIN_NAME);
        command.arg("-version");

        debug!("check command: {:?}", command);

        if let Ok(output) = command.output() {
            let regex = Regex::new(r"ffmpeg version (?P<version>(?:\d\.?)+)").unwrap();

            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                if let Some(captures) = regex.captures(line) {
                    if let Some(version) = captures.name("version") {
                        return Some(version.as_str().to_string());
                    }
                }
            }
        }
        None
    }

    fn execute(options: &FfmpegOptions) -> Result<(), io::Error> {
        // ffmpeg -nostdin -v error -activation_bytes <activation-bytes> \
        //  -ss <start_time> -to <end_time> -i <in>.aax \
        //  -c:a libmp3lame -ac 2 -q:a 2 \
        //  -metadata title="<title>" -metadata track="<curr/total>" \
        //  <out>.mp3

        let file = options.input_file.to_str().unwrap(); // do unwrap??
        let title = format!("title={}", options.title);
        let track = format!("track={}", options.track_nr);

        let mut command = Command::new(BIN_NAME);
        command.arg("-nostdin")
            .arg("-v").arg("error")
            .arg("-activation_bytes").arg(options.activation_bytes)
            .arg("-ss").arg(options.start)
            .arg("-to").arg(options.end)
            .arg("-i").arg(file)
            .arg("-codec:a").arg("libmp3lame")
            .arg("-qscale:a").arg("4") // Controls bitrate. Make configurable??
            //.arg("-ac").arg("2")
            .arg("-metadata").arg(&title)
            .arg("-metadata").arg(&track)
            .arg(options.output_file());

        debug!("transcode command: {:?}", command);
        
        let output = command.output()?;
        if output.status.success() {
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(io::Error::new(io::ErrorKind::Other, error))
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_installed() {
        let version = FFMPEG::is_installed().unwrap();
        assert_eq!(version, "4.1.3".to_string());
    }

    #[test]
    fn test_execute() {
        
    }
}