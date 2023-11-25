use std::{fs::File, io::Read, path::PathBuf};

pub mod b64;
pub mod time;

pub fn truncate_hex_string(input: &str, n: usize) -> String {
    let prefix = &input[..n];
    let suffix = &input[input.len() - n..];
    let ellipsis = "......";

    format!("{}{}{}", prefix, ellipsis, suffix)
}

pub async fn read_file(file_path: &str) -> String {
    // Construct the full file path
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(file_path);

    // Read the content of the file synchronously
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    content
}
