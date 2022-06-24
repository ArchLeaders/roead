use crate::aamp::{hash_name, CRC32};
use cached::proc_macro::cached;
use indexmap::IndexMap;
use once_cell::sync::Lazy;

const NAMES: &str = include_str!("../../include/oead/data/botw_hashed_names.txt");
const NUMBERED_NAMES: &str = include_str!("../../include/oead/data/botw_numbered_names.txt");

static NUMBERED_NAME_LIST: Lazy<Vec<&'static str>> = Lazy::new(|| NUMBERED_NAMES.split('\n').collect());

/// A table of names that is used to recover original names in binary parameter archives which store only name hashes.
#[derive(Clone, Default, Debug)]
pub struct NameTable {
    table: IndexMap<u32, &'static str>,
    own_table: IndexMap<u32, String>,
}

impl NameTable {
    /// Creates a new name table, optionally pre-filling it with BOTW strings
    pub fn new(use_botw_strings: bool) -> NameTable {
        let mut m: IndexMap<u32, &'static str> = IndexMap::default();
        if use_botw_strings {
            for name in NAMES.split('\n').map(|n| n.strip_suffix('\r').unwrap_or(n)) {
                let val = CRC32.checksum(name.as_bytes());
                m.insert(val, name);
            }
        }
        NameTable {
            table: m,
            own_table: IndexMap::new(),
        }
    }

    /// Add a known string to the name table.
    pub fn add_name<S: Into<String>>(&mut self, name: S) {
        let name = name.into();
        self.own_table.insert(hash_name(&name), name);
    }

    /// Add a known string reference to the name table. Must be static to avoid lifetime nonsense.
    pub fn add_name_ref(&mut self, name: &'static str) {
        self.table.insert(hash_name(name), name);
    }

    /// Gets the string associated with a specific hash, if present in the table
    pub fn get_name(&self, crc: u32) -> Option<&str> {
        self.table
            .get(&crc).copied()
            .or_else(|| self.own_table.get(&crc).map(|s| s.as_str()))
    }

    /// Tries to guess the name that is associated with the given hash and index (of the parameter / object / list in its parent).
    /// The table is automatically updated with any newly found names if an indice-based guess was necessary.
    pub fn guess_name(&mut self, crc: u32, parent_crc: u32, idx: usize) -> Option<String> {
        let result = self.guess_name_tmp(crc, parent_crc, idx);
        if let Some(s) = result.as_ref() {
            self.add_name(s);
        }
        result
    }

    /// Tries to guess the name that is associated with the given hash and index (of the parameter / object / list in its parent).
    /// The table is **not** automatically updated with any newly found names.
    pub fn guess_name_tmp(&self, crc: u32, parent_crc: u32, idx: usize) -> Option<String> {
        let parent = self.get_name(parent_crc);
        match parent {
            Some(parent_name) => {
                let mut matched = test_names(parent_name, idx, crc);
                if matched.is_none() {
                    if parent_name == "Children" {
                        matched = test_names("Child", idx, crc);
                    }
                    if matched.is_none() {
                        for suffix in &["s", "es", "List"] {
                            if parent_name.ends_with(suffix) {
                                matched = test_names(
                                    &parent_name[0..parent_name.len() - suffix.len()],
                                    idx,
                                    crc,
                                );
                                if matched.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
                match matched {
                    Some(s) => Some(s),
                    None => try_numbered_name(idx, crc),
                }
            }
            None => try_numbered_name(idx, crc),
        }
    }
}

fn test_names(parent: &str, idx: usize, crc: u32) -> Option<String> {
    for i in &[idx, idx + 1] {
        for name in &[
            [parent, i.to_string().as_str()].join(""),
            [parent, "_", i.to_string().as_str()].join(""),
            [parent, format!("{:02}", i).as_str()].join(""),
            [parent, "_", format!("{:02}", i).as_str()].join(""),
            [parent, format!("{:03}", i).as_str()].join(""),
            [parent, "_", format!("{:03}", i).as_str()].join(""),
        ] {
            if CRC32.checksum(name.as_bytes()) == crc {
                return Some(name.to_owned());
            }
        }
    }
    None
}

#[cached]
fn try_numbered_name(idx: usize, crc: u32) -> Option<String> {
    let mut opt = Option::None;
    for name in NUMBERED_NAME_LIST
        .iter()
        .map(|n| n.strip_suffix('\r').unwrap_or(n))
    {
        for i in 0..idx + 2 {
            let maybe: String = if name.contains('{') {
                rt_format(name, i)
            } else {
                name.to_string()
            };
            if CRC32.checksum(maybe.as_bytes()) == crc as u32 {
                opt = Some(maybe);
            }
        }
    }
    opt
}

#[inline]
fn rt_format(name: &str, i: usize) -> String {
    if name.contains("{}") {
        name.replace("{}", &format!("{}", i))
    } else if name.contains("{:02}") {
        name.replace("{:02}", &format!("{:02}", i))
    } else if name.contains("{:03}") {
        name.replace("{:03}", &format!("{:03}", i))
    } else if name.contains("{:04}") {
        name.replace("{:04}", &format!("{:04}", i))
    } else {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::NameTable;
    #[test]
    fn test_names() {
        let mut table = NameTable::new(true);
        table.add_name("JohnDavenant");
        assert_eq!(table.get_name(108636734).unwrap(), "JohnDavenant");
        assert_eq!(
            table.guess_name(663971094, 1393476043, 2).unwrap(),
            "Spine_02"
        );
        assert_eq!(table.get_name(3869088787).unwrap(), "Sword-Huge");
    }
}
