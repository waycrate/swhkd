use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::{fs, path};

#[derive(Debug)]
pub enum Error {
    ConfigNotFound,
    Io(std::io::Error),
    InvalidConfig(ParseError),
}

#[derive(Debug)]
pub enum ParseError {
    // u32 is the line number where an error occured
    UnknownSymbol(u32),
    MissingCommand(u32),
    CommandWithoutWhitespace(u32),
}

impl From<std::io::Error> for Error {
    fn from(val: std::io::Error) -> Self {
        if val.kind() == std::io::ErrorKind::NotFound {
            Error::ConfigNotFound
        } else {
            Error::Io(val)
        }
    }
}

#[derive(Debug)]
pub struct Hotkey {
    keysyms: Vec<evdev::Key>,
    command: String,
}

impl Hotkey {
    fn new(keysyms: Vec<evdev::Key>, command: String) -> Self {
        Hotkey { keysyms, command }
    }
}

pub fn load(path: path::PathBuf) -> Result<Vec<Hotkey>, Error> {
    let file_contents = load_file_contents(path)?;
    parse_contents(file_contents)
}

fn load_file_contents(path: path::PathBuf) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_contents(contents: String) -> Result<Vec<Hotkey>, Error> {
    let key_to_evdev_key: HashMap<&str, evdev::Key> = HashMap::from([
        ("q", evdev::Key::KEY_Q),
        ("w", evdev::Key::KEY_W),
        ("e", evdev::Key::KEY_E),
        ("r", evdev::Key::KEY_R),
        ("t", evdev::Key::KEY_T),
        ("y", evdev::Key::KEY_Y),
        ("u", evdev::Key::KEY_U),
        ("i", evdev::Key::KEY_I),
        ("o", evdev::Key::KEY_O),
        ("p", evdev::Key::KEY_P),
        ("a", evdev::Key::KEY_A),
        ("s", evdev::Key::KEY_S),
        ("d", evdev::Key::KEY_D),
        ("f", evdev::Key::KEY_F),
        ("g", evdev::Key::KEY_G),
        ("h", evdev::Key::KEY_H),
        ("j", evdev::Key::KEY_J),
        ("k", evdev::Key::KEY_K),
        ("l", evdev::Key::KEY_L),
        ("z", evdev::Key::KEY_Z),
        ("x", evdev::Key::KEY_X),
        ("c", evdev::Key::KEY_C),
        ("v", evdev::Key::KEY_V),
        ("b", evdev::Key::KEY_B),
        ("n", evdev::Key::KEY_N),
        ("m", evdev::Key::KEY_M),
        ("1", evdev::Key::KEY_1),
        ("2", evdev::Key::KEY_2),
        ("3", evdev::Key::KEY_3),
        ("4", evdev::Key::KEY_4),
        ("5", evdev::Key::KEY_5),
        ("6", evdev::Key::KEY_6),
        ("7", evdev::Key::KEY_7),
        ("8", evdev::Key::KEY_8),
        ("9", evdev::Key::KEY_9),
        ("0", evdev::Key::KEY_0),
    ]);

    let lines: Vec<&str> = contents.split('\n').collect();
    println!("{:?}", lines);
    // Go through each line, ignore comments and empty lines, mark lines starting with whitespace
    // as commands, and mark the other lines as keysyms. Mark means storing a line's type and the
    // line number in a vector.

    let mut lines_with_types: Vec<(&str, u32)> = Vec::new();
    for (line_number, line) in lines.iter().enumerate() {
        if line.trim().starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            lines_with_types.push(("command", line_number as u32));
        } else {
            lines_with_types.push(("keysym", line_number as u32));
        }
    }

    println!("{:?}", lines_with_types);

    // Go through lines_with_types, and add the next line over and over until the current line no
    // longer ends with backslash. (Only if the lines have the same type)
    let mut actual_lines: Vec<(&str, u32, String)> = Vec::new();
    let mut current_line_type = lines_with_types[0].0;
    let mut current_line_number = lines_with_types[0].1;
    let mut current_line_string = String::new();
    for (line_type, line_number) in lines_with_types {
        if line_type != current_line_type {
            current_line_type = line_type;
            current_line_number = line_number;
            current_line_string = String::new();
        }
        current_line_string.push_str(lines[line_number as usize].trim());
        if !current_line_string.ends_with('\\') {
            actual_lines.push((
                current_line_type,
                current_line_number,
                current_line_string.replace("\\", ""),
            ));
            current_line_type = line_type;
            current_line_number = line_number;
            current_line_string = String::new();
        }
    }

    println!("{:?}", actual_lines);

    let mut hotkeys: Vec<Hotkey> = Vec::new();

    // Go through actual_lines.
    // If the line is a keysym, check if it's valid. If it is, add it to the current hotkey. If
    // not, return an error.
    // Check the next line of the keysym. If it's a keysym, add an empty command to the current
    // hotkey. If it's a command, add it to the current hotkey. Then continue searching until we
    // find another keysym.

    for (i, item) in actual_lines.iter().enumerate() {
        let line_type = item.0;
        let line_number = item.1;
        let line = &item.2;
        let mut current_hotkey = Hotkey::new(Vec::new(), String::new());
        if line_type == "keysym" {
            // We split the line on '+' and trim each item
            let keysyms: Vec<&str> = line.split('+').map(|s| s.trim()).collect();
            for keysym in keysyms {
                if let Some(key) = key_to_evdev_key.get(keysym) {
                    current_hotkey.keysyms.push(*key);
                } else {
                    return Err(Error::InvalidConfig(ParseError::UnknownSymbol(line_number + 1)));
                }
            }
            if let Some(next_line) = actual_lines.get(i + 1) {
                if next_line.0 == "command" {
                    current_hotkey.command.push_str(&next_line.2.clone());
                }
            }
            hotkeys.push(current_hotkey);
        }
        if line_type == "command" {
            continue;
        }
    }
    Ok(hotkeys)
}

#[test]

fn test_parse_config() {
    let contents = "
# This is a comment

a + b+c
  firefox

d
  brave

e \
+f\
+g

  chrome
  #comment
    command to be ignored

h
    #no command
i
    ";
    let hotkeys = parse_contents(contents.to_string()).unwrap();
    assert_eq!(hotkeys.len(), 5);
    assert_eq!(hotkeys[0].keysyms, vec![evdev::Key::KEY_A, evdev::Key::KEY_B, evdev::Key::KEY_C]);
    assert_eq!(hotkeys[0].command, "firefox".to_string());
    assert_eq!(hotkeys[1].keysyms, vec![evdev::Key::KEY_D]);
    assert_eq!(hotkeys[1].command, "brave".to_string());
    assert_eq!(hotkeys[2].keysyms, vec![evdev::Key::KEY_E, evdev::Key::KEY_F, evdev::Key::KEY_G]);
    assert_eq!(hotkeys[2].command, "chrome".to_string());
    assert_eq!(hotkeys[3].keysyms, vec![evdev::Key::KEY_H]);
    assert_eq!(hotkeys[3].command, "".to_string());
    assert_eq!(hotkeys[4].keysyms, vec![evdev::Key::KEY_I]);
    assert_eq!(hotkeys[4].command, "".to_string());
}

#[test]

fn test_invalid_key() {
    let contents = "
invalid + a
    firefox
    ";
    let hotkeys = parse_contents(contents.to_string());
    assert!(hotkeys.is_err());
}
