use ezjs;

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct MyExpander {

}

impl ezjs::runtime::Expandable for MyExpander {
    fn hash(&self) -> u64 {
        return 0;
    }
}

pub fn main() {
    let mut rt = ezjs::new_runtime::<MyExpander>();

    println!("REPL of ezjs v0.1.0");
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        let content = fs::read_to_string(&args[i]).unwrap();
        let vmf = ezjs::build_function_from_code(&content).unwrap();
        if args.len() == 2 {
            ezjs::dump_function(&vmf);
        }
        ezjs::run_script(&mut rt, vmf).unwrap();
    }

    loop {
        print!("=>");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(_n) => {
                if line != "" {
                    let begin = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    
                    let vmf = ezjs::build_function_from_code(&line).unwrap();
                    let _ret = ezjs::run_script(&mut rt, vmf).unwrap();
                    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

                    println!("<{}>", end - begin);
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    rt.push_undefined();
}