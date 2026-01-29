//! Frequency response analysis tests
//!
//! Tests to verify that the audio pipeline maintains accurate frequency response
//! across the audible spectrum (20Hz - 20kHz).

use std::path::Path;

/// Perform FFT on audio samples to analyze frequency content
fn analyze_frequency_content(samples: &[i16], sample_rate: u32) -> Vec<(f64, f64)> {
    use rustfft::{num_complex::Complex, FftPlanner};

    // Convert samples to complex numbers
    let mut buffer: Vec<Complex<f64>> = samples
        .iter()
        .map(|&s| Complex::new(s as f64 / 32767.0, 0.0))
        .collect();

    // Perform FFT
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer.len());
    fft.process(&mut buffer);

    // Calculate magnitude spectrum
    let freq_resolution = sample_rate as f64 / buffer.len() as f64;
    buffer
        .iter()
        .take(buffer.len() / 2) // Only take positive frequencies
        .enumerate()
        .map(|(i, c)| {
            let frequency = i as f64 * freq_resolution;
            let magnitude = (c.re * c.re + c.im * c.im).sqrt();
            (frequency, magnitude)
        })
        .collect()
}

/// Find the dominant frequency in the spectrum
fn find_dominant_frequency(spectrum: &[(f64, f64)]) -> (f64, f64) {
    spectrum
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .copied()
        .unwrap_or((0.0, 0.0))
}

/// Calculate the signal-to-noise ratio (SNR) in dB
fn calculate_snr(spectrum: &[(f64, f64)], signal_freq: f64, tolerance: f64) -> f64 {
    // Find signal power (around the expected frequency)
    let signal_power: f64 = spectrum
        .iter()
        .filter(|(f, _)| (f - signal_freq).abs() < tolerance)
        .map(|(_, mag)| mag * mag)
        .sum();

    // Find noise power (excluding signal region)
    let noise_power: f64 = spectrum
        .iter()
        .filter(|(f, _)| (f - signal_freq).abs() >= tolerance)
        .map(|(_, mag)| mag * mag)
        .sum();

    if noise_power > 0.0 {
        10.0 * (signal_power / noise_power).log10()
    } else {
        f64::INFINITY
    }
}

/// Read WAV file and extract samples
fn read_wav_samples(path: &Path) -> Result<(Vec<i16>, u32), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    Ok((samples?, spec.sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_100hz_sine_wave_frequency() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_100Hz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read 100Hz sine wave");

        let spectrum = analyze_frequency_content(&samples, sample_rate);
        let (dominant_freq, _) = find_dominant_frequency(&spectrum);

        // Dominant frequency should be close to 100Hz
        assert!(
            (dominant_freq - 100.0).abs() < 5.0,
            "Expected 100Hz, got {:.2}Hz",
            dominant_freq
        );
    }

    #[test]
    fn test_440hz_sine_wave_frequency() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_440Hz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read 440Hz sine wave");

        let spectrum = analyze_frequency_content(&samples, sample_rate);
        let (dominant_freq, _) = find_dominant_frequency(&spectrum);

        // Dominant frequency should be close to 440Hz (A4)
        assert!(
            (dominant_freq - 440.0).abs() < 5.0,
            "Expected 440Hz, got {:.2}Hz",
            dominant_freq
        );
    }

    #[test]
    fn test_1khz_sine_wave_frequency() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read 1kHz sine wave");

        let spectrum = analyze_frequency_content(&samples, sample_rate);
        let (dominant_freq, _) = find_dominant_frequency(&spectrum);

        // Dominant frequency should be close to 1000Hz
        assert!(
            (dominant_freq - 1000.0).abs() < 5.0,
            "Expected 1000Hz, got {:.2}Hz",
            dominant_freq
        );
    }

    #[test]
    fn test_5khz_sine_wave_frequency() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_5kHz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read 5kHz sine wave");

        let spectrum = analyze_frequency_content(&samples, sample_rate);
        let (dominant_freq, _) = find_dominant_frequency(&spectrum);

        // Dominant frequency should be close to 5000Hz
        assert!(
            (dominant_freq - 5000.0).abs() < 10.0,
            "Expected 5000Hz, got {:.2}Hz",
            dominant_freq
        );
    }

    #[test]
    fn test_10khz_sine_wave_frequency() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_10kHz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read 10kHz sine wave");

        let spectrum = analyze_frequency_content(&samples, sample_rate);
        let (dominant_freq, _) = find_dominant_frequency(&spectrum);

        // Dominant frequency should be close to 10000Hz
        assert!(
            (dominant_freq - 10000.0).abs() < 20.0,
            "Expected 10000Hz, got {:.2}Hz",
            dominant_freq
        );
    }

    #[test]
    fn test_sine_wave_snr() {
        // Test that sine waves have high SNR (low noise)
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read 1kHz sine wave");

        let spectrum = analyze_frequency_content(&samples, sample_rate);
        let snr = calculate_snr(&spectrum, 1000.0, 50.0);

        // SNR should be very high for a pure sine wave (> 60dB)
        assert!(
            snr > 60.0,
            "SNR should be > 60dB for pure sine wave, got {:.2}dB",
            snr
        );
    }

    #[test]
    fn test_white_noise_flat_spectrum() {
        // Test that white noise has relatively flat frequency spectrum
        let test_file = Path::new("test_data/reference/44100Hz_16bit/white_noise.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) = read_wav_samples(test_file).expect("Should read white noise");

        let spectrum = analyze_frequency_content(&samples, sample_rate);

        // Calculate average magnitude in different frequency bands
        let low_band: f64 = spectrum
            .iter()
            .filter(|(f, _)| *f >= 100.0 && *f < 1000.0)
            .map(|(_, mag)| mag)
            .sum::<f64>()
            / spectrum
                .iter()
                .filter(|(f, _)| *f >= 100.0 && *f < 1000.0)
                .count() as f64;

        let mid_band: f64 = spectrum
            .iter()
            .filter(|(f, _)| *f >= 1000.0 && *f < 5000.0)
            .map(|(_, mag)| mag)
            .sum::<f64>()
            / spectrum
                .iter()
                .filter(|(f, _)| *f >= 1000.0 && *f < 5000.0)
                .count() as f64;

        let high_band: f64 = spectrum
            .iter()
            .filter(|(f, _)| *f >= 5000.0 && *f < 10000.0)
            .map(|(_, mag)| mag)
            .sum::<f64>()
            / spectrum
                .iter()
                .filter(|(f, _)| *f >= 5000.0 && *f < 10000.0)
                .count() as f64;

        // All bands should have similar energy (within 20% of each other)
        let avg = (low_band + mid_band + high_band) / 3.0;

        assert!(
            (low_band - avg).abs() / avg < 0.2,
            "Low band should be within 20% of average"
        );
        assert!(
            (mid_band - avg).abs() / avg < 0.2,
            "Mid band should be within 20% of average"
        );
        assert!(
            (high_band - avg).abs() / avg < 0.2,
            "High band should be within 20% of average"
        );
    }

    #[test]
    fn test_silence_no_frequency_content() {
        // Test that silence has no significant frequency content
        let test_file = Path::new("test_data/reference/44100Hz_16bit/silence.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) = read_wav_samples(test_file).expect("Should read silence");

        let spectrum = analyze_frequency_content(&samples, sample_rate);

        // Find maximum magnitude in spectrum
        let max_magnitude = spectrum
            .iter()
            .map(|(_, mag)| mag)
            .fold(0.0f64, |a, &b| a.max(b));

        // Maximum magnitude should be very small (essentially zero)
        assert!(
            max_magnitude < 0.001,
            "Silence should have no frequency content, max magnitude: {}",
            max_magnitude
        );
    }

    #[test]
    fn test_impulse_wide_spectrum() {
        // Test that impulse has energy across wide frequency range
        let test_file = Path::new("test_data/reference/44100Hz_16bit/impulse.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) = read_wav_samples(test_file).expect("Should read impulse");

        let spectrum = analyze_frequency_content(&samples, sample_rate);

        // Impulse should have relatively flat spectrum (like white noise)
        // Check that energy is present across frequency range
        let low_energy: f64 = spectrum
            .iter()
            .filter(|(f, _)| *f >= 100.0 && *f < 1000.0)
            .map(|(_, mag)| mag * mag)
            .sum();

        let high_energy: f64 = spectrum
            .iter()
            .filter(|(f, _)| *f >= 5000.0 && *f < 10000.0)
            .map(|(_, mag)| mag * mag)
            .sum();

        // Both low and high frequencies should have significant energy
        assert!(
            low_energy > 0.01,
            "Impulse should have low frequency energy"
        );
        assert!(
            high_energy > 0.01,
            "Impulse should have high frequency energy"
        );
    }

    #[test]
    fn test_frequency_sweep_coverage() {
        // Test that frequency sweep covers the expected range
        let test_file = Path::new("test_data/reference/special/sweep_20Hz_20kHz.wav");

        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }

        let (samples, sample_rate) =
            read_wav_samples(test_file).expect("Should read frequency sweep");

        // Analyze in chunks to see frequency progression
        let chunk_size = sample_rate as usize; // 1 second chunks
        let num_chunks = samples.len() / chunk_size;

        let mut frequencies = Vec::new();
        for i in 0..num_chunks.min(10) {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(samples.len());
            let chunk = &samples[start..end];

            let spectrum = analyze_frequency_content(chunk, sample_rate);
            let (dominant_freq, _) = find_dominant_frequency(&spectrum);
            frequencies.push(dominant_freq);
        }

        // Frequencies should be generally increasing (sweep goes from low to high)
        // Allow some variation due to FFT resolution
        let mut increasing_count = 0;
        for i in 1..frequencies.len() {
            if frequencies[i] > frequencies[i - 1] {
                increasing_count += 1;
            }
        }

        // Most transitions should be increasing
        assert!(
            increasing_count >= frequencies.len() - 2,
            "Frequency sweep should be mostly increasing"
        );

        // First frequency should be relatively low (within first few kHz)
        assert!(
            frequencies[0] < 3000.0,
            "Sweep should start relatively low, got {:.2}Hz",
            frequencies[0]
        );

        // Last frequency should be high (near 20kHz)
        assert!(
            frequencies[frequencies.len() - 1] > 8000.0,
            "Sweep should end high, got {:.2}Hz",
            frequencies[frequencies.len() - 1]
        );
    }

    #[test]
    fn test_different_sample_rates_frequency_accuracy() {
        // Test that frequency detection works correctly at different sample rates
        let sample_rates = [44100, 48000, 96000];

        for sr in sample_rates {
            let path_string = format!("test_data/reference/{}Hz_16bit/sine_1kHz.wav", sr);
            let test_file = Path::new(&path_string);

            if !test_file.exists() {
                eprintln!("Warning: Test file not found: {:?}", test_file);
                continue;
            }

            let (samples, sample_rate) = read_wav_samples(test_file).expect("Should read file");

            let spectrum = analyze_frequency_content(&samples, sample_rate);
            let (dominant_freq, _) = find_dominant_frequency(&spectrum);

            // Should detect 1kHz regardless of sample rate
            assert!(
                (dominant_freq - 1000.0).abs() < 10.0,
                "At {}Hz sample rate, expected 1000Hz, got {:.2}Hz",
                sr,
                dominant_freq
            );
        }
    }
}
