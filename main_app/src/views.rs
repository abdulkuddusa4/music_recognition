#![allow(dead_code, unused_variables, unused_imports)]


use std::any::type_name;
// use std::alloc::sync::Arc;
use std::sync::Arc;

use cot::db::{
    Model,
    query
};

use cot::request::extractors::{
    RequestDb
};

use cot::request::Request;
use cot::db::Database;

use cot::json::Json;
use serde_json::{
    json,
    Value
};

use crate::models::Song;
// use crate::utils::download_youtube_audio;

fn print_type_of<T>(obj: &T){
    println!("{}", type_name::<T>());
}


pub async fn test_view(
    request: Request,
    RequestDb(db): RequestDb
)->Json<Value>
{

    print_type_of(&db);
    return Json(json!({"name": "downloaded"}));
}