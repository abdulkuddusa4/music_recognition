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
    pub id: Auto<i64>,
    pub youtube_url: String,
}

impl Song{
    pub fn new(youtube_url:&str)->Song{
        Song{
            id: Auto::default(),
            youtube_url: youtube_url.to_string(),
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
        s.serialize_field("youtube_url", &self.youtube_url)?;
        s.end()
    }
}

#[model]
pub struct FingerPrint  {
    #[model(primary_key)]
    id: Auto<i64>,
    address: u32,
    anchor_time_ms: u32,
    song_id:       i64
}

impl FingerPrint{
    pub fn new(
        address: u32,
        anchor_time_ms: u32,
        song_id: i64
    )->FingerPrint
    {
        FingerPrint{
            id:Auto::default(),
            address,
            anchor_time_ms,
            song_id
        }
    }
}