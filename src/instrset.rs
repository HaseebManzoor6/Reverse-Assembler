/*
* instrset.rs
* structs for representing an instructions set
*/
use std::collections::HashMap;

#[path="bits.rs"]
pub mod bits;

use bits::{
    Wordt,
    Bitmask,
};

/*
 * Formatting types for printing parts of an instruction.
 */
pub enum Fmt {
    Addr,
    Signed,
    Unsigned,
    Binary,
    Ignore,
}

/*
 * An Instrfmt describes the parameters an instruction takes.
 * The opcode is not included.
 * fmt is a list of Fmt objects.
 *  For example an instruction which adds the value at an address with
 *  a constant and places their sum in a third address might have fmt:
 *   [(Fmt::Addr,0b...), (Fmt::Signed,0b...), (Fmt::Addr,0b...)]
 */
pub struct Instrfmt {
    pub fmt: Vec<(Fmt,Bitmask)>,
}

pub enum Node {
    Map(Maskmap),
    Instr((String,Instrfmt))
}

// Bit-masking hashmap. Keys are words under the bitmask, vals are &Node
pub struct Maskmap {
    pub mask: Bitmask,
    pub map: HashMap<Wordt, Node>
}

pub struct Instrset {
    pub wordsize: usize,
    pub endian_little: bool, // if false use big endian
    pub set: Maskmap,
}



/*
 * Given a Wordt and an instructions set,
 *  return the matching instruction name and Instrfmt
 * Apply (Bitwise OR) all bitmasks searched to *<mask_total>
 */
pub fn get_fmt<'a>(w: Wordt, mut set: &'a Maskmap, mask_total: &mut Bitmask) -> Option<&'a (String,Instrfmt)> {
    loop {
        *mask_total |= set.mask;
        match set.map.get(&(w&set.mask)) {
            None => {return None},
            Some(n) => match n {
                Node::Instr(tup) => {return Some(&tup)},
                Node::Map(m)   => {set=&m;}
            }
        }
    }
}
