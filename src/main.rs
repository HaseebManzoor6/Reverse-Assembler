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
            match parse::parse_file(&file) {
                Err((t,ln)) => {
                    println!("Line {}: Syntax error in file:",ln);
                    parse::err_msg(t);
                },
                Ok(d) => {
                    /*TODO*/
                }
            }

        },
        Err(why) => {
            println!("Couldn't open file {}: {}",&argv[1],why);
        },
    }
}
