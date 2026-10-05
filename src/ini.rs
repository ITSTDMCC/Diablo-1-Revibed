//! The subset of SimpleIni (CSimpleIniA: case-insensitive, multi-key, no spaces around `=`)
//! that `options.cpp` uses for diablo.ini. Third-party library; reimplemented from its behaviour.

#[derive(Default, Clone, Debug)]
struct Section {
    name: String,
    /// (key, value) in load/insert order; a key may repeat (multi-key mode).
    entries: Vec<(String, String)>,
}

#[derive(Default, Clone, Debug)]
pub struct Ini {
    sections: Vec<Section>,
}

fn eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

impl Ini {
    /// `CSimpleIni::LoadData`: `[section]` lines, `key=value` lines, `;`/`#` comments.
    /// Keys and values are trimmed. A UTF-8 BOM is skipped.
    pub fn parse(text: &str) -> Ini {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut ini = Ini::default();
        let mut current: Option<usize> = None;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix('[') {
                let name = rest.split(']').next().unwrap_or("").trim().to_string();
                current = Some(ini.section_index_or_insert(&name));
                continue;
            }
            let (k, v) = match line.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => (line, ""),
            };
            let idx = match current {
                Some(i) => i,
                None => ini.section_index_or_insert(""),
            };
            ini.sections[idx].entries.push((k.to_string(), v.to_string()));
        }
        ini
    }

    fn section_index(&self, name: &str) -> Option<usize> {
        self.sections.iter().position(|s| eq(&s.name, name))
    }

    fn section_index_or_insert(&mut self, name: &str) -> usize {
        self.section_index(name).unwrap_or_else(|| {
            self.sections.push(Section { name: name.to_string(), entries: Vec::new() });
            self.sections.len() - 1
        })
    }

    /// `GetValue`: first value of the key.
    pub fn get_value(&self, section: &str, key: &str) -> Option<&str> {
        let s = &self.sections[self.section_index(section)?];
        s.entries.iter().find(|(k, _)| eq(k, key)).map(|(_, v)| v.as_str())
    }

    /// `GetAllValues`
    pub fn get_all_values(&self, section: &str, key: &str) -> Option<Vec<&str>> {
        let s = &self.sections[self.section_index(section)?];
        let v: Vec<&str> = s.entries.iter().filter(|(k, _)| eq(k, key)).map(|(_, v)| v.as_str()).collect();
        if v.is_empty() { None } else { Some(v) }
    }

    /// `GetLongValue`: decimal or `0x` hex (strtol semantics on the whole value); default when
    /// missing or not a number.
    pub fn get_long_value(&self, section: &str, key: &str, default: i64) -> i64 {
        let Some(v) = self.get_value(section, key) else { return default };
        let (neg, digits) = match v.strip_prefix('-') {
            Some(d) => (true, d),
            None => (false, v.strip_prefix('+').unwrap_or(v)),
        };
        let parsed = if let Some(hex) = digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
            i64::from_str_radix(hex, 16).ok()
        } else {
            digits.parse::<i64>().ok()
        };
        match parsed {
            Some(n) => if neg { -n } else { n },
            None => default,
        }
    }

    /// `GetBoolValue`: t/y/1 or "on" are true, f/n/0 or "off" false, anything else the default.
    pub fn get_bool_value(&self, section: &str, key: &str, default: bool) -> bool {
        let Some(v) = self.get_value(section, key) else { return default };
        let b = v.as_bytes();
        match b.first().map(|c| c.to_ascii_lowercase()) {
            Some(b't' | b'y' | b'1') => true,
            Some(b'f' | b'n' | b'0') => false,
            Some(b'o') => match b.get(1).map(|c| c.to_ascii_lowercase()) {
                Some(b'n') => true,
                Some(b'f') => false,
                _ => default,
            },
            _ => default,
        }
    }

    /// `GetDoubleValue`
    pub fn get_double_value(&self, section: &str, key: &str, default: f64) -> f64 {
        self.get_value(section, key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    /// `SetValue(section, key, value, comment, forceReplace)`. Without `force_replace` a
    /// multi-key ini appends another value for the key.
    pub fn set_value(&mut self, section: &str, key: &str, value: &str, force_replace: bool) {
        let idx = self.section_index_or_insert(section);
        let s = &mut self.sections[idx];
        if force_replace {
            if let Some(pos) = s.entries.iter().position(|(k, _)| eq(k, key)) {
                s.entries[pos].1 = value.to_string();
                // force replace removes the other values of a multi-key
                let mut seen = false;
                s.entries.retain(|(k, _)| {
                    if eq(k, key) {
                        let keep = !seen;
                        seen = true;
                        keep
                    } else {
                        true
                    }
                });
                return;
            }
        }
        s.entries.push((key.to_string(), value.to_string()));
    }

    /// `SetLongValue(section, key, value, comment, useHex=false, forceReplace)`
    pub fn set_long_value(&mut self, section: &str, key: &str, value: i64, force_replace: bool) {
        self.set_value(section, key, &value.to_string(), force_replace);
    }

    /// `SetDoubleValue`: SimpleIni formats with "%f".
    pub fn set_double_value(&mut self, section: &str, key: &str, value: f64, force_replace: bool) {
        self.set_value(section, key, &format!("{value:.6}"), force_replace);
    }

    /// `Save`: sections and keys in load order, `key=value` (no spaces), CRLF line endings,
    /// a blank line between sections.
    pub fn save(&self) -> String {
        let mut out = String::new();
        for (i, s) in self.sections.iter().enumerate() {
            if i > 0 {
                out.push_str("\r\n");
            }
            if !s.name.is_empty() {
                out.push('[');
                out.push_str(&s.name);
                out.push_str("]\r\n");
            }
            for (k, v) in &s.entries {
                out.push_str(k);
                out.push('=');
                out.push_str(v);
                out.push_str("\r\n");
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_saves() {
        let mut ini = Ini::parse("[Audio]\r\nSound Volume=0\r\n[NetMsg]\r\nF9=a\r\nF9=b\r\n[Graphics]\r\nFullscreen=0\r\n");
        assert_eq!(ini.get_long_value("audio", "sound volume", 5), 0);
        assert_eq!(ini.get_all_values("NetMsg", "F9").unwrap(), vec!["a", "b"]);
        assert!(!ini.get_bool_value("Graphics", "Fullscreen", true));
        assert!(ini.get_bool_value("Graphics", "Missing", true));
        ini.set_value("NetMsg", "F9", "c", true);
        assert_eq!(ini.get_all_values("NetMsg", "F9").unwrap(), vec!["c"]);
        ini.set_value("NetMsg", "F9", "d", false);
        assert_eq!(ini.get_all_values("NetMsg", "F9").unwrap(), vec!["c", "d"]);
        assert_eq!(ini.save(), "[Audio]\r\nSound Volume=0\r\n\r\n[NetMsg]\r\nF9=c\r\nF9=d\r\n\r\n[Graphics]\r\nFullscreen=0\r\n");
    }
}
