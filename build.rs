use std::fs::read_dir;

const USER_BIN_DIR: &str = "./target/riscv64gc-unknown-none-elf/release";
// use std::fs::{read_dir,File};
fn main() -> std::io::Result<()> {
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
    for bin in bins {
        let path_elf = bin.to_str().unwrap();
        let path_bin = format!("{path_elf}.bin");
        std::process::Command::new("rust-objcopy")
            .args(["--binary-architecture=riscv64","--strip-all","-O","binary",path_elf,&path_bin])
            .status()
            .expect("failed to turn elf files into binary files");
    }    
    
    //println!("cargo:warning={:?}",bins);
    Ok(())
}