use std::{
    io::{
        Read,
        BufReader,
        Seek, SeekFrom,
    },
    fs::File,
};

#[path="instrset.rs"]
pub mod instrset;

use instrset::{
    Instrset,
    Fmt,
};

use instrset::bits as bits;
use bits::{
    Wordt,
    minimize,
};

pub enum DeasmErr {
    NoOp,
    Internal,
    Unaligned,
}

pub fn print_deasm_err(i: u64, e: DeasmErr) {
    eprintln!("At {:#x}: {} ",i,match e {
        DeasmErr::NoOp => "Unknown instruction exists in the binary",
        DeasmErr::Internal => "Internal error",
        DeasmErr::Unaligned => "Size of binary file is not a multiple of wordsize",
    });
}

/*
 * TODO move to bits.rs?
 * Make an unsigned number from little endian bytes
 */
fn wordt_from_le(bytes: &Vec<u8>) -> Wordt {
    let mut ret: Wordt=0;
    for i in 0..bytes.len() {
        ret+= (bytes[i] as Wordt) << (8*i)
    }
    return ret
}

fn deassemble_instr(w: Wordt, is: &Instrset) -> Result<(),DeasmErr> {
    let mut mask_total: Wordt = 0;
    match instrset::get_fmt(w,&is.set,&mut mask_total) {
        None => {
            eprintln!("Unrecognized instruction: {:#b}",w);
            return Err(DeasmErr::NoOp)
        },
        Some((name,ifmt)) => {
            // TODO lock stdout during loop
            // print instruction
            print!("{}",name);
            for (f,m) in &ifmt.fmt {
                match f {
                    Fmt::Addr => {
                        print!(" {:#x}",minimize(w,*m).0);},
                    Fmt::Unsigned => {
                        print!( " {}",minimize(w,*m).0);},
                    Fmt::Signed   => {
                        print!( " {}",bits::twoscomp(minimize(w,*m)));
                    },
                }
                mask_total |= m;
            }
            // default formatter for instructions without format provided
            // Use it if mask_total is less than
            //  maximum possible word of size <is.wordsize>,
            //  i.e. 0b11111111 for 1 byte wordsize
            if mask_total < !0&((1<<is.wordsize*8)-1) {
                print!(" {:#x}",minimize(w,!mask_total).0);
            }
            println!();
            return Ok(())
        }
    }
}

pub fn deassemble_file(mut f: &File, is: &Instrset) -> Result<(),(u64,DeasmErr)>{
    // Read <is.wordsize> bytes at a time
    let mut reader=BufReader::new(f);
    let mut buffer=Vec::<u8>::with_capacity(is.wordsize);
    buffer.resize(is.wordsize,0);
    let len: u64;
    let mut w: Wordt;
    let ws: u64=is.wordsize.try_into().unwrap();

    // get filesize, in bytes
    if let Err(why)=f.seek(SeekFrom::End(0)) {
        eprintln!("Internal error in file seek: {}",why);
        return Err((0,DeasmErr::Internal))
    }
    match f.stream_position() {
        Ok(p)    => {len = p;},
        Err(why) => {
            eprintln!("Internal error fetching filesize: {}",why);
            return Err((0,DeasmErr::Internal))
        },
    }
    if let Err(why)=f.rewind() {
        eprintln!("Internal error in file seek: {}",why);
        return Err((0,DeasmErr::Internal))
    }
    // check if wordsize is ok
    if len%ws != 0 {return Err((0,DeasmErr::Unaligned))}

    // read every instruction
    for i in 0..(len/ws) {
        match reader.read_exact(&mut buffer) {
            Ok(()) => {
                w=wordt_from_le(&buffer);
                match deassemble_instr(w,&is) {
                    Ok(()) => {},
                    Err(e) => {return Err((i,e))}
                }
            },
            Err(why) => {
                eprintln!("Internal error while de-assembling: {}",why);
                return Err((i,DeasmErr::Internal))
            },
        }
    }

    Ok(())
}
