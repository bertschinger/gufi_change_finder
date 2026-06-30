use std::{fs, io};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::ErrorKind;
use std::process::Command;

fn main() {
    let mut log_file = File::create("build.log").expect("Failed to create build.log file");
	
    let top_path = PathBuf::from(".").canonicalize().expect("failed to canonicalize top level path");

    log_file.write_all(b"Top level MarFS source directory: ").expect("Failed to write to build.log");
    log_file.write_all(b"\n").expect("Failed to write to build.log");

    let scoutwrap_clean = Command::new("make")
                            .arg("-C")
                            .arg("src/scoutwrap")
                            .arg("clean")
                            .status()
                            .expect("failed to clean src/scoutwrap");

    let scoutwrap_make = Command::new("make")
                            .arg("-C")
                            .arg("src/scoutwrap")
                            .status()
                            .expect("failed to make src/scoutwrap");

       
    let bindings_path = "src/bindings.rs";
    match fs::remove_file(bindings_path) {
        Ok(_) => { },
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                log_file.write_all(b"\nfailed to remove old bindings file: does not exist\n").expect("failed to write to build.log");
            }
            else {
                log_file.write_all(b"\nfailed to remove old bindings file\n").expect("failed to write to build.log");
            }
        }
    };


    // ScoutFS
    let scoutfs_path = PathBuf::from("src/scoutfs").canonicalize().expect("Cannot canonicalize path");
    
    log_file.write_all(b"Top level ScoutFS source directory: ").expect("Failed to write to build.log");
    log_file.write_all(scoutfs_path.clone().into_os_string().as_encoded_bytes()).expect("Failed to write to build.log");
    log_file.write_all(b"\n").expect("Failed to write to build.log");

    // ScoutFS headers

    let scoutwrap_path = top_path.join("src/scoutwrap/scoutwrap.h");
    let scoutwrap_path_str = scoutwrap_path.to_str().expect("Failed to convert header path to String");

    // ScoutFS is kernel code and does not provide libs; Need a user library wrapper for the ioctl
    
    log_file.write_all(b"Searching for libs in: ").expect("Failed to write to build.log");
    log_file.write_all(b"\n\n").expect("Failed to write to build.log");
    
    let bindings = bindgen::Builder::default()
                    .header(scoutwrap_path_str)
                    .clang_arg(format!("-I{}/src/scoutfs", top_path.to_str().unwrap()))
                    .clang_arg("-I/usr/include/libxml2")
                    .blocklist_item("^FP_.*$") // for some reason, FP_NAN, etc. are defined twice, so block them and use the libc variant
                    .generate()
                    .expect("Failed to generate bindings for {header_path_str");
    
    bindings.write_to_file("src/bindings.tmp").expect("Failed to write to bindings.tmp");
    let mut tmp_bindings = fs::OpenOptions::new()
            .read(true)
            .open("src/bindings.tmp")
            .unwrap();

    let mut real_bindings = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(bindings_path)
            .unwrap();

    let _result = io::copy(&mut tmp_bindings, &mut real_bindings);

    fs::remove_file("src/bindings.tmp").expect("Failed to remove src/bindings.tmp");

    //let log_string = format!("Wrote bindings from {header_path_str} to {bindings_path}\n"); 
    //log_file.write_all(log_string.as_bytes()).expect("Failed to write to build.log");

    let scoutwrap_lib_path = top_path.join("src/scoutwrap");
    
    println!("cargo:rustc-link-search={}", scoutwrap_lib_path.to_str().unwrap());
    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", scoutwrap_lib_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=scoutwrap");

}
