use crate::types::Type;

pub fn handle_ret(sequence: &mut Vec<String>) -> bool {
    if let Some(ctx) = crate::converter::CONTEXT.get() {
        let context = ctx.lock().unwrap();
        if context.types != Type::UN {
            if context.types != Type::X64 {
                sequence.push(format!("return ({})rax", context.types.get_expr_with_type()));
            }else {
                sequence.push(String::from("return rax"));
            }
        }else {
            sequence.push("return".to_string());
        }
    }
    true
}