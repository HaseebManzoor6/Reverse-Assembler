use std::{
    fs::File,
    io::{self, BufRead},
    u64,
    num::ParseIntError,
    collections::HashMap,
};

#[path="deassemble.rs"]
pub mod deassemble;
pub use deassemble::instrset as instrset;

use instrset::{
    Instrfmt,
    Node,
    Maskmap,
    Instrset,
};

use instrset::bits as bits;
use bits::{
    Wordt,
    Bitmask,
};

pub enum ErrType {
    NoWordsize,
    ExtraClosingBrace,
    NoMask,
    ZeroWordsize,
    ZeroMask,
    ParseNumber,
    Internal,

    UnknownFormat,

    Other,
}



pub fn err_msg(t: ErrType) {
    eprintln!("\t{}", match t {
        ErrType::NoWordsize => "Expected word size declaraction (like \"4 byte words\") at start of file",
        ErrType::NoMask => "Expected bit mask for opcodes (like \"mask b01110000 {\" for 3 bit opcodes) here",
        ErrType::ZeroWordsize => "Word size cannot be 0",

        ErrType::ZeroMask => "Bit masks cannot be 0",

        ErrType::ParseNumber => "Error parsing a number. Prefix numbers with \'0b\' for binary or \'0x\' for hexadecimal. Numbers are base 10 otherwise.",

        ErrType::ExtraClosingBrace => "Extra closing brace",

        ErrType::Internal => "Internal Error",

        ErrType::UnknownFormat => "Unrecognized format",

        ErrType::Other => "Malformed line",
    })
}

fn parse_number(text: &str) -> Result<Wordt, ParseIntError> {
    if let Some(s)=text.strip_prefix("0b")      {return Wordt::from_str_radix(s,2)}
    else if let Some(s)=text.strip_prefix("0x") {return Wordt::from_str_radix(s,16)}
    else                                        {return Wordt::from_str_radix(text,10)}
}

/*
 * Helper for gen_mask()
 * Parse a range of numbers from text,
 *  such as 3:7 -> bits 3,4,5,6,7 -> 0b11111000
 *  or      3   -> bit 3          -> 0b00001000
 */
fn parse_range(text: &str) -> Option<Bitmask> {
    let mut range: [Wordt; 2]=[0,0];
    let mut ret: Bitmask=0;

    if text.contains(':') {
        for (i,num) in text.split(':').enumerate() {
            if i>1 {return None}
            range[i]=match parse_number(num) {
                Ok(x) => x,
                Err(_) => {return None}
            };
            if i==1 {ret= ((1<<range[0])-1)
                         ^((1<<range[1])-1)
                         +(1<<range[1]);
            }
        }
        return Some(ret);
    }
    else {
        match parse_number(text) {
            Ok(x) => {return Some(1<<x)},
            Err(_) => {return None}
        }
    }
}

/*
 * Read a bitmask from a statement like:
 *  mask 0b01001
 * Returns the bitmask and the number of words read
 */
fn gen_mask(v: &Vec<&str>, start: usize) -> Option<(Bitmask,usize)> {
    if v.len()-1<=start {return None}
    let mut ranges: [Wordt; 2];
    let mut mask: Bitmask=0;

    if v[start]=="mask" {
        if let Ok(n)=parse_number(v[start+1]) {
            if n==0 {return None}

            return Some( (n,2) )
        }
    }
    else if v[start]=="bits" {
        for range in v[start+1].split('+') {
            mask |= match parse_range(range) {
                Some(m) => m,
                None => {return None}
            }
        }
        return Some((mask,2))
    }

    eprintln!("Expected bitmask, found nothing");
    None
}

#[cfg(test)]
mod genmask_tests {
    use crate::parse;
    #[test]
    fn test_genmask() {
        let result: (parse::Bitmask, usize);
        if let Some(m)=parse::gen_mask(
                &vec!["mask","0b1101"],
                0) {
            result=m;
        }
        else {assert!(0==1); return}
        assert!(
            result == (0b1101,2),
            "Actual: ({:#b},{})",result.0,result.1
            );
    }

    #[test]
    fn test_genmask_bits() {
        let result: (parse::Bitmask, usize);
        if let Some(m)=parse::gen_mask(
                &vec!["bits","0+1:3"],
                0) {
            result=m;
        }
        else {assert!(0==1); return}
        assert!(
            result == (0b1111,2),
            "Actual: ({:#b},{})",result.0,result.1
            );
    }
}

/*
 * Read wordsize
 */
fn parse_first_line(words: &Vec<&str>) -> Result<usize,ErrType> {

    if words.len()==3 && "words"==words[2] {
        if let Ok(n)=parse_number(&words[0]) {
            if n==0                  {return Err(ErrType::ZeroWordsize)}
            else if words[1]=="byte" {return Ok((n).try_into().unwrap())}
        }
        else {return Err(ErrType::ParseNumber)}
    }
    return Err(ErrType::NoWordsize)
}

/*
 * Read initial mask for the instruction set
 */
fn parse_second_line(words: &Vec<&str>, map: &mut Maskmap) -> Result<(),ErrType> {
    if let Some( (n,_) )=gen_mask(words,0) {
        if n==0 {return Err(ErrType::ZeroMask)}

        map.mask=n;
        return Ok(())
    }
    else {return Err(ErrType::NoMask)}
}

/*
 * Create an Instrfmt
 */
fn create_fmt(words: &Vec<&str>, mut start: usize) -> Result<Instrfmt,ErrType> {
    let mut fmt: Vec<(instrset::Fmt,Bitmask)>=Vec::new();
    let mut mask: Bitmask;
    let mut read: usize;

    while start<words.len() {
        // gen mask
        match gen_mask(words,start+1) {
            Some( (x,i) ) => {mask=x; read=i;},
            None => {return Err(ErrType::NoMask)}
        }

        // get format type
        match words[start] {
            "ADDR" => { fmt.push((instrset::Fmt::Addr,mask))},
            "UINT" => { fmt.push((instrset::Fmt::Unsigned,mask))},
            "INT"  => { fmt.push((instrset::Fmt::Signed,mask))},

            other => {
                eprintln!("Unrecognized format: {}",other);
                return Err(ErrType::UnknownFormat)
            }
        }

        start += read+1;
    }

    Ok(Instrfmt {fmt})
}

/*
 * Create either a Instrfmt or a Maskmap, which is returned and to be
 *  inserted into a Maskmap
 */
fn create_node(words: &Vec<&str>,mask: Bitmask) -> Result<(Wordt,Node),ErrType> {
    if words.len()<3 {return Err(ErrType::Other)}

    // n Will store the opcode for the new Node, under the containing Maskmap's mask
    let n: Wordt;
    match parse_number(words[0]) {
        Ok(x) => {n=bits::align(x,mask);},
        Err(why) => {
            eprintln!("Error parsing a number: {}",why);
            return Err(ErrType::ParseNumber)
        }
    }

    // instr
    // TODO can flatten String -> str in Instrfmt?
    if words[1]=="=" {
        match create_fmt(words,3) {
            Ok(fmt) => {return
                Ok((n,Node::Instr((words[2].to_string(),fmt))))
            },
            Err(why) => {return Err(why)}
        }
    }
    // map
    return match gen_mask(words,1) {
        Some( (m,_) ) => Ok((n,Node::Map(Maskmap{mask: m, map: HashMap::new()}))),
        None => Err(ErrType::NoMask)
    }
}

pub fn parse_file(file: &File) -> Result<Instrset, (ErrType,u64)> {
    // Curly {} braces represent nesting of Maskmaps. The Wordt is the index in the parent map
    let mut braces: Vec<(Wordt, Maskmap)> = Vec::new();
    let mut n_instrs: Wordt=0;

    let mut d=Instrset {
        wordsize: 0,
        set: Maskmap {mask: 0, map: HashMap::new()},
    };

    braces.push((0,d.set));

    let mut ln: u64=0; // lines in file
    let mut lines_parsed=0; // non-comment/empty lines
   
    for line in io::BufReader::new(file).lines() {
        ln+=1;
        match line {
            Ok(l) => {
                let words: Vec<&str> = l.split_whitespace().collect();

                // comments
                if words.len()==0 || words[0].starts_with('#') {continue;}

                // First line (wordsize declaration)
                if lines_parsed==0 { match parse_first_line(&words) {
                    Ok(n) => {d.wordsize=n;},
                    Err(why) => return Err((why,ln)),
                }}

                // Second line (first opcode mask)
                else if lines_parsed==1 { match parse_second_line(&words, &mut braces.last_mut().unwrap().1) {
                    Ok(()) => {},
                    Err(why) => return Err((why,ln)),
                }}

                // Closing braces
                else if words[0]=="}" && words.len()==1 {
                    if braces.len()==0 { return Err((ErrType::ExtraClosingBrace,ln)); }

                    let tmp=braces.pop().unwrap();
                    // final closing brace returns Instrset
                    if braces.len()==0 {
                        d.set=tmp.1;
                        eprintln!("Add {} opcodes",n_instrs);
                        return Ok(d)
                    }
                    // otherwise move temp Maskmap off braces stack and into parent Maskmap
                    else {braces.last_mut().unwrap().1.map.insert(tmp.0,Node::Map(tmp.1));}
                }

                // other lines
                else {match create_node(&words,braces.last_mut().unwrap().1.mask) {
                    Ok((i,n)) => match n {
                        Node::Instr((ref _name,ref _fmt)) => {
                            //println!("Add {} (opcode {:#b} under mask {:#b})",name,i,braces.last().unwrap().1.mask);
                            braces.last_mut().unwrap().1.map.insert(i,n);
                            n_instrs+=1;
                        },
                        Node::Map(map) => {braces.push((i,map));},
                    },
                    Err(why) => {return Err((why,ln))}
                }}


                lines_parsed+=1;
            },
            Err(why) => {
                eprintln!("Internal error: {}",why);
                return Err((ErrType::Internal,ln))
            }
        }
    }
    Err((ErrType::Other,ln))
}
