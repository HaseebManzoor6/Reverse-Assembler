use std::collections::BTreeSet;


#[path="instrset.rs"]
pub mod instrset;
use instrset::{
    binreader::Binreader,
    FmtType,
    Maskmap,
    bits as bits, bits::{
        Wordt, Bitmask,
        BitOp, BitOpType,
    },
};



pub type BranchTree = BTreeSet<u64>;

// TODO move to bits/instrset?
pub fn apply_bit_ops(ops: &Vec<BitOp>, w: &mut Wordt) {
    for op in ops {*w=match op.typ {
        BitOpType::AND => *w&op.val,
        BitOpType::OR => *w|op.val,
        BitOpType::XOR => *w^op.val,
        BitOpType::SL => *w<<op.val,
        BitOpType::SR => *w>>op.val,
    }}
}

pub fn add_branch_ups(br: &mut Binreader, tree: &mut BranchTree, set: &Maskmap) -> bool {
    let mut mask: Bitmask=0; // just so get_fmt can track it
    let mut dest: (Wordt, Wordt);

    for i in 0..br.n_instrs { match br.next() {
        Some(w) => { match instrset::get_fmt(w, set, &mut mask) {
            Some((_name,ifmt)) => { for f in &ifmt.fmt {
                dest = bits::minimize(w,f.mask);

                apply_bit_ops(&f.ops, &mut dest.0);

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
                // TODO combine this with error handling in deassemble.rs
                eprintln!("At {:#x}: Unrecognized instruction: {:#b}",i,w);
                return false
            },
        }},
        None => { return false },
    }}
    return true
}
