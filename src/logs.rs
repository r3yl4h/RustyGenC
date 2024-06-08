use std::fmt;
use crate::TIME;

#[allow(dead_code)]
pub enum CriticalErr {
    StackSaveEmpty,
    StackPtrIsNotRsp,
    NotFoundPathFile(Box<dyn fmt::Display>),
    InvalidFormat(Box<dyn fmt::Display>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Warning {
    UnknownLabel,
    WtchStack,
    BadStackPtrHigh,
    BadStackPtrLow,
}

#[allow(dead_code)]
pub enum LogLevel {
    CriticalErr(CriticalErr),
    Error(Error),
    Debug(DebugMsg),
    Warning(Warning),
    Custom(Severity),
}

pub enum DebugMsg {
    Terminate
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Severity {
    WARNING,
    ERROR,
}

#[allow(dead_code)]
pub enum Error {
    DestinationImmediate,
    DestinationLabel,
    InvalidInstruction,
    NotOp,
    Arithmetic(Box<dyn fmt::Display>),
    BadStackPtrHigh,
    BadStackPtrLow,
    UnknowLabel,
}


pub const ERROR_COLOR: &str = "\x1b[31m";
pub const WARNING_COLOR: &str = "\x1b[33m";
pub const DEBUG_GREEN: &str = "\x1b[32m";
pub const RESET_COLOR: &str = "\x1b[0m";



pub fn log_critical(error: CriticalErr, msg: String) {
    let formatted_msg = match error {
        CriticalErr::StackSaveEmpty => "There is no longer any save point in the stack, which is very serious in a program".to_string(),
        CriticalErr::StackPtrIsNotRsp => "The 1st element of the \"stack ptr\" vector is not sp".to_string(),
        CriticalErr::NotFoundPathFile(error_file) => format!("Error opening '{msg}' file : '{error_file}'"),
        CriticalErr::InvalidFormat(error_form) => format!("The file format of '{msg}' is invalid : '{error_form}'"),
    };
    eprintln!("[{ERROR_COLOR}Critical{RESET_COLOR}] -> {}", formatted_msg);
    std::process::exit(1);
}

pub fn log_error(error: Error, msg: String) {
    let formatted_msg = match error {
        Error::DestinationImmediate => String::from("The destination cannot be immediate"),
        Error::DestinationLabel => format!("The destination cannot be a label: '{msg}'"),
        Error::InvalidInstruction => format!("The instruction '{msg}' is not supported"),
        Error::NotOp => String::from("The instruction must be accompanied by 2 operands"),
        Error::Arithmetic(arithmetic_msg) => format!("{msg} ({arithmetic_msg})"),
        Error::BadStackPtrHigh => String::from("The value of the stack ptr has become higher than the top of the stack"),
        Error::BadStackPtrLow => String::from("The value of the stack ptr has become negative"),
        Error::UnknowLabel => format!("The label '{msg}' is unknown"),
    };
    eprintln!("LINE {}: [{}ERROR{}] -> {}",unsafe {crate::converter::LINE_COUNT}, ERROR_COLOR, RESET_COLOR, formatted_msg);
}

pub fn log_warning(warning: Warning, msg: String) {
    let formatted_msg = match warning {
        Warning::UnknownLabel => format!("unknown label '{}' (it's not critical but it's an error to resolve)", msg),
        Warning::WtchStack => "This use of the stack pointer is not recommended and may cause unpredictable behavior, which should be monitored".to_string(),
        Warning::BadStackPtrHigh => "the value of the stack ptr has become higher than the top of the stack, to monitor".to_string(),
        Warning::BadStackPtrLow => "the value of the stack ptr has become negative, this is abnormal and can cause serious problems, to be monitored".to_string(),
    };
    println!("LINE {}: [{}WARNING{}] -> {}" , unsafe {crate::converter::LINE_COUNT}, WARNING_COLOR, RESET_COLOR, formatted_msg);
}



pub fn log_debug(debug_msg: DebugMsg) {
    let (type_d, msg) = match debug_msg {
        DebugMsg::Terminate => ("Finished", format!("the code was generated in {:?}", unsafe {TIME.elapsed()}))
    };
    println!("[{DEBUG_GREEN}{type_d}{RESET_COLOR}] -> {msg}\n")
}



pub fn log_custom(severity: Severity, msg: String) {
    let formatted_msg = match severity {
        Severity::WARNING => format!("{}", msg),
        Severity::ERROR => format!("{}", msg),
    };
    match severity {
        Severity::WARNING => println!("[{}WARNING{}] -> {}", WARNING_COLOR, RESET_COLOR, formatted_msg),
        Severity::ERROR => eprintln!("[{}ERROR{}] -> {}", ERROR_COLOR, RESET_COLOR, formatted_msg),
    }
}



#[macro_export] macro_rules! print_msg {
    ($level:expr, $($arg:tt)*) => {
        match $level {
            LogLevel::CriticalErr(error) => log_critical(error, format!($($arg)*)),
            LogLevel::Error(error) => log_error(error, format!($($arg)*)),
            LogLevel::Warning(warning) => log_warning(warning, format!($($arg)*)),
            LogLevel::Custom(severity) => log_custom(severity, format!($($arg)*)),
            LogLevel::Debug(debug) => log_debug(debug),
        }
    }
}
