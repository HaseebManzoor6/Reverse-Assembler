use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead},
    u64,
    num::ParseIntError,
    collections::{
        HashMap,
    },
};

#[path="deassemble.rs"]
pub mod deassemble;

pub use deassemble::instrset::{
    self as instrset,
    Instrfmt,
    Fmt, FmtType,
    Node,
    Maskmap,
    Instrset,
    bits as bits, bits::{
        Wordt,
        Bitmask,
        BitOp, BitOpType,
    },
};


pub enum ErrType {
    NoWordsize(String),
    NoMask(String),
    ZeroWordsize,
    NoWordsizeUnits(String),
    ZeroMask(String),
    ParseNumber(String,ParseIntError),
    ExtraClosingBrace,
    Internal(std::io::Error),

    UnknownFormat(String),
    ExpectedNumber(String),

    Other,
}

impl Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        match self {
            ErrType::NoWordsize(line) =>
                write!(f,"Expected word size declaration \
                       (like \"4 byte little endian words\") at start of file. \
                       Found:\n{}",
                line),

            ErrType::NoMask(found) =>
                write!(f,"Expected bit mask for opcodes \
                       (like \"mask b01110000 {{\" for 3 bit opcodes) \
                       here. Found:\n{}",
                found),

            ErrType::ZeroWordsize =>
                write!(f,"Word size cannot be 0"),

            ErrType::NoWordsizeUnits(num) =>
                write!(f,"Wordsize was not given units; add \"bytes\" after: {}",num),

            ErrType::ZeroMask(mask) =>
                write!(f,"Bit masks cannot be 0. Erroneous bitmask:\n{}",
                mask),

            ErrType::ParseNumber(num,why) =>
                write!(f,"Couldn't parse \"{}\" as a number: {}.\n\
                   Prefix numbers with \'0b\' for binary or \'0x\' for hexadecimal. \
                   Numbers are base 10 otherwise.",
                   why, num),

            ErrType::ExtraClosingBrace => write!(f,"Extra closing brace"),

            ErrType::Internal(why) => write!(f,"I/O error: {}",why),

            ErrType::UnknownFormat(fmt) =>
                write!(f,"Unrecognized format: \"{}\"",fmt),

            ErrType::ExpectedNumber(lastword) =>
                write!(f,"Expected a number after \"{}\", found end of line",lastword),

            ErrType::Other => write!(f,"Malformed line"),
        }
    }
}

/*
 * Helper for error handling
 * Only to be called in error cases!
 * Concatenate all strings in <words> to a String
 */
fn wordsvec_to_string(words: &Vec<&str>) -> String {
    let mut ret=String::new();
    for word in words {
        ret+=word;
        ret+=" ";
    }
    ret
}



/*
 * String to int
 */
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
 * TODO more descriptive errors; ParseRangeErr type?
 */
fn parse_range(text: &str) -> Option<Bitmask> {
    let mut range: [Wordt; 2]=[0,0];
    let mut ret: Bitmask=0;

    // range
    if text.contains(':') {
        /*
         * Split on :, want syntax like 3:5 but not
         * 3:5:7 (+ union syntax is used for that, i.e. 3:5+6:7)
         */
        for (i,num) in text.split(':').enumerate() {
            if i>1 {return None}
            range[i]=match parse_number(num) {
                Ok(x) => x,
                Err(_) => {return None}
            };
            /*
             * Generate bitmask from [a,b]
             * i.e. range=[3,5] => ret=0b00111000
             */
            if i==1 {ret= ((1<<range[0])-1)
                         ^((1<<range[1])-1)
                         +(1<<range[1]);
            }
        }
        return Some(ret);
    }
    // single bit
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
fn gen_mask(v: &Vec<&str>, start: usize, reverse: usize) -> Option<(Bitmask,usize)> {
    if v.len()-1<start {return None}
    let mut mask: Bitmask=0;

    // look for mask [mask]
    if v[start]=="mask" {
        if let Ok(n)=parse_number(v[start+1]) {
            if n==0 {return None}

            mask=n;
            if reverse>0 {mask=bits::reverse(&mask,reverse);}
            return Some((mask,2))
        }
    }
    // look for bits [range]
    else {
        for range in v[start].split('+') {
            mask |= match parse_range(range) {
                Some(m) => m,
                None => {return None}
            }
        }
        if reverse>0 {mask=bits::reverse(&mask,reverse);}
        return Some((mask,1))
    }

    None
}

/*
 * Read wordsize and endianness
 */
fn parse_first_line(words: &Vec<&str>) -> Result<(usize,bool,bool),ErrType> {
    let mut native_endian: bool=true;
    let mut reversed: bool=false;
    let wordsize: usize;

    if (words.len()>=3 && words.len()<=6) && "words"==words[words.len()-1] {
        // wordsize from first 2 words
        match parse_number(&words[0]) {
            Ok(n) => {
                if n==0                  {return Err(ErrType::ZeroWordsize)}
                else if words[1]=="byte" {wordsize=n.try_into().unwrap()}
                else {return Err(ErrType::NoWordsizeUnits(words[0].to_string()))}
            },
            Err(why) => { return Err(ErrType::ParseNumber(words[0].to_string(),why)) }
        }

        // check remaining words for reversed flag and endianness
        let mut i=2; // number of words already read
        while i<words.len()-1 {
            eprintln!("[dbg] {}",words[i]);
            if words[i]=="reversed" {
                reversed=true;
            }
            else if words[i]=="nonnative" && words.len()>i+1 && words[i+1]=="endian" {
                i+=1;
                native_endian=false;
            }
            else if words[i]=="native" && words.len()>i+1 && words[i+1]=="endian" {
                // do nothing
                i+=1;
            }
            else {
                return Err(ErrType::NoWordsize(wordsvec_to_string(words)))
            }
            i+=1;
        }
        return Ok((wordsize,native_endian,reversed))
    }
    Err(ErrType::NoWordsize(wordsvec_to_string(words)))
}

/*
 * Read initial mask for the instruction set
 */
fn parse_second_line(words: &Vec<&str>, map: &mut Maskmap, reverse: usize) -> Result<(),ErrType> {
    if let Some( (n,_) )=gen_mask(words,0, reverse) {
        if n==0 {return Err(ErrType::ZeroMask(words[0].to_string()))}

        map.mask=n;
        return Ok(())
    }
    else {return Err(ErrType::NoMask(wordsvec_to_string(words)))}
}

/*
 * Create an Instrfmt
 * Must not be called on an empty line.
 */
fn create_fmt(words: &Vec<&str>, mut start: usize, reverse: usize) 
-> Result<Instrfmt,ErrType> {
    let mut fmt: Vec<Fmt>=Vec::new();
    let mut mask: Bitmask;
    let mut read: usize;

    let mut ops: Vec<BitOp>;
    let mut n: Wordt;
    let mut tmp: BitOpType;

    while start<words.len() {
        // gen mask
        match gen_mask(words,start+1, reverse) {
            Some( (x,i) ) => {mask=x; read=i;},
            None => {return Err(ErrType::NoMask(words[start+1].to_string()))}
        }

        // get BitOps
        ops=Vec::new();
        for i in start+read+1..words.len() {

            // get op
            tmp=match words[i] {
                "<<" => BitOpType::SL,
                ">>" => BitOpType::SR,
                "&" => BitOpType::AND,
                "|" => BitOpType::OR,
                "^" => BitOpType::XOR,

                _other => break,
            };

            // get number
            if i+1>=words.len() {return Err(
                    ErrType::ExpectedNumber(words.last()
                    .expect("Internal Error: create_fmt() called on empty line").to_string())
                    )}
            n=match parse_number(words[i+1]) {
                Ok(x) => x,
                Err(why) => {return Err(ErrType::ParseNumber(words[i+1].to_string(),why))}
            };

            ops.push(BitOp {typ: tmp, val: n});
            read+=2;
        }

        // get format type
        fmt.push(Fmt
           {typ: match words[start] {
                "addr" => FmtType::Addr,
                "uint" => FmtType::Unsigned,
                "int"  => FmtType::Signed,
                "bin"  => FmtType::Binary,

                "ubranch" => FmtType::Ubranch,
                "dbranch" => FmtType::Dbranch,
                "ibranch" => FmtType::Ibranch,
                "sbranch" => FmtType::Sbranch,

                "ignore" => FmtType::Ignore,

                other => {
                    return Err(ErrType::UnknownFormat(other.to_string()))
                },
            },
                mask: mask,
                ops: ops,
           }
        );

        start += read+1;
    }

    Ok(Instrfmt {fmt})
}

/*
 * Create either a Instrfmt or a Maskmap, which is returned and to be
 *  inserted into a Maskmap
 */
fn create_node(words: &Vec<&str>,mask: Bitmask,reverse: usize) -> Result<(Wordt,Node),ErrType> {
    if words.len()<3 {return Err(ErrType::Other)}

    // n Will store the opcode for the new Node, under the containing Maskmap's mask
    let n: Wordt;
    match parse_number(words[0]) {
        Ok(x) => {n=bits::align(x,mask);},
        Err(why) => {
            return Err(ErrType::ParseNumber(words[0].to_string(),why))
        }
    }

    // instr
    if words[1]=="=" {
        match create_fmt(words,3,reverse) {
            Ok(fmt) => {return
                Ok((n,Node::Instr((words[2].to_string(),fmt))))
            },
            Err(why) => {return Err(why)}
        }
    }
    // map
    return match gen_mask(words,1, reverse) {
        Some( (m,_) ) => Ok((n,Node::Map(Maskmap{mask: m, map: HashMap::new()}))),
        None => Err(ErrType::NoMask(wordsvec_to_string(words)))
    }
}

pub fn parse_file(file: &File) -> Result<Instrset, (ErrType,u64)> {
    // Curly {} braces represent nesting of Maskmaps. The Wordt is the index in the parent map
    let mut braces: Vec<(Wordt, Maskmap)> = Vec::new();
    // <reverse> should be set to 0 for no reversing, or wordsize to reverse all bitmasks by that
    // many bits. Should be set when reading first line of file
    let mut reverse: usize = 0;

    let mut d=Instrset {
        endian_little: true,
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
                    Ok((n,le,to_reverse)) => {
                        d.wordsize=n;
                        d.endian_little=le;
                        if to_reverse {reverse=d.wordsize;}
                    },
                    Err(why) => { return Err((why,ln)) },
                }}

                // Second line (first opcode mask)
                else if lines_parsed==1 { match parse_second_line(&words, &mut braces.last_mut().unwrap().1, reverse) {
                    Ok(()) => (),
                    Err(why) => return Err((why,ln)),
                }}

                // Closing braces
                else if words[0]=="}" && words.len()==1 {
                    if braces.len()==0 { return Err((ErrType::ExtraClosingBrace,ln)); }

                    let tmp=braces.pop().unwrap();
                    // final closing brace returns Instrset
                    if braces.len()==0 {
                        d.set=tmp.1;
                        return Ok(d)
                    }
                    // otherwise move temp Maskmap off braces stack and into parent Maskmap
                    else {braces.last_mut().unwrap().1.map.insert(tmp.0,Node::Map(tmp.1));}
                }

                // other lines
                else {match create_node(&words,braces.last_mut().unwrap().1.mask,reverse) {
                    Ok((i,n)) => match n {
                        Node::Instr((ref _name,ref _fmt)) => {
                            braces.last_mut().unwrap().1.map.insert(i,n);
                        },
                        Node::Map(map) => {braces.push((i,map));},
                    },
                    Err(why) => {return Err((why,ln))}
                }}


                lines_parsed+=1;
            },
            Err(why) => {
                return Err((ErrType::Internal(why),ln))
            }
        }
    }
    Err((ErrType::Other,ln))
}
