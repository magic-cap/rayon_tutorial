use cipher::StreamCipherSeek;
use getrandom;
use rayon::prelude::*;
use rayon_tutorial::*;

use std::fs::{File};
use std::io::prelude::*;
use std::io::{BufReader, Cursor, SeekFrom};
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

// use rustix::fs::FileExt;
// use std::mem;
// use std::thread;

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
    let outer_file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("foo").unwrap()));
    let mut index_hash:Vec<(usize,[u8;32])> = bri.into_iter()
        .enumerate()
        .map(|(block_from_zero,(plaintext_block,bytes_read))| (block_from_zero,(plaintext_block,bytes_read), Arc::clone(&outer_file)))
        .par_bridge()
        .map(|(block_from_zero,(mut plaintext_block,bytes_read),inner_file)|
             { // closure can use bindings from outer scope
                 let offset = blocksize * (block_from_zero + 1);
                 let mut key = key_from_bytes(key_bytes);
                 key.try_seek(offset).expect("this only fails if we encrypt truly massive files, what?");
                 // let inner_file: Arc<Mutex<File>> = Arc::clone(&outer_file);
                 encryptor(&mut key,&mut plaintext_block);
                 let crypt_block = plaintext_block; // it's been encrypted in-place
                 println!("{bytes_read} {block_from_zero} real actual block number is {}",block_from_zero+1);
                 // XXX doesn't write yet! will that work in parallel!?
                 let result = (block_from_zero + 1,hash_leaf(&crypt_block));
                 let mut actual_file: std::sync::MutexGuard<'_, File> = inner_file.lock().unwrap();
                 // actual_file.seek(SeekFrom::Start(0)).unwrap();
                 write_out_of_order(actual_file.deref_mut(), offset as u64, crypt_block);
                 return result;

             }
        ).collect();
    index_hash.sort(); // get 'em back in order
    for (block,hash) in index_hash {
        println!("block number {block} has hash {hash:?}");
    }
    Ok(())
}

// is it better to calculate the offset ahead of time? 🤔
fn write_out_of_order(f: &mut File, offset: u64, encrypted_block: Vec<u8>) -> Result<(), MagicCapError> {
    f.seek(SeekFrom::Start(offset))?;
    f.write(&encrypted_block)?;
    Ok(())
}
/*
const COUNT: usize = 1000;

fn main() {
    let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("foo.txt").unwrap()));
    static sfile = file;
    let v: [u32; COUNT] = [6; COUNT];

    let counter = Arc::new(Mutex::new(0));

    let mut threads = Vec::new();
    for _ in 0..COUNT {
        // let counter = Arc::clone(&counter);
        // let file = Arc::clone(&file);

        let thread = thread::spawn(|| {
            let counter = Arc::clone(&counter);
            let file = Arc::clone(&file);

            let mut i = counter.lock().unwrap();
            let file = file.lock().unwrap();

            let offset = (mem::size_of::<u32>() * (*i)) as u64;
            let bytes = unsafe { mem::transmute::<u32, [u8; 4]>(v[*i]) };
            file.write_all_at(&bytes, offset).unwrap();

            *i += 1;
        });
        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
}

// fn write_out_of_order() -> Result<(),MagicCapError> {
//     let mut f = File::create_new("foo")?;
//     f.seek(SeekFrom::Start(512))?;
//     f.write(b"22222222")?;
//     f.seek(SeekFrom::Start(0))?;
//     f.write(b"1111")?;
//     Ok(())
// }
*/
