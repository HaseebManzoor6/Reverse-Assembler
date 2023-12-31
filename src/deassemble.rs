use std::fmt::Display;

#[path="branch.rs"]
pub mod branch;
pub use branch::{
    instrset as instrset, instrset::{
        Instrset,
        FmtType,
        binreader::Binreader,
        bits as bits, bits::{
            Wordt,
            minimize,
        },
    }
};

enum DeasmErrType {
    UnknownOp(Wordt),
    Binread(instrset::binreader::BinReaderErr),
}
pub struct DeasmErr {
    typ: DeasmErrType,
    words_read: Wordt,
}
impl Display for DeasmErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        match &self.typ {
            DeasmErrType::UnknownOp(w) =>
                write!(f, "[At {:#x}] Unknown instruction: {:#x}",self.words_read,w),
            DeasmErrType::Binread(why) =>
                write!(f,"[At {:#x}] I/O error: {}",self.words_read,why),
        }
    }
}

fn deassemble_instr(w: Wordt, is: &Instrset, tree: &mut branch::BranchTree, i: &u64) -> Result<(),DeasmErr> {
    let mut mask_total: bits::Bitmask = 0;
    let mut d: (Wordt,Wordt); // data under current Fmt mask

    match instrset::get_fmt(w,&is.set,&mut mask_total) {
        None => {
            return Err(DeasmErr {
                typ: DeasmErrType::UnknownOp(w),
                words_read: 0, // will be set by deassemble_file
            })
        },
        Some((name,ifmt)) => {
            // TODO lock stdout during loop
            // print instruction
            print!("{}",name);
            for f in &ifmt.fmt {
                // Apply BitOps
                d=minimize(w,f.mask);
                bits::apply_bit_ops(f.ops.iter(),&mut d.0);

                match &f.typ {
                    FmtType::Addr => {
                        print!(" {:#x}",d.0);},
                    FmtType::Unsigned => {
                        print!( " {}",d.0);},
                    FmtType::Signed   => {
                        print!( " {}",bits::twoscomp(d));
                    },
                    FmtType::Binary => {
                        print!( " {:#b}",d.0);
                    },

                    FmtType::Ubranch => {print!(" label_{:#x}",(*i)-d.0);},
                    FmtType::Dbranch => {
                        print!(" label_{:#x}",(*i)+d.0);
                        tree.insert((*i)+d.0);
                    },
                    FmtType::Ibranch => {
                        if d.0>0 {
                            print!(" label_{:#x}",(*i)+d.0);
                            tree.insert(( *i)+d.0 );
                        }
                        else {
                            print!(" label_{:#x}",
                                (*i)-Wordt::from_le_bytes((-1*bits::twoscomp(d)).to_le_bytes())
                            )
                        }
                    },
                    FmtType::Sbranch => {
                        print!(" label_{:#x}",d.0);
                        if d.0>=(*i) {tree.insert(d.0);}
                    },

                    FmtType::Ignore => (),
                }
                mask_total |= f.mask;
            }
            // default formatter for instruction parts without format provided
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

pub fn deassemble_file(br: &mut Binreader, is: &Instrset, tree: &mut branch::BranchTree) -> Result<(),DeasmErr>{
    // read every instruction
    for i in 0..br.n_instrs {
        // check to generate labels
        match tree.first() {
            Some(n) => {
                if i==*n {
                    println!("label_{:#x}:",i);
                    tree.pop_first();
                }
            },
            None => (),
        }

        // deassemble instruction
        match br.next() {
            Ok(w) => { match deassemble_instr(w,&is,tree,&i) {
                Ok(()) => (),
                Err(mut e) => {
                    e.words_read=i;
                    return Err(e)
                }
            }},
            Err(why) => { return Err(DeasmErr {
                typ: DeasmErrType::Binread(why),
                words_read: i,
            })},
    }}

    Ok(())
}
