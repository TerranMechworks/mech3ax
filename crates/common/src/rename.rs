use std::collections::HashSet;

#[derive(Debug)]
#[repr(transparent)]
pub struct Rename {
    seen: HashSet<String>,
}

impl Rename {
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn insert(&mut self, name: &str) -> Option<String> {
        if self.seen.insert(name.to_string()) {
            return None;
        }

        let info = name.rsplit_once('.');
        for index in 1usize.. {
            let rename = match info {
                Some((stem, suffix)) => format!("{}-{}.{}", stem, index, suffix),
                None => format!("{}-{}", name, index),
            };
            if self.seen.insert(rename.clone()) {
                return Some(rename);
            }
        }
        panic!("ran out of renames");
    }
}
