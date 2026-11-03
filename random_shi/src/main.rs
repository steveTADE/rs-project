use fastrand;
use rayon::prelude::*;
use std::{collections::HashMap, io::{self, Stdout, StdoutLock, Write, stdout}, string};
//use std::collections::HashMap;
const RED         : &str = "\x1b[91m";
const COLOR_RESET : &str = "\x1b[0m";
const SOFT_RESET  : &str = "\u{1b}[2J\u{1b}[3J";
const HIDE_CURSOR : &str = "\u{1b}[?25l";
const CURSOR_HOME : &str = "\x1b[H";
const ARRAY_LENGTH:usize = 100;

//use fastrand;
pub fn main() {
    let target:i32 = fastrand::i32(0..=100) * 37/10;
    let return_coins_list = get_return_change(target);
    let golden_ration_length:i8 = 10;
    let factorial_number = 5;

    println!("exchange : {} > coins : {:?}",  target as f32 / 10.0, return_coins_list);

    println!("golden ratio > {:?}", get_golden_ratio_list(golden_ration_length));

    print!("hailstone sequence > ");
    get_hailstone_sequence(9);

    println!("\n{} factorial > {}", factorial_number, factorial(factorial_number));

}

fn get_golden_ratio_list(n:i8) -> Vec<i8>{
    let mut golden_ratio_list:Vec<i8> = vec![];
    let mut a: i8 = 0;
    let mut b = 1;
    for _ in 0..n {
        let tmp = a;
        a  = b;
        b += tmp;
        golden_ratio_list.push(a);
    }
    return golden_ratio_list;
}

/*
fn get_recursive_golden_ratio_list(n:i32, a:i32, b:i32){
    // print!("{}, ",a);
    if n < 1 {
        return;
    }
    get_recursive_golden_ratio_list(n - 1, b, b + a);
}
*/

fn get_hailstone_sequence(n:i32){
    print!("{}, ", n);
    if n <= 1{
        return;
    }
    else if n%2 == 0{
        get_hailstone_sequence(n / 2);
    }
    else{
        get_hailstone_sequence(n * 3 + 1);
    }
}

fn factorial(n:i32) -> i32{
    let mut output:i32 = 1; 
    for i in 1..(n + 1){
        output *= i;
    }
    return output;
}

fn get_return_change(mut target:i32) -> Vec<f32>{
    let mut output:Vec<f32> = vec![];
    let coins = [100, 50, 20, 10, 5, 2, 1]; // faster than hashset when small

    let mut index = 0;
    while target > 0 && index < coins.len() {
        let coin = coins[index];
        if coin <= target {
            target -= coin;
            output.push(coin as f32 / 10.0);
        } else {
            index += 1;
        }
    }
    //println!("j*b has ended"); // debug
    return output;
}

fn sum_of_array(){
    let array_a : [i32; ARRAY_LENGTH] = (1..=ARRAY_LENGTH as i32).collect::<Vec<i32>>().try_into().unwrap();
    let output  :  i32                = array_a.iter().sum();
    let sigma_output: i32             = ((ARRAY_LENGTH * (ARRAY_LENGTH + 1)) / 2 )as i32;   
    println!("{}\n{}", output, sigma_output);
}

fn a_b_to_c(){
    let array_a : [i32; ARRAY_LENGTH] = (1..=ARRAY_LENGTH as i32).collect::<Vec<i32>>().try_into().unwrap();
    let array_b : [i32; ARRAY_LENGTH] = (1..=ARRAY_LENGTH as i32).rev().collect::<Vec<i32>>().try_into().unwrap();
    let mut array_c : [i32; ARRAY_LENGTH] = [0; ARRAY_LENGTH];
    array_c.par_iter_mut().enumerate().for_each(|(i, x)|{
        *x = array_a[i] + array_b[i] - 1;
    });
    println!("{:?}\n{:?}\n{:?}",array_a,array_b,array_c)
}

fn bucket_sort(target : [i32 ; ARRAY_LENGTH]) -> [i32; ARRAY_LENGTH]{
    let mut tmp = [0; ARRAY_LENGTH];
    let mut output = [0; ARRAY_LENGTH];
    let mut pos = 0;
    for num in target {
        tmp[num as usize] += 1;
    }
    for (index, count) in tmp.iter().enumerate() {
        for _ in 0..*count {
            output[pos] = index as i32;
            pos += 1;
        }
    }
    output
}

fn plurality_exercise(){
    let mut candidates:HashMap<String, i32> = HashMap::new();
    candidates.insert("Alice".to_string(), 0);
    candidates.insert("Bob".to_string(), 0);
    candidates.insert("Charlie".to_string(), 0);

    println!("{:?}", candidates);
    let first_question   : String = format!("number of voters > ");
    let second_question  : String = format!("voters > ");
    let number_of_voters : i32    = get_int_input(first_question);
    for _ in 0..number_of_voters{
        loop {
            let vote:String = get_str_input(second_question.clone());
            if vote == "clear"{
                println!("{}{}", SOFT_RESET, CURSOR_HOME);
                continue;
            }
            let is_candidate = candidates.get(&vote);
            if is_candidate == None{
                println!("{}invalid vote{}", RED, COLOR_RESET);
                continue;
            }
            if let Some(count) = candidates.get_mut(&vote) {
                *count += 1;
            }            
            println!("{:?}", candidates);
            break;
        }
    }
    let mut winner = format!("");
    let mut max = 0;
    for (calidate, votes) in candidates{
        if max < votes{
            max = votes;
            winner = calidate;
        }
    }
    println!("the winner is > {}", winner);

}

fn get_int_input(question : String) -> i32 {
    let stdout             : Stdout         = stdout();
    let mut lock           : StdoutLock<'_> = stdout.lock();
    loop {
        let mut input_text = String::new(); // reset the input_text
        write!(lock, "{}", question).unwrap();
        lock.flush().unwrap();

        io::stdin()
            .read_line(&mut input_text)
            .expect("failed to read from stdin");

        let trimmed = input_text.trim();

        match trimmed.parse::<String>(){ // clear the screen if the user thinks its too clustered
            Ok(s) => if s == "clear" {
                write!(lock, "{}{}", SOFT_RESET, CURSOR_HOME).unwrap();
                continue;
            }
        };

        match trimmed.parse::<i32>() {
            Ok(i) => return i,
            Err(..) => write!(lock, "{}error not the right input{}\n", RED, COLOR_RESET).unwrap(),
        };
    }
}

fn get_str_input(question : String) -> String {
    let stdout             : Stdout         = stdout();
    let mut lock           : StdoutLock<'_> = stdout.lock();
    let mut input_text = String::new(); // reset the input_text
    write!(lock, "{}", question).unwrap();
    lock.flush().unwrap();
    io::stdin()
        .read_line(&mut input_text)
        .expect("failed to read from stdin");
    let trimmed = input_text.trim();
    match trimmed.parse::<String>(){ // clear the screen if the user thinks its too clustered
        Ok(s) => return s
    }
}

fn string_reversal(){
    let output : String = format!("hello world").chars().rev().collect::<String>();
    print!("{:?}", output);
}

fn calculate_sum_of_multiples(){
    let target_list = vec![3, 5];
    let max_number:i64 = 1000;
    let mut output:i64 = 0;
    for num in target_list.iter(){
        output += multiple_sum(*num, max_number);
        //println!("{}", num);
    }
    output -= multiple_sum(target_list.iter().product(), max_number);
    println!("{}", output);

}

fn multiple_sum(target : i64, max_number : i64) -> i64{
    let mut all_numbers = vec![0];
    for number in 0..max_number{
        if number%target == 0{
            all_numbers.push(number);
        }
    }
    return all_numbers.iter().sum();
}

fn difference_between_sum_squared_and_squared_sum(){
    let list : Vec<i32> = (1..=100).collect();
    let first_output  = sum_squared(&list);
    let second_output = squared_sum(&list);
    println!("|{} - {}| = {}", first_output, second_output, (first_output - second_output).abs());
}

fn sum_squared(list : &Vec<i32>) -> i32 {
    let mut output = 0; 
    for num in list.iter(){
        output += (*num).pow(2);
    }
    output
}

fn squared_sum(list : &Vec<i32>) -> i32{
    let output:i32 = list.iter().sum();
    output.pow(2)
}

fn pascal_triangle_example() ->ExitCode{
    let mut pascal_tri_list : Vec<Vec<i32>> = vec![vec![1], vec![1, 1]];
    let number_rows: usize = 5;
    if number_rows < 2 {
        println!("{:?}debug", pascal_tri_list);
        return ExitCode::SUCCESS
    }

    pascal_tri_list = pascal_triangle(pascal_tri_list, number_rows);
    println!("{:?}", pascal_tri_list); // get the result

    ExitCode::SUCCESS // exit the program

}

fn pascal_triangle(mut pascal_tri_list: Vec<Vec<i32>>, target: usize) -> Vec<Vec<i32>> {
    for x in 2..target {
        let mut new_list: Vec<i32> = Vec::with_capacity(x + 1);
        new_list.push(1);
        for i in 1..x {
            new_list.push(pascal_tri_list[x - 1][i - 1] + pascal_tri_list[x - 1][i]);
        }
        new_list.push(1);
        pascal_tri_list.push(new_list);
    }
    pascal_tri_list
}

fn decimal_to_binary(decimal: u32) -> String {
    format!("{:b}", decimal)
}

fn square_brackets (all_the_text: &[&str], mut buffer: String) -> String{
    buffer.clear();
    let mut max_length = 0;
    for &text in all_the_text.iter() {
        max_length = max_length.max (text.len());
    }

    let push_text = |buffer: &mut String, insert_text: &str, gap_text: &str| {
        buffer.push_str (BLOCK);
        buffer.push_str (gap_text);
        buffer.push_str (insert_text);
        buffer.push_str (&gap_text.repeat(max_length - insert_text.len() + 1));
        buffer.push_str (BLOCK);
        buffer.push ('\n');
    };

    push_text (&mut buffer, &BLOCK.repeat(max_length), BLOCK);
    for &text in all_the_text.iter() {
        push_text (&mut buffer, text, GAP);
    }
    push_text (&mut buffer, &BLOCK.repeat(max_length), BLOCK);
    buffer
}

fn Prime_numbers_list(){
        let max_number = 100;
    let mut Output : Vec<i32> = vec![];
    let mut condition = true;
    for current_number in 0..max_number
    {
        condition = true;
        for index in 2..current_number{
            if current_number%index == 0{
                condition = false;
                break;
            }
        }
        if condition{
            Output.push(current_number);
        }
    }
    println!("{:?}", Output);

}

fn Container_With_Most_Water(){
    let height    : [i32; 9] = [1, 8, 6, 2, 5, 4, 8, 3, 7];
    let mut left  : usize    = 0;
    let mut right : usize    = height.len() - 1;
    let mut output: i32      = 0;
    while left < right{
        let length    : usize = right - left;
        let max_height: i32 = height[left].min(height[right]);
        let tmp       : i32   = (length as i32) * max_height;
        print!("{}\n", tmp); // debug
        if tmp > output {
            output = tmp;
        }
        if height[left] < height[right]{
            left += 1;
        }
        else {
            right -= 1;
        }
    }
    println!("{}", output);
}

