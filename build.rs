use std::{fs::read_dir, io::ErrorKind, process::exit, process::Command};

fn main() {
    // Check if scdoc command exists
    match Command::new("scdoc").spawn() {
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                exit(0);
            }
        }
        _ => {}
    }

    let mut man_pages: Vec<(String, String)> = Vec::new();
    for path in read_dir("./docs").unwrap() {
        let path = path.unwrap();
        if path.file_type().unwrap().is_dir() {
            continue;
        }

        if let Some(file_name) = path.path().to_str() {
            let man_page_name = file_name.replace(".scd", ".gz");
            man_pages.push((file_name.to_string(), man_page_name));
        }
    }

    for man_page in man_pages {
        _ = Command::new("sh")
            .arg("-c")
            .arg(format!("scdoc <{}>{}", man_page.0, man_page.1))
            .spawn();
    }
}
