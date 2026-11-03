use std::{env, fs ::self, path::Path, process::ExitCode};
use std::io::{Stdout, StdoutLock, Write, stdin, stdout};
use crossterm::{event::{self, Event, KeyCode, KeyModifiers}, terminal};
use std::time::Duration;

const EXPECTED_ANSWER : [&str; 2] = ["y", "n"];
const SOFT_RESET  : &str = "\u{1b}[2J\u{1b}[3J";
const CURSOR_HOME : &str = "\x1b[H";
//const SHOW_CURSOR : &str = "\u{1b}[?25h";
//const HIDE_CURSOR : &str = "\u{1b}[?25l";

//TODO : implement usage of cursor like select text with user cursor
//TODO : autosave in a tmp file
//TODO : let the user scroll (make sure to allow scrolling left and right) if the file is too big
fn input(question: String, expected_ans: &[&str], allow_cancel : bool) -> i32 {
    let mut answer = String::new();
    let mut new_expected_ans : Vec<&str> = expected_ans.to_vec();
    if allow_cancel{
        new_expected_ans.push("c");
    }
    print!("{} > {:?} : ", question, new_expected_ans);
    let _ = stdout().flush();
    stdin().read_line(&mut answer).expect("Did not enter a correct string");
    answer = answer.trim().to_string();

    while !new_expected_ans.contains(&answer.as_str()) {
        answer.clear();
        print!("{} > {:?} : ", question, new_expected_ans); // reprint prompt so it's not confusing
        let _ = stdout().flush();
        stdin().read_line(&mut answer).expect("Did not enter a correct string");
        answer = answer.trim().to_string();
    }

    //println!("{}", answer);   //debug line
    new_expected_ans.iter().position(|&x| x == answer).unwrap() as i32 // output the answer
}

//HACK : ts don't work
#[allow(unused)]
fn save_changes(){
    // if we want the save changes to be printed down then we need padding
    println!("save changes");
} 

fn display(mut buffer : String, file_content : &Vec<Vec<u8>>) -> String{
    buffer.clear();
    buffer.push_str(SOFT_RESET); // this is to prevent smears when you del stuff
    buffer.push_str(CURSOR_HOME);
    let string_data: Vec<String> = file_content 
    .iter() 
    .map(|bytes| String::from_utf8_lossy(bytes).into_owned()) 
    .collect();
    for (index, output_line) in string_data.iter().enumerate(){
        buffer.push_str(output_line.as_str());
        if index + 1 < string_data.len() {
            buffer.push_str("\n\x1b[1G");
        }
    }
    buffer
}

#[allow(unused_mut)]
#[allow(unused_variables)]
fn main() -> ExitCode {
    /* #region initializing the editor */
    let stdout                 : Stdout         = stdout();
    let mut lock               : StdoutLock<'_> = stdout.lock();
    let raw                    : String         = env::args().skip(1).collect(); // get the raw path
    let (mut x, mut y)         : (usize,usize)  = (1, 1);
    let (mut width, mut height): (u16,u16)      = terminal::size().unwrap();
    //let mut flattened_index    = 0; // the one i'll actually use for stuff like the file_content
    //HACK : ts a problem, should allow user to just text something without constraint
    if raw == "".to_string(){ 
        write!(lock, "cannot edit nothing").unwrap();
        return ExitCode::SUCCESS;
    }
    let raw_path = Path::new(&raw); // turn it usable
    if !raw_path.exists(){
        println!("Files doesn't exist");
        let answer : i32 = input("File doesn't exist, do you want it to".to_string(), &EXPECTED_ANSWER, false);
        if answer == 0{ // if y
            let _raw_file = fs::File::create(&raw_path).unwrap();
        }
        else {
            println!("exiting program");
            return ExitCode::SUCCESS;
        }
    }
    let raw_file_content: String  = fs::read_to_string(&raw_path).unwrap(); 
    let mut buffer = String::with_capacity(raw.len() * 2);
    let mut file_content: Vec<Vec<u8>> = raw_file_content
        .lines()
        .map(|line| line.as_bytes().to_vec())
        .collect();
    /* #endregion */
    terminal::enable_raw_mode().unwrap();
    write!(lock, "{}{}", SOFT_RESET, CURSOR_HOME).unwrap();
    buffer = display(buffer, &file_content);
    write!(lock, "{}", &buffer).unwrap();
    lock.flush().unwrap();
    //TODO : add delete key
    loop{
        let condition = event::poll(Duration::from_millis(50)).unwrap();
        if  condition && let Event::Key(key_event) = event::read().unwrap(){
            let user_input = key_event.code;
            match (user_input, key_event.modifiers) {
                /* #region special function keys                */
                //HACK : the file_content[y].len() - 1 in every fucking function key is visually annoying me but it works
                //TODO : auto-indentation with tab
                (KeyCode::Tab,   KeyModifiers::NONE) =>{
                    //write!(lock, "\x1b[10;0H{}", string_data[y].chars().nth(x).unwrap()).unwrap(); // debug
                }
                (KeyCode::Left,  KeyModifiers::NONE) => {
                    if x != 0{
                        x -= 1;
                    }
                    else if x == 0 && y != 0{
                            y -= 1;
                            x  = file_content[y].len() - 1;
                    }
                }
                (KeyCode::Right, KeyModifiers::NONE) => {
                    if x < file_content[y].len() - 1 {
                        x += 1;
                    } else if y < file_content.len() - 1 {
                        // wrap to start of next line
                        y += 1;
                        x = 0;
                    }
                }
                (KeyCode::Up,    KeyModifiers::NONE) => {
                    if y != 0 {
                        y -= 1;
                    }
                    let max_length = file_content[y].len() - 1;
                    if x >= max_length{
                        x = max_length;
                    }
                }
                (KeyCode::Down,  KeyModifiers::NONE) => {
                    if y < file_content.len() - 1 {
                        y += 1;
                    }
                    let max_length = file_content[y].len() - 1;
                    if x >= max_length{
                        x = max_length;
                    }
                }
                /* #endregion */
                /* #region ctrl function */
                /*TODO : implement ctrl commands
                    - ctrl + z          undo
                    - ctrl + x          cut
                    - ctrl + c          copy
                    - ctrl + v          paste
                    - ctrl + s          save
                    - ctrl + a          select everything
                    - ctrl + h          help 
                    - ctrl + backspace  word deletion
                    - ctrl + arrow </>  word shifting
                    - ctrl + q          exit program (prompt if you wanna save or not, anything else cancels the actions)
                */

                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    println!("currently does nothing");
                }
                (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                    println!("currently does nothing");
                }
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    save_changes();
                }
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => { // controls quit
                    terminal::disable_raw_mode().unwrap();
                    let padding = height as usize - y;
                    let answer = input(format!("\n\x1b[{padding};0Hdo you wanna save changes"), &EXPECTED_ANSWER, true);
                    if answer == 0{ // save changes
                        save_changes();
                        println!("quit program");
                        break;
                    }
                    if answer == 1{ // just quit
                        println!("quit program");
                        break;
                    }
                    write!(lock, "\x1b[1A\x1b[2K\r").unwrap();
                    terminal::enable_raw_mode().unwrap();
                }
                //HACK : janky code
                (KeyCode::Left,      KeyModifiers::CONTROL) => {
                    let mut tmp : usize = x;
                    if tmp == 0 && y > 0{
                        y  -= 1;
                        tmp = file_content[y].len() - 1;
                    }
                    while tmp != 0 && file_content[y][tmp - 1] == b' ' {
                        tmp -= 1;
                    }
                    while tmp != 0 && file_content[y][tmp - 1] != b' ' {
                        tmp -= 1;
                    }
                    x = tmp;
                }
                (KeyCode::Right,      KeyModifiers::CONTROL) => {
                    let mut tmp : usize = x;
                    if tmp == file_content[y].len() - 1 && y < file_content.len() - 1{
                        y  += 1;
                        tmp = 0;
                    }
                    while tmp < file_content[y].len() - 1 && file_content[y][tmp + 1] == b' ' {
                        tmp += 1;
                    }
                    while tmp < file_content[y].len() - 1 && file_content[y][tmp + 1] != b' ' {
                        tmp += 1;
                    }
                    x = tmp;
                }
                /* #endregion */
                _ => { // update the string and display
                    if let KeyCode::Char(character) = user_input {
                        let mut encoded = [0; 4];
                        let character_bytes = character.encode_utf8(&mut encoded).as_bytes();
                        file_content[y].splice(x..x, character_bytes.iter().copied());
                        x += character_bytes.len();
                        buffer = display(buffer, &file_content);
                        write!(lock, "{}", &buffer).unwrap();
                    }
                }
            
            }
            write!(lock, "\x1b[{};{}H", y + 1, x + 1).unwrap(); //only will run this if the user input something
            lock.flush().unwrap();
        }
    }
    terminal::disable_raw_mode().unwrap();
    ExitCode::SUCCESS
}
