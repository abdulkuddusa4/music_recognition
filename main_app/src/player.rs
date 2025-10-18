use rodio::{OutputStream, Sink, Source};
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;
use std::path::Path;

pub fn play_audio(samples: Vec<f32>, sample_rate:u32) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    let source = AudioSource {
        samples,
        sample_rate,
        channels: 1,
        current_frame: 0,
    };
    
    sink.append(source);
    sink.sleep_until_end();
    
    Ok(())
}

struct AudioSource {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    current_frame: usize,
}

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame < self.samples.len() {
            let sample = self.samples[self.current_frame];
            self.current_frame += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / self.sample_rate as f32,
        ))
    }
}

fn convert_to_mono(audio_buf: &AudioBufferRef, num_channels: usize, output: &mut Vec<f32>) {
    // ... (use the convert_to_mono function from before)
}