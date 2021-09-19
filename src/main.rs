use std::process::Command;
use std::vec::Vec;
use std::{thread, time};
use std::{time::Duration};
use rand::Rng;
use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

// Tetronimos list of seven 4x4 pieces layed in straight line
static TETRONIMO: [&str; 7] = [
    ".X...X...X...X..",
    "..X..XX...X.....",
    ".....XX..XX.....",
    "..X..XX..X......",
    ".X...XX...X.....",
    ".X...X...XX.....",
    "..X...X..XX.....",
];

const SCREEN_WIDTH: usize = 50; // Console Screen Size X (columns)
const SCREEN_HEIGHT: usize = 25; // Console Screen Size Y (rows)
const FIELD_WIDTH: usize = 12;
const FIELD_HEIGHT: usize = 18;
const FIELD_X_MIDDLE: usize = FIELD_WIDTH / 2 - 2;

type KeyMap = [bool;4];

//0d 0  1  2  3   90d 12  8  4  0  180d 15 14 13 12  270d 3  7 11 15 
//   4  5  6  7       13  9  5  1       11 10  9  8       2  6 10 14
//   8  9 10 11       14 10  6  2        7  6  5  4       1  5  9 13      
//  12 13 14 15       15 11  7  3        3  2  1  0       0  4  8 12 
fn rotate_piece(px: usize, py: usize, r: usize) -> usize {
    let pi = match  r % 4 {
        0 => py * 4 + px,        // 0 degrees
        1 => 12 + py - (px * 4), // 90 degrees
        2 => 15 - (py * 4) - px, // 180 degrees
        3 => 3 - py + (px * 4),  // 270 degrees
        _ => 0,                  // none of the above
    };
    return pi;
}

// renders a buffer on screen
fn render(screen_width:usize, screen_height: usize, buffer: &[char]) {
    // a pseudo OS agnostic way to clear the screen
    assert!(Command::new("clear").status().or_else(|_| Command::new("cls").status()).unwrap().success());

    for x in 0..screen_width * screen_height {
        print!("{}", buffer[x]);
        if x % screen_width == 0 {
            println!("{}", "");
        }
    }   
}

fn print(pos: usize, text: String,  buffer: &mut[char; SCREEN_WIDTH * SCREEN_HEIGHT]) {
    let mut offset = 0;
    for s in text.chars() { 
        buffer[pos + offset] = s;
        offset += 1;
    }
}

fn sprite(x:usize, y:usize, field: &[char]) -> char {
    let sprite_list = [' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', '=', '#'];
    let char_index = field[y * FIELD_WIDTH + x] as usize - '0' as usize;
    return sprite_list[char_index];
}

fn does_piece_fit(tetromino: usize, rotation: usize, pos_x:usize, pos_y: usize, field: &[char]) -> bool {
    for x in 0..4 {
        for y in 0..4 {
            let piece_index = rotate_piece(x, y, rotation) as usize;
            let field_index = (pos_y + y) * FIELD_WIDTH + (pos_x + x);
            //(pos_x + x) as i32 >= 0 && 
            if pos_x + x < FIELD_WIDTH  {
                // (pos_y + y) as i32 >= 0 && 
                if pos_y + y < FIELD_HEIGHT  {
                    let tetro = TETRONIMO[tetromino].chars().nth(piece_index).unwrap();
                    if tetro != '.' && field[field_index] != '0' {
                        return false;
                    }
                }
            }

        }
    }
    return true;
}

fn handle_input() -> Result<KeyMap> {
        // Wait up to 50ms for another event
        if poll(Duration::from_millis(50))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = read()?;
        
            if event == Event::Key(KeyCode::Left.into()) {
                return Ok([false, true, false, false]);
            }

            if event == Event::Key(KeyCode::Right.into()) {
                return Ok([false, false, true, false]);
            } 

            if event == Event::Key(KeyCode::Down.into()) {
                return Ok([false, false, false, true]);
            } 
            
            if event == Event::Key(KeyCode::Char('z').into()) {
                return Ok([true, false, false, false]);
            }
      
        } else {
            return Ok([false, false, false, false]);
        }
    return Ok([false, false, false, false]);
}

fn main() -> Result<()> {
    // start display buffer
    let mut screen : [char; SCREEN_WIDTH * SCREEN_HEIGHT] = [' '; SCREEN_WIDTH * SCREEN_HEIGHT];

    // Create play field buffer
    let mut field : [char; FIELD_WIDTH * FIELD_HEIGHT] = ['0'; FIELD_WIDTH * FIELD_HEIGHT]; 

    // set boundaries a in the field both sides and bottom //TODO optimize to iterate only on borders
    for x in 0..FIELD_WIDTH { // Board Boundary
        for y in 0..FIELD_HEIGHT {
            if x == 0 || x == FIELD_WIDTH - 1 || y == FIELD_HEIGHT - 1 {
                field[y * FIELD_WIDTH + x] = '9';
            }
        }
    }
   
    // game over flag
    let mut is_over = false;

    let mut current_piece = 0;
    let mut current_rotation = 0;
    let mut current_x = FIELD_X_MIDDLE;
    let mut current_y = 0;
    let mut current_keys : KeyMap = [false;4];
    let mut game_speed = 10;
    let mut game_speed_counter = 0;
    let mut force_down = false;
    let mut piece_count = 0;
    let mut score = 0;
    // let mut is_key_hit = false;
    let mut lines: Vec<usize> = Vec::new();
    lines.clear();
    enable_raw_mode()?; // TODO why ???

    loop {
        // TIMING ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// 
        //let tick_time = time::Duration::from_millis(50);
        //thread::sleep(tick_time);
        if game_speed_counter == game_speed {
            force_down = true;
        }
        
        // EOF TIMING //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        
        // GAME LOGIC //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        
        // move left
        current_x = if current_keys[1] && current_x > 0 && does_piece_fit(current_piece, current_rotation, current_x - 1, current_y, &field) {current_x - 1} else {current_x};
        
        // move right
        current_x = if current_keys[2] && does_piece_fit(current_piece, current_rotation, current_x + 1, current_y, &field) {current_x + 1} else {current_x};

        // move down
        current_y = if current_keys[3] && does_piece_fit(current_piece, current_rotation, current_x, current_y + 1, &field) {current_y + 1} else {current_y};

        // rotate the piece
        current_rotation = if current_keys[0] && does_piece_fit(current_piece, current_rotation + 1, current_x, current_y, &field) {current_rotation + 1} else {current_rotation};

        if force_down {
            if does_piece_fit(current_piece, current_rotation, current_x, current_y + 1, &field) {
                current_y = current_y + 1;
            } else {
                
                // lock the current piece into the field
                for x in 0..4 {
                    for y in 0..4 {
                        if TETRONIMO[current_piece].chars().nth(rotate_piece(x, y, current_rotation)).unwrap() != '.' {
                            let c = (current_piece + 1).to_string().chars().nth(0).unwrap();
                            field[(current_y + y) * FIELD_WIDTH + current_x + x] = c;
                        }
                    }
                }

                piece_count += 1;
                if piece_count % 10 == 0 { // increase each 10 pieces is locked to the field
                    if game_speed >= 10 {
                        game_speed -= 1; // get the game faster
                    }
                }

                // check if we have any lines
                for y in 0..4 {
                    if current_y + y < FIELD_HEIGHT - 1 {
                        let mut is_line = true;
                        for x in 1..(FIELD_WIDTH-1) {
                            is_line = is_line && field[(current_y + y) * FIELD_WIDTH + x] != '0';
                        }

                        if is_line {
                            // remove line, set to =
                            for x in 1..(FIELD_WIDTH-1) {
                                field[(current_y + y) * FIELD_WIDTH + x] = '8';
                            }
                            lines.push(current_y + y);
                        }
                    }
                }

                score += 25; // 25 points just for having a piece on the field
                // gives a increasing score for removing more lines at the same time
                score += if !lines.is_empty() {(1 << lines.len()) * 100} else { 0 };

                // select next piece
                let new_piece = rand::thread_rng().gen_range(0..7);
                current_piece = new_piece;
                current_rotation = 0;
                current_x = FIELD_X_MIDDLE;
                current_y = 0;

                // piece hit something cannot go on
                is_over = !does_piece_fit(current_piece, current_rotation, current_x, current_y, &field);

            }
            force_down = false;
            game_speed_counter = 0;
            
        } else {
            game_speed_counter += 1;
        }



        // EOF GAME LOGIC //////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        
        // RENDER //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        
        // draw field
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                screen[(y + 2) * SCREEN_WIDTH + (x + 2)] = sprite(x, y, &field);
            }
        }

        // draw current piece
        for x in 0..4 {
            for y in 0..4 {
                if TETRONIMO[current_piece].chars().nth(rotate_piece(x, y, current_rotation)).unwrap() != '.' {
                    screen[(current_y + y + 2) * SCREEN_WIDTH + current_x + x + 2] =  ((current_piece + 65) as u8) as char;
                }
            }
        }

        if !lines.is_empty() {
            render(SCREEN_WIDTH, SCREEN_HEIGHT, &screen);
            let tick_time = time::Duration::from_millis(400);
            thread::sleep(tick_time);
            for line in &lines {
                for x in 1..(FIELD_WIDTH - 1) {
                    for y in (1..*line + 1).rev() {
                        let c = field[(y - 1) * FIELD_WIDTH + x];
                        field[y * FIELD_WIDTH + x] = c;
                    }
                    field[x] = '0';
                }
            }

            lines.clear();
            assert!(lines.is_empty());
        }
      
        // print score on the buffer
        print(66, format!("Score:  {}", score), &mut screen);

        // print title
        print(5, "TETRIS!".to_string(), &mut screen);
       
        render(SCREEN_WIDTH, SCREEN_HEIGHT, &screen);

        // EOF RENDER //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

        // USER INPUT //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
      
        if let Ok(v) = handle_input() {
            current_keys = v;
        }
       
        // EOF USER INPUT //////////////////////////////////////////////////////////////////////////////////////////////////////////////////
      
        if is_over {
            break;
        }
    }

    println!(""); // this is here to avoid the awkward behaviour of printing game over in the end of the line
    println!("Game Over!");

    disable_raw_mode() // TODO why ??? 
 

}
