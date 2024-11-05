use std::{
    env,
    error::Error,
    fs::{self},
    io::{self, Write},
    process::{Command, Stdio},
};

macro_rules! print_error {
    ($msg:expr) => {
        if false {
            println!("{}", $msg);
        } else {
            println!("?");
        }
    };
}

type Address = (usize, usize);

struct Editor {
    path: String,
    buffer: Vec<String>,
    current: usize,
    run_command: String,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            path: String::new(),
            buffer: vec![],
            current: 0,
            run_command: String::new(),
        }
    }

    pub fn main_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let line = Editor::read_line()?;
            if line.is_empty() {
                continue;
            }

            // https://www.gnu.org/software/ed/manual/ed_manual.html#Commands
            match line.chars().next().unwrap() {
                // insert after current line
                'a' if line.len() == 1 => loop {
                    let line = Editor::read_line()?;
                    if line == "." {
                        break;
                    }
                    if self.buffer.is_empty() {
                        self.buffer.push(line);
                    } else {
                        self.current += 1;
                        self.buffer.insert(self.current, line);
                    }
                },
                // change line
                'c' if line.len() == 1 => {
                    if self.current >= self.buffer.len() {
                        print_error!("Invalid address");
                    } else {
                        self.buffer[self.current] = Editor::read_line()?;
                    }
                }
                // delete line
                'd' if line.len() == 1 => {
                    if self.current >= self.buffer.len() {
                        print_error!("Invalid address");
                    } else {
                        self.buffer.remove(self.current);
                    }
                }
                // open file
                'e' if line.starts_with("e ") => {
                    self.open_file(line[2..].into());
                }
                // insert before current line
                'i' if line.len() == 1 => loop {
                    let line = Editor::read_line()?;
                    if line == "." {
                        break;
                    }
                    if self.buffer.is_empty() {
                        self.buffer.push(line);
                    } else {
                        self.current += 1;
                        self.buffer.insert(self.current - 1, line);
                    }
                },
                // print the line preceded by its line number
                'n' if line.len() == 1 => match self.buffer.get(self.current) {
                    Some(line) => println!("{}\t{}", self.current + 1, line),
                    None => print_error!("Invalid address"),
                },
                // print line
                'p' if line.len() == 1 => {
                    self.print_current_line();
                }
                // run a shell command
                'r' => {
                    if line.contains(" ") {
                        self.run_command = (&line[2..]).into();
                    }

                    if self.run_command.is_empty() {
                        print_error!("Empty command");
                    } else {
                        Editor::run_command(self.run_command.clone())?;
                    }
                }
                // replace
                's' => {
                    let mut parts = line.split("/");
                    _ = parts.next();

                    match (parts.next(), parts.next()) {
                        (Some(pattern), Some(sub)) => {
                            let re = regex::Regex::new(&regex::escape(pattern)).unwrap();

                            match self.buffer.get(self.current) {
                                Some(line) => {
                                    self.buffer[self.current] =
                                        re.replace_all(line, sub).to_string();
                                    self.print_current_line();
                                }
                                None => print_error!("Invalid address"),
                            }
                        }
                        _ => print_error!("Missing pattern delimiter"),
                    }
                }
                // exit
                'q' if line.len() == 1 => break,
                // save and quit
                'w' if line == "wq" => {
                    self.save_file()?;
                    break;
                }
                // save as
                'w' if line.starts_with("w ") => {
                    self.path = line[2..].into();
                    self.save_file()?;
                }
                // save
                'w' if line.len() == 1 => {
                    self.save_file()?;
                }
                // select an address and print it
                '1'..='9' | '.' | '$' => {
                    let (start, end) = self.parse_address(&line)?;
                    if end >= self.buffer.len() {
                        print_error!("Invalid address");
                        continue;
                    }

                    for i in start..=end {
                        self.current = i;
                        self.print_current_line();
                    }
                    self.current = start;
                }
                // print the line number
                '=' => match self.buffer.get(self.current) {
                    Some(_) => println!("{}", self.current + 1),
                    None => print_error!("Invalid address"),
                },
                // comment
                '#' => {}
                _ => print_error!("Unknown command"),
            }
        }
        Ok(())
    }

    pub fn open_file(&mut self, path: String) {
        self.path.clone_from(&path);

        let data = fs::read_to_string(path);

        match data {
            Ok(data) => {
                self.buffer = data.lines().map(str::to_owned).collect();
                if !self.buffer.is_empty() {
                    self.current = self.buffer.len() - 1;
                }
                self.print_buffer_size();
            }
            Err(err) => println!("{}", err),
        }
    }

    fn save_file(&self) -> Result<(), Box<dyn Error>> {
        assert!(!self.path.is_empty());
        let mut file = fs::File::create(self.path.clone())?;
        file.write_all(self.buffer.join("\n").as_bytes())?;
        self.print_buffer_size();
        Ok(())
    }

    // https://www.gnu.org/software/ed/manual/ed_manual.html#Line-addressing
    fn parse_address(&self, s: &str) -> Result<Address, Box<dyn Error>> {
        if s.contains(",") {
            let mut parts = s.split(",");
            let start = self.parse_line_number(parts.next().unwrap())?;
            let end = self.parse_line_number(parts.next().unwrap())?;
            Ok((start, end))
        } else {
            let n = self.parse_line_number(s)?;
            Ok((n, n))
        }
    }

    fn parse_line_number(&self, s: &str) -> Result<usize, Box<dyn Error>> {
        match s.chars().next() {
            Some('1'..='9') => Ok(s.parse::<usize>()? - 1),
            Some('.') => Ok(self.current),
            Some('$') => Ok(self.buffer.len() - 1),
            _ => Err("failed to parse line number".into()),
        }
    }

    fn print_current_line(&self) {
        match self.buffer.get(self.current) {
            Some(line) => println!("{}", line),
            None => print_error!("Invalid address"),
        }
    }

    fn print_buffer_size(&self) {
        println!("{}", self.buffer.iter().map(|x| x.len()).sum::<usize>());
    }

    fn run_command(command: String) -> Result<(), Box<dyn Error>> {
        if cfg!(target_os = "windows") {
            Command::new("cmd").arg("/C").arg(command).spawn()?.wait()?;
        } else {
            Command::new("sh").arg("-c").arg(command).spawn()?.wait()?;
        }
        Ok(())
    }

    fn read_line() -> Result<String, io::Error> {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        line.pop(); // TODO: does it work on windows?
        Ok(line.to_owned())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut editor = Editor::new();

    let mut args = env::args();
    _ = args.next();
    if let Some(path) = args.next() {
        editor.open_file(path.clone());
    }

    editor.main_loop()?;
    Ok(())
}
