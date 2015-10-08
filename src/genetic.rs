use std::cmp;
use std::mem;
use rand::{Rng,thread_rng};
use bit_vec::BitVec;
use expr;

const MAX_GENS: usize = 1000;
const CHROMOSOME_MIN: usize = 3;
const CHROMOSOME_MAX: usize = 101;
const MUTATION_RATE: f64 = 0.01;
const CROSSOVER_RATE: f64 = 0.70;
const EPSILON: f64 = 1e-9;

/// A single phenotype.
#[derive(Clone)]
// See the impl below
pub struct Chromosome {
    pub bits: BitVec,
    pub fitness: f64
}

fn randrange(lo: f64, hi: f64) -> f64 { thread_rng().gen_range(lo, hi) }

fn randbit() -> bool { randrange(0.0, 1.0) < 0.5 }

/// Convert a number from its binary representation in a BitVec to a usize.
pub fn from_binary(b: &BitVec) -> usize {
    let bytes = b.to_bytes();
    let n = bytes.len();
    let mut acc: usize = 0;
    for (i, byte) in bytes.iter().enumerate() {
        acc |= (*byte as usize) << (n - 1 - i) * 8;
    }
    acc
}

/// Convert a number to its binary representation.
pub fn to_binary(x: usize) -> BitVec {
    let mut n = x;
    let mut bytes: Vec<u8> = Vec::new();
    while (n > 0) {
        bytes.push((n & 0xff) as u8);
        n >>= 8;
    }
    BitVec::from_bytes(&bytes)
}

/// Return a string of 0s and 1s, given a BitVec.
pub fn bitstring(b: &BitVec) -> String {
    let mut ret = String::new();
    for bit in b.iter() {
        ret.push(if bit { '1' } else { '0' });
    }
    ret
}


/// Decodes a 4 bit number to a string symbol it represents. Returns the empty
/// string for invalid numbers.
/// For n from 0 through 9, returns the string representation of the digit.
/// For n = 10 through 14, the operators "+", "-", "*", "/", "**" are returned
/// in that order.
fn get_symbol(n: u8) -> String {
    match n {
        0 ... 9 => n.to_string(),
             10 => String::from("+"),
             11 => String::from("-"),
             12 => String::from("*"),
             13 => String::from("/"),
             14 => String::from("**"),
              _ => String::from(""),
    }
}


/// Decodes a bitvec into an expression. Note that the expression returned
/// may very well be malformed. All this function does is go over bit
/// quadruplets, substituting each with the value returned from `get_symbol()`.
fn decode(b: &BitVec) -> String {
    let mut e = String::new();
    for byte in b.to_bytes().iter() {
        let hi = 0xf & ((*byte as i32) >> 4);
        let lo = 0xf & (*byte as i32);
        e.push_str(&get_symbol(hi as u8));
        e.push_str(&get_symbol(lo as u8));
    }
    e
}

/// Try to evaluate the expression encoded in a bit vector and return it.
fn value(b: &BitVec) -> Option<f64> { expr::eval(&decode(b)).ok() }

/// Roulette select a chromosome from a population.
fn select<'a>(population: &'a [Chromosome], total_fitness: f64) -> &'a Chromosome {
    loop {
        let slice = randrange(0.0, 1.0) * total_fitness;
        let mut acc = 0f64;
        for c in population {
            acc += c.fitness;
            if acc >= slice {
                return c;
            }
        }
    }
}


impl Chromosome {
    /// Construct a new Chromosome from a bit pattern and a target number.
    pub fn new(bits: BitVec, target: f64) -> Chromosome {
        let fitness = value(&bits)
                      .map(|v| -> f64 {
                          // NaN can result because of a divide by zero.
                          if v.is_nan() {
                              0f64
                          } else {
                              1f64 / (1f64 + (v - target).abs())
                          }
                      })
                      .unwrap_or(0f64);
        Chromosome { bits: bits, fitness: fitness }
    }

    /// Construct a Chromosome with a random bit pattern, given a target number.
    pub fn random(target: f64) -> Chromosome {
        let size = thread_rng().gen_range(CHROMOSOME_MIN, CHROMOSOME_MAX) * 4;
        let bits = BitVec::from_fn(size, |_| randbit());
        Chromosome::new(bits, target)
    }

    /// Return the expression (possibly malformed) represented by this chromosome.
    pub fn decode(&self) -> String { decode(&self.bits) }

    /// Return the value that the expression encoded by this chromosome evaluates
    /// to. If the encoded expression is malformed, return None.
    pub fn value(&self) -> Option<f64> { value(&self.bits) }

    /// Crossover two chromosomes according to CROSSOVER_RATE.
    /// This is one cause of variation in the gene pool.
    pub fn crossover(&self, them: &Chromosome, target: f64) -> (Chromosome, Chromosome) {
        if randrange(0.0, 1.0) >= CROSSOVER_RATE {
            return ((*self).clone(), (*them).clone());
        }

        let m = self.bits.len();
        let n = them.bits.len();
        let k = cmp::max(m, n);
        let lim = thread_rng().gen_range(0, k);

        let mut b1 = BitVec::new();
        for i in 0..cmp::min(m, lim+1) {
            b1.push(self.bits.get(i).unwrap()); 
        }

        let mut b2 = BitVec::new();
        for i in 0..cmp::min(n, lim+1) {
            b2.push(them.bits.get(i).unwrap());
        }

        for i in lim..k {
            if i < m {
                b2.push(self.bits.get(i).unwrap());
            }
            if i < n {
                b1.push(them.bits.get(i).unwrap());
            }
        }

        (Chromosome::new(b1, target), Chromosome::new(b2, target))
    }

    /// Return a mutated chromosome, according to MUTATION_RATE.
    /// This is another cause for variation in the gene pool (the other
    /// being crossover), although mutations are comparatively very, very
    /// rare (as reflected in the MUTATION_RATE constant).
    pub fn mutate(&self, target: f64) -> Chromosome {
        let b: BitVec = self.bits.iter().map(|bit| -> bool {
            if randrange(0f64, 1f64) <= MUTATION_RATE { !bit } else { bit }
        }).collect();
        Chromosome::new(b, target)
    }
}

/// Breed one generation of chromosomes and return the new population.
fn ga_epoch(population: &[Chromosome], target: f64) -> Vec<Chromosome> {
    let fitness: f64 = population.iter()
                                 .map(|c| c.fitness)
                                 .fold(0f64, |a, b| a + b);
    let mut new_population = Vec::new();
    loop {
        let (c1, c2) = select(&population, fitness).crossover(
            select(&population, fitness),
            target
        );
        let (c1, c2) = (c1.mutate(target), c2.mutate(target));
        new_population.push(c1);
        new_population.push(c2);
        if new_population.len() >= population.len() {
            break;
        }
    }    
    new_population
}

pub fn ga(popsize: usize, target: f64) -> (usize, Option<Chromosome>) {
    let mut pop = Vec::new();
    for i in 0..popsize {
        pop.push(Chromosome::random(target));
    }

    for i in 0..MAX_GENS {
        if i % 10 == 9 || i + 10 >= MAX_GENS {
            println!("Generation {} of {}", i+1, MAX_GENS);
        }
        for c in pop.iter() {
            if (1f64 - c.fitness).abs() <= EPSILON {
                return (i, Some(c.clone()))
            }
        }
        pop = ga_epoch(&pop, target);
    }
    (MAX_GENS, None)
}

