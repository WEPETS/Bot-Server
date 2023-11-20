pub mod b64;
pub mod time;

pub fn truncate_hex_string(input: &str, n: usize) -> String {
    let prefix = &input[..n];
    let suffix = &input[input.len() - n..];
    let ellipsis = "......";

    format!("{}{}{}", prefix, ellipsis, suffix)
}
