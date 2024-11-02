use std::{env, error::Error, fs, io};

fn read_line() -> Result<String, io::Error> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    _ = args.next();

    let mut buffer: Vec<String> = vec![];
    let mut current = 0;

    if let Some(path) = args.next() {
        let data = fs::read_to_string(path)?.trim().to_owned();
        println!("{}", data.len());

        buffer = data.lines().map(str::to_owned).collect();
        if !buffer.is_empty() {
            current = buffer.len() - 1;
        }
    };

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
                buffer.insert(current, line);
                current += 1;
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
