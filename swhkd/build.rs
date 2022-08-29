use std::{
    fs::{read_dir, File, OpenOptions},
    io::ErrorKind,
    path::Path,
    process::{exit, Command, Stdio},
};

fn main() {
    if let Err(e) = Command::new("scdoc")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let ErrorKind::NotFound = e.kind() {
            exit(0);
        }
    }

    let mut man_pages: Vec<(String, String)> = Vec::new();
    for path in read_dir("../docs").unwrap() {
        let path = path.unwrap();
        if path.file_type().unwrap().is_dir() {
            continue;
        }

        if let Some(file_name) = path.path().to_str() {
            if path.path().extension().unwrap().to_str().unwrap() == "gz" {
                continue;
            }

            let man_page_name = file_name.replace(".scd", ".gz");
            man_pages.push((file_name.to_string(), man_page_name));
        }
    }

    for man_page in man_pages {
        let output =
            OpenOptions::new().write(true).create(true).open(Path::new(&man_page.1)).unwrap();
        _ = Command::new("scdoc")
            .stdin(Stdio::from(File::open(man_page.0).unwrap()))
            .stdout(output)
            .spawn();
    }
}
