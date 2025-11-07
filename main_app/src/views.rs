use std::any::type_name;

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
use cot::db::{
    query,
    Model
};
use cot::response::{Response, ResponseExt};
use cot::json::Json;
use cot::html::Html;
use cot::Body;
use cot::bytes::Buf;

use num_complex::Complex;

use askama::Template;
use symphonia::core::errors::Error;

use main_app::player::play_audio;
use crate::my_random::random_string;
use crate::download_helpers::download_youtube_audio;
use crate::models::{
    Song,
    FingerPrint
};
use crate::shazam::MatchError;
use crate::shazam::Match;


fn print_type_of<T>(value: &T){
    println!("{}", type_name::<T>());
}
#[derive(Template)]
#[template(path = "upload.html")]
struct UploadTemplate {
    youtube_url: String,
    errors: Vec<String>,
    success: String
}

union number_32{
    as_u32: u32,
    as_f32: f32,
    as_arr: [u8;4]
}

pub async fn upload_view(mut request: Request, RequestDb(mut db): RequestDb)->Response{
    if request.method() == Method::POST{
        let form_result = crate::forms::MusicUploadForm::from_request(&mut request).await.unwrap();
        println!("DATABASE TYPE");
        print_type_of(&db);
        match form_result{
            FormResult::Ok(form) => {

                println!("youtube url: {}", form.youtube_url);
                let file_path = format!("output/{}.mp3", random_string(5 as usize));

                if query!(Song, $youtube_url==form.youtube_url.clone()).all(&db).await.unwrap().len()>0{
                    let template = UploadTemplate{
                        youtube_url:form.youtube_url,
                        errors: vec!["The video is already uploaded.".to_string()],
                        success: "".to_string()
                    };
                    return Response::new(
                        Body::fixed(template.render().unwrap())
                    );
                }

                let res = download_youtube_audio(&form.youtube_url[..], &file_path[..]).await;
                if res != Ok(()){
                    let template = UploadTemplate{
                        youtube_url:form.youtube_url,
                        errors: vec!["failed to download the video. try again later".to_string()],
                        success: "".to_string()
                    };
                    return Response::new(
                        Body::fixed(template.render().unwrap())
                    );               
                }

                let audio_data:Result<(Vec<f32>, u32), _> = main_app::utils::fetch_audio_data(&file_path);

                let mut spectrogram_option:Option<(Vec<Vec<Complex<f64>>>, f64)> = match audio_data{
                    Ok((audio_samples, sample_rate)) => {
                        let audio_duration = (audio_samples.len() as f64) / sample_rate as f64;
                        let spectogram_result = crate::shazam::spectogram::spectrogram(
                            &audio_samples.into_iter().map(|value| value as f64).collect::<Vec<f64>>()[..],
                            sample_rate as usize
                        );
                        if let Ok(spectrogram) = spectogram_result{
                            Some((spectrogram, audio_duration))
                        }
                        else {
                            None
                        }
                    },
                    Err(_) => None
                };
                std::fs::remove_file(&file_path[..]);

                if  spectrogram_option == None{                    
                    let template = UploadTemplate{
                        youtube_url:form.youtube_url,
                        errors: vec![],
                        success: "failed to save audio".to_string()
                    };
                    return Response::new(
                        Body::fixed(template.render().unwrap())
                    );
                }


                let (spectrogram, audio_duration) = spectrogram_option.unwrap();
                let peaks = crate::shazam::spectogram::extract_peaks(&spectrogram, audio_duration);

                let mut song = Song::new(&form.youtube_url);
                song.save(&db).await;
                let song_id :i64 = song.id.unwrap();
                let fingerprints = crate::shazam::fingerprint::fingerprint(peaks, song_id);

                println!("SAVING FINGERPRINTS...");
                for (address, couple) in fingerprints{
                    let mut fingerprint = FingerPrint::new(
                        address,
                        couple.anchor_time_ms,
                        couple.song_id
                    );
                    fingerprint.save(&db).await;
                }
                println!("DONE DB upDATE..");

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

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    error: String,
    success: String,
    results: Vec<String>
}


pub async fn search_view(
    mut request: Request,
    RequestDb(mut db): RequestDb
)->Response
{
    if let Some((audio_sample, sample_rate)) = get_request_audio_data(request).await{
        let duration: f64 = audio_sample.len() as f64 / sample_rate as f64;
        let spectogram = crate::shazam::spectogram::spectrogram(
            &audio_sample.iter().map(|x| *x as f64).collect::<Vec<f64>>()[..],
            sample_rate as usize
        );

        let matches = crate::shazam::find_matches(
            &db,
            &audio_sample.iter().map(|x| *x as f64).collect::<Vec<f64>>()[..],
            duration,
            sample_rate as usize
        ).await;

        let search_results = Vec::<String>::new();

        match matches{
            Ok((found_songs, duration)) => {
                print_type_of(&found_songs);
                // println!("TUMI HAKIS FASDF SAFS: {:?}", found_songs);
                let mut songs: Vec<String> = Vec::new();
                for mtch in found_songs{
                    // println!("match>>>>>");
                    let song_match: Match = mtch;
                    songs.push(song_match.youtube_url);
                }

                let template = SearchTemplate{
                    error: "".to_string(),
                    success: "".to_string(),
                    results: songs                };
                return Response::new(
                    Body::fixed(template.render().unwrap())
                );
            },
            _ => {
                let template = SearchTemplate{
                    error: "some error occured".to_string(),
                    success: "".to_string(),
                    results: vec!{"".to_string()}
                };
                return Response::new(
                    Body::fixed(template.render().unwrap())
                );
            }
        }
    }




    // if request.method() == Method::POST{
    //     let form_result = crate::forms::RecordedSampleForm::from_request(&mut request);
    // }
    let template = SearchTemplate{
        error: "".to_string(),
        success: "".to_string(),
        results: vec!{"https://www.youtube.com/watch?v=TH6OzKUB9Sg".to_string()}
    };
    Response::new(
        Body::fixed(template.render().unwrap())
    )
}


pub async fn get_request_audio_data(
    request: Request,
) -> Option<(Vec<f32>, u32)> {
    
    if request.method() == Method::POST {
        // Get the Content-Type header and clone the boundary
        let boundary = request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|content_type| {
                content_type
                    .split("boundary=")
                    .nth(1)
                    .map(|s| s.trim().to_string())  // Convert to owned String
            });
        
        let boundary = match boundary {
            Some(b) => b,
            None => {
                return None;
            }
        };
        
        
        // Now we can safely move request
        let (parts, body) = request.into_parts();
        let body_bytes = body.collect().await.unwrap().to_bytes();
        
        println!("Body size: {} bytes", body_bytes.len());
        
        // Create a stream for multer
        let stream = futures_util::stream::once(async move {
            Ok::<_, std::io::Error>(body_bytes)
        });
        
        // Parse with multer (boundary is now owned, so no borrow issue)
        let mut multipart = multer::Multipart::new(stream, boundary);
                
        // Process each field
        let mut sample_rate: Option<u32> = None;
        let mut audio_samples: Option<Vec<f32>> = None;

        while let Some(mut field) = multipart.next_field().await.unwrap() {
            let field_name = field.name().unwrap_or("unknown").to_string();
            let file_name = field.file_name().map(|s| s.to_string());

            if field_name == "sample_rate"{
                sample_rate = Some(field.text().await.unwrap().parse().unwrap());
            }
            else if let Some(original_filename) = file_name {                
                let data = field.bytes().await.unwrap();
                let mut audio_samples_local = Vec::<f32>::new();
                let mut shifter=0;
                let mut cur_val=0_u32;
                for v in &data{
                    // println!("d: {}", v);
                    cur_val=cur_val | ((*v as u32)<<shifter*8);
                    if shifter == 3{
                        let val=unsafe {
                            number_32{as_u32: cur_val}.as_f32
                        };
                        // println!("NUMBER: {:?}", unsafe {
                        //     number_32{as_u32: cur_val}.as_arr
                        // });
                        audio_samples_local.push(val);
                        shifter=0;
                        cur_val=0_u32;

                    }else{
                        shifter+=1;
                    }
                }
                audio_samples = Some(audio_samples_local);
            }
        }
        if sample_rate == None || audio_samples == None{
            return None;
        }
        // println!("SAMPLES>>: {:?}", audio_samples);
        // println!("samples: {}", audio_samples.clone().unwrap()[2000]);
        // println!("samples: {}", audio_samples.clone().unwrap()[2355]);
        // println!("samples: {}", audio_samples.clone().unwrap()[2388]);
        return Some((audio_samples.unwrap(), sample_rate.unwrap()));
    }
    return None;
}
