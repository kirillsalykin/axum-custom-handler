use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
// use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    pub code: Cow<'static, str>,
    // pub message: Option<Cow<'static, str>>,
    pub params: HashMap<Cow<'static, str>, Value>,
}

impl ErrorEntry {
    pub fn new(code: &'static str) -> Self {
        Self {
            code: Cow::from(code),
            // message: None,
            params: HashMap::new(),
        }
    }
}

// impl std::error::Error for Error {
//     fn description(&self) -> &str {
//         &self.code
//     }
//     fn cause(&self) -> Option<&dyn std::error::Error> {
//         None
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Error {
    Struct(HashMap<Cow<'static, str>, Error>),
    List(BTreeMap<usize, Box<Error>>),
    Entry(ErrorEntry),
}

impl Error {
    pub fn entry(code: &'static str) -> Self {
        Error::Entry(ErrorEntry::new(code))
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorMap(pub HashMap<Cow<'static, str>, Error>);
