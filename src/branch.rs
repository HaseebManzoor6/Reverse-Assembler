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
    Unrecognized(u64, Wordt),
    Binread(u64,instrset::binreader::BinReaderErr),
}
impl Display for GenLabelsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        match self {
            GenLabelsErr::Unrecognized(when,what) =>
                write!(f,"[At {:#x}] Unknown instruction: {:#x}",when,what),
            GenLabelsErr::Binread(when,why) =>
                write!(f,"[At {:#x}] I/O error: {}",when,why),
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
        Ok(w) => { match instrset::get_fmt(w, set, &mut mask) {
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
        Err(why) => { return Err(GenLabelsErr::Binread(i,why)) },
    }}

    match br.rewind() {
        Ok(()) => Ok(()),
        Err(why) => Err(GenLabelsErr::Binread(br.n_instrs,why)),
    }
}
