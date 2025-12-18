use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::fs::read_to_string;
use std::str::Split;

fn main() {
    // read a csv file to a string
    let my_string = read_to_string("tmp182oee02.csv").unwrap();

    let my_vec: Vec<String> = my_string
        .lines()
        .filter(|x| *x != String::new())
        .map(|x| x.to_string())
        .collect();
    println!("{my_vec:?}");

    let my_vec: Vec<Vec<String>> = my_vec
        .into_par_iter()
        .map(|x| {
            let x = x.split(",").map(|x| x.to_string()).collect::<Vec<String>>();
            let res: Vec<String> = x
                .into_par_iter()
                .map(|_| "Hello world!".to_string())
                .collect();
            res
        })
        .collect();

    // this should now print a vec of vecs
    // where every single value is the "Hello world!" string
    println!("{:?}", my_vec);
}
