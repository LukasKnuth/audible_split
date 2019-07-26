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
        .version("1.0")
        .author("Lukas Knuth")
        .about("Takes an Audible .aax file and splits it into an .mp3 file per chapter.")
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
            .help("The output folder which contains all transcoded .mp3 files")
            .default_value("output/")
            .takes_value(true)
        )
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Print debug information like executed commands")
        ).get_matches();
    
    // Get paramters from CLI 
    let input = matches.value_of("input").unwrap().to_string();
    let output = matches.value_of("output").unwrap().to_string();
    let activation_bytes = matches.value_of("activation_bytes").unwrap().to_string();
    
    // init logger
    let log_level = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    SimpleLogger::init(log_level, logger_config()).expect("Couldn't initialize logger!");

    // Run the actual program.
    let result = audible_split::run(input, output, activation_bytes);
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