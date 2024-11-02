use std::{env, error::Error, fs, io};

fn read_line() -> Result<String, io::Error> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    _ = args.next();

    let mut buffer: Vec<String> = match args.next() {
        Some(path) => fs::read_to_string(path)?
            .split("\n")
            .map(|s| s.to_string())
            .collect(),
        None => vec![],
    };

    loop {
        let line = read_line()?;
        if line.is_empty() {
            continue;
        }

        match line.as_str() {
            "a" => loop {
                let line = read_line()?;
                if line == "." {
                    break;
                }
                buffer.push(line);
            },
            _ => {
                if let Ok(n) = line.trim().parse::<usize>() {
                    match buffer.get(n - 1) {
                        Some(line) => println!("{}", line),
                        None => println!("?"),
                    }
                } else {
                    println!("?")
                }
            }
        }
    }
}
