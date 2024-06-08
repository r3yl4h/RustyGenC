pub fn im_for_dest_mem(im: &str, types: crate::types::Type, op2: &str, wait2types: &mut bool, sequence: &mut Vec<String>) {
    let expr_type = if types != crate::types::Type::UN {
        crate::types::get_expr_with_len(types.get_size_with_type())
    } else {
        crate::types::get_expr_with_len(crate::types::Type::get_type_of_reg(op2).get_size_with_type())
    };
    *wait2types = true;
    if expr_type != "" {
        sequence.push(format!("*({expr_type}*){im}"));
    }else {
        sequence.push(format!("*{im}"));
    }
}