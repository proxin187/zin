extern crate ncurses;
mod config;

use std::env;
use std::process;
use std::fs;
use std::io::{prelude::*, BufReader};

#[derive(Debug)]
struct Window {
    cursor_col: i32,
    cursor_row: i32,
    win_row: i32,
    win_width: i32,
    win_height: i32,
}

#[derive(Debug)]
struct Buffer {
    buf_name: String,
    buffer: Vec<Vec<String>>,
    fd: fs::File,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SyntaxModes {
    Str,
    Normal,
    Comment,
}

#[derive(Debug)]
pub struct Syntax {
    keywords: Vec<String>,
    symbols: Vec<String>,
    types: Vec<String>,
    operators: Vec<String>,
    string: String,
    comment: String,
    mode: SyntaxModes,
}

#[derive(Debug)]
struct Visual {
    start: (i32, i32),
    end: (i32, i32),
}

#[derive(Debug)]
struct Matched {
    current_match: usize,
    matches: Vec<(i32, i32)>,
}

#[derive(Eq, PartialEq, Debug)]
enum Modes {
    Insert,
    Normal,
    Visual,
    Command,
}

impl Syntax {
    fn empty() -> Syntax {
        Syntax {
            keywords: Vec::new(),
            symbols: Vec::new(),
            types: Vec::new(),
            operators: Vec::new(),
            string: String::new(),
            comment: String::new(),
            mode: SyntaxModes::Normal,
        }
    }

    fn new(filepath: &str) -> Syntax {
        if filepath.ends_with(".rs") {
            return config::rs::init();
        } else {
            return Self::empty();
        }
    }

    fn color_of_token(&mut self, token: String) -> (i16, String) {
        if self.mode == SyntaxModes::Normal {
            if self.keywords.contains(&token) {
                return (5, token);
            } else if self.types.contains(&token) {
                return (6, token);
            } else if self.operators.contains(&token) {
                return (7, token);
            } else if token == self.string {
                self.mode = SyntaxModes::Str;
                return (8, token);
            } else if token == self.comment {
                self.mode = SyntaxModes::Comment;
                return (9, token);
            } else {
                return (1, token);
            }
        } else if self.mode == SyntaxModes::Str {
            if token == self.string {
                self.mode = SyntaxModes::Normal;
            }
            return (8, token);
        } else if self.mode == SyntaxModes::Comment {
            return (9, token);
        } else {
            return (1, token);
        }
    }

    fn highlight_line(&mut self, line: &Vec<String>) -> Vec<(i16, String)> {
        let mut token = String::new();
        let mut highlighted: Vec<(i16, String)> = Vec::new();
        self.mode = SyntaxModes::Normal;
        let comment_pos: usize;
        match line.join("").find(&self.comment) {
            Some(value) => {
                comment_pos = value;
            },
            _ => {
                comment_pos = 100000;
            },
        }

        let mut index = 0;
        while index < line.len() + 1 {
            if index < line.len() {
                let character = &line[index];
                if index == comment_pos {
                    highlighted.push(self.color_of_token(token));
                    highlighted.push(self.color_of_token(self.comment.clone()));
                    token = String::new();
                    index += self.comment.len() - 1;
                } else if self.symbols.contains(character) {
                    highlighted.push(self.color_of_token(token));
                    highlighted.push(self.color_of_token(character.clone()));
                    token = String::new();
                } else {
                    token = token + character;
                }
            } else {
                highlighted.push(self.color_of_token(token));
                token = String::new();
            }
            index += 1;
        }

        return highlighted;
    }
}

impl Window {
    fn display(
        &self,
        buffer: &Buffer,
        screen: *mut i8,
        mode: &Modes,
        command: &String,
        syntax: &mut Syntax
    ) {
        ncurses::wmove(screen, 0, 0);

        // render text
        let mut index = self.win_row;
        while index < self.win_row + self.win_height -  2 {
            let lines: Vec<String>;
            if buffer.buffer.len() <= index as usize {
                lines = vec![String::from("~")];
            } else {
                lines = buffer.buffer[index as usize].clone();
            }

            let mut counter = 0;
            let highlighted_line = syntax.highlight_line(&lines);
            for token in highlighted_line {
                ncurses::attron(ncurses::COLOR_PAIR(token.0));
                ncurses::waddstr(screen, &token.1);
                ncurses::attroff(ncurses::COLOR_PAIR(token.0));
                counter += token.1.len();
            }
            ncurses::attron(ncurses::COLOR_PAIR(1));
            for _ in counter..self.win_width as usize - 1 {
                ncurses::waddstr(screen, " ");
            }
            ncurses::waddstr(screen, "\n");
            ncurses::attroff(ncurses::COLOR_PAIR(1));
            index += 1;
        }


        // render bar
        let mut counter = 0;
        while counter < self.win_width as usize {
            if counter == 0 {
                let attr: i16;
                let mode = &format!(" {:?} ", mode).to_uppercase();
                if mode == " NORMAL " {
                    attr = 2;
                } else {
                    attr = 4;
                }
                ncurses::attron(ncurses::COLOR_PAIR(attr));
                ncurses::waddstr(screen, mode);
                ncurses::attroff(ncurses::COLOR_PAIR(attr));
                counter += mode.len() - 1;
            } else if counter == self.win_width as usize / 2 - buffer.buf_name.len() / 2 {
                ncurses::attron(ncurses::COLOR_PAIR(3));
                ncurses::waddstr(screen, &buffer.buf_name);
                ncurses::attroff(ncurses::COLOR_PAIR(3));
                counter += buffer.buf_name.len() - 1;
            } else {
                ncurses::attron(ncurses::COLOR_PAIR(3));
                ncurses::waddstr(screen, " ");
                ncurses::attroff(ncurses::COLOR_PAIR(3));
            }
            counter += 1;
        }

        counter = 0;
        ncurses::attron(ncurses::COLOR_PAIR(1));
        while counter < self.win_width as usize {
            match counter {
                0 => {
                    ncurses::waddstr(screen, command);
                    counter += command.len();
                },
                _ => {
                    ncurses::waddstr(screen, " ");
                },
            }
            counter += 1;
        }
        ncurses::attroff(ncurses::COLOR_PAIR(1));

        ncurses::wmove(screen, self.cursor_row - self.win_row, self.cursor_col);
    }

    fn init_colors(&self, configuration: &config::Config) {
        ncurses::start_color();

        ncurses::init_color(
            1,
            rgb(configuration.foreground.red as f32) as i16,
            rgb(configuration.foreground.green as f32) as i16,
            rgb(configuration.foreground.blue as f32) as i16,
        );

        ncurses::init_color(
            4,
            rgb(configuration.foreground1.red as f32) as i16,
            rgb(configuration.foreground1.green as f32) as i16,
            rgb(configuration.foreground1.blue as f32) as i16,
        );

        ncurses::init_color(
            2,
            rgb(configuration.background.red as f32) as i16,
            rgb(configuration.background.green as f32) as i16,
            rgb(configuration.background.blue as f32) as i16,
        );

        ncurses::init_color(
            5,
            rgb(configuration.background1.red as f32) as i16,
            rgb(configuration.background1.green as f32) as i16,
            rgb(configuration.background1.blue as f32) as i16,
        );

        ncurses::init_color(
            3,
            rgb(configuration.green.red as f32) as i16,
            rgb(configuration.green.green as f32) as i16,
            rgb(configuration.green.blue as f32) as i16,
        );

        ncurses::init_color(
            6,
            rgb(configuration.orange.red as f32) as i16,
            rgb(configuration.orange.green as f32) as i16,
            rgb(configuration.orange.blue as f32) as i16,
        );

        ncurses::init_color(
            7,
            rgb(configuration.yellow.red as f32) as i16,
            rgb(configuration.yellow.green as f32) as i16,
            rgb(configuration.yellow.blue as f32) as i16,
        );

        ncurses::init_color(
            8,
            rgb(configuration.quartz.red as f32) as i16,
            rgb(configuration.quartz.green as f32) as i16,
            rgb(configuration.quartz.blue as f32) as i16,
        );

        ncurses::init_pair(1, 1, 2);
        ncurses::init_pair(2, 2, 3);
        ncurses::init_pair(3, 1, 5);
        ncurses::init_pair(4, 2, 6);
        ncurses::init_pair(5, 7, 2);
        ncurses::init_pair(6, 8, 2);
        ncurses::init_pair(7, 6, 2);
        ncurses::init_pair(8, 3, 2);
        ncurses::init_pair(9, 5, 2);
    }

    fn left(&mut self) {
        if self.cursor_col != 0 {
            self.cursor_col = self.cursor_col - 1;
        }
    }

    fn right(&mut self, buffer: &Buffer) {
        if self.cursor_col != buffer.buffer[self.cursor_row as usize].len() as i32 {
            self.cursor_col = self.cursor_col + 1;
        }
    }

    fn down(&mut self, buffer: &Buffer) {
        if self.cursor_row + 1 != buffer.buffer.len() as i32 && self.cursor_row - self.win_row != self.win_height - 3 {
            self.cursor_row = self.cursor_row + 1;
        } else if self.cursor_row - self.win_row == self.win_height - 3 && self.cursor_row + 1 != buffer.buffer.len() as i32 {
            self.cursor_row = self.cursor_row + 1;
            self.win_row = self.win_row + 1;
        }
    }

    fn up(&mut self) {
        if self.cursor_row - self.win_row != 0 {
            self.cursor_row = self.cursor_row - 1;
        } else if self.win_row != 0 {
            self.cursor_row = self.cursor_row - 1;
            self.win_row = self.win_row - 1;
        }
    }

    fn clamp_col(&mut self, buffer: &Buffer) {
        if buffer.buffer[self.cursor_row as usize].len() < self.cursor_col as usize {
            self.cursor_col = buffer.buffer[self.cursor_row as usize].len() as i32;
        }
    }

    fn check_move(&mut self, buffer: &Buffer, char_code: i32) -> bool {
        if char_code as i32 == ncurses::KEY_LEFT {
            self.left();
        } else if char_code as i32 == ncurses::KEY_RIGHT {
            self.right(&buffer);
        } else if char_code as i32 == ncurses::KEY_DOWN {
            self.down(&buffer);
        } else if char_code as i32 == ncurses::KEY_UP {
            self.up();
        } else {
            return false;
        }
        return true;
    }

    fn next_match(&mut self, matched: &mut Matched) {
        if matched.current_match != matched.matches.len() - 1 {
            matched.current_match += 1;
        }
        self.cursor_row = matched.matches[matched.current_match].0;
        self.cursor_col = matched.matches[matched.current_match].1;
        self.win_row = self.cursor_row;
    }

    fn earlier_match(&mut self, matched: &mut Matched) {
        if matched.current_match != 0 {
            matched.current_match -= 1;
        }
        self.cursor_row = matched.matches[matched.current_match].0;
        self.cursor_col = matched.matches[matched.current_match].1;
        self.win_row = self.cursor_row;
    }
}

impl Buffer {
    fn new(filepath: &str) -> Buffer {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filepath);

        if file.is_err() {
            println!("Cant open {}, make shure you have permision to read/write", filepath);
            process::exit(1);
        }

        let file = file.unwrap();

        let reader = BufReader::new(file.try_clone().expect("Failed to clone file descriptor"));
        let mut buf: Vec<Vec<String>> = Vec::new();
        for line in reader.lines() {
            let mut line_vec: Vec<String> = Vec::new();
            for byte in line.unwrap().bytes() {
                line_vec.push(String::from_utf8(vec![byte]).unwrap());
            }
            buf.push(line_vec);
        }

        if buf.len() == 0 {
            buf.push(vec![]);
        }

        return Buffer {
            buf_name: String::from(filepath),
            buffer: buf,
            fd: file,
        }
    }

    fn insert(&mut self, window: &Window, char_code: u8) {
        let character = String::from_utf8(vec![char_code]);

        if character.is_err() {
            return;
        }

        let character = character.unwrap();

        let mut new_line: Vec<String> = Vec::new();
        let mut count = 0;
        if self.buffer[window.cursor_row as usize].len() == 0 {
            new_line.push(character);
            self.buffer[window.cursor_row as usize] = new_line;
            return;
        } else if window.cursor_col == self.buffer[window.cursor_row as usize].len() as i32 {
            self.buffer[window.cursor_row as usize].push(character);
            return;
        }
        for old_character in &self.buffer[window.cursor_row as usize] {
            if count == window.cursor_col {
                new_line.push(character.clone());
            }
            new_line.push(String::from(old_character));
            count += 1;
        }
        self.buffer[window.cursor_row as usize] = new_line;
    }

    fn delete_line(&mut self, window: &Window) {
        self.buffer.remove(window.cursor_row as usize);
    }

    fn delete(&mut self, window: &mut Window) {
        if window.cursor_col == 0 && window.cursor_row != 0 {
            let old_line = self.buffer[window.cursor_row as usize].clone();
            self.buffer.remove(window.cursor_row as usize);
            window.up();
            window.cursor_col = self.buffer[window.cursor_row as usize].len() as i32;
            self.buffer[window.cursor_row as usize].extend(old_line);
        } else if window.cursor_col != 0 {
            self.buffer[window.cursor_row as usize].remove(window.cursor_col as usize - 1);
            window.left();
        }

    }

    fn get_identation(&self, line: usize) -> usize {
        let mut counter = 0;
        for character in &self.buffer[line] {
            if character != " " {
                return counter;
            }
            counter += 1;
        }
        return 0;
    }

    fn newline_down(&mut self, window: &mut Window) {
        self.buffer.insert(window.cursor_row as usize + 1, Vec::new());
        window.down(&self);
    }

    fn newline(&mut self, window: &mut Window) {
        let identation_count = self.get_identation(window.cursor_row as usize);
        let mut count = window.cursor_col as usize;
        let mut old_line: Vec<String> = Vec::new();
        let mut identation: Vec<String> = Vec::new();
        for _ in 0..identation_count {
            identation.push(String::from(" "));
        }
        old_line.extend(identation);
        while count < self.buffer[window.cursor_row as usize].len() {
            old_line.push(self.buffer[window.cursor_row as usize][count].clone());
            count += 1;
        }
        for _ in 0 .. count - window.cursor_col as usize {
            self.buffer[window.cursor_row as usize].pop().unwrap();
        }
        self.buffer.insert(window.cursor_row as usize + 1, old_line);
        window.down(&self);
        if identation_count == 0 {
            window.cursor_col = 0;
        } else {
            window.cursor_col = identation_count as i32;
        }
    }

    fn yank(&self, visual: &mut Visual) -> Vec<Vec<String>> {
        let mut yanked: Vec<Vec<String>> = Vec::new();
        let mut line: Vec<String> = Vec::new();
        if visual.start.0 == visual.end.0 {
            let range: std::ops::Range<i32>;
            if visual.start.1 < visual.end.1 {
                range = visual.start.1 .. visual.end.1;
            } else {
                range = visual.end.1 .. visual.start.1;
            }
            for character in range {
                line.push(self.buffer[visual.start.0 as usize][character as usize].clone());
            }
            yanked.push(line);
        } else {
            for index in visual.start.0 .. visual.end.0 + 1 {
                let row = &self.buffer[index as usize];
                let mut line: Vec<String> = Vec::new();
                if index == visual.start.0 {
                    for character in visual.start.1 as usize .. row.len() {
                        line.push(row[character].clone());
                    }
                } else if index == visual.end.0 {
                    for character in 0 .. visual.end.1 as usize {
                        line.push(row[character].clone());
                    }
                } else {
                    for character in row {
                        line.push(character.clone());
                    }
                }
                yanked.push(line);
            }
        }
        return yanked;
    }

    fn paste(&mut self, window: &Window, text: Vec<Vec<String>>) {
        let mut index = window.cursor_row as usize;
        for line in text {
            self.buffer.insert(index, line);
            index += 1;
        }
    }

    fn line_to_string(&self, line: &Vec<String>) -> String {
        let mut str_line = String::new();
        for character in line {
            str_line = str_line + &character;
        }
        return str_line;
    }

    fn find(&self, value: &str, ) -> Option<Vec<(i32, i32)>> {
        let mut row = 0;
        let mut matched: Vec<(i32, i32)> = Vec::new();
        for line in &self.buffer {
            let line = self.line_to_string(line);
            let found = line.find(value);
            if found.is_some() {
                let found = found.unwrap();
                matched.push((row, found as i32));
            }
            row += 1;
        }
        if matched.len() == 0 {
            return None;
        } else {
            return Some(matched);
        }
    }

    fn handle_command(
        &mut self,
        window: &mut Window,
        command: String,
        matched: &mut Matched,
    ) -> String {
        let command = command.split(" ").collect::<Vec<&str>>();
        match command[0] {
            ":q" => {
                ncurses::endwin();
                process::exit(1);
            },
            ":E" => {
                return format!("\"{}\", {}B written", self.buf_name.clone(), self.write());
            },
            ":F" => {
                let pos = self.find(command[1]);
                if pos.is_some() {
                    let pos = pos.unwrap();
                    matched.matches = pos;
                    window.cursor_row = matched.matches[matched.current_match].0;
                    window.cursor_col = matched.matches[matched.current_match].1;
                    window.win_row = window.cursor_row;
                    return format!("Found: {} at {:?}", command[1], matched.matches);
                } else {
                    return format!("Couldn't find: {}", command[1]);
                }
            },
            _ => {
                return format!("Unknown command: {}", command.join(" "));
            },
        }
    }

    fn write(&mut self) -> u64 {
        self.fd.set_len(0).unwrap();
        self.fd.rewind().unwrap();

        for characters in &self.buffer {
            let mut line = String::new();
            for character in characters {
                line = line + character;
            }
            line = line + "\n";
            self.fd.write_all(line.as_bytes()).unwrap();
        }
        return self.fd.metadata().unwrap().len();
    }
}

fn rgb(num: f32) -> f32 {
    return (1000 as f32 / 100 as f32) * ((num / 256 as f32) * 100 as f32);
}

fn main() {
    let argv = env::args().collect::<Vec<String>>();
    let argc = argv.len();
    if argc < 2 {
        println!("Usage: zin <file>");
        process::exit(1);
    }

    let configuration = config::Config::init();

    let screen = ncurses::initscr();
    ncurses::noecho();
    ncurses::cbreak();
    ncurses::raw();
    ncurses::keypad(screen, true);
    ncurses::set_escdelay(0);

    let mut clipboard: Vec<Vec<Vec<String>>> = Vec::new();
    let mut matches = Matched {
        current_match: 0,
        matches: Vec::new(),
    };

    let mut syntax = Syntax::new(&argv[1]);
    let mut buffer = Buffer::new(&argv[1]);
    let mut window = Window {
        cursor_col: 0,
        cursor_row: 0,
        win_row: 0,
        win_height: ncurses::LINES(),
        win_width: ncurses::COLS(),
    };

    let mut command = String::new();
    let mut mode = Modes::Normal;
    let mut visual = Visual {
        start: (0, 0),
        end: (0, 0),
    };

    window.init_colors(&configuration);

    loop {
        window.clamp_col(&buffer);
        window.display(&buffer, screen, &mode, &command, &mut syntax);
        let char_code = ncurses::getch();
        if mode == Modes::Normal {
            if char_code == configuration.insert_mode {
                println!("\x1b[6 q"); // change cursor to bar
                mode = Modes::Insert;
            } else if char_code == configuration.visual_mode {
                visual.start = (window.cursor_row, window.cursor_col);
                visual.end = (window.cursor_row, window.cursor_col);
                mode = Modes::Visual;
            } else if char_code == configuration.paste {
                let top_clipboard = clipboard.pop();
                if top_clipboard.is_some() {
                    buffer.paste(&window, top_clipboard.unwrap());
                } else {
                    /* Clip board is empty */
                }
            } else if char_code == 58 {
                command = String::new() + ":";
                mode = Modes::Command;
            } else if char_code == 100 {
                let key = ncurses::getch();
                if key == 100 {
                    buffer.delete_line(&window);
                }
            } else if char_code == 111 {
                buffer.newline_down(&mut window);
            } else if char_code == 110 {
                window.next_match(&mut matches);
            } else if char_code == 98 {
                window.earlier_match(&mut matches);
            } else {
                window.check_move(&buffer, char_code);
            }
        } else if mode == Modes::Insert {
            if char_code == configuration.normal_mode {
                println!("\x1b[1 q"); // change cursor to block
                mode = Modes::Normal;
            } else if char_code == ncurses::KEY_BACKSPACE {
                buffer.delete(&mut window);
            } else if char_code == 10 { // if 10 doesnt work try ncurses::KEY_ENTER
                buffer.newline(&mut window);
            } else if window.check_move(&buffer, char_code) {
            } else {
                buffer.insert(&window, char_code as u8);
                window.right(&buffer);
            }
        } else if mode == Modes::Visual {
            if char_code == configuration.normal_mode {
                visual.start = (0, 0);
                visual.end = (0, 0);
                mode = Modes::Normal;
            } else if char_code == configuration.yank {
                clipboard.push(buffer.yank(&mut visual));
                mode = Modes::Normal;
            } else {
                window.check_move(&buffer, char_code);
                visual.end = (window.cursor_row, window.cursor_col);
            }
        } else if mode == Modes::Command {
            let character = &String::from_utf8(vec![char_code as u8]).unwrap();
            if character == "\n" {
                mode = Modes::Normal;
                command = buffer.handle_command(&mut window, command, &mut matches);
            } else if character == "\x1B" {
                mode = Modes::Normal;
                command = String::new();
            } else if char_code == ncurses::KEY_BACKSPACE {
                command.pop().unwrap();
            } else {
                command = command + character;
            }
        }
    }
}



