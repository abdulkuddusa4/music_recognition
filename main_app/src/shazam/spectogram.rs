#![allow(dead_code)]

use num_complex::Complex;
use realfft::RealFftPlanner;
use std::f64::consts::PI;

const DSP_RATIO: usize = 4;
const FREQ_BIN_SIZE: usize = 1024;
const MAX_FREQ: f64 = 5000.0; // 5kHz
const HOP_SIZE: usize = FREQ_BIN_SIZE / 32;

#[derive(Debug)]
pub enum ShazamError {
    DownsampleError(String),
    InvalidSampleRate(String),
    FftError(String),
}

impl std::fmt::Display for ShazamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShazamError::DownsampleError(msg) => write!(f, "Downsample error: {}", msg),
            ShazamError::InvalidSampleRate(msg) => write!(f, "Invalid sample rate: {}", msg),
            ShazamError::FftError(msg) => write!(f, "FFT error: {}", msg),
        }
    }
}

impl std::error::Error for ShazamError {}

pub fn spectrogram(
    sample: &[f64],
    sample_rate: usize,
) -> Result<Vec<Vec<Complex<f64>>>, ShazamError> {
    let filtered_sample = low_pass_filter(MAX_FREQ, sample_rate as f64, sample);

    let downsampled_sample =
        downsample(&filtered_sample, sample_rate, sample_rate / DSP_RATIO)?;

    let num_of_windows = downsampled_sample.len() / (FREQ_BIN_SIZE - HOP_SIZE);
    let mut spectrogram_result = Vec::with_capacity(num_of_windows);

    // Precompute Hamming window
    let window: Vec<f64> = (0..FREQ_BIN_SIZE)
        .map(|i| 0.54 - 0.46 * (2.0 * PI * i as f64 / (FREQ_BIN_SIZE as f64 - 1.0)).cos())
        .collect();

    // Create FFT planner
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(FREQ_BIN_SIZE);
    let mut scratch = fft.make_scratch_vec();

    // Perform STFT
    for i in 0..num_of_windows {
        let start = i * HOP_SIZE;
        let end = (start + FREQ_BIN_SIZE).min(downsampled_sample.len());

        let mut bin = vec![0.0; FREQ_BIN_SIZE];
        let copy_len = end - start;
        bin[..copy_len].copy_from_slice(&downsampled_sample[start..end]);

        // Apply Hamming window
        for j in 0..FREQ_BIN_SIZE {
            bin[j] *= window[j];
        }

        // Perform FFT using realfft
        let mut spectrum = fft.make_output_vec();
        fft.process_with_scratch(&mut bin, &mut spectrum, &mut scratch)
            .map_err(|e| ShazamError::FftError(format!("FFT processing failed: {:?}", e)))?;

        spectrogram_result.push(spectrum);
    }

    Ok(spectrogram_result)
}

/// Low-pass filter that attenuates high frequencies above the cutoff frequency.
/// Uses the transfer function H(s) = 1 / (1 + sRC), where RC is the time constant.
pub fn low_pass_filter(cutoff_frequency: f64, sample_rate: f64, input: &[f64]) -> Vec<f64> {
    let rc = 1.0 / (2.0 * PI * cutoff_frequency);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut filtered_signal = Vec::with_capacity(input.len());
    let mut prev_output = 0.0;

    for (i, &x) in input.iter().enumerate() {
        let output = if i == 0 {
            x * alpha
        } else {
            alpha * x + (1.0 - alpha) * prev_output
        };
        filtered_signal.push(output);
        prev_output = output;
    }

    filtered_signal
}

/// Downsamples the input audio from original_sample_rate to target_sample_rate
pub fn downsample(
    input: &[f64],
    original_sample_rate: usize,
    target_sample_rate: usize,
) -> Result<Vec<f64>, ShazamError> {
    if target_sample_rate == 0 || original_sample_rate == 0 {
        return Err(ShazamError::InvalidSampleRate(
            "Sample rates must be positive".to_string(),
        ));
    }
    if target_sample_rate > original_sample_rate {
        return Err(ShazamError::InvalidSampleRate(
            "Target sample rate must be less than or equal to original sample rate".to_string(),
        ));
    }

    let ratio = original_sample_rate / target_sample_rate;
    if ratio == 0 {
        return Err(ShazamError::InvalidSampleRate(
            "Invalid ratio calculated from sample rates".to_string(),
        ));
    }

    let mut resampled = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let end = (i + ratio).min(input.len());

        let sum: f64 = input[i..end].iter().sum();
        let avg = sum / (end - i) as f64;
        resampled.push(avg);

        i += ratio;
    }

    Ok(resampled)
}

#[derive(Debug, Clone)]
pub struct Peak {
    pub time: f64,
    pub freq: Complex<f64>,
}

/// Analyzes a spectrogram and extracts significant peaks in the frequency domain over time.
pub fn extract_peaks(spectrogram: &Vec<Vec<Complex<f64>>>, audio_duration: f64) -> Vec<Peak> {
    if spectrogram.is_empty() {
        return Vec::new();
    }

    #[derive(Clone)]
    struct Maxies {
        max_mag: f64,
        max_freq: Complex<f64>,
        freq_idx: usize,
    }

    let bands = [
        (0, 10),
        (10, 20),
        (20, 40),
        (40, 80),
        (80, 160),
        (160, 512),
    ];

    let mut peaks = Vec::new();
    let bin_duration = audio_duration / spectrogram.len() as f64;

    for (bin_idx, bin) in spectrogram.iter().enumerate() {
        let mut max_mags = Vec::new();
        let mut max_freqs = Vec::new();
        let mut freq_indices = Vec::new();

        // Find maximum in each frequency band
        for &(min, max) in &bands {
            let mut maxx = Maxies {
                max_mag: 0.0,
                max_freq: Complex::new(0.0, 0.0),
                freq_idx: min,
            };

            for (idx, &freq) in bin[min..max].iter().enumerate() {
                let magnitude = freq.norm();
                if magnitude > maxx.max_mag {
                    let freq_idx = min + idx;
                    maxx = Maxies {
                        max_mag: magnitude,
                        max_freq: freq,
                        freq_idx,
                    };
                }
            }

            max_mags.push(maxx.max_mag);
            max_freqs.push(maxx.max_freq);
            freq_indices.push(maxx.freq_idx as f64);
        }

        // Calculate the average magnitude
        let max_mags_sum: f64 = max_mags.iter().sum();
        let avg = max_mags_sum / max_freqs.len() as f64;

        // Add peaks that exceed the average magnitude
        for (i, &max_mag) in max_mags.iter().enumerate() {
            if max_mag > avg {
                let peak_time_in_bin = freq_indices[i] * bin_duration / bin.len() as f64;

                // Calculate the absolute time of the peak
                let peak_time = bin_idx as f64 * bin_duration + peak_time_in_bin;

                peaks.push(Peak {
                    time: peak_time,
                    freq: max_freqs[i],
                });
            }
        }
    }

    peaks
}
