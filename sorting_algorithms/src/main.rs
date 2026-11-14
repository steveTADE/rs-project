use std::{fmt::NumBuffer, time::Instant};
use fastrand;
use rayon::prelude::*;
use std::{collections::HashMap, io::{self, Stdout, StdoutLock, Write, stdout}, string};
//use std::collections::HashMap;
const RED         : &str = "\x1b[91m";
const COLOR_RESET : &str = "\x1b[0m";
const SOFT_RESET  : &str = "\u{1b}[2J\u{1b}[3J";
const HIDE_CURSOR : &str = "\u{1b}[?25l";
const CURSOR_HOME : &str = "\x1b[H";
type IntegerType = i16;
const ARRAY_LENGTH : usize = IntegerType::MAX as usize;

fn main() {
    let something: [IntegerType; ARRAY_LENGTH] = std::array::from_fn(|_| fastrand::i16(0..IntegerType::MAX));

    let start = Instant::now();
    let _bucket_output = bucket_sort(something);
    let bucket_time = start.elapsed();

    let mut sort_output = something;
    let start = Instant::now();
    sort_output.sort();
    let sort_time = start.elapsed();

    println!("bucket sort time > ({:?})", bucket_time);
    println!("standard sort time > ({:?})", sort_time);
}
#[inline(always)]
fn bucket_sort(target : [IntegerType ; ARRAY_LENGTH]) -> [IntegerType; ARRAY_LENGTH]{
    let mut tmp = [0; IntegerType::MAX as usize];
    let mut output : [IntegerType; ARRAY_LENGTH] = [0; ARRAY_LENGTH];
    let mut start = 0;
    for num in target {
        tmp[num as usize] += 1;
    }
    for (index, count) in tmp.iter().enumerate() {
            let end = start + count;
            output[start..end].fill(index as IntegerType);
            start = end;
    }
    output
}

fn display(content : [IntegerType ; ARRAY_LENGTH]){
    
    std::write!(content);
}