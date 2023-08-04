use std::{
    fmt::Display,
    collections::BTreeSet,
};


#[path="instrset.rs"]
pub mod instrset;
use instrset::{
    binreader::Binreader,
    FmtType,
    Maskmap,
    bits as bits, bits::{
        Wordt, Bitmask,
    },
};



pub type BranchTree = BTreeSet<u64>;


pub enum GenLabelsErr {
    IO(u64),
    Unrecognized(u64, Wordt),
    IORewind(std::io::Error),
}
impl Display for GenLabelsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        match self {
            GenLabelsErr::IO(when) =>
                write!(f,"[At {:#x}] Internal I/O error: could not get next instruction",when),
            GenLabelsErr::Unrecognized(when,what) =>
                write!(f,"[At {:#x}] Unknown instruction: {:#x}",when,what),
            GenLabelsErr::IORewind(why) =>
                write!(f,"[After generating labels] Failed to rewind file: {}",why),
        }
    }
}



/*
 * Add any labels in the file wrapped by <br> who branch upwards to the
 *  labels set <tree>
 */
pub fn add_branch_ups(br: &mut Binreader, tree: &mut BranchTree, set: &Maskmap) -> Result<(),GenLabelsErr> {
    let mut mask: Bitmask=0; // just so get_fmt can track it
    let mut dest: (Wordt, Wordt);

    for i in 0..br.n_instrs { match br.next() {
        Some(w) => { match instrset::get_fmt(w, set, &mut mask) {
            Some((_name,ifmt)) => { for f in &ifmt.fmt {
                dest = bits::minimize(w,f.mask);

                bits::apply_bit_ops(f.ops.iter(), &mut dest.0);

                match &f.typ {
                    FmtType::Ubranch => {tree.insert(i-dest.0);},
                    FmtType::Sbranch => { if dest.0<i {tree.insert(dest.0);} },
                    FmtType::Ibranch => { if dest.0<=0 {tree.insert(
                                i
                                -bits::Wordt::from_le_bytes((-1*bits::twoscomp(dest)).to_le_bytes())
                                );}},

                    _other => (),
                }
            }},
            None => {
                return Err(GenLabelsErr::Unrecognized(i,w))
            },
        }},
        None => { return Err(GenLabelsErr::IO(i)) },
    }}

    match br.rewind() {
        Ok(()) => Ok(()),
        Err(why) => Err(GenLabelsErr::IORewind(why)),
    }
}
