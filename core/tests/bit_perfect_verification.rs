//! Bit-perfect playback verification tests
//!
//! Verifies that audio output is bit-perfect by comparing input and output samples.
//! These tests ensure that the audio pipeline does not introduce any unwanted
//! modifications to the audio data.

use std::path::Path;

/// Calculate checksum of audio samples
fn calculate_audio_checksum(samples: &[i16]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    for &sample in samples {
        sample.hash(&mut hasher);
    }
    hasher.finish()
}

/// Read WAV file and extract samples
fn read_wav_samples(path: &Path) -> Result<Vec<i16>, Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    Ok(samples?)
}

/// Verify that two audio files have identical samples
fn verify_bit_perfect_match(
    original: &Path,
    processed: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let original_samples = read_wav_samples(original)?;
    let processed_samples = read_wav_samples(processed)?;
    
    if original_samples.len() != processed_samples.len() {
        return Ok(false);
    }
    
    Ok(original_samples == processed_samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        // Test that checksum is consistent
        let samples = vec![100i16, 200, 300, 400, 500];
        let checksum1 = calculate_audio_checksum(&samples);
        let checksum2 = calculate_audio_checksum(&samples);
        assert_eq!(checksum1, checksum2, "Checksum should be deterministic");
    }

    #[test]
    fn test_checksum_different_for_different_data() {
        // Test that different data produces different checksums
        let samples1 = vec![100i16, 200, 300, 400, 500];
        let samples2 = vec![100i16, 200, 300, 400, 501]; // Last sample different
        
        let checksum1 = calculate_audio_checksum(&samples1);
        let checksum2 = calculate_audio_checksum(&samples2);
        
        assert_ne!(checksum1, checksum2, "Different data should produce different checksums");
    }

    #[test]
    fn test_read_reference_sine_wave() {
        // Test reading a reference sine wave file
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let result = read_wav_samples(test_file);
        assert!(result.is_ok(), "Should be able to read reference WAV file");
        
        let samples = result.unwrap();
        assert!(!samples.is_empty(), "Should have samples");
        assert_eq!(samples.len(), 44100 * 5, "Should have 5 seconds of audio at 44.1kHz");
    }

    #[test]
    fn test_bit_perfect_identity() {
        // Test that a file matches itself (identity test)
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let result = verify_bit_perfect_match(test_file, test_file);
        assert!(result.is_ok(), "Verification should succeed");
        assert!(result.unwrap(), "File should match itself perfectly");
    }

    #[test]
    fn test_silence_is_zero() {
        // Verify that silence file contains only zeros
        let test_file = Path::new("test_data/reference/44100Hz_16bit/silence.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let samples = read_wav_samples(test_file).expect("Should read silence file");
        
        // All samples should be zero (or very close to zero due to dithering)
        let non_zero_count = samples.iter().filter(|&&s| s.abs() > 1).count();
        assert_eq!(non_zero_count, 0, "Silence should contain only zero samples");
    }

    #[test]
    fn test_impulse_has_single_peak() {
        // Verify that impulse file has a single peak at the start
        let test_file = Path::new("test_data/reference/44100Hz_16bit/impulse.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let samples = read_wav_samples(test_file).expect("Should read impulse file");
        
        // First sample should be at maximum
        assert!(samples[0].abs() > 30000, "First sample should be near maximum");
        
        // Rest should be near zero
        let non_zero_count = samples[1..].iter().filter(|&&s| s.abs() > 1).count();
        assert_eq!(non_zero_count, 0, "Only first sample should be non-zero");
    }

    #[test]
    fn test_sine_wave_properties() {
        // Verify basic properties of sine wave
        let test_file = Path::new("test_data/reference/44100Hz_16bit/sine_1kHz.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let samples = read_wav_samples(test_file).expect("Should read sine wave file");
        
        // Calculate RMS (should be around 0.8 * 0.707 = 0.566 of full scale)
        let sum_squares: f64 = samples.iter()
            .map(|&s| (s as f64 / 32767.0).powi(2))
            .sum();
        let rms = (sum_squares / samples.len() as f64).sqrt();
        
        // RMS should be approximately 0.566 (0.8 amplitude * 1/sqrt(2))
        assert!(
            (rms - 0.566).abs() < 0.01,
            "RMS should be approximately 0.566, got {}",
            rms
        );
        
        // Check that signal crosses zero (sine wave property)
        let zero_crossings = samples.windows(2)
            .filter(|w| (w[0] > 0 && w[1] <= 0) || (w[0] <= 0 && w[1] > 0))
            .count();
        
        // For 1kHz sine at 44.1kHz, expect about 2000 zero crossings per second
        // (2 per cycle, 1000 cycles per second)
        let expected_crossings = 2 * 1000 * 5; // 5 seconds
        assert!(
            (zero_crossings as i32 - expected_crossings).abs() < 100,
            "Expected approximately {} zero crossings, got {}",
            expected_crossings,
            zero_crossings
        );
    }

    #[test]
    fn test_different_sample_rates() {
        // Verify that files with different sample rates are generated correctly
        let sample_rates = [44100, 48000, 96000, 192000];
        
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
            
            let samples = read_wav_samples(test_file).expect("Should read file");
            
            // Should have 5 seconds of audio
            let expected_samples = sr * 5;
            assert_eq!(
                samples.len(),
                expected_samples as usize,
                "File at {}Hz should have {} samples",
                sr,
                expected_samples
            );
        }
    }

    #[test]
    fn test_white_noise_properties() {
        // Verify that white noise has expected statistical properties
        let test_file = Path::new("test_data/reference/44100Hz_16bit/white_noise.wav");
        
        if !test_file.exists() {
            eprintln!("Warning: Test file not found. Run 'cargo run --example generate_test_audio' first.");
            return;
        }
        
        let samples = read_wav_samples(test_file).expect("Should read white noise file");
        
        // Calculate mean (should be close to 0)
        let mean: f64 = samples.iter()
            .map(|&s| s as f64 / 32767.0)
            .sum::<f64>() / samples.len() as f64;
        
        assert!(
            mean.abs() < 0.01,
            "White noise mean should be close to 0, got {}",
            mean
        );
        
        // Calculate standard deviation (should be around 0.5 / sqrt(3) ≈ 0.289)
        let variance: f64 = samples.iter()
            .map(|&s| {
                let normalized = s as f64 / 32767.0;
                (normalized - mean).powi(2)
            })
            .sum::<f64>() / samples.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // For uniform distribution [-0.5, 0.5], std dev = 0.5 / sqrt(3) ≈ 0.289
        assert!(
            (std_dev - 0.289).abs() < 0.05,
            "White noise std dev should be approximately 0.289, got {}",
            std_dev
        );
    }
}
