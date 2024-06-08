pub fn handle_neg(sequence: &mut Vec<String>) -> bool {
    sequence.push(format!(" = -{}", sequence.join("")));
    true
}