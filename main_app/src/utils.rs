#![allow(
    unused_imports,
    dead_code,
    unused_variables,
)]
use std::sync::Arc;

use cot::db::Database;


pub fn create_and_store_finger_print(
    db: &Arc<Database>,
    file_path: String
){

}

use ffmpeg_next as ffmpeg;
use anyhow::Result;

pub fn load_audio_samples(path: &str) -> Result<Vec<f32>> {
    // Initialize FFmpeg
    ffmpeg::init()?;

    // Open input file
    let mut ictx = ffmpeg::format::input(&path)?;

    // Find the best audio stream
    let input_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
    let stream_index = input_stream.index();

    // Get codec context
    let codec_params = input_stream.parameters();
    let decoder = ffmpeg::codec::context::Context::from_parameters(codec_params)?;
    let mut decoder = decoder.decoder().audio()?;

    // Set up resampler to convert to mono f32
    let mut resampler = ffmpeg::software::resampling::Context::get(
        decoder.format(),
        decoder.channel_layout(),
        decoder.rate(),
        ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
        ffmpeg::channel_layout::MONO,
        decoder.rate(),
    )?;

    let mut output_samples: Vec<f32> = Vec::new();
    let mut decoded = ffmpeg::frame::Audio::empty();

    for (stream, packet) in ictx.packets() {
        if stream.index() == stream_index {
            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut resampled = ffmpeg::frame::Audio::empty();
                resampler.run(&decoded, &mut resampled)?;

                // Collect samples as f32
                let planes = resampled.data(0);
                let len = resampled.samples();
                let slice = unsafe {
                    std::slice::from_raw_parts(planes.as_ptr() as *const f32, len)
                };
                output_samples.extend_from_slice(slice);
            }
        }
    }

    // flush decoder
    decoder.send_eof()?;
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut resampled = ffmpeg::frame::Audio::empty();
        resampler.run(&decoded, &mut resampled)?;
        let planes = resampled.data(0);
        let len = resampled.samples();
        let slice = unsafe {
            std::slice::from_raw_parts(planes.as_ptr() as *const f32, len)
        };
        output_samples.extend_from_slice(slice);
    }

    Ok(output_samples)
}
