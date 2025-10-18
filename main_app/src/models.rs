#![allow(dead_code,unused_variables)]

use cot::db::{
    model,
    Auto
};
use serde::ser::{
    Serialize,
    Serializer,
    SerializeStruct
};


#[model]
pub struct Song{
    #[model(primary_key)]
    id: Auto<i64>,
    pub name: String,
    pub meta_info: String
}

impl Song{
    pub fn new(name:&str, meta_info:&str)->Song{
        Song{
            id: Auto::default(),
            name: name.to_string(),
            meta_info: meta_info.to_string()
        }
    }
}

impl Serialize for Song{
    fn serialize <S>(
        &self, serializer: S
    )->Result<S::Ok, S::Error>
    where
    S: Serializer
    {
        let mut s = serializer.serialize_struct("Post", 3)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("meta_info", &self.meta_info)?;
        s.end()
    }
}

#[model]
struct FingerPrint  {
    #[model(primary_key)]
    id: Auto<i64>,
    address: u32,
    AnchorTimeMs: u32,
    SongID:       i64
}

impl FingerPrint{
    pub fn new(
        address: u32,
        AnchorTimeMs: u32,
        SongID: i64
    )->FingerPrint{
        FingerPrint{
            id:Auto::default(),
            address,
            AnchorTimeMs,
            SongID
        }
    }
}