use std::{
    fs::File,
    io::{self, BufRead},
    u64,
    num::ParseIntError,
    collections::HashMap,
};

type Wordt = u64;

struct Instrfmt {
    name: String,
}

enum Node {
    Map(Maskmap),
    Instr(Instrfmt)
}

// Bit-masking hashmap. Keys are words under the bitmask, vals are &Node
struct Maskmap {
    mask: Wordt,
    map: HashMap<Wordt, Node>
}

struct Instrset {
    wordsize: Wordt,
    set: Maskmap,
}

enum ErrType {
    NoWordsize,
    ExtraClosingBrace,
    NoMask,
    ZeroWordsize,
    ZeroMask,
    ParseNumber,
    ExtraClosingBraceBefore,
    InternalHashmap,
    InsertWithoutMap,
    Other,
}



fn err_msg(t: ErrType) {
    println!("\t{}", match t {
        ErrType::NoWordsize => "Expected word size declaraction (like \"32 bit words\" or \"4 byte words\") at start of file",
        ErrType::NoMask => "Expected bit mask declaration for opcode (like \"mask b01110000 {...\" for 3 bit opcodes) after word size declaration)",
        ErrType::ZeroWordsize => "Word size cannot be 0",

        ErrType::ZeroMask => "Bit masks cannot be 0",

        ErrType::ParseNumber => "Error parsing a number. Prefix numbers with \'b\' for binary or \'x\' for hexadecimal. Numbers are base 10 otherwise.",

        ErrType::ExtraClosingBrace => "Extra closing brace",

        ErrType::InsertWithoutMap => "Can't place an instruction here; define a bit mask first",
        ErrType::InternalHashmap => "Internal error",

        ErrType::ExtraClosingBraceBefore => "Extra closing brace before this line",

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

fn parse_second_line(words: &Vec<&str>, map: &mut Maskmap) -> Option<ErrType> {
    if let Some(n)=gen_mask(words,0) {
        if n==0 {return Some(ErrType::ZeroMask)}

        map.mask=n;
        return None
    }
    else {return Some(ErrType::NoMask)}
}

fn create_node(words: &Vec<&str>) -> Result<(Wordt,Node),ErrType> {
    if words.len()<3 {return Err(ErrType::Other)}

    let n: Wordt;
    match parse_number(words[0]) {
        Ok(x) => {n=x;},
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

pub fn parse_file(file: &File) {
    // Curly {} braces represent nesting of Maskmaps. The Wordt is the index in the parent map
    let mut braces: Vec<(Wordt, Maskmap)> = Vec::new();

    let mut d=Instrset {
        wordsize: 0,
        set: Maskmap {mask: 0, map: HashMap::new()},
    };

    braces.push((0,d.set));

    let mut ln: u64=0; // lines in file
    let mut lines_parsed=0; // non-comment/empty lines
   
    let mut e: Option<ErrType>;

    for line in io::BufReader::new(file).lines() {
        ln+=1;
        match line {
            Ok(l) => {
                let words: Vec<&str> = l.split_whitespace().collect();

                // comments
                if words.len()==0 || words[0].starts_with('#') {continue;}

                // First line (wordsize declaration)
                if lines_parsed==0 { match parse_first_line(&words) {
                    Ok(n) => {d.wordsize=n; e=None;},
                    Err(why) => {e=Some(why);},
                }}

                // Second line (first opcode mask)
                else if lines_parsed==1 { e=parse_second_line(&words, &mut braces.last_mut().unwrap().1); }

                // Closing braces
                else if words[0]=="}" && words.len()==1 {
                    if braces.len()==0 { e=Some(ErrType::ExtraClosingBrace); }
                    else {
                        let tmp=braces.pop().unwrap();
                        if braces.len()==0 {d.set=tmp.1;}
                        else {braces.last_mut().unwrap().1.map.insert(tmp.0,Node::Map(tmp.1));}
                        e=None;
                    }
                }

                // other lines
                else {match create_node(&words) {
                    Ok((i,n)) => match n {
                        Node::Instr(_) => {braces.last_mut().unwrap().1.map.insert(i,n); e=None;},
                        Node::Map(map) => {braces.push((i,map)); e=None;},
                    },
                    Err(why) => {e=Some(why);}
                }}


                if let Some(t)=e {
                    println!("Line {}: Syntax error in file:",ln);
                    err_msg(t);
                    break;
                }

                lines_parsed+=1;
            },
            Err(why) => {
                println!("Internal error: {}",why);
                break;
            }
        }
    }
}
