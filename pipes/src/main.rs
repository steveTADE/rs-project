use std::io::{Stdout, stdout, Write, StdoutLock};
use std::time::Duration;
use fastrand;
use std::process::ExitCode;
use crossterm::terminal::{self, size};
use crossterm::event::{self, Event, KeyCode};

const SOFT_RESET  : &str = "\u{1b}[2J\u{1b}[3J";
const CURSOR_HOME : &str = "\x1b[H";
const SHOW_CURSOR : &str = "\u{1b}[?25h";
const HIDE_CURSOR : &str = "\u{1b}[?25l";
use std::fmt::Write as FmtWrite;

use clap::Parser;

//TODO : make it so that i can use more styles
const TRACKS: [[&str; 12]; 4] = [[
    "┃", "┃", "━", "━",
    "┏", "┏", "┓", "┓",
    "┗", "┗", "┛", "┛",
], [
    "│", "│", "─", "─",
    "╭", "╭", "╮", "╮",
    "╰", "╰", "╯", "╯",
], [
    "│", "│", "─", "─",
    "┌", "┌", "┐", "┐", 
    "└", "└", "┘", "┘", 
], [
    "║", "║", "═", "═",
    "╔", "╔", "╗", "╗", 
    "╚", "╚", "╝", "╝",
], ];
const DIRECTIONS  : [[i32; 2]; 4]  = [[0, -1], [1, 0], [0, 1], [-1, 0]]; // 0: up, 1: right, 2: down, 3: left
const MAX_DIRECTION_INDEX : i32 = DIRECTIONS.len() as i32 - 1;

#[derive(Parser, Debug)]
#[command(about = "pipes.sh rewritten in rust but shittier\n
q : quit\n
r : resets all pipes and place them back in the get_current_direction\n
c : change style of the pipes\n
up/down arrow : increase/decrease probability of straight lines\n
right/left    : increase/decrease refresh time")]
struct Args {
    /// Refresh time between frames in milliseconds
    #[arg(short, long, default_value_t = 35)]
    refresh_time: u64,
    /// number of pipes
    #[arg(short, long, default_value_t = 1)]
    pipe_amount : i32,
    /// probability of a straight pipe
    #[arg(short, long, default_value_t = 85)]
    straight_pipe_probability : i32,
    ///reset after x characters, can't set to no limit yet
    #[arg(short, long, default_value_t = 20_000)]
    limit_reset : i32,
}


#[inline(always)]
fn get_current_direction (mut direction_index: i32, prob_of_straight_lines : i32) -> i32 {
    if fastrand::i32(0..100) > prob_of_straight_lines{ // will generate curve pipes only when above the threshold
        direction_index += if fastrand::bool() { 1 } else { -1 };
        if direction_index < 0{
            direction_index = MAX_DIRECTION_INDEX;
        }
        else if direction_index > MAX_DIRECTION_INDEX {
            direction_index = 0;
        }
    }
    return direction_index;
}

fn track_index (key:(i32, i32)) -> i8 {
    match key {
        (0, 0) => 0, (2, 2) => 1, (1, 1) => 2, (3, 3) => 3,
        (0, 1) => 4, (3, 2) => 5, (1, 2) => 6, (0, 3) => 7,
        (2, 1) => 8, (3, 0) => 9, (1, 0) => 10, (2, 3) => 11,
        _ => unreachable!(),
    }
}

struct Pipe {
    pointer: [i32; 2],
    direction_current: i32,
    direction_next: i32,
    color : (u8, u8, u8)
}

fn main () -> ExitCode{
    let args                                  : Args           = Args::parse();
    let stdout                                : Stdout         = stdout();
    let mut lock                              : StdoutLock<'_> = stdout.lock();
    let mut refresh_time                      : u64            = args.refresh_time;
    let mut prob_of_straight_lines            : i32            = args.straight_pipe_probability; 
    let (mut width,mut height)  : (usize, usize) = (size().unwrap().0 as usize, size().unwrap().1 as usize);
    let num_of_pipes                          : i32            = args.pipe_amount;
    let reset_limit                           : i32            = args.limit_reset;
    let mut reset_counter                     : i32            = 0;
    let mut pipe_array                        : Vec<Pipe>      = (0..num_of_pipes)
                                                                .map(|_| Pipe {
                                                                    pointer: [width as i32 / 2, height  as i32 / 2],
                                                                    direction_current: 0,
                                                                    direction_next: 0,
                                                                    color : (fastrand::u8(..), fastrand::u8(..), fastrand::u8(..))
                                                                })
                                                                .collect();
    let mut updates: Vec<(usize, i8, (u8, u8, u8))>            = Vec::with_capacity(num_of_pipes as usize);
    let mut buffer : String                                    = String::with_capacity(width * height * 32);
    let mut pipe_style                        : usize          = 1;
    terminal::enable_raw_mode().unwrap();
    write!(lock, "{}{}{}", HIDE_CURSOR, CURSOR_HOME, SOFT_RESET).unwrap();
    //FIXME : Spaghetti code
    loop{
        //allow the terminal to be resized in real time and adapt to it
        let (new_width, new_height) = {
            let (w, h) = size().unwrap();
            (w as usize, h as usize)
        };
        if (new_width, new_height) != (width, height) {
            width  = new_width;
            height = new_height;
            write!(lock, "{}{}", CURSOR_HOME, SOFT_RESET).unwrap();
        }
        //reset the grid if it pass the threshold
        reset_counter += num_of_pipes;
        if reset_counter > reset_limit{
            write!(lock, "{}{}", CURSOR_HOME, SOFT_RESET).unwrap();
            reset_counter = 0;
        }

        updates.clear();
        for pipe in pipe_array.iter_mut(){
            pipe.direction_current = pipe.direction_next;
            pipe.direction_next = get_current_direction(pipe.direction_next, prob_of_straight_lines);
            pipe.pointer[0] += DIRECTIONS[pipe.direction_current as usize][0];
            pipe.pointer[1] += DIRECTIONS[pipe.direction_current as usize][1];
            if pipe.pointer[0] < 0 || pipe.pointer[0] >= width as i32
                || pipe.pointer[1] < 0 || pipe.pointer[1] >= height as i32 {
                pipe.pointer[0] = (pipe.pointer[0] + width as i32) % width as i32;
                pipe.pointer[1] = (pipe.pointer[1] + height as i32) % height as i32;
                pipe.color = (fastrand::u8(..), fastrand::u8(..), fastrand::u8(..));
            }
            let idx = pipe.pointer[1] as usize * width + pipe.pointer[0] as usize;
            let glyph = track_index((pipe.direction_current, pipe.direction_next));
            updates.push( (idx, glyph, pipe.color) );
        };

        buffer.clear();
        for (idx, glyph, color) in &updates {
            let y = idx / width  + 1;
            let x = idx % width + 1;
            write!(buffer, "\x1b[{};{}H\x1b[38;2;{};{};{}m{}", y, x, color.0, color.1, color.2, TRACKS[pipe_style][*glyph as usize]).unwrap();
        }
        lock.flush().unwrap();
        write!(lock, "{}", buffer).unwrap();
        //sleep + actions
        /*TODO : give some more actions other than quitting
        B	    Toggle bold effect
        C	    Toggle no colors
        K	    Toggle keeping pipe color and type when crossing edges
        */
        
        let condition = event::poll(Duration::from_millis(refresh_time)).unwrap();
        if  condition && let Event::Key(key_event) = event::read().unwrap(){
            match key_event.code  {
                KeyCode::Right => refresh_time    += 5,
                KeyCode::Left  => if refresh_time !=  0 {refresh_time -= 5},
                KeyCode::Up    => if prob_of_straight_lines < 100 {prob_of_straight_lines += 5},
                KeyCode::Down  => if prob_of_straight_lines > 0   {prob_of_straight_lines -= 5},
                KeyCode::Char('c') => if pipe_style >= TRACKS.len() - 1 {pipe_style = 0} else {pipe_style += 1},
                KeyCode::Char('r') => {
                    for pipes in pipe_array.iter_mut(){
                        pipes.pointer[0] = width as i32 / 2;
                        pipes.pointer[1] = height  as i32 / 2;
                    }
                }
                KeyCode::Char('q') => break,
                _                  => {}, 
            }
        }
    }
    terminal::disable_raw_mode().unwrap();
    write!(lock, "{}{}{}j*b ended",SOFT_RESET, CURSOR_HOME, SHOW_CURSOR).unwrap();
    lock.flush().unwrap();
    ExitCode::SUCCESS
}