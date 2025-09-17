use std::collections::HashMap;

pub type Bytes = Vec<u8>;

pub trait Backend: Send + Sync {
    fn read(&self, path: &str) -> Option<Bytes>;
    fn write(&mut self, path: &str, data: &[u8]) -> bool { let _ = (path, data); false }
    fn exists(&self, path: &str) -> bool { self.read(path).is_some() }
    fn list(&self, path: &str) -> Vec<String> { let _ = path; Vec::new() }
    fn mkdir(&mut self, path: &str) -> bool { let _ = path; false }
}

// Simple in-memory backend with hierarchical directories.
#[derive(Default)]
pub struct MemBackend {
    files: HashMap<String, Bytes>,
    dirs: HashMap<String, ()>,
}

fn norm(path: &str) -> String {
    let mut out = String::from("/");
    for part in path.split('/') {
        if part.is_empty() || part == "." { continue; }
        if part == ".." { continue; }
        if !out.ends_with('/') { out.push('/'); }
        out.push_str(part);
    }
    if out.len() > 1 && out.ends_with('/') { out.pop(); }
    out
}

impl MemBackend {
    pub fn new() -> Self { let mut s = Self::default(); s.dirs.insert("/".into(), ()); s }
}

impl Backend for MemBackend {
    fn read(&self, path: &str) -> Option<Bytes> { self.files.get(&norm(path)).cloned() }
    fn write(&mut self, path: &str, data: &[u8]) -> bool {
        let p = norm(path);
        // ensure parent dir exists
        if let Some(idx) = p.rfind('/') {
            let parent = if idx == 0 { "/" } else { &p[..idx] };
            if !self.dirs.contains_key(parent) { return false; }
        }
        self.files.insert(p, data.to_vec()); true
    }
    fn exists(&self, path: &str) -> bool { self.files.contains_key(&norm(path)) || self.dirs.contains_key(&norm(path)) }
    fn list(&self, path: &str) -> Vec<String> {
        let base = norm(path);
        let mut out = Vec::new();
        let prefix = if base == "/" { "/".to_string() } else { format!("{}/", base) };
        for k in self.files.keys() {
            if k.starts_with(&prefix) {
                let rest = &k[prefix.len()..];
                if let Some(i) = rest.find('/') { out.push(rest[..i].to_string()); } else { out.push(rest.to_string()); }
            }
        }
        for k in self.dirs.keys() {
            if k.starts_with(&prefix) {
                let rest = &k[prefix.len()..];
                if rest.is_empty() { continue; }
                if let Some(i) = rest.find('/') { out.push(rest[..i].to_string()); } else { out.push(rest.to_string()); }
            }
        }
        out.sort(); out.dedup(); out
    }
    fn mkdir(&mut self, path: &str) -> bool { self.dirs.insert(norm(path), ()).is_none() }
}

// Mount table that dispatches to backends by longest-prefix path match
pub struct Vfs {
    mounts: Vec<(String, Box<dyn Backend>)>,
}

impl Vfs {
    pub fn new() -> Self { Self { mounts: Vec::new() } }
    pub fn mount(&mut self, at: &str, backend: Box<dyn Backend>) {
        let p = norm(at);
        self.mounts.push((p, backend));
        self.mounts.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    }
    fn route_mut(&mut self, path: &str) -> Option<(&str, &mut Box<dyn Backend>, String)> {
        let p = norm(path);
        for (mp, be) in self.mounts.iter_mut() {
            let is_match = if mp == "/" { true } else { p == *mp || p.starts_with(&(mp.clone() + "/")) };
            if is_match {
                let sub = if mp == "/" {
                    if p == "/" { "/".to_string() } else { format!("/{}", &p[1..]) }
                } else if p == *mp { "/".to_string() } else { format!("/{}", &p[mp.len()+1..]) };
                return Some((mp.as_str(), be, sub));
            }
        }
        None
    }
    fn route(&self, path: &str) -> Option<(&str, &Box<dyn Backend>, String)> {
        let p = norm(path);
        for (mp, be) in &self.mounts {
            let is_match = if mp == "/" { true } else { p == *mp || p.starts_with(&(mp.clone() + "/")) };
            if is_match {
                let sub = if mp == "/" {
                    if p == "/" { "/".to_string() } else { format!("/{}", &p[1..]) }
                } else if p == *mp { "/".to_string() } else { format!("/{}", &p[mp.len()+1..]) };
                return Some((mp.as_str(), be, sub));
            }
        }
        None
    }
    pub fn read(&self, path: &str) -> Option<Bytes> { self.route(path).and_then(|(_, b, sub)| b.read(&sub)) }
    pub fn write(&mut self, path: &str, data: &[u8]) -> bool { self.route_mut(path).map(|(_, b, sub)| b.write(&sub, data)).unwrap_or(false) }
    pub fn exists(&self, path: &str) -> bool { self.route(path).map(|(_, b, sub)| b.exists(&sub)).unwrap_or(false) }
    pub fn list(&self, path: &str) -> Vec<String> { self.route(path).map(|(_, b, sub)| b.list(&sub)).unwrap_or_default() }
    pub fn mkdir(&mut self, path: &str) -> bool { self.route_mut(path).map(|(_, b, sub)| b.mkdir(&sub)).unwrap_or(false) }
}
