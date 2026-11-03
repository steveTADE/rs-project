use rayon::prelude::*;
use std::thread;
use std::time::Duration;
use fastrand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
//use std::env;


//TODO: make grid size mutable  
//region variable
//⬛⬜██
const ALIVE            : &str = "\x1b[37m⬜ ";
const DEAD             : &str = "\x1b[30m⬛ ";
const SOFT_RESET       : &str = "\u{1b}[2J\u{1b}[3J";
const HIDE_CURSOR      : &str = "\u{1b}[?25l";
const SHOW_CURSOR      : &str = "\u{1b}[?25h";
const CURSOR_HOME      : &str = "\u{1b}[H";
const ADD_TO_CONDITION : i32  = 1;                           //default is 1
const COND1            : i32  = 1 + ADD_TO_CONDITION;
const COND2            : i32  = 2 + ADD_TO_CONDITION;
const ALIVE_CONDITION  : i32  = 3;                               //default is 3
const GRID_SIZE        : i32  = 50;                             //default is 25
const STARTING_AMOUNT  : i32  = GRID_SIZE * 15;
const OUT_OF_BOUND: &[i32; 2] = &[-1, GRID_SIZE];
const RANGE       : [i32; 3]  = [-1, 0, 1];
const REFRESH_TIME: u64       = 75;                              // count in ms
const GRID_SIZE_FLATTENED : usize = (GRID_SIZE * GRID_SIZE) as usize;
//endregion
// we will set all false cells as 'dead'


fn start_game(mut grid : [bool; GRID_SIZE_FLATTENED]) -> [bool; GRID_SIZE_FLATTENED]{
    // there will be overlaps but lets ignore it
    for _ in 0..STARTING_AMOUNT{ // starts filling random cells with 'alive' ones
        let rand_x = fastrand::i32(0..GRID_SIZE);
        let rand_y = fastrand::i32(0..GRID_SIZE);
        let index= (rand_x * GRID_SIZE + rand_y) as usize; // converts numbers to usize wahever it is
        grid[index] = true;
    }
    return grid;
}


fn display_grid(grid : [bool; GRID_SIZE_FLATTENED]){
    let mut buffer = String::with_capacity(GRID_SIZE_FLATTENED as usize * 3); // rough capacity guess
    buffer.push_str(SOFT_RESET);
    buffer.push_str(CURSOR_HOME);

    for index in 0..GRID_SIZE_FLATTENED {
        let col = index as i32 % GRID_SIZE;
        if grid[index] {
            buffer.push_str(ALIVE);
        } else {
            buffer.push_str(DEAD);
        }

        if col == GRID_SIZE - 1 {
            buffer.push('\n');
        }
    }
    print!("{}", buffer);
}

fn check_cell(grid : [bool; GRID_SIZE_FLATTENED] , cell_x : i32, cell_y : i32, current_state_of_cell : bool) -> bool{
    let mut count:i32 = 0; 
    for y in RANGE{
        for x in RANGE{
            // prevent out of bounds
            let tmp_condition_y = OUT_OF_BOUND.iter().any(|&tmp| tmp == cell_y + y); 
            let tmp_condition_x = OUT_OF_BOUND.iter().any(|&tmp| tmp == cell_x + x);

            let index: usize = ( (cell_y + y) * GRID_SIZE + (cell_x + x) ) as usize; // converts numbers to usize wahever it is
            if tmp_condition_y || tmp_condition_x || y == 0 && x == 0{ 
                continue;
            }
            else if grid[index] == false{ // if the cell is dead skip
                continue;
            }
            count += 1;
        }
    }
    count == ALIVE_CONDITION || (current_state_of_cell && (count == COND1 || count == COND2))

}

fn grid_check(grid : [bool; GRID_SIZE_FLATTENED]) -> [bool; GRID_SIZE_FLATTENED]{
    let mut tmp_grid:[bool; GRID_SIZE_FLATTENED] = grid;
    tmp_grid.par_iter_mut().enumerate().for_each(|(index, cell)| {
        let i: i32 = index as i32 / GRID_SIZE; // this returns the Y
        let j = index as i32 % GRID_SIZE; // this returns the X
        *cell = check_cell(grid, j, i, grid[index]);
    });
    return tmp_grid;
}
/*
fn clear_prev_cells(mut grid: [bool; GRID_SIZE_FLATTENED],changed_grid : Vec<(i32, i32)>) -> [bool; GRID_SIZE_FLATTENED]{
    for (i,j) in changed_grid{
        let index:usize = (i * GRID_SIZE + j) as usize; // converts numbers to usize wahever it is
        grid[index] = false; // only refreshes the set here
    } 
    return grid;
}
*/
fn main(){
    let mut grid = [false; GRID_SIZE_FLATTENED]; // initialize all cells of grid as dead
    print!("{}{}" , SOFT_RESET, HIDE_CURSOR);
    grid = start_game(grid);
    //TOOD : if possible change the method in check allow exit loops
    
    loop { // main loop
        display_grid(grid);
        thread::sleep(Duration::from_millis(REFRESH_TIME)); // allat just to sleep
        let prev_grid = grid;
        grid = grid_check(grid);
        // make sure if quit on pressing q
        //terminal::enable_raw_mode().unwrap();
        //let condition = event::poll(Duration::from_millis(100)).unwrap();
        //if  condition && let Event::Key(key_event) = event::read().unwrap(){
        //    if key_event.code == KeyCode::Char('q') {
        //        break;
        //    }
        //}
        //terminal::disable_raw_mode().unwrap();
        if prev_grid == grid{ //compare to see if they are the same
            break;
        }
        //TODO: add a check if the game stop (pull it straight from java ver)
    }
    print!("{}{}j*b ended{}", SOFT_RESET, CURSOR_HOME, SHOW_CURSOR);
}
