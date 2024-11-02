use std::{
    env,
    error::Error,
    fs::{self},
    io::{self, Write},
};

fn read_line() -> Result<String, io::Error> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

fn get_buffer_size(buffer: &[String]) -> usize {
    buffer.iter().map(|x| x.len()).sum()
}

fn open_file(
    path: String,
    buffer: &mut Vec<String>,
    current: &mut usize,
) -> Result<(), Box<dyn Error>> {
    *buffer = fs::read_to_string(path)?
        .trim()
        .lines()
        .map(str::to_owned)
        .collect();
    if !buffer.is_empty() {
        *current = buffer.len() - 1;
    }
    println!("{}", get_buffer_size(buffer));
    Ok(())
}

fn write_file(path: String, buffer: &[String]) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(path)?;
    file.write_all(buffer.join("\n").as_bytes())?;
    println!("{}", get_buffer_size(buffer));
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    _ = args.next();

    let mut path = String::new();
    let mut buffer: Vec<String> = vec![];
    let mut current = 0;

    if let Some(p) = args.next() {
        path = p;
        open_file(path.clone(), &mut buffer, &mut current)?;
    }

    loop {
        let line = read_line()?;
        if line.is_empty() {
            continue;
        }

        match line.chars().next().unwrap() {
            // append
            'a' if line.len() == 1 => loop {
                let line = read_line()?;
                if line == "." {
                    break;
                }
                current += 1;
                buffer.insert(current, line);
            },
            // print line number
            'n' if line.len() == 1 => {
                println!("{}\t{}", current + 1, buffer.get(current).unwrap());
            }
            // print line
            'p' | '.' if line.len() == 1 => {
                println!("{}", buffer.get(current).unwrap());
            }
            // exit
            'q' if line.len() == 1 => break,
            // select last line and print it
            '$' if line.len() == 1 => {
                current = buffer.len() - 1;
                println!("{}", buffer.get(current).unwrap());
            }
            'e' if line.starts_with("e ") => {
                path = line[2..].into();
                open_file(path.clone(), &mut buffer, &mut current)?;
            }
            'w' => {
                write_file(path.clone(), &buffer)?;
            }
            // select line and print it
            '0'..='9' => {
                let n = line.trim().parse::<usize>().unwrap() - 1;
                match buffer.get(n) {
                    Some(line) => {
                        current = n;
                        println!("{}", line);
                    }
                    None => println!("?"),
                }
            }
            _ => {
                println!("?")
            }
        }
    }

    Ok(())
}
