use crate::converter;

pub fn handle_jmp(code: &mut Vec<String>, liner: Vec<&str>, skip2: &mut bool) -> bool {
    code.push(format!("{}goto {};", unsafe {converter::IDENT.clone()}, liner.last().unwrap()));
    *skip2 = true;
    true
}