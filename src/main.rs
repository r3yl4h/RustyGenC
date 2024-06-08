use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ptr::addr_of;
use std::time::Instant;
use once_cell::sync::Lazy;
use crate::logs::*;
use std::path::Path;

mod converter;
mod logs;
mod stack;
mod eval;
mod label;
mod types;
mod function;
mod operand;
mod instruction;
mod flow_control;


static mut ARCH: types::Type = types::Type::X64;


static mut TIME: Lazy<Instant> = Lazy::new(|| Instant::now());



fn main() {
    Lazy::force(unsafe {&*addr_of!(TIME)});
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: RustygenC <input file(s)>");
    }else {
        let input_path = &args[1];
        match File::open(input_path) {
            Ok(input_file) => {
                let asm_code = BufReader::new(input_file).lines().collect::<Result<Vec<String>, _>>();
                match asm_code {
                    Ok(asm_code) => {
                        for line in converter::converter(&asm_code) {
                            println!("{line}");
                        }
                    }
                    Err(e) => print_msg!(LogLevel::CriticalErr(CriticalErr::InvalidFormat(Box::new(e))), "{}", Path::new(input_path).file_name().unwrap().to_str().unwrap()),
                }
            }
            Err(e1) => print_msg!(LogLevel::CriticalErr(CriticalErr::NotFoundPathFile(Box::new(e1))), "{input_path}")
        }
    }
}

