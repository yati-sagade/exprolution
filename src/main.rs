extern crate rand;
extern crate exprolution;
use std::env;
use rand::Rng;
use exprolution::expr;
use exprolution::genetic::{Chromosome,ga};


fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Need a number");
        return;
    }
    let num = args[1].parse::<f64>().unwrap();
    match ga(500, num) {
        (ngens, Some(ref c)) => {
            println!("Found a solution in {} generations:", ngens);
            println!("\t{}", c.decode());
        },
        (ngens, None) => {
            println!("Could not find a solution in {} generations.", ngens);
        }
    };    
}

