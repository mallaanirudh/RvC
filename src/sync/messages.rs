use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    GetRefs,
    GetObjects(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    Refs(HashMap<String, String>),
    Objects(Vec<(String, Vec<u8>)>),
}
