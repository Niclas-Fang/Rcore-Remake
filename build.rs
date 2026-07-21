use std::fs::read_dir;

const USER_BIN_DIR: &str = "./target/riscv64gc-unknown-none-elf/release";
// use std::fs::{read_dir,File};
fn main() -> std::io::Result<()> {
    let mut bins = vec![];
    for entry in read_dir(USER_BIN_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_none() {
            bins.push(path);
        }
    }

    let bins: Vec<_> = read_dir(USER_BIN_DIR)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().is_none() && path.is_file() && !path.file_name()?.to_str()?.starts_with(".") {
                return Some(path);
            } else {
                return None;
            }
        })
        .collect();
    
    println!("cargo:warning={:?}",bins);
    Ok(())
}