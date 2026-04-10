use crate::domain::hash::Hash;

pub struct Commit {
    pub tree_hash: Hash,
    pub parent_hash: Option<Hash>,
    pub author: String,
    pub message: String,
}

impl Commit {
    pub fn prepare(&self) -> Vec<u8> {
        let mut body = String::new();

        body.push_str(&format!("tree {}\n", self.tree_hash.as_str()));

        if let Some(parent) = &self.parent_hash {
            body.push_str(&format!("parent {}\n", parent.as_str()));
        }

        body.push_str(&format!("author {}\n", self.author));
        body.push_str(&format!("\n{}\n", self.message));

        let header = format!("commit {}\0", body.len());
        let mut full_data = Vec::with_capacity(header.len() + body.len());
        full_data.extend_from_slice(header.as_bytes());
        full_data.extend_from_slice(body.as_bytes());
        full_data
    }

    pub fn parse(data: &[u8]) -> Commit {
        let s = String::from_utf8_lossy(data);
        let mut tree_hash = Hash::new(String::new());
        let mut parent_hash = None;
        let mut author = String::new();
        let mut message = String::new();

        let mut lines = s.lines();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                message = lines.collect::<Vec<_>>().join("\n");
                break;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            match parts[0] {
                "tree" => tree_hash = Hash::new(parts[1].to_string()),
                "parent" => parent_hash = Some(Hash::new(parts[1].to_string())),
                "author" => author = parts[1].to_string(),
                _ => {}
            }
        }

        Commit { tree_hash, parent_hash, author, message }
    }
}
