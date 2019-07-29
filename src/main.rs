use audible_split;
use std::process;
extern crate clap;
use clap::{Arg, App};

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{SimpleLogger, LevelFilter, Config};

fn main() {
    // Setup CLI interface
    let matches = App::new("Audbile Split")
        .version(clap::crate_version!()) // uses version from Cargo.toml
        .author("Lukas Knuth")
        .about("Takes an Audible .aax file and splits it into an .mp3 file per chapter.
\nCode & Bugtracker at https://github.com/LukasKnuth/audible_split")
        .arg(Arg::with_name("input")
            .short("i").long("input")
            .value_name("FILE.aax")
            .help("The Audbile .aax input file")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("activation_bytes")
            .short("ab").long("activation_bytes")
            .value_name("HEX")
            .help("The activation bytes to decode the given Audbile .aax file")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("output")
            .short("o").long("output")
            .value_name("FOLDER")
            .help("The output folder which contains all transcoded .mp3 files. \
            Will be created recursively if not existing.")
            .default_value("output/")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("quality")
            .short("q").long("quality")
            .value_name("0..9")
            .help("Value for libmp3lame qscale setting.")
            .long_help("Value for libmp3lame qscale setting. \
            Generally, a smaller number yields better quality but higher filesize. \
            See https://trac.ffmpeg.org/wiki/Encode/MP3 for more information and guidence.\n")
            .default_value("6")
            .validator(validate_config)
            .takes_value(true)
        )
        .arg(
            Arg::with_name("name_format")
            .long("format")
            .value_name("FORMAT")
            .help("Format the file-name for output MP3 files.")
            .long_help("The .mp3 extension will be added if absent. Character escaping and removal \
            of spaces is done automatically. Available placeholders are:
    {title} - The title of the AudioBook
    {track_nr} - The number of the chapter
    {track_title} - The title of the track\n")
            .default_value(audible_split::DEFAULT_FORMAT)
            .takes_value(true)
        )
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Print debug information like executed commands")
        ).get_matches();
    
    // Get paramters from CLI 
    let params = audible_split::RunParameters {
        input_file: matches.value_of("input").unwrap().to_string(),
        output_folder: matches.value_of("output").unwrap().to_string(),
        activation_bytes: matches.value_of("activation_bytes").unwrap().to_string(),
        quality: matches.value_of("quality").unwrap().parse().unwrap(),
        output_format: matches.value_of("name_format").unwrap().to_string()
    };
    
    // init logger
    let log_level = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    SimpleLogger::init(log_level, logger_config()).expect("Couldn't initialize logger!");

    // Run the actual program.
    let result = audible_split::run(params);
    match result {
        Ok(_) => {
            info!("All chapters completed successfully");
            process::exit(0);
        },
        Err(_) => {
            error!("Not every chapter transcoded successfully! See above erros.");
            process::exit(1);
        }
    }
}

fn logger_config() -> Config {
    Config {
        time: None,
        time_format: None,
        ..Config::default()
    }
}

fn validate_config(input: String) -> Result<(), String> {
    let parse_res: Result<u8, _> = input.parse();
    if let Ok(nr) = parse_res {
        if nr <= 9 { // >= 0 is implied because of unsigend type
            Ok(())
        } else {
            Err(format!("Value {} is not in allowed range of 0..9", nr))
        }
    } else {
        Err(format!("Value \"{}\" couldn't be parsed to number.", input))
    }
}