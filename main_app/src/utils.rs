#![allow(
    unused_imports,
    dead_code,
    unused_variables,
)]
use std::sync::Arc;
use std::any::type_name;

use cot::db::Database;


pub fn create_and_store_finger_print(
    db: &Arc<Database>,
    file_path: String
){

}
fn print_type_of<T>(obj: &T){
    println!("{}", type_name::<T>());
}

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;
use std::path::Path;

pub fn fetch_audio_data<P: AsRef<Path>>(path: P) -> Result<(Vec<f32>, u32), Error> {
    // Open the media source
    let file = File::open(path.as_ref())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a probe hint using the file extension
    let mut hint = Hint::new();
    if let Some(extension) = path.as_ref().extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    // Probe the media source
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;

    let mut format = probed.format;

    // Find the default audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(Error::Unsupported("No supported audio tracks found"))?;

    let track_id = track.id;
    let num_channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);
    let sample_rate = track.codec_params.sample_rate.ok_or(Error::Unsupported("Sample rate not found"))?;

    // Create a decoder for the track
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    // Decode all packets and collect samples
    let mut samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(err),
        };

        // Skip packets that don't belong to our track
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet
        let decoded = decoder.decode(&packet)?;

        // Convert to f32 and mix to mono if needed
        convert_to_mono(&decoded, num_channels, &mut samples);
    }

    Ok((samples, sample_rate))
}

fn convert_to_mono(audio_buf: &AudioBufferRef, num_channels: usize, output: &mut Vec<f32>) {
    match audio_buf {
        AudioBufferRef::F32(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                // Mono audio - just copy
                output.extend_from_slice(planes.planes()[0]);
            } else {
                // Multi-channel - average all channels
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += channel[i];
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::F64(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| s as f32));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += channel[i];
                    }
                    output.push((sum / num_channels as f64) as f32);
                }
            }
        }
        AudioBufferRef::S32(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| s as f32 / i32::MAX as f32));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += channel[i] as f32 / i32::MAX as f32;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::S24(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();
            const S24_MAX: f32 = 8388607.0; // 2^23 - 1

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| s.into_i32() as f32 / S24_MAX));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += channel[i].into_i32() as f32 / S24_MAX;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::S16(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| s as f32 / i16::MAX as f32));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += channel[i] as f32 / i16::MAX as f32;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::S8(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| s as f32 / i8::MAX as f32));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += channel[i] as f32 / i8::MAX as f32;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::U32(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| (s as f32 / u32::MAX as f32) * 2.0 - 1.0));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += (channel[i] as f32 / u32::MAX as f32) * 2.0 - 1.0;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::U24(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();
            const U24_MAX: f32 = 16777215.0; // 2^24 - 1

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| (s.into_u32() as f32 / U24_MAX) * 2.0 - 1.0));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += (channel[i].into_u32() as f32 / U24_MAX) * 2.0 - 1.0;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::U16(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += (channel[i] as f32 / u16::MAX as f32) * 2.0 - 1.0;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
        AudioBufferRef::U8(buf) => {
            let planes = buf.planes();
            let num_frames = buf.frames();

            if num_channels == 1 {
                output.extend(planes.planes()[0].iter().map(|&s| (s as f32 / u8::MAX as f32) * 2.0 - 1.0));
            } else {
                for i in 0..num_frames {
                    let mut sum = 0.0;
                    for channel in planes.planes() {
                        sum += (channel[i] as f32 / u8::MAX as f32) * 2.0 - 1.0;
                    }
                    output.push(sum / num_channels as f32);
                }
            }
        }
    }
}


