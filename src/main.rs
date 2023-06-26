use std::{
    env,
    fs::File,
};

mod parse;


fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() <= 1 {
        println!("Usage: {} [filename]",&argv[0]);
        return
    }

    match File::open(&argv[1]) {
        Ok(file) => {
            parse::parse_file(&file);
        },
        Err(why) => {
            println!("Couldn't open file {}: {}",&argv[1],why);
        },
    }
}
