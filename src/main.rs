use std::{
    env,
    fs::File,
};

mod parse;
use parse::instrset as instrset;
use parse::deassemble as deassemble;
use deassemble::branch as branch;

use instrset::{
    Instrset,
    binreader::Binreader,
};

use branch::BranchTree;


fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() <= 1 {
        eprintln!("Usage: {} [filename]",&argv[0]);
        return
    }

    // Make instructions set from script
    let is: Instrset;
    eprintln!("== Read Script ==");
    match File::open(&argv[1]) {
        Ok(file) => {
            match parse::parse_file(&file,argv.len()==4 && argv[3]=="-reverse") {
                Err((t,ln)) => {
                    eprintln!("Line {}: Syntax error in file: {}",ln,t);
                    return
                },
                Ok(d) => { 
                    is=d;
                    eprintln!("Finished parsing file {}",&argv[1]);
                }
            }

        },
        Err(why) => {
            eprintln!("Couldn't open script file {}: {}",&argv[1],why);
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
    // Open file
    let mut binreader = match Binreader::new(is.wordsize, &argv[2], is.endian_little) {
        Some(br) => br,
        None => { return },
    };

    // find any branch labels who move upwards
    eprintln!("== Generate Branch Labels ==");
    let mut branches: BranchTree = BranchTree::new();
    match branch::add_branch_ups(&mut binreader,&mut branches, &is.set) {
        Ok(()) => {eprintln!("Generated branches for file {}",argv[2]);},
        Err(why) => eprintln!("{}",why),
    };
        

    // deassemble
    eprintln!("== Deassemble ==");
    match deassemble::deassemble_file(&mut binreader,&is,&mut branches) {
        Ok(()) => { eprintln!("Done reading file {}",argv[2]); },
        Err(why) => {eprintln!("{}",why); return}
    }
}

