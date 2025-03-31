use std::collections::HashSet;
use std::path::Path;
use sweet::KeyAttribute;
use sweet::{Definition, SwhkdParser};
use sweet::{ModeInstruction, ParseError};

pub fn load(path: &Path) -> Result<Vec<Mode>, ParseError> {
    let config_self = sweet::SwhkdParser::from(sweet::ParserInput::Path(path))?;
    parse_contents(config_self)
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub keysym: evdev::Key,
    pub modifiers: HashSet<Modifier>,
    pub send: bool,
    pub on_release: bool,
}

impl PartialEq for KeyBinding {
    fn eq(&self, other: &Self) -> bool {
        self.keysym == other.keysym
            // Comparisons are order independent without manual iterations
            && self.modifiers == other.modifiers
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
    fn modifiers(&self) -> &HashSet<Modifier>;
    fn is_send(&self) -> bool;
    fn is_on_release(&self) -> bool;
}

impl KeyBinding {
    pub fn new(keysym: evdev::Key, modifiers: HashSet<Modifier>) -> Self {
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
    fn modifiers(&self) -> &HashSet<Modifier> {
        &self.modifiers
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
    pub mode_instructions: Vec<ModeInstruction>,
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
        Hotkey { keybinding, command, mode_instructions: vec![] }
    }

    /// Accepts both Vec<Modifier> and HashSet<Modifier> and stored as HashSet<Modifier>
    #[cfg(test)]
    pub fn new(
        keysym: evdev::Key,
        modifiers: impl IntoIterator<Item = Modifier>,
        command: String,
    ) -> Self {
        Hotkey {
            keybinding: KeyBinding::new(keysym, modifiers.into_iter().collect()),
            command,
            mode_instructions: vec![],
        }
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
    fn modifiers(&self) -> &HashSet<Modifier> {
        &self.keybinding.modifiers
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

pub fn parse_contents(contents: SwhkdParser) -> Result<Vec<Mode>, ParseError> {
    let mut default_mode = Mode::default();

    for binding in &contents.bindings {
        default_mode.hotkeys.push(Hotkey {
            keybinding: sweet_def_to_kb(&binding.definition),
            command: binding.command.clone(),
            mode_instructions: binding.mode_instructions.clone(),
        });
    }
    default_mode.unbinds.extend(contents.unbinds.iter().map(sweet_def_to_kb));

    let mut modes = vec![default_mode];

    for sweet::Mode { name, oneoff, swallow, bindings, unbinds } in contents.modes {
        let mut pushmode =
            Mode { name, options: ModeOptions { swallow, oneoff }, ..Default::default() };

        for binding in bindings {
            let hotkey = Hotkey {
                keybinding: sweet_def_to_kb(&binding.definition),
                command: binding.command,
                mode_instructions: binding.mode_instructions.clone(),
            };
            // Replace existing hotkeys with same keybinding
            pushmode.hotkeys.retain(|h| h.keybinding.keysym != hotkey.keybinding.keysym);
            pushmode.hotkeys.push(hotkey);
        }
        pushmode.unbinds.extend(unbinds.iter().map(sweet_def_to_kb));

        modes.push(pushmode);
    }
    Ok(modes)
}

/// Convert sweet::Definition to KeyBinding
fn sweet_def_to_kb(def: &Definition) -> KeyBinding {
    let modifiers: HashSet<Modifier> = def
        .modifiers
        .iter()
        .filter_map(|m| match m {
            sweet::Modifier::Super => Some(Modifier::Super),
            sweet::Modifier::Any => Some(Modifier::Any),
            sweet::Modifier::Control => Some(Modifier::Control),
            sweet::Modifier::Alt => Some(Modifier::Alt),
            sweet::Modifier::Altgr => Some(Modifier::Altgr),
            sweet::Modifier::Shift => Some(Modifier::Shift),
            sweet::Modifier::Omission => None,
        })
        .collect();

    KeyBinding {
        keysym: def.key.key,
        modifiers,
        send: def.key.attribute == KeyAttribute::Send,
        on_release: def.key.attribute == KeyAttribute::OnRelease,
    }
}
