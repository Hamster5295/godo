use std::io;

use io::Error;

pub struct Version {
    tag: String,
    mono: bool,
}

pub fn new(tag: String, mono: bool) -> Version {
    Version { tag, mono }
}

pub fn parse(name: String) -> Result<Version, Error> {
    let results: Vec<&str> = name.split(&[' ', '_']).collect();
    if results.len() < 2 {
        Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Wrong format! Require Godot_[version]_[mono] or Godot [version] [mono], found {}",
                name
            ),
        ))
    } else {
        Ok(Version {
            tag: results[1].to_string(),
            mono: results.len() >= 3,
        })
    }
}

impl Version {
    pub fn dir_name(&self) -> String {
        let mut result = format!("Godot_{}", self.tag);
        if self.mono {
            result += "_mono";
        }
        result
    }

    pub fn version_name(&self) -> String {
        format!("Godot {}", self.short_name())
    }

    pub fn short_name(&self) -> String {
        let mut result = format!("{}", self.tag);
        if self.mono {
            result += " mono";
        }
        result
    }

    pub fn tag(&self) -> String {
        self.tag.clone()
    }

    pub fn mono(&self) -> bool {
        self.mono
    }
}
