use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let url = "https://cdn.jsdelivr.net/npm/mermaid@latest/dist/mermaid.min.js";
    let out_path = Path::new("src").join("mermaid.min.js");

    let response = reqwest::blocking::get(url)
        .expect("Failed to fetch URL")
        .text()
        .expect("Failed to read response text");

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let mut file = fs::File::create(&out_path).expect("Failed to create file");
    file.write_all(response.as_bytes())
        .expect("Failed to write to file");

    println!("cargo:rerun-if-changed=build.rs");
}
