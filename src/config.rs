use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
// use std::str::pattern::Pattern;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    ConfigNotFound,
    Io(std::io::Error),
    InvalidConfig(ParseError),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // u32 is the line number where an error occured
    UnknownSymbol(PathBuf, u32),
    InvalidModifier(PathBuf, u32),
    InvalidKeysym(PathBuf, u32),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            Error::ConfigNotFound => "Config file not found.".fmt(f),

            Error::Io(io_err) => format!("I/O Error while parsing config file: {}", io_err).fmt(f),
            Error::InvalidConfig(parse_err) => match parse_err {
                ParseError::UnknownSymbol(path, line_nr) => format!(
                    "Error parsing config file {:?}. Unknown symbol at line {}.",
                    path, line_nr
                )
                .fmt(f),
                ParseError::InvalidKeysym(path, line_nr) => format!(
                    "Error parsing config file {:?}. Invalid keysym at line {}.",
                    path, line_nr
                )
                .fmt(f),
                ParseError::InvalidModifier(path, line_nr) => format!(
                    "Error parsing config file {:?}. Invalid modifier at line {}.",
                    path, line_nr
                )
                .fmt(f),
            },
        }
    }
}

pub const IMPORT_STATEMENT: &str = "include";

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub path: PathBuf,
    pub contents: String,
    pub imports: Vec<PathBuf>,
}

pub fn load_file_contents(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

impl Config {
    pub fn get_imports(contents: &str) -> Result<Vec<PathBuf>, Error> {
        let mut imports = Vec::new();
        for line in contents.lines() {
            if line.split(' ').next().unwrap() == IMPORT_STATEMENT {
                if let Some(import_path) = line.split(' ').nth(1) {
                    imports.push(Path::new(import_path).to_path_buf());
                }
            }
        }
        Ok(imports)
    }

    pub fn new(path: &Path) -> Result<Self, Error> {
        let contents = load_file_contents(path)?;
        let imports = Self::get_imports(&contents)?;
        Ok(Config { path: path.to_path_buf(), contents, imports })
    }

    pub fn load_to_configs(&self) -> Result<Vec<Self>, Error> {
        let mut configs = Vec::new();
        for import in &self.imports {
            configs.push(Self::new(import)?)
        }
        Ok(configs)
    }

    pub fn load_and_merge(mut configs: Vec<Self>) -> Result<Vec<Self>, Error> {
        let mut prev_count = 0;
        let mut current_count = configs.len();
        while prev_count != current_count {
            prev_count = configs.len();
            for config in configs.clone() {
                for import in Self::load_to_configs(&config)? {
                    if !configs.contains(&import) {
                        configs.push(import);
                    }
                }
            }
            current_count = configs.len();
        }
        Ok(configs)
    }
}

pub fn load(path: &Path) -> Result<Vec<Hotkey>, Error> {
    let mut hotkeys = Vec::new();
    let configs = vec![Config::new(path)?];
    for config in Config::load_and_merge(configs)? {
        for hotkey in parse_contents(path.to_path_buf(), config.contents)? {
            if !hotkeys.contains(&hotkey) {
                hotkeys.push(hotkey);
            }
        }
    }
    Ok(hotkeys)
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub keysym: evdev::Key,
    pub modifiers: Vec<Modifier>,
    pub send: bool,
    pub on_release: bool,
}

impl PartialEq for KeyBinding {
    fn eq(&self, other: &Self) -> bool {
        self.keysym == other.keysym
            && self.modifiers.iter().all(|modifier| other.modifiers.contains(modifier))
            && self.modifiers.len() == other.modifiers.len()
            && self.send == other.send
            && self.on_release == other.on_release
    }
}

pub trait Prefix {
    fn send(self) -> Self;
    fn on_release(self) -> Self;
}

pub trait Value {
    fn keysym(&self) -> evdev::Key;
    fn modifiers(&self) -> Vec<Modifier>;
    fn is_send(&self) -> bool;
    fn is_on_release(&self) -> bool;
}

impl KeyBinding {
    pub fn new(keysym: evdev::Key, modifiers: Vec<Modifier>) -> Self {
        KeyBinding { keysym, modifiers, send: false, on_release: false }
    }
    pub fn on_release(mut self) -> Self {
        self.on_release = true;
        self
    }
}

impl Prefix for KeyBinding {
    fn send(mut self) -> Self {
        self.send = true;
        self
    }
    fn on_release(mut self) -> Self {
        self.on_release = true;
        self
    }
}

impl Value for KeyBinding {
    fn keysym(&self) -> evdev::Key {
        self.keysym
    }
    fn modifiers(&self) -> Vec<Modifier> {
        self.clone().modifiers
    }
    fn is_send(&self) -> bool {
        self.send
    }
    fn is_on_release(&self) -> bool {
        self.on_release
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hotkey {
    pub keybinding: KeyBinding,
    pub command: String,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Modifier {
    Super,
    Alt,
    Control,
    Shift,
}

impl Hotkey {
    pub fn from_keybinding(keybinding: KeyBinding, command: String) -> Self {
        Hotkey { keybinding, command }
    }
    #[cfg(test)]
    pub fn new(keysym: evdev::Key, modifiers: Vec<Modifier>, command: String) -> Self {
        Hotkey { keybinding: KeyBinding::new(keysym, modifiers), command }
    }
}

impl Prefix for Hotkey {
    fn send(mut self) -> Self {
        self.keybinding.send = true;
        self
    }
    fn on_release(mut self) -> Self {
        self.keybinding.on_release = true;
        self
    }
}

impl Value for &Hotkey {
    fn keysym(&self) -> evdev::Key {
        self.keybinding.keysym
    }
    fn modifiers(&self) -> Vec<Modifier> {
        self.keybinding.clone().modifiers
    }
    fn is_send(&self) -> bool {
        self.keybinding.send
    }
    fn is_on_release(&self) -> bool {
        self.keybinding.on_release
    }
}

pub fn parse_contents(path: PathBuf, contents: String) -> Result<Vec<Hotkey>, Error> {
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
        ("escape", evdev::Key::KEY_ESC),
        ("backspace", evdev::Key::KEY_BACKSPACE),
        ("capslock", evdev::Key::KEY_CAPSLOCK),
        ("return", evdev::Key::KEY_ENTER),
        ("enter", evdev::Key::KEY_ENTER),
        ("tab", evdev::Key::KEY_TAB),
        ("space", evdev::Key::KEY_SPACE),
        ("plus", evdev::Key::KEY_KPPLUS),
        ("minus", evdev::Key::KEY_MINUS),
        ("-", evdev::Key::KEY_MINUS),
        ("equal", evdev::Key::KEY_EQUAL),
        ("=", evdev::Key::KEY_EQUAL),
        ("grave", evdev::Key::KEY_GRAVE),
        ("`", evdev::Key::KEY_GRAVE),
        ("print", evdev::Key::KEY_SYSRQ),
        ("volumeup", evdev::Key::KEY_VOLUMEUP),
        ("xf86audioraisevolume", evdev::Key::KEY_VOLUMEUP),
        ("volumedown", evdev::Key::KEY_VOLUMEDOWN),
        ("xf86audiolowervolume", evdev::Key::KEY_VOLUMEDOWN),
        ("mute", evdev::Key::KEY_MUTE),
        ("xf86audiomute", evdev::Key::KEY_MUTE),
        ("brightnessup", evdev::Key::KEY_BRIGHTNESSUP),
        ("xf86monbrightnessup", evdev::Key::KEY_BRIGHTNESSUP),
        ("brightnessdown", evdev::Key::KEY_BRIGHTNESSDOWN),
        ("xf86monbrightnessdown", evdev::Key::KEY_BRIGHTNESSDOWN),
        ("xf86audioplay", evdev::Key::KEY_PLAYPAUSE),
        ("xf86audioprev", evdev::Key::KEY_PREVIOUSSONG),
        ("xf86audionext", evdev::Key::KEY_NEXTSONG),
        ("xf86audiostop", evdev::Key::KEY_STOP),
        ("xf86audiomedia", evdev::Key::KEY_MEDIA),
        (",", evdev::Key::KEY_COMMA),
        ("comma", evdev::Key::KEY_COMMA),
        (".", evdev::Key::KEY_DOT),
        ("dot", evdev::Key::KEY_DOT),
        ("period", evdev::Key::KEY_DOT),
        ("/", evdev::Key::KEY_SLASH),
        ("question", evdev::Key::KEY_QUESTION),
        ("slash", evdev::Key::KEY_SLASH),
        ("backslash", evdev::Key::KEY_BACKSLASH),
        ("leftbrace", evdev::Key::KEY_LEFTBRACE),
        ("[", evdev::Key::KEY_LEFTBRACE),
        ("bracketleft", evdev::Key::KEY_LEFTBRACE),
        ("rightbrace", evdev::Key::KEY_RIGHTBRACE),
        ("]", evdev::Key::KEY_RIGHTBRACE),
        ("bracketright", evdev::Key::KEY_RIGHTBRACE),
        (";", evdev::Key::KEY_SEMICOLON),
        ("semicolon", evdev::Key::KEY_SEMICOLON),
        ("'", evdev::Key::KEY_APOSTROPHE),
        ("apostrophe", evdev::Key::KEY_APOSTROPHE),
        ("left", evdev::Key::KEY_LEFT),
        ("right", evdev::Key::KEY_RIGHT),
        ("up", evdev::Key::KEY_UP),
        ("down", evdev::Key::KEY_DOWN),
        ("pause", evdev::Key::KEY_PAUSE),
        ("home", evdev::Key::KEY_HOME),
        ("delete", evdev::Key::KEY_DELETE),
        ("insert", evdev::Key::KEY_INSERT),
        ("end", evdev::Key::KEY_END),
        ("pause", evdev::Key::KEY_PAUSE),
        ("prior", evdev::Key::KEY_PAGEDOWN),
        ("next", evdev::Key::KEY_PAGEUP),
        ("pagedown", evdev::Key::KEY_PAGEDOWN),
        ("pageup", evdev::Key::KEY_PAGEUP),
        ("f1", evdev::Key::KEY_F1),
        ("f2", evdev::Key::KEY_F2),
        ("f3", evdev::Key::KEY_F3),
        ("f4", evdev::Key::KEY_F4),
        ("f5", evdev::Key::KEY_F5),
        ("f6", evdev::Key::KEY_F6),
        ("f7", evdev::Key::KEY_F7),
        ("f8", evdev::Key::KEY_F8),
        ("f9", evdev::Key::KEY_F9),
        ("f10", evdev::Key::KEY_F10),
        ("f11", evdev::Key::KEY_F11),
        ("f12", evdev::Key::KEY_F12),
        ("f13", evdev::Key::KEY_F13),
        ("f14", evdev::Key::KEY_F14),
        ("f15", evdev::Key::KEY_F15),
        ("f16", evdev::Key::KEY_F16),
        ("f17", evdev::Key::KEY_F17),
        ("f18", evdev::Key::KEY_F18),
        ("f19", evdev::Key::KEY_F19),
        ("f20", evdev::Key::KEY_F20),
        ("f21", evdev::Key::KEY_F21),
        ("f22", evdev::Key::KEY_F22),
        ("f23", evdev::Key::KEY_F23),
        ("f24", evdev::Key::KEY_F24),
    ]);

    let mod_to_mod_enum: HashMap<&str, Modifier> = HashMap::from([
        ("ctrl", Modifier::Control),
        ("control", Modifier::Control),
        ("super", Modifier::Super),
        ("mod4", Modifier::Super),
        ("alt", Modifier::Alt),
        ("mod1", Modifier::Alt),
        ("shift", Modifier::Shift),
    ]);

    let lines: Vec<&str> = contents.split('\n').collect();

    // Go through each line, ignore comments and empty lines, mark lines starting with whitespace
    // as commands, and mark the other lines as keysyms. Mark means storing a line's type and the
    // line number in a vector.
    let mut lines_with_types: Vec<(&str, u32)> = Vec::new();
    for (line_number, line) in lines.iter().enumerate() {
        if line.trim().starts_with('#')
            || line.split(' ').next().unwrap() == IMPORT_STATEMENT
            || line.trim().is_empty()
        {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            lines_with_types.push(("command", line_number as u32));
        } else {
            lines_with_types.push(("keysym", line_number as u32));
        }
    }

    // Edge case: return a blank vector if no lines detected
    if lines_with_types.is_empty() {
        return Ok(vec![]);
    }

    let mut actual_lines: Vec<(&str, u32, String)> = Vec::new();

    if contents.contains('\\') {
        // Go through lines_with_types, and add the next line over and over until the current line no
        // longer ends with backslash. (Only if the lines have the same type)
        let mut current_line_type = lines_with_types[0].0;
        let mut current_line_number = lines_with_types[0].1;
        let mut current_line_string = String::new();
        let mut continue_backslash;

        for (line_type, line_number) in lines_with_types {
            if line_type != current_line_type {
                current_line_type = line_type;
                current_line_number = line_number;
                current_line_string = String::new();
            }

            let line_to_add = lines[line_number as usize].trim();
            continue_backslash = line_to_add.ends_with('\\');

            let line_to_add = line_to_add.strip_suffix('\\').unwrap_or(line_to_add);

            current_line_string.push_str(line_to_add);

            if !continue_backslash {
                actual_lines.push((current_line_type, current_line_number, current_line_string));
                current_line_type = line_type;
                current_line_number = line_number;
                current_line_string = String::new();
            }
        }
    } else {
        for (line_type, line_number) in lines_with_types {
            actual_lines.push((
                line_type,
                line_number,
                lines[line_number as usize].trim().to_string(),
            ));
        }
    }

    drop(lines);

    let mut hotkeys: Vec<Hotkey> = Vec::new();

    for (i, item) in actual_lines.iter().enumerate() {
        let line_type = item.0;
        let line_number = item.1;
        let line = &item.2;

        if line_type != "keysym" {
            continue;
        }

        let next_line = actual_lines.get(i + 1);
        if next_line.is_none() {
            break;
        }
        let next_line = next_line.unwrap();

        if next_line.0 != "command" {
            continue; // this should ignore keysyms that are not followed by a command
        }

        let extracted_keys = extract_curly_brace(line);
        let extracted_commands = extract_curly_brace(&next_line.2);

        'hotkey_parse: for (key, command) in extracted_keys.iter().zip(extracted_commands.iter()) {
            let keybinding = parse_keybind(
                path.clone(),
                key,
                line_number + 1,
                &key_to_evdev_key,
                &mod_to_mod_enum,
            )?;
            let hotkey = Hotkey::from_keybinding(keybinding, command.to_string());

            // Ignore duplicate hotkeys
            for i in hotkeys.iter() {
                if i.keybinding == hotkey.keybinding {
                    continue 'hotkey_parse;
                }
            }

            hotkeys.push(hotkey);
        }
    }

    Ok(hotkeys)
}

// We need to get the reference to key_to_evdev_key
// and mod_to_mod enum instead of recreating them
// after each function call because it's too expensive
fn parse_keybind(
    path: PathBuf,
    line: &str,
    line_nr: u32,
    key_to_evdev_key: &HashMap<&str, evdev::Key>,
    mod_to_mod_enum: &HashMap<&str, Modifier>,
) -> Result<KeyBinding, Error> {
    let line = line.split('#').next().unwrap();
    let tokens: Vec<String> =
        line.split('+').map(|s| s.trim().to_lowercase()).filter(|s| s != "_").collect();

    let mut tokens_new = Vec::new();
    for mut token in tokens {
        while token.trim().starts_with('_') {
            token = token.trim().strip_prefix('_').unwrap().to_string();
        }
        tokens_new.push(token.trim().to_string());
    }

    let last_token = tokens_new.last().unwrap().trim();

    // Check if last_token is prefixed with @ or ~ or even both.
    // If prefixed @, on_release = true; if prefixed ~, send = true
    let send = last_token.starts_with('~') || last_token.starts_with("@~");
    let on_release = last_token.starts_with('@') || last_token.starts_with("~@");

    // Delete the @ and ~ in the last token
    fn strip_at(token: &str) -> &str {
        if token.starts_with('@') {
            let token = token.strip_prefix('@').unwrap();
            strip_tilde(token)
        } else if token.starts_with('~') {
            strip_tilde(token)
        } else {
            token
        }
    }

    fn strip_tilde(token: &str) -> &str {
        if token.starts_with('~') {
            let token = token.strip_prefix('~').unwrap();
            strip_at(token)
        } else if token.starts_with('@') {
            strip_at(token)
        } else {
            token
        }
    }

    let last_token = strip_at(last_token);

    // Check if each token is valid
    for token in &tokens_new {
        let token = strip_at(token);
        if key_to_evdev_key.contains_key(token) {
            // Can't have a keysym that's like a modifier
            if token != last_token {
                return Err(Error::InvalidConfig(ParseError::InvalidModifier(path, line_nr)));
            }
        } else if mod_to_mod_enum.contains_key(token) {
            // Can't have a modifier that's like a keysym
            if token == last_token {
                return Err(Error::InvalidConfig(ParseError::InvalidKeysym(path, line_nr)));
            }
        } else {
            return Err(Error::InvalidConfig(ParseError::UnknownSymbol(path, line_nr)));
        }
    }

    // Translate keypress into evdev key
    let keysym = key_to_evdev_key.get(last_token).unwrap();

    let modifiers: Vec<Modifier> = tokens_new[0..(tokens_new.len() - 1)]
        .iter()
        .map(|token| *mod_to_mod_enum.get(token.as_str()).unwrap())
        .collect();

    let mut keybinding = KeyBinding::new(*keysym, modifiers);
    if send {
        keybinding = keybinding.send();
    }
    if on_release {
        keybinding = keybinding.on_release();
    }
    Ok(keybinding)
}

pub fn extract_curly_brace(line: &str) -> Vec<String> {
    if !line.contains('{') || !line.contains('}') || !line.is_ascii() {
        return vec![line.to_string()];
    }

    // go through each character in the line and mark the position of each { and }
    // if a { is not followed by a  }, return the line as is
    let mut brace_positions: Vec<usize> = Vec::new();
    let mut flag = false;
    for (i, c) in line.chars().enumerate() {
        if c == '{' {
            if flag {
                return vec![line.to_string()];
            }
            brace_positions.push(i);
            flag = true;
        } else if c == '}' {
            if !flag {
                return vec![line.to_string()];
            }
            brace_positions.push(i);
            flag = false;
        }
    }

    // now we have a list of positions of { and }
    // we should extract the items between each pair of braces and store them in a vector
    let mut items: Vec<String> = Vec::new();
    let mut remaining_line: Vec<String> = Vec::new();
    let mut start_index = 0;
    for i in brace_positions.chunks(2) {
        items.push(line[i[0] + 1..i[1]].to_string());
        remaining_line.push(line[start_index..i[0]].to_string());
        start_index = i[1] + 1;
    }

    // now we have a list of items between each pair of braces
    // we should extract the items between each comma and store them in a vector
    let mut tokens_vec: Vec<Vec<String>> = Vec::new();
    for item in items {
        // Edge case: escape periods
        // example:
        // ```
        // super + {\,, .}
        //    riverctl focus-output {previous, next}
        // ```
        let item = item.replace("\\,", "comma");

        let items: Vec<String> = item.split(',').map(|s| s.trim().to_string()).collect();
        tokens_vec.push(handle_ranges(items));
    }

    fn handle_ranges(items: Vec<String>) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        for item in items {
            if !item.contains('-') {
                output.push(item);
                continue;
            }
            let mut range = item.split('-').map(|s| s.trim());

            let begin_char: &str = if let Some(b) = range.next() {
                b
            } else {
                output.push(item);
                continue;
            };

            let end_char: &str = if let Some(e) = range.next() {
                e
            } else {
                output.push(item);
                continue;
            };

            // Do not accept range values that are longer than one char
            // Example invalid: {ef-p} {3-56}
            // Beginning of the range cannot be greater than end
            // Example invalid: {9-4} {3-2}
            if begin_char.len() != 1 || end_char.len() != 1 || begin_char > end_char {
                output.push(item);
                continue;
            }

            // In swhkd we will parse the full range using ASCII values.

            let begin_ascii_val = begin_char.parse::<char>().unwrap() as u8;
            let end_ascii_val = end_char.parse::<char>().unwrap() as u8;

            for ascii_number in begin_ascii_val..=end_ascii_val {
                output.push((ascii_number as char).to_string());
            }
        }
        output
    }

    // now write the tokens back to the line and output a vector
    let mut output: Vec<String> = Vec::new();
    // generate a cartesian product iterator for all the vectors in tokens_vec
    let cartesian_product_iter = tokens_vec.iter().multi_cartesian_product();
    for tokens in cartesian_product_iter.collect_vec() {
        let mut line_to_push = String::new();
        for i in 0..remaining_line.len() {
            line_to_push.push_str(&remaining_line[i]);
            line_to_push.push_str(tokens[i]);
        }
        if brace_positions[brace_positions.len() - 1] < line.len() - 1 {
            line_to_push.push_str(&line[brace_positions[brace_positions.len() - 1] + 1..]);
        }
        output.push(line_to_push);
    }
    output
}
