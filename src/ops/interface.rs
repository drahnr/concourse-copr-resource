use std::vec::*;
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use serde::ser::{Serialize,Serializer,SerializeSeq};
use std::fmt;


extern crate serde;
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct ResourceSource {
    pub login: String,
    pub username: String,
    pub token: String,
    pub url: String,
    pub srpm_path : String,
}


#[derive(Serialize, Deserialize)]
pub struct ResourceParams {
    pub project_id : u32,
    pub chroots: Vec<String>,
    pub enable_net: bool,
    pub max_n_bytes: u64,
}


#[derive(Serialize, Deserialize)]
pub struct ResourceVersion {
	pub digest : [u8; 32],
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



#[cfg(test)]
mod tests {
    use ::ops::interface::*;
    #[test]
    fn ser() {
        let s = r#"{"digest":[171,205,239,1,35,69,103,137]}"#;
        let o = ResourceVersion { digest : b"\xab\xcd\xef\x01\x23\x45\x67\x89".clone(), };
        let g = serde_json::to_string(&o).unwrap();
        assert!(g.as_str() == s);
    }
    #[test]
    fn de() {
        let s = r#"{"digest":[171,205,239,1,35,69,103,137]}"#;
        let o = ResourceVersion { digest : b"\xab\xcd\xef\x01\x23\x45\x67\x89".clone(), };
        let g : ResourceVersion = serde_json::from_str(s).unwrap();
        println!("parsed obj: <{}>", g);
        println!("expected obj: <{}>", o);
        assert!(g == o);
    }
    #[test]
    fn serde() {
        let s = r#"{"digest":[171,205,239,1,35,69,103,137]}"#;
        let g : ResourceVersion = serde_json::from_str(s).unwrap();
        assert!(serde_json::to_string(&g).unwrap() == s);
    }
    #[test]
    fn deser() {
        let o = ResourceVersion { digest : b"\xab\xcd\xef\x01\x23\x45\x67\x89".clone(), };
        let g = serde_json::to_string(&o).unwrap();
        println!("{}", g);
        let g2 = serde_json::from_str::<ResourceVersion>(g.as_str()).unwrap();
        assert!(g2 == o);
    }
}
