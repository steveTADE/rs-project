use clap::{Parser};
use std::process::ExitCode;
use std::thread;
use std::time::Duration;
use std::io::{Stdout, StdoutLock, Write, stdout};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;

const ALIVE      : &str           = "\x1b[37m██";
const DEAD       : &str           = "\x1b[90m██";
const ANT        : &str           = "\x1b[31m██";
const SOFT_RESET : &str           = "\u{1b}[2J\u{1b}[3J";
const HIDE_CURSOR: &str           = "\u{1b}[?25l";
const SHOW_CURSOR: &str           = "\u{1b}[?25h";
const CURSOR_HOME: &str           = "\x1b[H";
const DIRECTION  : [[i32; 2]; 4]  = [[0, 1], [1, 0], [0, -1], [-1, 0]]; // up, right, down, left
const MAX_DIRECTION_INDEX : usize = 3;

#[derive(Parser, Debug)]
#[command(about = "Langton's Ant simulation\n\x1b[31mred\x1b[37m dot is the ant")]
struct Args {
    /// Refresh time between frames in milliseconds
    #[arg(short, long, default_value_t = 35)]
    refresh_time: u64,
    /// Determines the grid size , pretty self explanatory
    #[arg(short, long, default_value_t = 25)]
    grid_size: i32,
    /// If the ant can warp across the screen or just die when it hits the walls of the grid
    #[arg(short, long, default_value_t = false)]
    warp: bool,
}

fn get_current_direction (grid : &Vec<bool>, ant_index : usize, mut  direction_index : usize) -> usize{
    if !grid[ant_index] {
        direction_index += 1;
    }
    else {
        if direction_index == 0{
            direction_index = MAX_DIRECTION_INDEX;
        }
        else {
            direction_index -= 1;
        }
    }

    if direction_index > MAX_DIRECTION_INDEX{
        direction_index = 0;
    }
    direction_index
}

fn display (grid : &Vec<bool>, grid_size : i32, ant_index : usize, mut buffer : String) -> String {
    //buffer.push_str(SOFT_RESET); // make terminal flicker at 1-10ms
    buffer.push_str(CURSOR_HOME);
    let mut counter:i32 = 0;
    for (index, &cell) in grid.iter().enumerate(){
        if  index == ant_index {
            buffer.push_str(ANT);
        }
        else if cell {
            buffer.push_str(ALIVE);
        } 
        else {
            buffer.push_str(DEAD);
        }
        counter += 1;
        if counter == grid_size {
            buffer.push('\n');
            counter = 0;
        }
    }
    buffer
}

fn main () -> ExitCode {
    let stdout             : Stdout         = stdout();
    let mut lock           : StdoutLock<'_> = stdout.lock();
    let args               : Args           = Args  ::parse();
    let refresh_time       : u64            = args.refresh_time;
    let grid_size          : i32            = args.grid_size;
    let grid_size_flattened: usize          = (grid_size * grid_size) as usize;
    let mut grid           : Vec<bool>      = vec![false; grid_size_flattened]; // initialize the entire grid with false
    let mut ant_x          : i32            = grid_size/2;  // spawn ant in the middle of the grid
    let mut ant_y          : i32            = grid_size/2; // spawn ant in the middle of the grid
    let mut ant_index      : usize          = (ant_y * grid_size + ant_x) as usize;
    let mut direction_index: usize          = 0; // start pointing up
    let ant_warp           : bool           = args.warp; //don't warp it on default
    let mut buffer         : String         = String::with_capacity(grid_size_flattened * 11); // rough capacity guess

    write!(lock, "{}{}", SOFT_RESET, HIDE_CURSOR).unwrap();
    loop {
        buffer.clear();
        direction_index = get_current_direction(&grid, ant_index, direction_index);
        ant_x          += DIRECTION[direction_index][0];
        ant_y          += DIRECTION[direction_index][1];

        if ant_x < 0 || ant_x >= grid_size || ant_y < 0 || ant_y >= grid_size {
            if !ant_warp {
                // exit loop and end the job cause the ant is dead
                write!(lock, "ant is dead end j*b{}", SHOW_CURSOR).unwrap();
                break;
            }
            else {
                ant_x = (ant_x + grid_size) % grid_size;
                ant_y = (ant_y + grid_size) % grid_size;
            }
        }

        ant_index       = (ant_y * grid_size + ant_x) as usize;
        grid[ant_index] = !grid[ant_index];
        buffer = display(&grid, grid_size, ant_index, buffer);
        write!(lock, "{}", buffer).unwrap();
        lock.flush().unwrap();
        thread::sleep(Duration::from_millis(refresh_time));
        terminal::enable_raw_mode().unwrap();
        let condition = event::poll(Duration::from_millis(refresh_time)).unwrap();
        if  condition && let Event::Key(key_event) = event::read().unwrap(){
            if key_event.code == KeyCode::Char('q') {
                break;
            }
        }
        terminal::disable_raw_mode().unwrap();
    }

    lock.flush().unwrap(); // just to be sure
    ExitCode::SUCCESS
}
