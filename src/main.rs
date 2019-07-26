use audible_split;
use std::process;
extern crate clap;
use clap::{Arg, App};

fn main() {
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
        ).get_matches();
    
    let input = matches.value_of("input").unwrap().to_string();
    let output = matches.value_of("output").unwrap().to_string();
    let activation_bytes = matches.value_of("activation_bytes").unwrap().to_string();

    let result = audible_split::run(input, output, activation_bytes);
    match result {
        Ok(_) => {
            println!("All chapters completed successfully");
            process::exit(0);
        },
        Err(e) => {
            eprintln!("Error while transcoding: {}", e);
            process::exit(1);
        }
    }
}