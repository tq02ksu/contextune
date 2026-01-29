//! Audio checksum calculation utilities
//!
//! Provides functions to calculate checksums of audio data for verification
//! and integrity checking. Supports multiple checksum algorithms.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Checksum algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// Simple hash-based checksum (fast)
    Simple,
    /// CRC32 checksum
    Crc32,
    /// MD5 hash (for compatibility)
    Md5,
    /// SHA256 hash (secure)
    Sha256,
}

/// Audio checksum result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioChecksum {
    /// The checksum algorithm used
    pub algorithm: ChecksumAlgorithm,
    /// The checksum value as hex string
    pub value: String,
    /// Number of samples processed
    pub sample_count: usize,
}

impl AudioChecksum {
    /// Create a new checksum
    pub fn new(algorithm: ChecksumAlgorithm, value: String, sample_count: usize) -> Self {
        Self {
            algorithm,
            value,
            sample_count,
        }
    }
}

/// Calculate simple hash-based checksum of i16 samples
pub fn calculate_simple_checksum(samples: &[i16]) -> AudioChecksum {
    let mut hasher = DefaultHasher::new();

    for &sample in samples {
        sample.hash(&mut hasher);
    }

    let hash = hasher.finish();

    AudioChecksum::new(
        ChecksumAlgorithm::Simple,
        format!("{:016x}", hash),
        samples.len(),
    )
}

/// Calculate simple hash-based checksum of f32 samples
pub fn calculate_simple_checksum_f32(samples: &[f32]) -> AudioChecksum {
    let mut hasher = DefaultHasher::new();

    for &sample in samples {
        // Convert to bits for consistent hashing
        sample.to_bits().hash(&mut hasher);
    }

    let hash = hasher.finish();

    AudioChecksum::new(
        ChecksumAlgorithm::Simple,
        format!("{:016x}", hash),
        samples.len(),
    )
}

/// Calculate simple hash-based checksum of f64 samples
pub fn calculate_simple_checksum_f64(samples: &[f64]) -> AudioChecksum {
    let mut hasher = DefaultHasher::new();

    for &sample in samples {
        // Convert to bits for consistent hashing
        sample.to_bits().hash(&mut hasher);
    }

    let hash = hasher.finish();

    AudioChecksum::new(
        ChecksumAlgorithm::Simple,
        format!("{:016x}", hash),
        samples.len(),
    )
}

/// Calculate CRC32 checksum of i16 samples
pub fn calculate_crc32_checksum(samples: &[i16]) -> AudioChecksum {
    let mut crc = crc32fast::Hasher::new();

    // Convert samples to bytes
    for &sample in samples {
        crc.update(&sample.to_le_bytes());
    }

    let checksum = crc.finalize();

    AudioChecksum::new(
        ChecksumAlgorithm::Crc32,
        format!("{:08x}", checksum),
        samples.len(),
    )
}

/// Calculate MD5 hash of i16 samples
pub fn calculate_md5_checksum(samples: &[i16]) -> AudioChecksum {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();

    // Convert samples to bytes
    for &sample in samples {
        hasher.update(&sample.to_le_bytes());
    }

    let result = hasher.finalize();

    AudioChecksum::new(
        ChecksumAlgorithm::Md5,
        format!("{:x}", result),
        samples.len(),
    )
}

/// Calculate SHA256 hash of i16 samples
pub fn calculate_sha256_checksum(samples: &[i16]) -> AudioChecksum {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    // Convert samples to bytes
    for &sample in samples {
        hasher.update(&sample.to_le_bytes());
    }

    let result = hasher.finalize();

    AudioChecksum::new(
        ChecksumAlgorithm::Sha256,
        format!("{:x}", result),
        samples.len(),
    )
}

/// Calculate checksum using specified algorithm
pub fn calculate_checksum(samples: &[i16], algorithm: ChecksumAlgorithm) -> AudioChecksum {
    match algorithm {
        ChecksumAlgorithm::Simple => calculate_simple_checksum(samples),
        ChecksumAlgorithm::Crc32 => calculate_crc32_checksum(samples),
        ChecksumAlgorithm::Md5 => calculate_md5_checksum(samples),
        ChecksumAlgorithm::Sha256 => calculate_sha256_checksum(samples),
    }
}

/// Verify that two checksums match
pub fn verify_checksum(checksum1: &AudioChecksum, checksum2: &AudioChecksum) -> bool {
    checksum1.algorithm == checksum2.algorithm
        && checksum1.value == checksum2.value
        && checksum1.sample_count == checksum2.sample_count
}

/// Calculate RMS (Root Mean Square) of audio samples
pub fn calculate_rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f64 = samples
        .iter()
        .map(|&s| {
            let normalized = s as f64 / i16::MAX as f64;
            normalized * normalized
        })
        .sum();

    (sum_squares / samples.len() as f64).sqrt()
}

/// Calculate peak amplitude of audio samples
pub fn calculate_peak(samples: &[i16]) -> i16 {
    samples.iter().map(|&s| s.abs()).max().unwrap_or(0)
}

/// Audio statistics
#[derive(Debug, Clone)]
pub struct AudioStats {
    /// Number of samples
    pub sample_count: usize,
    /// RMS (Root Mean Square) value
    pub rms: f64,
    /// Peak amplitude
    pub peak: i16,
    /// Minimum sample value
    pub min: i16,
    /// Maximum sample value
    pub max: i16,
    /// Average sample value
    pub average: f64,
}

/// Calculate comprehensive audio statistics
pub fn calculate_audio_stats(samples: &[i16]) -> AudioStats {
    if samples.is_empty() {
        return AudioStats {
            sample_count: 0,
            rms: 0.0,
            peak: 0,
            min: 0,
            max: 0,
            average: 0.0,
        };
    }

    let rms = calculate_rms(samples);
    let peak = calculate_peak(samples);
    let min = *samples.iter().min().unwrap();
    let max = *samples.iter().max().unwrap();
    let average = samples.iter().map(|&s| s as f64).sum::<f64>() / samples.len() as f64;

    AudioStats {
        sample_count: samples.len(),
        rms,
        peak,
        min,
        max,
        average,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_checksum_deterministic() {
        let samples = vec![100i16, 200, 300, 400, 500];

        let checksum1 = calculate_simple_checksum(&samples);
        let checksum2 = calculate_simple_checksum(&samples);

        assert_eq!(checksum1.value, checksum2.value);
        assert_eq!(checksum1.sample_count, 5);
    }

    #[test]
    fn test_simple_checksum_different_data() {
        let samples1 = vec![100i16, 200, 300, 400, 500];
        let samples2 = vec![100i16, 200, 300, 400, 501];

        let checksum1 = calculate_simple_checksum(&samples1);
        let checksum2 = calculate_simple_checksum(&samples2);

        assert_ne!(checksum1.value, checksum2.value);
    }

    #[test]
    fn test_crc32_checksum() {
        let samples = vec![100i16, 200, 300, 400, 500];

        let checksum = calculate_crc32_checksum(&samples);

        assert_eq!(checksum.algorithm, ChecksumAlgorithm::Crc32);
        assert_eq!(checksum.sample_count, 5);
        assert_eq!(checksum.value.len(), 8); // CRC32 is 8 hex chars
    }

    #[test]
    fn test_md5_checksum() {
        let samples = vec![100i16, 200, 300, 400, 500];

        let checksum = calculate_md5_checksum(&samples);

        assert_eq!(checksum.algorithm, ChecksumAlgorithm::Md5);
        assert_eq!(checksum.sample_count, 5);
        assert_eq!(checksum.value.len(), 32); // MD5 is 32 hex chars
    }

    #[test]
    fn test_sha256_checksum() {
        let samples = vec![100i16, 200, 300, 400, 500];

        let checksum = calculate_sha256_checksum(&samples);

        assert_eq!(checksum.algorithm, ChecksumAlgorithm::Sha256);
        assert_eq!(checksum.sample_count, 5);
        assert_eq!(checksum.value.len(), 64); // SHA256 is 64 hex chars
    }

    #[test]
    fn test_verify_checksum_match() {
        let samples = vec![100i16, 200, 300, 400, 500];

        let checksum1 = calculate_simple_checksum(&samples);
        let checksum2 = calculate_simple_checksum(&samples);

        assert!(verify_checksum(&checksum1, &checksum2));
    }

    #[test]
    fn test_verify_checksum_mismatch() {
        let samples1 = vec![100i16, 200, 300, 400, 500];
        let samples2 = vec![100i16, 200, 300, 400, 501];

        let checksum1 = calculate_simple_checksum(&samples1);
        let checksum2 = calculate_simple_checksum(&samples2);

        assert!(!verify_checksum(&checksum1, &checksum2));
    }

    #[test]
    fn test_calculate_rms() {
        // Test with known values
        let samples = vec![0i16, 16384, 0, -16384]; // Half of max amplitude
        let rms = calculate_rms(&samples);

        // RMS should be approximately 0.5 / sqrt(2) ≈ 0.354
        assert!(
            (rms - 0.354).abs() < 0.01,
            "Expected RMS ~0.354, got {}",
            rms
        );
    }

    #[test]
    fn test_calculate_peak() {
        let samples = vec![100i16, -200, 300, -400, 500];
        let peak = calculate_peak(&samples);

        assert_eq!(peak, 500);
    }

    #[test]
    fn test_audio_stats() {
        let samples = vec![100i16, 200, 300, 400, 500];
        let stats = calculate_audio_stats(&samples);

        assert_eq!(stats.sample_count, 5);
        assert_eq!(stats.min, 100);
        assert_eq!(stats.max, 500);
        assert_eq!(stats.average, 300.0);
        assert!(stats.rms > 0.0);
        assert_eq!(stats.peak, 500);
    }

    #[test]
    fn test_empty_samples() {
        let samples: Vec<i16> = vec![];
        let stats = calculate_audio_stats(&samples);

        assert_eq!(stats.sample_count, 0);
        assert_eq!(stats.rms, 0.0);
        assert_eq!(stats.peak, 0);
    }

    #[test]
    fn test_f32_checksum() {
        let samples = vec![0.1f32, 0.2, 0.3, 0.4, 0.5];

        let checksum1 = calculate_simple_checksum_f32(&samples);
        let checksum2 = calculate_simple_checksum_f32(&samples);

        assert_eq!(checksum1.value, checksum2.value);
        assert_eq!(checksum1.sample_count, 5);
    }

    #[test]
    fn test_f64_checksum() {
        let samples = vec![0.1f64, 0.2, 0.3, 0.4, 0.5];

        let checksum1 = calculate_simple_checksum_f64(&samples);
        let checksum2 = calculate_simple_checksum_f64(&samples);

        assert_eq!(checksum1.value, checksum2.value);
        assert_eq!(checksum1.sample_count, 5);
    }

    #[test]
    fn test_all_algorithms() {
        let samples = vec![100i16, 200, 300, 400, 500];

        let algorithms = [
            ChecksumAlgorithm::Simple,
            ChecksumAlgorithm::Crc32,
            ChecksumAlgorithm::Md5,
            ChecksumAlgorithm::Sha256,
        ];

        for algorithm in algorithms {
            let checksum = calculate_checksum(&samples, algorithm);
            assert_eq!(checksum.algorithm, algorithm);
            assert_eq!(checksum.sample_count, 5);
            assert!(!checksum.value.is_empty());
        }
    }
}
