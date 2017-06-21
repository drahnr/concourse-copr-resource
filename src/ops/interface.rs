use std::vec::*;
use std::fmt;

extern crate serde;

#[derive(Serialize, Deserialize)]
pub struct ResourceSource {
    pub login: String,
    pub token: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceParams {
    pub rpmbuild_dir: String,
    pub project_id: Option<u32>,
    pub regex: Option<String>,
    pub chroots: Option<Vec<String>>,
    pub enable_net: Option<bool>,
    pub max_n_bytes: Option<u64>,
}

impl Default for ResourceParams {
    fn default() -> Self {
        let mut v = Vec::new();
        v.push(String::from("fedora-25-x86_64"));
        ResourceParams {
            rpmbuild_dir: ".".to_string(),
            project_id: None,
            regex: Some(r".*\.src\.rpm".to_string()),
            chroots: Some(v),
            enable_net: Some(false),
            max_n_bytes: Some(1_000_000_000),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResourceVersion {
    pub digest: [u8; 32],
}

impl PartialEq for ResourceVersion {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl fmt::Display for ResourceVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &byte in self.digest.iter() {
            write!(f, "{:X}", byte)?;
        }
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use ops::interface::*;
//     #[test]
//     fn ser() {
//         let s = r#"{"digest":[171,205,239,1,35,69,103,137]}"#;
//         let o = ResourceVersion { digest: b"\xab\xcd\xef\x01\x23\x45\x67\x89".clone() };
//         let g = serde_json::to_string(&o).unwrap();
//         assert!(g.as_str() == s);
//     }
//     #[test]
//     fn de() {
//         let s = r#"{"digest":[171,205,239,1,35,69,103,137]}"#;
//         let o = ResourceVersion { digest: b"\xab\xcd\xef\x01\x23\x45\x67\x89".clone() };
//         let g: ResourceVersion = serde_json::from_str(s).unwrap();
//         assert!(g == o);
//     }
//     #[test]
//     fn serde() {
//         let s = r#"{"digest":[171,205,239,1,35,69,103,137]}"#;
//         let g: ResourceVersion = serde_json::from_str(s).unwrap();
//         assert!(serde_json::to_string(&g).unwrap() == s);
//     }
//     #[test]
//     fn deser() {
//         let o = ResourceVersion { digest: b"\xab\xcd\xef\x01\x23\x45\x67\x89".clone() };
//         let g = serde_json::to_string(&o).unwrap();
//         let g2 = serde_json::from_str::<ResourceVersion>(g.as_str()).unwrap();
//         assert!(g2 == o);
//     }
// }
