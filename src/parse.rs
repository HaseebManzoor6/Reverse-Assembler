use std::{
    fs::File,
    io::{self, BufRead},
    u64,
    num::ParseIntError,
    collections::HashMap,
};

#[path="bits.rs"]
mod bits;
use bits::Wordt;

#[path="instrset.rs"]
mod instrset;
use instrset::{
    Instrfmt,
    Node,
    Maskmap,
    Instrset,
};

pub enum ErrType {
    NoWordsize,
    ExtraClosingBrace,
    NoMask,
    ZeroWordsize,
    ZeroMask,
    ParseNumber,
    Internal,
    Other,
}



pub fn err_msg(t: ErrType) {
    println!("\t{}", match t {
        ErrType::NoWordsize => "Expected word size declaraction (like \"32 bit words\" or \"4 byte words\") at start of file",
        ErrType::NoMask => "Expected bit mask declaration for opcode (like \"mask b01110000 {...\" for 3 bit opcodes) after word size declaration)",
        ErrType::ZeroWordsize => "Word size cannot be 0",

        ErrType::ZeroMask => "Bit masks cannot be 0",

        ErrType::ParseNumber => "Error parsing a number. Prefix numbers with \'b\' for binary or \'x\' for hexadecimal. Numbers are base 10 otherwise.",

        ErrType::ExtraClosingBrace => "Extra closing brace",

        ErrType::Internal => "Internal Error",

        ErrType::Other => "Malformed line",
    })
}

fn parse_number(text: &str) -> Result<Wordt, ParseIntError> {
    if let Some(s)=text.strip_prefix('b')      {return Wordt::from_str_radix(s,2)}
    else if let Some(s)=text.strip_prefix('x') {return Wordt::from_str_radix(s,16)}
    else                                       {return Wordt::from_str_radix(text,10)}
}

fn gen_mask(v: &Vec<&str>, start: usize) -> Option<Wordt> {
    if v.len()-1<=start {return None}

    if v[start]=="mask" {
        if let Ok(n)=parse_number(v[start+1]) {
            if n==0 {return None}

            return Some(n)
        }
    }

    None
}

/*
 * Read wordsize
 */
fn parse_first_line(words: &Vec<&str>) -> Result<Wordt,ErrType> {

    if words.len()==3 && "words"==words[2] {
        if let Ok(n)=parse_number(&words[0]) {
            if n==0                  {return Err(ErrType::ZeroWordsize)}
            else if words[1]=="byte" {return Ok(n*8)}
            else if words[1]=="bit"  {return Ok(n)}
        }
        else {return Err(ErrType::ParseNumber)}
    }
    return Err(ErrType::NoWordsize)
}

/*
 * Read initial mask for the instruction set
 */
fn parse_second_line(words: &Vec<&str>, map: &mut Maskmap) -> Option<ErrType> {
    if let Some(n)=gen_mask(words,0) {
        if n==0 {return Some(ErrType::ZeroMask)}

        map.mask=n;
        return None
    }
    else {return Some(ErrType::NoMask)}
}

/*
 * Create either a Instrfmt or a Maskmap, which is returned and to be
 *  inserted into a Maskmap
 */
fn create_node(words: &Vec<&str>,mut mask: Wordt) -> Result<(Wordt,Node),ErrType> {
    if words.len()<3 {return Err(ErrType::Other)}

    // n Will store the opcode for the new Node, under the containing Maskmap's mask
    let n: Wordt;
    match parse_number(words[0]) {
        Ok(x) => {n=bits::unmask(x,mask);},
        Err(_) => {return Err(ErrType::ParseNumber)}
    }

    // instr
    // TODO can flatten String -> str in Instrfmt?
    if words[1]=="=" { return Ok((n,Node::Instr(Instrfmt {name: words[2].to_string()}))) }
    // map
    return match gen_mask(words,1) {
        Some(m) => Ok((n,Node::Map(Maskmap{mask: m, map: HashMap::new()}))),
        None => Err(ErrType::NoMask)
    }
}

pub fn parse_file(file: &File) -> Result<Instrset, (ErrType,u64)> {
    // Curly {} braces represent nesting of Maskmaps. The Wordt is the index in the parent map
    let mut braces: Vec<(Wordt, Maskmap)> = Vec::new();

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
                    Some(why) => return Err((why,ln)),
                    None => {},
                }}

                // Closing braces
                else if words[0]=="}" && words.len()==1 {
                    if braces.len()==0 { return Err((ErrType::ExtraClosingBrace,ln)); }

                    let tmp=braces.pop().unwrap();
                    // final closing brace returns Instrset
                    if braces.len()==0 {d.set=tmp.1; return Ok(d)}
                    // otherwise move temp Maskmap off braces stack and into parent Maskmap
                    else {braces.last_mut().unwrap().1.map.insert(tmp.0,Node::Map(tmp.1));}
                }

                // other lines
                else {match create_node(&words,braces.last_mut().unwrap().1.mask) {
                    Ok((i,n)) => match n {
                        Node::Instr(_) => {braces.last_mut().unwrap().1.map.insert(i,n);},
                        Node::Map(map) => {braces.push((i,map));},
                    },
                    Err(why) => {return Err((why,ln))}
                }}


                lines_parsed+=1;
            },
            Err(why) => {
                println!("Internal error: {}",why);
                return Err((ErrType::Internal,ln))
            }
        }
    }
    Err((ErrType::Other,ln))
}
