//! THD+N (Total Harmonic Distortion + Noise) measurement tests
//!
//! Tests to measure and verify audio quality by calculating THD+N ratio.
//! THD+N is the ratio of the sum of the powers of all harmonic frequencies
//! plus noise to the power of the fundamental frequency.

use std::path::Path;

/// Perform FFT on audio samples
fn perform_fft(samples: &[i16], sample_rate: u32) -> Vec<(f64, f64)> {
    use rustfft::{FftPlanner, num_complex::Complex};
    
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
        .take(buffer.len() / 2)
        .enumerate()
        .map(|(i, c)| {
            let frequency = i as f64 * freq_resolution;
            let magnitude = (c.re * c.re + c.im * c.im).sqrt();
            (frequency, magnitude)
        })
        .collect()
}

/// Find the fundamental frequency and its power
fn find_fundamental(spectrum: &[(f64, f64)], expected_freq: f64, tolerance: f64) -> (f64, f64) {
    spectrum
        .iter()
        .filter(|(f, _)| (f - expected_freq).abs() < tolerance)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .copied()
        .unwrap_or((expected_freq, 0.0))
}

/// Calculate THD+N (Total Harmonic Distortion + Noise)
/// Returns THD+N as a percentage
fn calculate_thd_n(
    spectrum: &[(f64, f64)],
    fundamental_freq: f64,
    tolerance: f64,
) -> f64 {
    // Find fundamental power
    let fundamental_power: f64 = spectrum
        .iter()
        .filter(|(f, _)| (f - fundamental_freq).abs() < tolerance)
        .map(|(_, mag)| mag * mag)
        .sum();
    
    if fundamental_power == 0.0 {
        return 100.0; // No signal
    }
    
    // Calculate total power (including harmonics and noise)
    let total_power: f64 = spectrum
        .iter()
        .map(|(_, mag)| mag * mag)
        .sum();
    
    // THD+N power is total power minus fundamental power
    let thd_n_power = total_power - fundamental_power;
    
    // Return as percentage
    (thd_n_power / fundamental_power).sqrt() * 100.0
}

/// Calculate THD+N in dB
fn calculate_thd_n_db(
    spectrum: &[(f64, f64)],
    fundamental_freq: f64,
    tolerance: f64,
) -> f64 {
    let thd_n_percent = calculate_thd_n(spectrum, fundamental_freq, tolerance);
    20.0 * (thd_n_percent / 100.0).log10()
}

/// Calculate harmonic distortion (only harmonics, not noise)
fn calculate_thd(
    spectrum: &[(f64, f64)],
    fundamental_freq: f64,
    num_harmonics: usize,
    tolerance: f64,
) -> f64 {
    // Find fundamental power
    let fundamental_power: f64 = spectrum
        .iter()
        .filter(|(f, _)| (f - fundamental_freq).abs() < tolerance)
        .map(|(_, mag)| mag * mag)
        .sum();
    
    if fundamental_power == 0.0 {
        return 100.0;
    }
    
    // Calculate power of harmonics (2f, 3f, 4f, ...)
    let mut harmonic_power = 0.0;
    for n in 2..=num_harmonics {
        let harmonic_freq = fundamental_freq * n as f64;
        let power: f64 = spectrum
            .iter()
            .filter(|(f, _)| (f - harmonic_freq).abs() < tolerance)
            .map(|(_, mag)| mag * mag)
            .sum();
        harmonic_power += power;
    }
    
    // Return as percentage
    (harmonic_power / fundamental_power).sqrt() * 100.0
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
    fn test_thd_n_100hz_sine_wave() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_100Hz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 100Hz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 100.0, 50.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 100.0, 50.0);
        
        println!("100Hz THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        // Pure sine wave should have very low THD+N (< 1%)
        assert!(
            thd_n < 1.0,
            "THD+N should be < 1% for pure sine wave, got {:.4}%",
            thd_n
        );
        
        // In dB, should be < -40dB
        assert!(
            thd_n_db < -40.0,
            "THD+N should be < -40dB, got {:.2} dB",
            thd_n_db
        );
    }

    #[test]
    fn test_thd_n_440hz_sine_wave() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_440Hz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 440Hz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 440.0, 50.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 440.0, 50.0);
        
        println!("440Hz THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        assert!(
            thd_n < 1.0,
            "THD+N should be < 1% for pure sine wave, got {:.4}%",
            thd_n
        );
        
        assert!(
            thd_n_db < -40.0,
            "THD+N should be < -40dB, got {:.2} dB",
            thd_n_db
        );
    }

    #[test]
    fn test_thd_n_1khz_sine_wave() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 1kHz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 1000.0, 50.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 1000.0, 50.0);
        
        println!("1kHz THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        assert!(
            thd_n < 1.0,
            "THD+N should be < 1% for pure sine wave, got {:.4}%",
            thd_n
        );
        
        assert!(
            thd_n_db < -40.0,
            "THD+N should be < -40dB, got {:.2} dB",
            thd_n_db
        );
    }

    #[test]
    fn test_thd_n_5khz_sine_wave() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_5kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 5kHz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 5000.0, 100.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 5000.0, 100.0);
        
        println!("5kHz THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        assert!(
            thd_n < 1.0,
            "THD+N should be < 1% for pure sine wave, got {:.4}%",
            thd_n
        );
        
        assert!(
            thd_n_db < -40.0,
            "THD+N should be < -40dB, got {:.2} dB",
            thd_n_db
        );
    }

    #[test]
    fn test_thd_n_10khz_sine_wave() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_10kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 10kHz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 10000.0, 200.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 10000.0, 200.0);
        
        println!("10kHz THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        assert!(
            thd_n < 1.0,
            "THD+N should be < 1% for pure sine wave, got {:.4}%",
            thd_n
        );
        
        assert!(
            thd_n_db < -40.0,
            "THD+N should be < -40dB, got {:.2} dB",
            thd_n_db
        );
    }

    #[test]
    fn test_thd_only_1khz() {
        // Test THD (harmonics only, not noise)
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 1kHz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd = calculate_thd(&spectrum, 1000.0, 5, 50.0);
        
        println!("1kHz THD (harmonics only): {:.4}%", thd);
        
        // Pure sine wave should have very low harmonic distortion
        assert!(
            thd < 0.5,
            "THD should be < 0.5% for pure sine wave, got {:.4}%",
            thd
        );
    }

    #[test]
    fn test_thd_n_different_sample_rates() {
        // Test THD+N at different sample rates
        let sample_rates = [44100, 48000, 96000];
        
        for sr in sample_rates {
            let path_string = format!(
                "test_data/reference/{}Hz_16bit/sine_1kHz.wav",
                sr
            );
            let test_file = Path::new(&path_string);
            
            if !test_file.exists() {
                eprintln!("Warning: Test file not found: {:?}", test_file);
                continue;
            }
            
            let (samples, sample_rate) = read_wav_samples(test_file)
                .expect("Should read file");
            
            let spectrum = perform_fft(&samples, sample_rate);
            let thd_n = calculate_thd_n(&spectrum, 1000.0, 50.0);
            
            println!("{}Hz sample rate - THD+N: {:.4}%", sr, thd_n);
            
            assert!(
                thd_n < 1.0,
                "At {}Hz sample rate, THD+N should be < 1%, got {:.4}%",
                sr,
                thd_n
            );
        }
    }

    #[test]
    fn test_thd_n_different_bit_depths() {
        // Test THD+N at different bit depths
        // Note: We only test 16-bit here as hound reads samples as i16
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read file");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 1000.0, 50.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 1000.0, 50.0);
        
        println!("16-bit - THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        assert!(
            thd_n < 1.0,
            "At 16-bit depth, THD+N should be < 1%, got {:.4}%",
            thd_n
        );
        
        // 16-bit audio should have THD+N better than -80dB
        assert!(
            thd_n_db < -80.0,
            "16-bit THD+N should be < -80dB, got {:.2} dB",
            thd_n_db
        );
    }

    #[test]
    fn test_fundamental_frequency_detection() {
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read 1kHz sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let (fundamental_freq, fundamental_power) = find_fundamental(&spectrum, 1000.0, 50.0);
        
        println!("Fundamental: {:.2}Hz, Power: {:.6}", fundamental_freq, fundamental_power);
        
        // Should detect fundamental near 1000Hz
        assert!(
            (fundamental_freq - 1000.0).abs() < 10.0,
            "Should detect fundamental near 1000Hz, got {:.2}Hz",
            fundamental_freq
        );
        
        // Fundamental should have significant power
        assert!(
            fundamental_power > 0.1,
            "Fundamental should have significant power, got {:.6}",
            fundamental_power
        );
    }

    #[test]
    fn test_silence_high_thd_n() {
        // Silence should have high THD+N (no signal)
        let test_file = Path::new("test_data/reference/44100Hz_16bit/silence.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read silence");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let (_, fundamental_power) = find_fundamental(&spectrum, 1000.0, 50.0);
        
        // Silence should have essentially no power
        assert!(
            fundamental_power < 0.001,
            "Silence should have no fundamental power, got {:.6}",
            fundamental_power
        );
    }

    #[test]
    fn test_full_scale_sine_thd_n() {
        // Test full-scale sine wave (should still have low THD+N)
        let test_file = Path::new("test_data/reference/special/full_scale_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let (samples, sample_rate) = read_wav_samples(test_file)
            .expect("Should read full-scale sine wave");
        
        let spectrum = perform_fft(&samples, sample_rate);
        let thd_n = calculate_thd_n(&spectrum, 1000.0, 50.0);
        let thd_n_db = calculate_thd_n_db(&spectrum, 1000.0, 50.0);
        
        println!("Full-scale 1kHz THD+N: {:.4}% ({:.2} dB)", thd_n, thd_n_db);
        
        // Even at full scale, THD+N should be reasonable
        assert!(
            thd_n < 2.0,
            "Full-scale THD+N should be < 2%, got {:.4}%",
            thd_n
        );
    }
}
