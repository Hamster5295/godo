use std::cmp::Ordering;

pub struct Version {
    tag: String,
    mono: bool,
}

pub fn new(tag: String, mono: bool) -> Version {
    Version { tag, mono }
}

pub fn parse(name: String) -> Option<Version> {
    let results: Vec<&str> = name.split(&[' ', '_']).collect();
    if results.len() < 2 {
        None
    } else {
        Some(Version {
            tag: results[1].to_string(),
            mono: results.len() >= 3,
        })
    }
}

pub fn compare(tag1: String, tag2: String) -> Ordering {
    let mut v1: Vec<&str> = tag1.split('.').collect();
    while v1.len() < 3 {
        v1.push("0");
    }
    let mut v2: Vec<&str> = tag2.split('.').collect();
    while v2.len() < 3 {
        v2.push("0");
    }

    let mut cmp = v1[0].cmp(v2[0]);
    if cmp != Ordering::Equal {
        return cmp;
    }
    cmp = v1[1].cmp(v2[1]);
    if cmp != Ordering::Equal {
        return cmp;
    }
    cmp = v1[2].cmp(v2[2]);
    if cmp != Ordering::Equal {
        return cmp;
    }
    Ordering::Equal
}

pub fn compare_opt(ver1: &Option<Version>, ver2: &Option<Version>) -> Ordering {
    if let Some(v1) = ver1 {
        if let Some(v2) = ver2 {
            compare(v1.tag(), v2.tag())
        } else {
            Ordering::Greater
        }
    } else {
        if let Some(_v2) = ver2 {
            Ordering::Less
        }else{
            Ordering::Equal
        }
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
