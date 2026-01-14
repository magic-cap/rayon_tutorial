use std::io;
use std::io::{BufReader, Read, Write};

pub fn print_hello_world() {
    let _ = io::stdout().write_all(b"Hello, world!\n");
}

/// BufReader Iterator, Read r => r -> BlockSize Int -> (PlainText ByteString, Size Int)
/// takes a blocksize and a Read or BufRead input. Self::Item is (Vec<u8>,amount_read)
#[derive(Debug)]
pub struct BufReaderIterator<R>
where
    R: Read,
{
    reader: BufReader<R>,
    blocksize: usize,
}

impl<R> BufReaderIterator<R>
where
    R: Read,
{
    pub fn new(reader: BufReader<R>, blocksize: usize) -> Self {
        Self { reader, blocksize }
    }
}

impl<R> Iterator for BufReaderIterator<R>
where
    R: Read,
{
    type Item = (Vec<u8>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.blocksize);
        buf.resize(self.blocksize, 0u8);
        let bytes_read = self.reader.read(&mut buf);
        match bytes_read {
            Ok(v) => {
                if v > 0 {
                    Some((buf, v))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor};
    use proptest::prelude::*;
    proptest!{
        #![proptest_config(ProptestConfig {
            cases: 5 as u32, .. ProptestConfig::default()
        })]
        #[test]
        fn bufreader_iterates(input: Vec<u8>, size in 5..30usize) {
            let c = Cursor::new(input);
            let br = BufReader::new(c);
            let mut bri = BufReaderIterator::new(br,size);
            let Some(_i) = bri.next() else {
                panic!("well that failed");
            };

        }
    }
}
