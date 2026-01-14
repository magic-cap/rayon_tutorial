use std::io::{Cursor};
use std::io::{BufReader};
use rayon_tutorial::*;
use rayon::prelude::*;
use getrandom;
use cipher::StreamCipherSeek;
fn main() -> Result<(), MagicCapError> {
    print_hello_world();
    let blocksize = 1024;
    let mut crud:Vec<u8> = vec![0u8; 5000];
    getrandom::fill(&mut crud)?;
    let c = Cursor::new(crud);
    let br = BufReader::new(c);
    let bri = BufReaderIterator::new(br,blocksize);
    let (mut key,_key_bytes) = make_key().unwrap();
    // this compiles and runs!
    // let v:Vec<()> = bri.into_iter().map(|(plaintext_block,bytes_read)| println!("{bytes_read}")).collect();
    let _v:Vec<()> = bri.into_iter()
        .enumerate()
        // .par_iter()
        .map(|(block_from_zero,(mut plaintext_block,bytes_read))|
             { // closure can use bindings from outer scope
                 let offset = blocksize * (block_from_zero + 1);
                 key.try_seek(offset).unwrap(); // unwrap is 😨
                 encryptor(&mut key,&mut plaintext_block);
                 println!("{bytes_read} {block_from_zero} real actual block number is {}",block_from_zero+1)
             }
        ).collect();
    Ok(())

}
