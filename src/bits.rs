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

/*
 * Two's complement to obtain a signed integer from
 *  the first <size> bits of <w>
 */
// TODO not correct
pub fn twoscomp((w,size): (Wordt, Wordt)) -> Signt {
    // Copy bits from w to ret
    // TODO unsafe transmute instead? Need to handle size difference
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

#[cfg(test)]
mod bits_tests {
    use crate::parse::bits;
    #[test]
    fn test_minimize() {
        let result=bits::minimize(0b10000011,0b11000011);
        let maxbit: bits::Wordt = 1 << (bits::Wordt::BITS-1);
        assert!(result.0==0b1011 && result.1==4,
            "Actual: ({:#b},{}); MAXBIT={:#b}",result.0,result.1,maxbit);
    }

    #[test]
    fn test_align() {
        let result=bits::align(0b1011,0b11000011);
        assert!(result==0b10000011,
            "Actual: {:#b}",result);
    }

    #[test]
    fn test_twoscomp() {
        let result=bits::twoscomp((0b1011,4));
        assert!(result== -5,
                "Actual: {:#b}={}",result,result);
        assert!(bits::twoscomp((0b011,3))==3);
    }
}
