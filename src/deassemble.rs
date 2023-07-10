#[path="instrset.rs"]
pub mod instrset;

use instrset::{
    Instrset,
    Fmt,
};

use instrset::binreader::{
    Binreader,
};

use instrset::bits as bits;
use bits::{
    Wordt,
    minimize,
};

pub enum DeasmErr {
    NoOp,
    Internal,
}

pub fn print_deasm_err(i: u64, e: DeasmErr) {
    eprintln!("At {:#x}: {} ",i,match e {
        DeasmErr::NoOp => "Unknown instruction exists in the binary",
        DeasmErr::Internal => "Internal error",
    });
}

fn deassemble_instr(w: Wordt, is: &Instrset) -> Result<(),DeasmErr> {
    //eprintln!("deassemble: {:#b}",w);
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
                    Fmt::Binary => {
                        print!( " {:#b}",minimize(w,*m).0);
                    },

                    Fmt::Ubranch => {print!(" {:#x}",minimize(w,*m).0);},
                    Fmt::Dbranch => {print!(" {:#x}",minimize(w,*m).0);},
                    Fmt::Ibranch => {print!(" {:#x}",minimize(w,*m).0);},
                    Fmt::Sbranch => {print!(" {:#x}",minimize(w,*m).0);},

                    Fmt::Ignore => (),
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

pub fn deassemble_file(br: &mut Binreader, is: &Instrset) -> Result<(),(u64,DeasmErr)>{
    // read every instruction
    for i in 0..br.n_instrs() {
        match br.next() {
            Some(w) => {
                match deassemble_instr(w,&is) {
                    Ok(()) => {},
                    Err(e) => {return Err((i,e))}
                }
            },
            None => { return Err((i,DeasmErr::Internal)) },
        }
    }

    Ok(())
}
