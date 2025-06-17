mod components;
mod errors;
mod parser;
mod syntax;
mod tokenizer;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(f) = args.get(1) {
        let contents = std::fs::read_to_string(f).unwrap();
        match syntax::Chapter::from_str(&contents) {
            Ok(mut chap) => {
                if let Some(o) = args.get(2) {
                    chap.process();
                    chap.to_html(o).unwrap();
                } else {
                    println!("{chap:?}")
                }
            }
            Err(e) => println!("{}", e.user_msg(Some(f))),
        }
    } else {
        println!("Provide a chapter file");
    }
}
