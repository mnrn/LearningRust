extern crate rand;

use std::io::{
    BufReader,
    BufRead,
};
use rand::{ 
    SeedableRng,
    Rng,
    XorShiftRng,
};

enum Affix {
    Prefix,
    Suffix,
}


fn spankle(buffer: &mut std::string::String, rng: &mut rand::XorShiftRng) {
    let ts = vec![(Some(Affix::Suffix), "app"), (Some(Affix::Suffix), "site"), (Some(Affix::Suffix), "time"), (Some(Affix::Prefix), "get"), (None, "")];
    let t = &ts[rng.gen_range(0, ts.len() - 1)];

    match t.0 {
        Some(Affix::Prefix) => {
            println!("echo: {}{}", t.1, buffer.trim_right());
        }
        Some(Affix::Suffix) => {
            println!("echo: {}{}", buffer.trim_right(), t.1);
        }
        None => {
            println!("{}", buffer.trim_right());
        }
    }
    buffer.clear();
}


fn main() {
    let stdin = std::io::stdin();               // A handle to the standard input stream of a process.
    let mut reader = BufReader::new(stdin);     // Creates a new BufReader with a default buffer capacity.
    
    let seed = [1,1,1,1, 2,2,2,2, 3,3,3,3, 4,4,4,4];
    let mut rng = XorShiftRng::from_seed(seed);
    
    let mut buffer = String::new();
    loop {
        match reader.read_line(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                spankle(&mut buffer, &mut rng);
            }
            Err(e) => {
                eprintln!("error: {}", e);  // Output to stderr.
            }
        };
    }
}
