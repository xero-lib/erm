use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Args {
    // Directory to recursively remove empty directories from
    path: Option<String>,
}

fn main() {
    let path = match Args::parse().path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap()
    };
    
    if let Err(e) = std::fs::read_dir(&path) {
        panic!("Unable to read target dir: {}", e.to_string());
    }

    delete_recursive(path.to_owned());

    match std::fs::remove_dir(&path) {
        Ok(_) => println!("Removed {}", path.to_str().unwrap()),
        Err(e) => eprintln!("Failed to remove {}: {}", path.to_str().unwrap(), e.kind())
    }
}

fn delete_recursive(path: PathBuf) {
    let dir = match std::fs::read_dir(&path) {
        Ok(dir) => dir,
        Err(e) => { 
            eprintln!("Unable to read dir {}: {e}", path.to_string_lossy());
            return;
        },
    };
    
    for i in dir {
        match &i {
            Ok(e) => match e.file_type() {
                Ok(t) if t.is_dir() => delete_recursive(e.path()),
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

}

#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn rmdirs() -> Result<(), std::io::Error> {
    std::fs::create_dir_all("test/contains_dirs/hello")?;
    std::fs::create_dir_all("test/contains_dirs/world/inside")?;
    
    delete_recursive(PathBuf::from("test"));

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

    delete_recursive(PathBuf::from("test"));
    let result = std::fs::read_dir("test");

    std::fs::remove_dir_all("test")?;

    match result {
        Ok(_) => assert!(true, "Properly recognized files were remaining"),
        Err(_) => assert!(false, "Deleted even though files remained"),
    }

    Ok(())
}

