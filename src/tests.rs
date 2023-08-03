/*
 * NOTE: This file currently does not get loaded anywhere
 * It's just to "keep" testcases around that have already passed
 */

#[cfg(test)]
mod bits_tests {
    use crate::parse::bits;

    #[test]
    fn test_reverse() {
        let a: bits::Wordt=0b01000000000000000000000000000001;
        let result=bits::reverse(&a,4);
        assert!(result==0b10000000000000000000000000000010,
        "actual: {:#b}",result);
    }
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

#[cfg(test)]
mod genmask_tests {
    use crate::parse;
    #[test]
    fn test_genmask() {
        let result: (parse::Bitmask, usize);
        if let Some(m)=parse::gen_mask(
                &vec!["mask","0b1101"],
                0, 0) {
            result=m;
        }
        else {assert!(0==1); return}
        assert!(
            result == (0b1101,2),
            "Actual: ({:#b},{})",result.0,result.1
            );
    }

    #[test]
    fn test_genmask_bits() {
        let result: (parse::Bitmask, usize);
        if let Some(m)=parse::gen_mask(
                &vec!["0+1:2+15"],
                0, 2) {
            result=m;
        }
        else {assert!(0==1); return}
        assert!(
            result == (0b1110000000000001,1),
            "Actual: ({:#b},{})",result.0,result.1
            );
    }
}

