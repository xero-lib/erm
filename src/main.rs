use std::{path::PathBuf, fs::ReadDir};

use clap::Parser;

#[derive(Parser)]
struct Args {
    // Directory to recursively remove empty directories and/or files from
    path: Option<String>,
    // Delete both empty files and empty directories
    #[arg(short, long, num_args(0))]
    dirs: bool,    // Delete only empty files while ignoring empty directories
    #[arg(short, long, num_args(0))]
    files: bool,
}

fn main() {
    let args = Args::parse();

    let path = match args.path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap()
    };
    
    if let Err(e) = std::fs::read_dir(&path) {
        panic!("Unable to read target dir: {}", e.to_string());
    }

    if args.dirs == args.files { delete_recursive(path); } else
    if args.dirs { delete_dirs_recursive(path); } else
    if args.files { delete_files_recursive(path); }
}

fn get_dir(path: &PathBuf) -> Result<ReadDir, ()> {
    match std::fs::read_dir(&path) { 
        Ok(d) => Ok(d),
        Err(e) => {
            eprintln!("Unable to read dir {}: {}", path.to_string_lossy(), e.to_string());
            Err(())
        }
    }
}

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return,
        }
    }
}

// macro_rules! unwrap_or_continue {
//     ( $e:expr, $t: ) => {
//         match $e {
//             Ok(x) => x,
//             Err(_) => continue,
//         }
//     };
// }

fn delete_dirs_recursive(path: PathBuf) {
    let dir = unwrap_or_return!(get_dir(&path));
    
    for i in dir {
        match &i {
            Ok(e) => match e.file_type() {
                Ok(t) if t.is_dir() => delete_dirs_recursive(e.path()),
                _ => continue,
            },
            Err(e) => {
                eprintln!("Unable to enter directory {}: {e:#?}", i.as_ref().unwrap().path().to_string_lossy());
                continue;
            }
        }

        let unwrapped = i.unwrap();
        match std::fs::remove_dir(unwrapped.path()) {
            Ok(_) => println!("Removed {}", unwrapped.path().to_string_lossy()),
            Err(e) => eprintln!("Failed to remove {}: {e}", unwrapped.file_name().to_string_lossy())
        }
    }

    match std::fs::remove_dir(&path) {
        Ok(_) => println!("Removed {}", path.to_str().unwrap()),
        Err(e) => eprintln!("Failed to remove {}: {}", path.to_str().unwrap(), e.kind())
    }
}

fn delete_files_recursive(path: PathBuf) {
    let dir = unwrap_or_return!(get_dir(&path));

    for i in dir {
        match &i {
            Ok(entry) => {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => { eprintln!("Error fetching metadata for {}", e.to_string()); continue }
                };

                if meta.file_type().is_dir() { delete_files_recursive(entry.path()) } else
                if meta.file_type().is_file() && meta.len() == 0 {
                    if let Err(e) = std::fs::remove_file(&entry.path()) {
                        eprintln!("Unable to remove empty file {}: {}", entry.path().to_string_lossy(), e.to_string());
                        continue;
                    }
                }
            },
            Err(e) => {
                eprintln!("Unable to enter directory {}: {e:#?}", i.as_ref().unwrap().path().to_string_lossy());
                continue;
            }
        }
    }
}

fn delete_recursive(path: PathBuf) {
    let dir = unwrap_or_return!(get_dir(&path));

    for i in dir {
        match &i {
            Ok(entry) => {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => { eprintln!("Error fetching metadata for {}", e.to_string()); continue }
                };

                if meta.file_type().is_dir() { delete_recursive(entry.path()) } else
                if meta.file_type().is_file() && meta.len() == 0 {
                    if let Err(e) = std::fs::remove_file(&entry.path()) {
                        eprintln!("Unable to remove empty file {}: {}", entry.path().to_string_lossy(), e.to_string());
                        continue;
                    }
                }
            },
            Err(e) => {
                eprintln!("Unable to enter directory {}: {e:#?}", i.as_ref().unwrap().path().to_string_lossy());
                continue;
            }
        }

        let unwrapped = i.unwrap();
        match std::fs::remove_dir(unwrapped.path()) {
            Ok(_) => println!("Removed {}", unwrapped.path().to_string_lossy()),
            Err(e) => eprintln!("Failed to remove {}: {e}", unwrapped.file_name().to_string_lossy())
        }
    }

    match std::fs::remove_dir(&path) {
        Ok(_) => println!("Removed {}", path.to_str().unwrap()),
        Err(e) => eprintln!("Failed to remove {}: {}", path.to_str().unwrap(), e.kind())
    }
}

#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn rmdirs() -> Result<(), std::io::Error> {
    std::fs::create_dir_all("test/contains_dirs/hello")?;
    std::fs::create_dir_all("test/contains_dirs/world/inside")?;
    
    delete_dirs_recursive(PathBuf::from("test"));

    match std::fs::read_dir("test") {
        Ok(_) => assert!(false, "Directories were not properly deleted."),
        Err(_) => assert!(true, "Directories were properly deleted."),
    }

    Ok(())
}


#[test]
#[serial]
fn rmfiles() -> Result<(), std::io::Error> {
    use std::io::Write;
    std::fs::create_dir_all("test/contains_files_and_dirs/hello")?;
    std::fs::create_dir_all("test/contains_files_and_dirs/world")?;
    let mut file = std::fs::File::create("test/contains_files_and_dirs/hello/hello.txt")?;
    file.write_all("hello".as_bytes())?;

    delete_dirs_recursive(PathBuf::from("test"));
    let result = std::fs::read_dir("test");

    std::fs::remove_dir_all("test")?;

    match result {
        Ok(_) => assert!(true, "Properly recognized files were remaining"),
        Err(_) => assert!(false, "Deleted even though files remained"),
    }

    Ok(())
}

