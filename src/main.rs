use audible_split;
use std::process;

fn main() {
    
    let input = String::from("test.aax");
    let output = String::from("out/");
    let activation_bytes = String::from("something");

    let result_code = audible_split::run(input, output, activation_bytes);
    process::exit(result_code);

}