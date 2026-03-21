
pub fn strings_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    for i in 0..a.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
    }
    true
}