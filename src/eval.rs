use std::fmt;

#[derive(Debug)]
pub enum EvalError {
    InvalidNumber(String),
    EmptyExpression,
    InvalidOperator(char),
    InvalidHex(char),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::InvalidNumber(s) => write!(f, "Invalid number: '{}'", s),
            EvalError::EmptyExpression => write!(f, "Empty expression"),
            EvalError::InvalidOperator(c) => write!(f, "Invalid operator: '{}'", c),
            EvalError::InvalidHex(c) => write!(f, "Invalid hexadecimal character: '{}'", c),
        }
    }
}




pub fn calcule_st_addr(expression: &str) -> Result<u64, EvalError> {
    let mut nums = Vec::new();
    let mut ops = Vec::new();
    let mut num_buffer = String::new();

    for c in expression.chars().filter(|c| *c != ' ') {
        match c {
            '0'..='9' | 'a'..='f' | 'A'..='F' => {
                num_buffer.push(c);
            }
            'x' | 'X' => {
                if num_buffer == "0" {
                    num_buffer.push(c);
                } else {
                    return Err(EvalError::InvalidHex(c));
                }
            }
            'h' | 'H' => {
                if !num_buffer.is_empty() {
                    num_buffer.push(c);
                    nums.push(parse_number(&num_buffer)?);
                    num_buffer.clear();
                } else {
                    return Err(EvalError::InvalidHex(c));
                }
            }
            '+' | '-' | '*' | '/' => {
                if !num_buffer.is_empty() {
                    nums.push(parse_number(&num_buffer)?);
                    num_buffer.clear();
                }
                while !ops.is_empty() && precedence(ops.last().unwrap()) >= precedence(&c) {
                    let op = ops.pop().unwrap();
                    let num2 = nums.pop().ok_or(EvalError::EmptyExpression)?;
                    let num1 = nums.pop().ok_or(EvalError::InvalidOperator(op))?;
                    nums.push(apply_operator(num1, num2, op)?);
                }
                ops.push(c);
            }
            _ => return Err(EvalError::InvalidOperator(c)),
        }
    }

    if !num_buffer.is_empty() {
        nums.push(parse_number(&num_buffer)?);
    }

    while let Some(op) = ops.pop() {
        let num2 = nums.pop().ok_or(EvalError::EmptyExpression)?;
        let num1 = nums.pop().ok_or(EvalError::InvalidOperator(op))?;
        nums.push(apply_operator(num1, num2, op)?);
    }

    nums.pop().ok_or(EvalError::EmptyExpression)
}



fn parse_number(num_buffer: &str) -> Result<u64, EvalError> {
    if num_buffer.ends_with('h') || num_buffer.ends_with('H') {
        u64::from_str_radix(&num_buffer[..num_buffer.len() - 1], 16).map_err(|_| EvalError::InvalidNumber(num_buffer.to_string()))
    } else if num_buffer.starts_with("0x") || num_buffer.starts_with("0X") {
        u64::from_str_radix(&num_buffer[2..], 16).map_err(|_| EvalError::InvalidNumber(num_buffer.to_string()))
    } else {
        num_buffer.parse().map_err(|_| EvalError::InvalidNumber(num_buffer.to_string()))
    }
}



fn precedence(op: &char) -> u8 {
    match op {
        '+' | '-' => 1,
        '*' | '/' => 2,
        _ => 0,
    }
}



fn apply_operator(num1: u64, num2: u64, op: char) -> Result<u64, EvalError> {
    match op {
        '+' => Ok(num1 + num2),
        '-' => Ok(num1 - num2),
        '*' => Ok(num1 * num2),
        '/' => Ok(num1 / num2),
        _ => Err(EvalError::InvalidOperator(op)),
    }
}
