use std::collections::HashMap;
use std::{
    fmt,
    path::{Path, PathBuf},
};
use sweet::token::KeyAttribute;
use sweet::{Definition, SwhkdParser};

#[derive(Debug)]
pub enum Error {
    ConfigNotFound,
    Io(std::io::Error),
    InvalidConfig(ParseError),
}

#[derive(Debug, PartialEq, Eq)]
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
        match self {
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

/// TODO: implement these in the parser crate
pub const MODE_ENTER_STATEMENT: &str = "@enter";
pub const MODE_ESCAPE_STATEMENT: &str = "@escape";

pub fn load(path: &Path) -> Result<Vec<Mode>, Error> {
    let config_self = sweet::SwhkdParser::from(sweet::ParserInput::Path(path)).unwrap();
    parse_contents(config_self)
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
    Altgr,
    Control,
    Shift,
    Any,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Mode {
    pub name: String,
    pub hotkeys: Vec<Hotkey>,
    pub unbinds: Vec<KeyBinding>,
    pub options: ModeOptions,
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            name: "normal".to_string(),
            hotkeys: vec![],
            unbinds: vec![],
            options: ModeOptions::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModeOptions {
    pub swallow: bool,
    pub oneoff: bool,
}

pub fn parse_contents(contents: SwhkdParser) -> Result<Vec<Mode>, Error> {
    let mut default_mode = Mode::default();

    for binding in &contents.bindings {
        default_mode.hotkeys.push(Hotkey {
            keybinding: sweet_def_to_kb(&binding.definition),
            command: binding.command.clone(),
        });
    }
    for unbind in contents.unbinds {
        default_mode.unbinds.push(sweet_def_to_kb(&unbind));
    }

    let mut modes = vec![default_mode];

    for sweet::Mode { name, oneoff, swallow, bindings, unbinds } in contents.modes {
        let mut pushmode =
            Mode { name, options: ModeOptions { swallow, oneoff }, ..Default::default() };
        for binding in bindings {
            let hotkey = Hotkey {
                keybinding: sweet_def_to_kb(&binding.definition),
                command: binding.command,
            };
            pushmode.hotkeys.retain(|h| h.keybinding != hotkey.keybinding);
            pushmode.hotkeys.push(hotkey);
        }
        for unbind in unbinds {
            pushmode.unbinds.push(sweet_def_to_kb(&unbind));
        }
        modes.push(pushmode);
    }
    Ok(modes)
}

/// This is a temporary function as glue code between
/// the SWEET parser and the structures here.
/// TODO: Bake this into the parser itself.
fn sweet_def_to_kb(def: &Definition) -> KeyBinding {
    // Don't forget to update valid key list on the man page if you do change this list.
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
        ("plus", evdev::Key::KEY_KPPLUS), // Shouldn't this be kpplus?
        ("kp0", evdev::Key::KEY_KP0),
        ("kp1", evdev::Key::KEY_KP1),
        ("kp2", evdev::Key::KEY_KP2),
        ("kp3", evdev::Key::KEY_KP3),
        ("kp4", evdev::Key::KEY_KP4),
        ("kp5", evdev::Key::KEY_KP5),
        ("kp6", evdev::Key::KEY_KP6),
        ("kp7", evdev::Key::KEY_KP7),
        ("kp8", evdev::Key::KEY_KP8),
        ("kp9", evdev::Key::KEY_KP9),
        ("kpasterisk", evdev::Key::KEY_KPASTERISK),
        ("kpcomma", evdev::Key::KEY_KPCOMMA),
        ("kpdot", evdev::Key::KEY_KPDOT),
        ("kpenter", evdev::Key::KEY_KPENTER),
        ("kpequal", evdev::Key::KEY_KPEQUAL),
        ("kpjpcomma", evdev::Key::KEY_KPJPCOMMA),
        ("kpleftparen", evdev::Key::KEY_KPLEFTPAREN),
        ("kpminus", evdev::Key::KEY_KPMINUS),
        ("kpplusminus", evdev::Key::KEY_KPPLUSMINUS),
        ("kprightparen", evdev::Key::KEY_KPRIGHTPAREN),
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
        ("xf86audiomedia", evdev::Key::KEY_MEDIA),
        ("xf86audiomicmute", evdev::Key::KEY_MICMUTE),
        ("micmute", evdev::Key::KEY_MICMUTE),
        ("xf86audionext", evdev::Key::KEY_NEXTSONG),
        ("xf86audioplay", evdev::Key::KEY_PLAYPAUSE),
        ("xf86audioprev", evdev::Key::KEY_PREVIOUSSONG),
        ("xf86audiostop", evdev::Key::KEY_STOP),
        ("xf86monbrightnessdown", evdev::Key::KEY_BRIGHTNESSDOWN),
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
        ("scroll_lock", evdev::Key::KEY_SCROLLLOCK),
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

    // Don't forget to update modifier list on the man page if you do change this list.
    let mod_to_mod_enum: HashMap<&str, Modifier> = HashMap::from([
        ("ctrl", Modifier::Control),
        ("control", Modifier::Control),
        ("super", Modifier::Super),
        ("mod4", Modifier::Super),
        ("alt", Modifier::Alt),
        ("mod1", Modifier::Alt),
        ("altgr", Modifier::Altgr),
        ("mod5", Modifier::Altgr),
        ("shift", Modifier::Shift),
        ("any", Modifier::Any),
    ]);

    let keysym = key_to_evdev_key.get(def.key.key.to_lowercase().as_str()).cloned().unwrap();
    let modifiers = def
        .modifiers
        .iter()
        .map(|sweet::token::Modifier(modifier)| {
            mod_to_mod_enum.get(modifier.to_lowercase().as_str()).cloned().unwrap()
        })
        .collect();
    let send = def.key.attribute == KeyAttribute::Send;
    let on_release = def.key.attribute == KeyAttribute::OnRelease;
    KeyBinding { keysym, modifiers, send, on_release }
}
