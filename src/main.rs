use std::{
    env,
    fs::File,
};

mod parse;
use parse::instrset as instrset;
use parse::deassemble as deassemble;

use instrset::{
    Instrset,
};


fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() <= 1 {
        println!("Usage: {} [filename]",&argv[0]);
        return
    }

    // Make instructions set from script
    let is: Instrset;
    match File::open(&argv[1]) {
        Ok(file) => {
            match parse::parse_file(&file) {
                Err((t,ln)) => {
                    println!("Line {}: Syntax error in file:",ln);
                    parse::err_msg(t);
                    return
                },
                Ok(d) => { 
                    is=d;
                    println!("Finished parsing file {}",&argv[1]);
                }
            }

        },
        Err(why) => {
            println!("Couldn't open script file {}: {}",&argv[1],why);
            return
        },
    }

    /*
    let a=0b10000000;
    match parse::instrset::deassemble(a,is.set) {
        Some(fmt) => {println!("{:#b} matches {}",a,fmt);},
        None      => {println!("No match for {:#b}",a);},
    }
    */

    // De-assemble a binary file
    if argv.len()==3 { match File::open(&argv[2]) {
        Ok(file) => {
            match deassemble::deassemble_file(&file,&is) {
                None => { println!("Done reading file {}",argv[2]); },
                Some((ln,e)) => {deassemble::print_deasm_err(ln,e); return}
            }
        },
        Err(why) => {
            println!("Couldn't open binary file {}: {}",&argv[2],why);
            return
        },
    }}
}

