use std::path::Path;

use serde_json::{json, Value};

use http_body_util::BodyExt;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use cot::http::method::Method;
use cot::form::{
    Form,
    FormResult
};
use cot::request::{Request, RequestExt};
use cot::request::extractors::RequestDb;
use cot::response::{Response, ResponseExt};
use cot::json::Json;
use cot::html::Html;
use cot::Body;



use askama::Template;



#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTemplate {
    youtube_url: String,
    errors: Vec<String>,
    success: String
}

pub async fn test_view(mut request: Request)->Response{
    if request.method() == Method::POST{
        let form_result = crate::forms::MusicUploadForm::from_request(&mut request).await.unwrap();
        match form_result{
            FormResult::Ok(form) => {
                let template = UploadTemplate{
                    youtube_url:form.youtube_url,
                    errors: vec![],
                    success: "".to_string()
                };
                Response::new(
                    Body::fixed(template.render().unwrap())
                ) 
            }
            _ =>{
                let template = UploadTemplate{
                    youtube_url:"".to_string(),
                    errors: vec![],
                    success: "".to_string()
                };
                Response::new(
                    Body::fixed(template.render().unwrap())
                )               
            }
        }
    }
    else{
        let template = UploadTemplate{
            youtube_url:"".to_string(),
            errors: vec![],
            success: "".to_string()
        };
        Response::new(
            Body::fixed(template.render().unwrap())
        )
    }
}