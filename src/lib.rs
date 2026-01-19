use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek, StreamCipherError};
use ctr;
use getrandom;
use bitcoin_hashes::{HashEngine, sha256d};
use rs_merkle::{Hasher, MerkleTree};
use std::io::{BufReader, Read, Write,stdout};
use thiserror::Error;
// use rayon::prelude::*;

pub fn print_hello_world() {
    let _ = stdout().write_all(b"Hello, world!\n");
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


/*
// don't know how to do this part
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
*/

// key pieces

type TahoeAesCtr = ctr::Ctr128BE<aes::Aes128>;
// mutates key and plain_text_block in place
pub fn encryptor(key: &mut TahoeAesCtr, plain_text_block: &mut Vec<u8>) -> () {
    key.apply_keystream(plain_text_block);
    // return plain_text_block; mutated in place!
}

pub fn new_key() -> Result<(TahoeAesCtr, [u8; 16]), MagicCapError> {
    let iv = [0u8; 16]; // 16 bytes of 0's
    let key_bytes = new_key_bytes()?;
    let key = TahoeAesCtr::new(&key_bytes.into(), &iv.into());
    Ok((key, key_bytes))
}

pub fn new_key_bytes() -> Result<[u8;16], MagicCapError> {
    let mut key_bytes = [0u8; 16];
    let _ = getrandom::fill(&mut key_bytes)?;
    Ok(key_bytes)
}

pub fn key_from_bytes(key_bytes: [u8; 16]) -> TahoeAesCtr {
    let iv = [0u8; 16];
    TahoeAesCtr::new(&key_bytes.into(), &iv.into())
}

pub fn key_from_bytes_with_offset(key_bytes: [u8; 16],offset: usize) -> Result<TahoeAesCtr,MagicCapError> {
    let iv = [0u8; 16];
    let mut key = TahoeAesCtr::new(&key_bytes.into(), &iv.into());
    key.try_seek(offset)?;
    Ok(key)

}

// merkle tree things
pub fn hash_leaf(leaf: &Vec<u8>) -> [u8; 32] {
    TahoeLeaf::hash(leaf.as_slice())
}

// from binrw-tahoe experiments -- mirroring the Tahoe way of
// using tagged hashes for merkel nodes, with different tags for
// leaves vs. interior vs. empty nodes.
#[derive(Clone)]
pub struct TahoeLeaf {}

impl Hasher for TahoeLeaf {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        //why not "Hash" as return type?
        //let mut engine = sha256d::Hash::engine();
        //engine.input(data);
        //sha256d::Hash::from_engine(engine).to_byte_array()
        let hashed = hash_things(data, b"allmydata_crypttext_segment_v1");
        // let hash = tagged_hash::<32>(, data);
        let mut ret = [0; 32];
        ret.copy_from_slice(hashed.as_slice());
        ret
    }
}

fn hash_things(data: &[u8],tag: &[u8]) -> [u8; 32] {
    let hash = tagged_hash::<32>(tag, data);
    let mut ret = [0; 32];
    ret.copy_from_slice(hash.as_slice());
    ret
}

// todo: Chris' code had "truncate_to" as an arg ... and then we
// wnated to do that as const-generics ... but "sha256d" _is_ just
// always 32 bytes so what does the truncate_to even do?
// tagged_hash<16>
pub fn tagged_hash<const TAGSIZE: usize>(tag: &[u8], val: &[u8]) -> [u8; TAGSIZE] {
    if TAGSIZE > 32 {
        panic!("illegal tag size");
    }
    let mut engine = sha256d::Hash::engine();
    engine.input(&netstring(tag));
    engine.input(val);
    let raw = *sha256d::Hash::from_engine(engine).as_byte_array();
    let mut rtn: [u8; TAGSIZE] = [0u8; TAGSIZE];
    rtn.copy_from_slice(&raw[0..TAGSIZE]);
    return rtn;
}

// pulled from "lafs"
pub fn netstring(s: &[u8]) -> Vec<u8> {
    //format!("{}:{},", s.len(), std::str::from_utf8(s).unwrap()).into_bytes()

    // what Python does is output BYTES here, where we have some
    // number of ASCII-numeral bytes that represent the length, then a
    // ':' byte, and then 32 arbitrary bytes of key
    let tag = format!("{}:", s.len());
    // stuff two byte-sequences together; better way?
    [tag.as_bytes(), s, b","].concat()
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
    #[error("Cipher seek failed")]
    StreamCipherError(#[from] StreamCipherError) // XXX exactly what trait bounds are missing for #[source] and #[from] ?

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
