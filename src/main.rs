use std::io::{Cursor};
use std::io::{BufReader,SeekFrom};
use std::io::prelude::*;
use std::fs::{File};
use rayon_tutorial::*;
use rayon::prelude::*;
use getrandom;
use cipher::StreamCipherSeek;
fn main() -> Result<(), MagicCapError> {
    // write_out_of_order()?;
    // Ok(())
    print_hello_world();
    let blocksize = 1024;
    let mut crud:Vec<u8> = vec![0u8; 5000];
    getrandom::fill(&mut crud)?;
    let c = Cursor::new(crud);
    let br = BufReader::new(c);
    let bri = BufReaderIterator::new(br,blocksize);
    let key_bytes = new_key_bytes().unwrap();
    // this compiles and runs!
    // let v:Vec<()> = bri.into_iter().map(|(plaintext_block,bytes_read)| println!("{bytes_read}")).collect();
    let mut index_hash:Vec<(usize,[u8;32])> = bri.into_iter()
        .enumerate()
        .par_bridge()
        .map(|(block_from_zero,(mut plaintext_block,bytes_read))|
             { // closure can use bindings from outer scope
                 let offset = blocksize * (block_from_zero + 1);
                 let mut key = key_from_bytes(key_bytes);
                 key.try_seek(offset).expect("this only fails if we encrypt truly massive files, what have you done?");
                 encryptor(&mut key,&mut plaintext_block);
                 let crypt_block = plaintext_block; // it's been encrypted in-place
                 println!("{bytes_read} {block_from_zero} real actual block number is {}",block_from_zero+1);
                 // XXX doesn't write yet! will that work in parallel!?
                 return (block_from_zero + 1,hash_leaf(&crypt_block));
             }
        ).collect();
    index_hash.sort(); // get 'em back in order
    for (block,hash) in index_hash {
        println!("block number {block} has hash {hash:?}");
    }
    Ok(())

}

fn write_out_of_order() -> Result<(),MagicCapError> {
    let mut f = File::create_new("foo")?;
    f.seek(SeekFrom::Start(512))?;
    f.write(b"22222222")?;
    f.seek(SeekFrom::Start(0))?;
    f.write(b"1111")?;
    Ok(())

}
