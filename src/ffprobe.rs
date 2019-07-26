//! Wraps the `ffprobe`-cli executable and parses its output for usage in the tool.

use std::io;
use std::process::Command;
use std::collections::HashMap;
use serde_json::{self, Value};
use serde::{Deserialize};
use crate::{CliTool};

const BIN_NAME: &str = "ffprobe";

#[derive(Deserialize, Debug)]
pub struct FFPROBEoutput {
    pub format: ProbeFormat,
    pub chapters: Vec<ProbeChapter>,
}

#[derive(Deserialize, Debug)]
pub struct ProbeFormat {
    pub tags: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct ProbeChapter {
    #[serde(rename = "start_time")]
    pub start: String,
    #[serde(rename = "end_time")]
    pub end: String,
    #[serde(rename = "id")]
    pub track_nr: u32,
    pub tags: HashMap<String, String>,
}

pub struct FFPROBE {}

impl FFPROBE {

}
impl CliTool<&str, FFPROBEoutput, io::Error> for FFPROBE {

    fn is_installed() -> Option<String> {
        let mut command = Command::new(BIN_NAME);
        command.arg("-v").arg("quiet")
            .arg("-print_format").arg("json=c=1")
            .arg("-show_versions");

        if let Ok(output) = command.output() {
            let result: serde_json::Result<Value> = serde_json::from_slice(&output.stdout);
            if let Ok(data) = result {
                if let Value::String(version) = &data["program_version"]["version"] {
                    return Some(version.clone());
                }
            }
        }
        None
    }

    fn execute<'a>(options: &'a str) -> Result<FFPROBEoutput, io::Error> {
        // ffprobe -v quiet -print_format json -show_chapters -show_format <file>.aax
        // format-object has info like album, title, etz
        // chapters-array has all chapters.

        let mut command = Command::new(BIN_NAME);
        command.arg("-v").arg("quiet")
            .arg("-print_format").arg("json=c=1")
            .arg("-show_chapters")
            .arg("-show_format")
            .arg(options);

        let out = command.output()?;

        let result: serde_json::Result<FFPROBEoutput> = serde_json::from_slice(&out.stdout);
        match result {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_output() {
        let output = FFPROBE::execute("..\\qualityland.aax").unwrap();
        assert_eq!(output.format.tags["title"], "QualityLand: Dunkle Edition".to_string());
        assert_eq!(output.chapters.len(), 78);
        assert_eq!(output.chapters[0].end, "78.994286".to_string());
        assert_eq!(output.chapters[1].start, "78.994286".to_string());
        assert_eq!(output.chapters[2].tags["title"], "Chapter 3".to_string());
    }

    #[test]
    fn test_installed() {
        let installed = FFPROBE::is_installed().unwrap();
        assert_eq!(installed, "4.1.3".to_string());
    }
}