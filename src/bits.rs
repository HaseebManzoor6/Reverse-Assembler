/*
 * Wordt must be an unsigned numeric type large enough to store
 *  any word in instruction sets being parsed. At minimum it is 8 bytes.
 * 
 * Wordt must implement these traits: 
 *    Clone
 *
 * Wordt must have a from_str_radix function.
 * It must also have a field BITS which stores the type's size in bits
 *
 * u64 or u128 are the only tested types!
 */
pub type Wordt = u64;
/*
 * Signt is for formatting signed numbers.
 * It should be a signed numeric type with capacity (Number of bits)
 *  with identical size as Wordt, and the same trait restrictions.
 * 
 * Only i64 and i128 are tested.
 */
pub type Signt = i64;

/* Bitmask is an alias to Wordt */
pub type Bitmask = Wordt;

const MAXBIT: Wordt=1 << (Wordt::BITS-1);

pub enum BitOpType {
    AND,
    OR,
    XOR,
    SL, SR // bitshifts
}

/*
 * Represents performing operation <typ> with operand <val>
 *  onto some existing Wordt
 */
pub struct BitOp {
    pub typ: BitOpType,
    pub val: Wordt,
}

/*
 * Two's complement to obtain a signed integer from
 *  the first <size> bits of <w>
 */
pub fn twoscomp((w,size): (Wordt, Wordt)) -> Signt {
    // Copy bits from w to ret
    let ret: Signt = Signt::from_le_bytes(w.to_le_bytes());
    // sign bit=0 (positive)
    if 0==w&(1<<(size-1)) { return ret }
    else {
        //return (-1*(ret & ((1<<(size-1))-1)))+1
        return   -1*
                 ((!ret+1)      // negation
                 & (1<<size)-1) // ...of relevant bits
                 
    }
}

/*
 * Return <w>, shifted so that all bits are "under"
 *  the <bitmask>
 * <w> must already be under <mask>
 * TODO rework. Currently slow
 */
pub fn align(mut w: Wordt, mut mask: Bitmask) -> Wordt {
    let mut ret: Wordt=0;

    for _ in 0..(Wordt::BITS) {
        ret >>= 1;
        if mask&1 != 0 {
            if w&1 != 0 {ret |= MAXBIT;}
            w >>= 1;
        }
        mask >>= 1;
    }
    ret
}

/*
 * Return the bits of <n> under (Bitwise AND) <mask>,
 *  adjacent and as left shifted as much as possible.
 * Returns (<n> under <mask>, number of high bits in <mask>)
 */
// TODO rework. Currently very slow (64 loops every time)
pub fn minimize(mut w: Wordt, mut mask: Bitmask) -> (Wordt,Wordt) {
    let mut ret: Wordt=0;
    let mut size: Wordt=0; // mask size

    for _ in 0..(Wordt::BITS) {
        if mask&MAXBIT !=0 {
            ret<<=1;
            if 0!=w&MAXBIT { ret |= 1; }
            size+=1;
        }
        w<<=1;
        mask<<=1;
    }

    (ret,size)
}

/*
 * Reverse <size> bits in <w>
 */
pub fn reverse(w: &Wordt, size: usize) -> Wordt {
    let mut ret: Wordt = 0;
    for i in 0..(8*size) {
        ret<<=1;
        if w&(1<<i)!=0 {ret|=1;}
    }
    ret
}

/*
 * Make an unsigned number from little endian bytes
 */
pub fn wordt_from_le(bytes: &Vec<u8>) -> Wordt {
    let mut ret: Wordt=0;
    for i in 0..bytes.len() {
        ret|= (bytes[i] as Wordt) << (8*i)
    }
    return ret
}
/*
 * Make an unsigned number from big endian bytes
 */
pub fn wordt_from_be(bytes: &Vec<u8>) -> Wordt {
    let mut ret: Wordt=0;
    for i in 0..bytes.len() {
        ret|= (bytes[bytes.len()-i-1] as Wordt) << (8*i)
    }
    return ret
}


