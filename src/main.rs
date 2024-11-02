use std::{
    env,
    error::Error,
    fs::{self},
    io::{self, Write},
};

struct Editor {
    path: String,
    buffer: Vec<String>,
    current: usize,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            path: String::new(),
            buffer: vec![],
            current: 0,
        }
    }

    pub fn main_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let line = self.read_line()?;
            if line.is_empty() {
                continue;
            }

            // https://www.gnu.org/software/ed/manual/ed_manual.html#Commands
            match line.chars().next().unwrap() {
                // insert after current line
                'a' if line.len() == 1 => loop {
                    let line = self.read_line()?;
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
                // delete line
                'd' if line.len() == 1 => {
                    if self.current >= self.buffer.len() {
                        println!("?");
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
                    let line = self.read_line()?;
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
                // print line number
                'n' if line.len() == 1 => match self.buffer.get(self.current) {
                    Some(line) => println!("{}\t{}", self.current + 1, line),
                    None => println!("?"),
                },
                // print line
                'p' | '.' if line.len() == 1 => {
                    self.print_current_line();
                }
                // exit
                'q' if line.len() == 1 => break,
                // save file
                'w' if line == "wq" => {
                    self.write_file(self.path.clone())?;
                    break;
                }
                'w' if line.len() == 1 => {
                    self.write_file(self.path.clone())?;
                }
                // select last line and print it
                '$' if line.len() == 1 => {
                    self.current = self.buffer.len() - 1;
                    self.print_current_line();
                }
                '0'..='9' => {
                    if line.contains(",") {
                        // print multiple lines
                        let mut parts = line.split(",");
                        let start = self.parse_line_number(parts.next().unwrap())?;
                        let end = self.parse_line_number(parts.next().unwrap())?;

                        let old_current = self.current;
                        for i in start..=end {
                            self.current = i;
                            self.print_current_line();
                        }
                        self.current = old_current;
                    } else {
                        // select line and print it
                        let n = self.parse_line_number(&line)?;
                        match self.buffer.get(n) {
                            Some(_) => {
                                self.current = n;
                                self.print_current_line();
                            }
                            None => println!("?"),
                        }
                    }
                }
                _ => {
                    println!("?")
                }
            }
        }
        Ok(())
    }

    pub fn open_file(&mut self, path: String) {
        self.path.clone_from(&path);

        let data = fs::read_to_string(path);

        match data {
            Ok(data) => {
                self.buffer = data.trim().lines().map(str::to_owned).collect();
                if !self.buffer.is_empty() {
                    self.current = self.buffer.len() - 1;
                }
                self.print_buffer_size();
            }
            Err(err) => println!("{}", err),
        }
    }

    fn write_file(&self, path: String) -> Result<(), Box<dyn Error>> {
        assert!(!path.is_empty());
        let mut file = fs::File::create(path)?;
        file.write_all(self.buffer.join("\n").as_bytes())?;
        self.print_buffer_size();
        Ok(())
    }

    // https://www.gnu.org/software/ed/manual/ed_manual.html#Line-addressing
    fn parse_line_number(&self, s: &str) -> Result<usize, Box<dyn Error>> {
        match s.chars().next().unwrap() {
            '0'..='9' => Ok(s.parse::<usize>()? - 1),
            '.' => Ok(self.current),
            '$' => Ok(self.buffer.len() - 1),
            _ => Err("invalid line number".into()),
        }
    }

    fn read_line(&self) -> Result<String, io::Error> {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        Ok(line.trim().to_owned())
    }

    fn print_current_line(&self) {
        match self.buffer.get(self.current) {
            Some(line) => println!("{}", line),
            None => println!("?"),
        }
    }

    fn print_buffer_size(&self) {
        println!("{}", self.buffer.iter().map(|x| x.len()).sum::<usize>());
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
