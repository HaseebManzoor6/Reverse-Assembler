/*
 * file.rs - helpers for reading binary files
 */
use std::{
    io::{
        BufReader,
        Seek, SeekFrom,
        Read,
    },
    fs::File,
};

#[path="bits.rs"]
pub mod bits;
pub use bits::Wordt as Wordt;

pub struct Binreader {
    buffer: Vec<u8>,
    reader: BufReader<File>,
    pub n_instrs: u64,
    endian_little: bool,
}

impl Binreader {
    /*
     * Next word from file.
     * May fail on internal file errors
     */
    pub fn next(&mut self) -> Option<Wordt> {
        match self.reader.read_exact(&mut self.buffer) {
            Ok(()) => {
                return Some(match self.endian_little {
                    true  => bits::wordt_from_le(&self.buffer),
                    false => bits::wordt_from_be(&self.buffer),
                })
            },
            Err(why) => {
                eprintln!("Internal error reading binary file: {}",why);
                return None
            },
        }
    }

    /*
     * Rewind binary file back to the start
     */
    pub fn rewind(&mut self) -> Result<(),std::io::Error> {
        self.reader.rewind()
    }

    /*
     * New Binary file reader with given wordsize
     * May fail if:
     *  - wordsize cannot fit into a u64
     *  - Error opening file
     *  - file size cannot be read
     *  - file size is not a multiple of wordsize
     */
    pub fn new(wordsize: usize, filepath: &String, endian_little: bool) 
    -> Option<Binreader> {
        let fsize: u64;
        let ws: u64= match wordsize.try_into() {
            Ok(u) => u,
            Err(_) => {return None},
        };

        // open file
        let mut f: File = match File::open(filepath) {
            Ok(file) => file,
            Err(why) => {
                eprintln!("Couldn't open binary file {}: {}",
                          filepath, why);
                return None
            },
        };


        // get filesize in bytes
        if let Err(why)=f.seek(SeekFrom::End(0)) {
            eprintln!("Internal error in file seek: {}",why);
            return None
        }
        match f.stream_position() {
            Ok(p)    => {fsize = p;},
            Err(why) => {
                eprintln!("Internal error fetching filesize: {}",why);
                return None
            },
        }
        if let Err(why)=f.rewind() {
            eprintln!("Internal error in file seek: {}",why);
            return None
        }

        // check if wordsize is ok
        if fsize%ws != 0 {
            eprintln!("Filesize ({}) is not a multiple of wordsize ({})",fsize,ws);
            return None
        }

        // create buffer
        let mut buffer=Vec::<u8>::with_capacity(wordsize);
        buffer.resize(wordsize,0);

        Some(Binreader {
            buffer: buffer,
            reader: BufReader::new(f),
            n_instrs: fsize/ws,
            endian_little: endian_little,
        })
    }
}
