//! Generate reference audio test files for audio quality testing.
//!
//! This example generates various test signals:
//! - Sine waves at different frequencies
//! - White noise
//! - Impulses
//! - Silence
//!
//! Run with: cargo run --example generate_test_audio

use std::f64::consts::PI;
use std::fs::create_dir_all;
use std::path::Path;

/// Generate a sine wave signal
fn generate_sine_wave(frequency: f64, duration: f64, sample_rate: u32, amplitude: f64) -> Vec<f64> {
    let num_samples = (sample_rate as f64 * duration) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f64 / sample_rate as f64;
            amplitude * (2.0 * PI * frequency * t).sin()
        })
        .collect()
}

/// Generate white noise signal
fn generate_white_noise(duration: f64, sample_rate: u32, amplitude: f64) -> Vec<f64> {
    use rand::Rng;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut rng = rand::thread_rng();
    (0..num_samples)
        .map(|_| amplitude * rng.gen_range(-1.0..1.0))
        .collect()
}

/// Generate an impulse signal
fn generate_impulse(duration: f64, sample_rate: u32, amplitude: f64) -> Vec<f64> {
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut signal = vec![0.0; num_samples];
    if !signal.is_empty() {
        signal[0] = amplitude;
    }
    signal
}

/// Generate silence
fn generate_silence(duration: f64, sample_rate: u32) -> Vec<f64> {
    let num_samples = (sample_rate as f64 * duration) as usize;
    vec![0.0; num_samples]
}

/// Save signal as WAV file
fn save_wav(
    filename: &str,
    signal: &[f64],
    sample_rate: u32,
    bit_depth: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(filename).parent() {
        create_dir_all(parent)?;
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: bit_depth,
        sample_format: if bit_depth == 32 {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    };

    let mut writer = hound::WavWriter::create(filename, spec)?;

    match bit_depth {
        16 => {
            for &sample in signal {
                let sample_i16 = (sample * i16::MAX as f64) as i16;
                writer.write_sample(sample_i16)?;
            }
        }
        24 => {
            for &sample in signal {
                let sample_i32 = (sample * 8388607.0) as i32;
                writer.write_sample(sample_i32)?;
            }
        }
        32 => {
            for &sample in signal {
                writer.write_sample(sample as f32)?;
            }
        }
        _ => return Err(format!("Unsupported bit depth: {}", bit_depth).into()),
    }

    writer.finalize()?;
    println!("Generated: {}", filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating reference audio test files...");
    println!("Output directory: test_data/reference");
    println!();

    let base_dir = "test_data/reference";
    let duration = 5.0; // seconds
    let sample_rates = [44100, 48000, 96000, 192000];
    let bit_depths = [16, 24];

    // Test frequencies
    let test_frequencies = [
        (100.0, "100Hz"),
        (440.0, "440Hz"), // A4
        (1000.0, "1kHz"), // Standard reference
        (5000.0, "5kHz"),
        (10000.0, "10kHz"),
    ];

    for sample_rate in sample_rates {
        for bit_depth in bit_depths {
            let sr_dir = format!("{}/{}Hz_{}bit", base_dir, sample_rate, bit_depth);

            // Generate sine waves
            for (freq, name) in test_frequencies {
                let signal = generate_sine_wave(freq, duration, sample_rate, 0.8);
                let filename = format!("{}/sine_{}.wav", sr_dir, name);
                save_wav(&filename, &signal, sample_rate, bit_depth)?;
            }

            // Generate white noise
            let signal = generate_white_noise(duration, sample_rate, 0.5);
            let filename = format!("{}/white_noise.wav", sr_dir);
            save_wav(&filename, &signal, sample_rate, bit_depth)?;

            // Generate impulse
            let signal = generate_impulse(duration, sample_rate, 1.0);
            let filename = format!("{}/impulse.wav", sr_dir);
            save_wav(&filename, &signal, sample_rate, bit_depth)?;

            // Generate silence
            let signal = generate_silence(duration, sample_rate);
            let filename = format!("{}/silence.wav", sr_dir);
            save_wav(&filename, &signal, sample_rate, bit_depth)?;

            println!();
        }
    }

    // Generate special test files
    println!("Generating special test files...");

    // Full scale sine wave (for clipping tests)
    let signal = generate_sine_wave(1000.0, 1.0, 44100, 1.0);
    save_wav(
        &format!("{}/special/full_scale_1kHz.wav", base_dir),
        &signal,
        44100,
        16,
    )?;

    // Very short impulse (for transient response)
    let signal = generate_impulse(0.1, 44100, 1.0);
    save_wav(
        &format!("{}/special/short_impulse.wav", base_dir),
        &signal,
        44100,
        16,
    )?;

    // Logarithmic frequency sweep (20Hz to 20kHz)
    let duration = 10.0;
    let sample_rate = 44100;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let f0 = 20.0;
    let f1 = 20000.0;
    let k = (f1 - f0) / duration;

    let signal: Vec<f64> = (0..num_samples)
        .map(|i| {
            let t = i as f64 / sample_rate as f64;
            0.8 * (2.0 * PI * (f0 * t + k * t * t / 2.0)).sin()
        })
        .collect();

    save_wav(
        &format!("{}/special/sweep_20Hz_20kHz.wav", base_dir),
        &signal,
        sample_rate,
        16,
    )?;

    println!();
    println!("✓ All reference audio files generated successfully!");

    // Count generated files
    let count = std::fs::read_dir(base_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "wav"))
        .count();
    println!("Total files: {}", count);

    Ok(())
}
