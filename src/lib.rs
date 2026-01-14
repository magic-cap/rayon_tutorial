use std::io;
use std::io::{BufReader, Read, Write};
use aes::cipher::{KeyIvInit, StreamCipher};
use ctr;
use getrandom;
use thiserror::Error;
use rayon::prelude::*;

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
impl<R> ParallelIterator for BufReaderIterator<R>
where R: Read + Send,
{
    type Item = (Vec<u8>, usize);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item> {
        todo!()
    }
}

// key pieces

type TahoeAesCtr = ctr::Ctr128BE<aes::Aes128>;
// mutates key and plain_text_block in place
pub fn encryptor(key: &mut TahoeAesCtr, plain_text_block: &mut Vec<u8>) -> () {
    key.apply_keystream(plain_text_block);
    // return plain_text_block; mutated in place!
}

pub fn make_key() -> Result<(TahoeAesCtr, [u8; 16]), MagicCapError> {
    let mut key_bytes = [0u8; 16];
    let _ = getrandom::fill(&mut key_bytes)?;
    let iv = [0u8; 16]; // 16 bytes of 0's
    let key = TahoeAesCtr::new(&key_bytes.into(), &iv.into());
    Ok((key, key_bytes))
}

                                                                                                                                                                                                                                                                                                                                            // error struct
#[derive(Error,Debug)]
pub enum MagicCapError {
    #[error("merkle root invalid, file integrity could not be verified.")]
    MerkleRootInvalid(#[source] rs_merkle::Error),
    #[error("Failed to base32 decode Key hash.")]
    HashInvalid(#[source] #[from] data_encoding::DecodeError),
    #[error("Random failed, good luck")]
    RandomError(#[source] #[from] getrandom::Error),
    #[error("File open/read/write/close failed")]
    FileError(#[source] #[from] std::io::Error),
    // #[error("CapnProto error, failed to read or write metadata")]
    // CapnProtoError(#[source] #[from] capnp::Error),
    #[error("Metadata hash does not match expected, do you have the wrong encrypted file?")]
    MerkleRootDoesNotMatch,

    // do we only get one single wrapper per concrete type? yes, unless wrapping in another enum!
    // or if you don't use source / from, which both call ~into~
    // #[error("Failed to base32 decode hash.")]
    // MetaHashInvalid(#[from] data_encoding::DecodeError),
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
        fn bufreader_iterates(input: Vec<u8>, blocksize in 5..30usize) {
            let c = Cursor::new(input);
            let br = BufReader::new(c);
            let mut bri = BufReaderIterator::new(br,blocksize);
            while let Some((chunk,read_amount)) = bri.next() {
                // final read_amount will be less than size
                let size_match = chunk.len() == blocksize || chunk.len() == read_amount;
                assert!(size_match);
            }
            // How do I combine while, let, and else?
            // } else {
            //     panic!("well that failed");
            // };

        }

        // #[test]
        // fn crypterator_iterates(input: Vec<u8>, blocksize in 5..30usize) {
        //     let c = Cursor::new(input);
        //     let br = BufReader::new(c);
        //     let mut bri = BufReaderIterator::new(br,blocksize);
        //     let (mut key,key_bytes) = make_key()?;
        //     // bri.into_iter().map(|plaintext_block

        // }
    }
}
