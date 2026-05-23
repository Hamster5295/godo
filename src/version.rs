use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreRelease {
    Stable,
    Alpha(u32),
    Beta(u32),
    Rc(u32),
    Dev(u32),
}

impl PreRelease {
    pub fn is_stable(&self) -> bool {
        matches!(self, PreRelease::Stable)
    }

    fn priority(&self) -> u32 {
        match self {
            PreRelease::Stable => 5,
            PreRelease::Rc(_) => 4,
            PreRelease::Beta(_) => 3,
            PreRelease::Alpha(_) => 2,
            PreRelease::Dev(_) => 1,
        }
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreRelease::Stable => write!(f, "stable"),
            PreRelease::Alpha(n) => write!(f, "alpha{n}"),
            PreRelease::Beta(n) => write!(f, "beta{n}"),
            PreRelease::Rc(n) => write!(f, "rc{n}"),
            PreRelease::Dev(n) => write!(f, "dev{n}"),
        }
    }
}

impl PartialOrd for PreRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority().cmp(&other.priority()) {
            Ordering::Equal => match (self, other) {
                (PreRelease::Alpha(a), PreRelease::Alpha(b))
                | (PreRelease::Beta(a), PreRelease::Beta(b))
                | (PreRelease::Rc(a), PreRelease::Rc(b))
                | (PreRelease::Dev(a), PreRelease::Dev(b)) => a.cmp(b),
                _ => Ordering::Equal,
            },
            ord => ord,
        }
    }
}

fn parse_pre(s: &str) -> Option<PreRelease> {
    match s {
        "stable" => Some(PreRelease::Stable),
        _ if s.starts_with("alpha") => s[5..].parse().ok().map(PreRelease::Alpha),
        _ if s.starts_with("beta") => s[4..].parse().ok().map(PreRelease::Beta),
        _ if s.starts_with("rc") => s[2..].parse().ok().map(PreRelease::Rc),
        _ if s.starts_with("dev") => s[3..].parse().ok().map(PreRelease::Dev),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GodotVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: PreRelease,
    pub mono: bool,
}

impl GodotVersion {
    pub fn is_stable(&self) -> bool {
        self.pre.is_stable()
    }

    pub fn folder_name(&self) -> String {
        let base = format!("{}.{}.{}-{}", self.major, self.minor, self.patch, self.pre);
        if self.mono {
            format!("{base}-mono")
        } else {
            base
        }
    }

    pub fn version_key(&self) -> String {
        format!("{}.{}.{}-{}", self.major, self.minor, self.patch, self.pre)
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        let tag = tag.strip_prefix('v').unwrap_or(tag);
        let (version_part, pre_part) = if let Some(idx) = tag.find('-') {
            (&tag[..idx], &tag[idx + 1..])
        } else {
            (tag, "stable")
        };

        let pre = parse_pre(pre_part)?;
        let parts: Vec<&str> = version_part.split('.').collect();
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1)?.parse().ok()?;
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(GodotVersion {
            major,
            minor,
            patch,
            pre,
            mono: false,
        })
    }

    pub fn from_folder(name: &str) -> Option<Self> {
        let mono = name.ends_with("-mono");
        let name = if mono { &name[..name.len() - 5] } else { name };

        let (version_part, pre_part) = if let Some(idx) = name.find('-') {
            (&name[..idx], &name[idx + 1..])
        } else {
            return None;
        };

        let pre = parse_pre(pre_part)?;
        let parts: Vec<&str> = version_part.split('.').collect();
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1)?.parse().ok()?;
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(GodotVersion {
            major,
            minor,
            patch,
            pre,
            mono,
        })
    }
}

impl fmt::Display for GodotVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}-{}",
            self.major, self.minor, self.patch, self.pre
        )?;
        if self.mono {
            write!(f, "-mono")?;
        }
        Ok(())
    }
}

impl PartialOrd for GodotVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GodotVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| self.pre.cmp(&other.pre))
    }
}

#[derive(Debug, Clone)]
pub struct VersionQuery {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub pre: Option<PreRelease>,
    pub mono: Option<bool>,
}

impl VersionQuery {
    pub fn from_input(input: &str) -> Option<Self> {
        let input = input.trim();
        let (version_part, pre_part) = if let Some(idx) = input.find('-') {
            (&input[..idx], Some(&input[idx + 1..]))
        } else {
            (input, None)
        };

        let pre = pre_part.and_then(parse_pre);
        let parts: Vec<&str> = version_part.split('.').collect();
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok());
        let patch = parts.get(2).and_then(|s| s.parse().ok());

        Some(VersionQuery {
            major,
            minor,
            patch,
            pre,
            mono: None,
        })
    }

    pub fn matches_loose(&self, version: &GodotVersion) -> bool {
        if self.major != version.major {
            return false;
        }
        if let Some(minor) = self.minor {
            if minor != version.minor {
                return false;
            }
        }
        if let Some(patch) = self.patch {
            if patch != version.patch {
                return false;
            }
        }
        if let Some(mono) = self.mono {
            if mono != version.mono {
                return false;
            }
        }
        true
    }
}
