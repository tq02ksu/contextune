//! 64-bit float audio processing
//!
//! Handles sample format conversion, volume control, and audio processing in 64-bit precision

use crate::audio::format::{AudioFormat, SampleFormat};
use crate::Result;

/// Convert samples from various formats to f64 normalized range [-1.0, 1.0]
pub trait SampleConverter {
    /// Convert to f64 samples
    fn to_f64(&self) -> Vec<f64>;
}

/// Convert u8 samples to f64 (unsigned 8-bit: 0-255 -> -1.0 to 1.0)
impl SampleConverter for &[u8] {
    fn to_f64(&self) -> Vec<f64> {
        self.iter()
            .map(|&sample| (sample as f64 / u8::MAX as f64) * 2.0 - 1.0)
            .collect()
    }
}

/// Convert i8 samples to f64 (signed 8-bit: -128 to 127 -> -1.0 to 1.0)
impl SampleConverter for &[i8] {
    fn to_f64(&self) -> Vec<f64> {
        self.iter()
            .map(|&sample| sample as f64 / i8::MAX as f64)
            .collect()
    }
}

/// Convert u16 samples to f64 (unsigned 16-bit: 0-65535 -> -1.0 to 1.0)
impl SampleConverter for &[u16] {
    fn to_f64(&self) -> Vec<f64> {
        self.iter()
            .map(|&sample| (sample as f64 / u16::MAX as f64) * 2.0 - 1.0)
            .collect()
    }
}

/// Convert i16 samples to f64 (signed 16-bit: -32768 to 32767 -> -1.0 to 1.0)
impl SampleConverter for &[i16] {
    fn to_f64(&self) -> Vec<f64> {
        self.iter()
            .map(|&sample| sample as f64 / i16::MAX as f64)
            .collect()
    }
}

/// Convert i32 samples to f64 (signed 32-bit: -2147483648 to 2147483647 -> -1.0 to 1.0)
impl SampleConverter for &[i32] {
    fn to_f64(&self) -> Vec<f64> {
        self.iter()
            .map(|&sample| sample as f64 / i32::MAX as f64)
            .collect()
    }
}

/// Convert f32 samples to f64 (already normalized)
impl SampleConverter for &[f32] {
    fn to_f64(&self) -> Vec<f64> {
        self.iter().map(|&sample| sample as f64).collect()
    }
}

/// Convert f64 samples to f64 (no conversion needed)
impl SampleConverter for &[f64] {
    fn to_f64(&self) -> Vec<f64> {
        self.to_vec()
    }
}

/// Convert f64 samples back to various formats
pub struct SampleFormatConverter;

impl SampleFormatConverter {
    /// Convert f64 samples to u8 (clamps to valid range)
    pub fn f64_to_u8(samples: &[f64]) -> Vec<u8> {
        samples
            .iter()
            .map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                ((clamped + 1.0) * 0.5 * u8::MAX as f64) as u8
            })
            .collect()
    }

    /// Convert f64 samples to i8 (clamps to valid range)
    pub fn f64_to_i8(samples: &[f64]) -> Vec<i8> {
        samples
            .iter()
            .map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                (clamped * i8::MAX as f64) as i8
            })
            .collect()
    }

    /// Convert f64 samples to u16 (clamps to valid range)
    pub fn f64_to_u16(samples: &[f64]) -> Vec<u16> {
        samples
            .iter()
            .map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                ((clamped + 1.0) * 0.5 * u16::MAX as f64) as u16
            })
            .collect()
    }

    /// Convert f64 samples to i16 (clamps to valid range)
    pub fn f64_to_i16(samples: &[f64]) -> Vec<i16> {
        samples
            .iter()
            .map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                (clamped * i16::MAX as f64) as i16
            })
            .collect()
    }

    /// Convert f64 samples to i32 (clamps to valid range)
    pub fn f64_to_i32(samples: &[f64]) -> Vec<i32> {
        samples
            .iter()
            .map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                (clamped * i32::MAX as f64) as i32
            })
            .collect()
    }

    /// Convert f64 samples to f32
    pub fn f64_to_f32(samples: &[f64]) -> Vec<f32> {
        samples.iter().map(|&sample| sample as f32).collect()
    }

    /// Convert f64 samples to the specified format
    pub fn convert_from_f64(samples: &[f64], target_format: SampleFormat) -> Vec<u8> {
        match target_format {
            SampleFormat::U8 => {
                let converted = Self::f64_to_u8(samples);
                converted.iter().flat_map(|&s| s.to_le_bytes()).collect()
            }
            SampleFormat::I8 => {
                let converted = Self::f64_to_i8(samples);
                converted.iter().flat_map(|&s| s.to_le_bytes()).collect()
            }
            SampleFormat::U16 => {
                let converted = Self::f64_to_u16(samples);
                converted.iter().flat_map(|&s| s.to_le_bytes()).collect()
            }
            SampleFormat::I16 => {
                let converted = Self::f64_to_i16(samples);
                converted.iter().flat_map(|&s| s.to_le_bytes()).collect()
            }
            SampleFormat::I24 => {
                // Convert to i32 first, then take only 3 bytes
                let converted = Self::f64_to_i32(samples);
                converted
                    .iter()
                    .flat_map(|&s| {
                        let bytes = s.to_le_bytes();
                        [bytes[0], bytes[1], bytes[2]] // Take only first 3 bytes
                    })
                    .collect()
            }
            SampleFormat::I32 => {
                let converted = Self::f64_to_i32(samples);
                converted.iter().flat_map(|&s| s.to_le_bytes()).collect()
            }
            SampleFormat::F32 => {
                let converted = Self::f64_to_f32(samples);
                converted.iter().flat_map(|&s| s.to_le_bytes()).collect()
            }
            SampleFormat::F64 => samples.iter().flat_map(|&s| s.to_le_bytes()).collect(),
        }
    }
}

/// Audio processor for 64-bit float processing
pub struct AudioProcessor {
    /// Current audio format
    format: AudioFormat,
    /// Current volume level (0.0 to 1.0)
    volume: f64,
    /// Target volume for ramping
    target_volume: f64,
    /// Volume ramp step per sample
    ramp_step: f64,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            volume: 1.0,
            target_volume: 1.0,
            ramp_step: 0.0,
        }
    }

    /// Get the current format
    pub fn format(&self) -> &AudioFormat {
        &self.format
    }

    /// Get the current volume level (0.0 to 1.0)
    pub fn volume(&self) -> f64 {
        self.volume
    }

    /// Set the volume level immediately (0.0 to 1.0)
    /// For smooth volume changes, use set_volume_ramped instead
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
        self.target_volume = self.volume;
        self.ramp_step = 0.0;
    }

    /// Set the volume level with ramping to prevent clicks
    ///
    /// # Arguments
    /// * `volume` - Target volume level (0.0 to 1.0)
    /// * `ramp_duration_ms` - Duration of the ramp in milliseconds
    pub fn set_volume_ramped(&mut self, volume: f64, ramp_duration_ms: f64) {
        let target = volume.clamp(0.0, 1.0);
        self.target_volume = target;

        if ramp_duration_ms <= 0.0 {
            self.volume = target;
            self.ramp_step = 0.0;
        } else {
            // Calculate step per sample
            let total_samples =
                (self.format.sample_rate as f64 * ramp_duration_ms / 1000.0).max(1.0);
            self.ramp_step = (target - self.volume) / total_samples;
        }
    }

    /// Apply volume to f64 samples
    /// This method applies the current volume and handles ramping if active
    pub fn apply_volume(&mut self, samples: &mut [f64]) {
        if self.ramp_step.abs() > 1e-10 {
            // Apply ramping
            for sample in samples.iter_mut() {
                *sample *= self.volume;
                self.volume += self.ramp_step;

                // Check if we've reached the target
                if (self.ramp_step > 0.0 && self.volume >= self.target_volume)
                    || (self.ramp_step < 0.0 && self.volume <= self.target_volume)
                {
                    self.volume = self.target_volume;
                    self.ramp_step = 0.0;
                }
            }
        } else {
            // Apply constant volume
            for sample in samples.iter_mut() {
                *sample *= self.volume;
            }
        }
    }

    /// Apply volume to f64 samples without modifying the processor state
    /// Useful for preview or one-off volume adjustments
    pub fn apply_volume_static(samples: &mut [f64], volume: f64) {
        let vol = volume.clamp(0.0, 1.0);
        for sample in samples.iter_mut() {
            *sample *= vol;
        }
    }

    /// Convert volume from decibels to linear scale
    ///
    /// # Arguments
    /// * `db` - Volume in decibels (typically -60.0 to 0.0)
    ///
    /// # Returns
    /// Linear volume scale (0.0 to 1.0)
    pub fn db_to_linear(db: f64) -> f64 {
        if db <= -60.0 {
            0.0
        } else {
            10_f64.powf(db / 20.0)
        }
    }

    /// Convert volume from linear scale to decibels
    ///
    /// # Arguments
    /// * `linear` - Linear volume (0.0 to 1.0)
    ///
    /// # Returns
    /// Volume in decibels
    pub fn linear_to_db(linear: f64) -> f64 {
        if linear <= 0.0 {
            -60.0
        } else {
            20.0 * linear.log10()
        }
    }

    /// Convert samples to f64 based on the current format
    pub fn convert_to_f64(&self, samples: &[u8]) -> Result<Vec<f64>> {
        let sample_size = self.format.sample_format.size_bytes();
        let num_samples = samples.len() / sample_size;

        let f64_samples = match self.format.sample_format {
            SampleFormat::U8 => samples.to_f64(),
            SampleFormat::I8 => {
                let i8_samples: Vec<i8> = samples.iter().map(|&b| b as i8).collect();
                i8_samples.as_slice().to_f64()
            }
            SampleFormat::U16 => {
                let u16_samples: Vec<u16> = samples
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();
                u16_samples.as_slice().to_f64()
            }
            SampleFormat::I16 => {
                let i16_samples: Vec<i16> = samples
                    .chunks_exact(2)
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();
                i16_samples.as_slice().to_f64()
            }
            SampleFormat::I24 => {
                // Convert 3-byte samples to i32
                samples
                    .chunks_exact(3)
                    .map(|chunk| {
                        // Sign-extend 24-bit to 32-bit
                        let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]);
                        let sign_extended = if value & 0x00800000 != 0 {
                            value | 0xFF000000u32 as i32
                        } else {
                            value
                        };
                        sign_extended as f64 / (1i32 << 23) as f64
                    })
                    .collect()
            }
            SampleFormat::I32 => {
                let i32_samples: Vec<i32> = samples
                    .chunks_exact(4)
                    .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                i32_samples.as_slice().to_f64()
            }
            SampleFormat::F32 => {
                let f32_samples: Vec<f32> = samples
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                f32_samples.as_slice().to_f64()
            }
            SampleFormat::F64 => samples
                .chunks_exact(8)
                .map(|chunk| {
                    f64::from_le_bytes([
                        chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6],
                        chunk[7],
                    ])
                })
                .collect(),
        };

        // Verify we got the expected number of samples
        if f64_samples.len() != num_samples {
            return Err(crate::Error::AudioFormat(format!(
                "Sample conversion mismatch: expected {} samples, got {}",
                num_samples,
                f64_samples.len()
            )));
        }

        Ok(f64_samples)
    }

    /// Convert f64 samples back to the current format
    pub fn convert_from_f64(&self, samples: &[f64]) -> Vec<u8> {
        SampleFormatConverter::convert_from_f64(samples, self.format.sample_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_to_f64_conversion() {
        let samples: Vec<u8> = vec![0, 128, 255];
        let f64_samples = samples.as_slice().to_f64();

        assert_eq!(f64_samples.len(), 3);
        assert!((f64_samples[0] - (-1.0)).abs() < 0.01); // 0 -> -1.0
        assert!((f64_samples[1] - 0.0).abs() < 0.01); // 128 -> ~0.0
        assert!((f64_samples[2] - 1.0).abs() < 0.01); // 255 -> 1.0
    }

    #[test]
    fn test_i8_to_f64_conversion() {
        let samples: Vec<i8> = vec![-128, 0, 127];
        let f64_samples = samples.as_slice().to_f64();

        assert_eq!(f64_samples.len(), 3);
        assert!((f64_samples[0] - (-1.0)).abs() < 0.01); // -128 -> ~-1.0
        assert_eq!(f64_samples[1], 0.0); // 0 -> 0.0
        assert!((f64_samples[2] - 1.0).abs() < 0.01); // 127 -> 1.0
    }

    #[test]
    fn test_i16_to_f64_conversion() {
        let samples: Vec<i16> = vec![-32768, 0, 32767];
        let f64_samples = samples.as_slice().to_f64();

        assert_eq!(f64_samples.len(), 3);
        assert!((f64_samples[0] - (-1.0)).abs() < 0.01);
        assert_eq!(f64_samples[1], 0.0);
        assert!((f64_samples[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_f32_to_f64_conversion() {
        let samples: Vec<f32> = vec![-1.0, 0.0, 0.5, 1.0];
        let f64_samples = samples.as_slice().to_f64();

        assert_eq!(f64_samples.len(), 4);
        assert_eq!(f64_samples[0], -1.0);
        assert_eq!(f64_samples[1], 0.0);
        assert_eq!(f64_samples[2], 0.5);
        assert_eq!(f64_samples[3], 1.0);
    }

    #[test]
    fn test_f64_to_i16_conversion() {
        let samples: Vec<f64> = vec![-1.0, 0.0, 0.5, 1.0];
        let i16_samples = SampleFormatConverter::f64_to_i16(&samples);

        assert_eq!(i16_samples.len(), 4);
        assert_eq!(i16_samples[0], -32767);
        assert_eq!(i16_samples[1], 0);
        assert!((i16_samples[2] - 16383).abs() <= 1);
        assert_eq!(i16_samples[3], 32767);
    }

    #[test]
    fn test_f64_to_f32_conversion() {
        let samples: Vec<f64> = vec![-1.0, 0.0, 0.5, 1.0];
        let f32_samples = SampleFormatConverter::f64_to_f32(&samples);

        assert_eq!(f32_samples.len(), 4);
        assert_eq!(f32_samples[0], -1.0);
        assert_eq!(f32_samples[1], 0.0);
        assert_eq!(f32_samples[2], 0.5);
        assert_eq!(f32_samples[3], 1.0);
    }

    #[test]
    fn test_roundtrip_i16() {
        let original: Vec<i16> = vec![-32768, -16384, 0, 16384, 32767];
        let f64_samples = original.as_slice().to_f64();
        let converted = SampleFormatConverter::f64_to_i16(&f64_samples);

        assert_eq!(converted.len(), original.len());
        for (orig, conv) in original.iter().zip(converted.iter()) {
            assert!((orig - conv).abs() <= 1); // Allow for rounding error
        }
    }

    #[test]
    fn test_audio_processor_i16() {
        let format = AudioFormat::new(44100, 2, SampleFormat::I16);
        let processor = AudioProcessor::new(format);

        // Create some i16 samples as bytes
        let i16_samples: Vec<i16> = vec![-32768, 0, 32767];
        let bytes: Vec<u8> = i16_samples.iter().flat_map(|&s| s.to_le_bytes()).collect();

        // Convert to f64
        let f64_samples = processor.convert_to_f64(&bytes).unwrap();
        assert_eq!(f64_samples.len(), 3);
        assert!((f64_samples[0] - (-1.0)).abs() < 0.01);
        assert_eq!(f64_samples[1], 0.0);
        assert!((f64_samples[2] - 1.0).abs() < 0.01);

        // Convert back
        let converted_bytes = processor.convert_from_f64(&f64_samples);
        assert_eq!(converted_bytes.len(), bytes.len());
    }

    #[test]
    fn test_audio_processor_f32() {
        let format = AudioFormat::new(48000, 2, SampleFormat::F32);
        let processor = AudioProcessor::new(format);

        // Create some f32 samples as bytes
        let f32_samples: Vec<f32> = vec![-1.0, 0.0, 0.5, 1.0];
        let bytes: Vec<u8> = f32_samples.iter().flat_map(|&s| s.to_le_bytes()).collect();

        // Convert to f64
        let f64_samples = processor.convert_to_f64(&bytes).unwrap();
        assert_eq!(f64_samples.len(), 4);
        assert_eq!(f64_samples[0], -1.0);
        assert_eq!(f64_samples[1], 0.0);
        assert_eq!(f64_samples[2], 0.5);
        assert_eq!(f64_samples[3], 1.0);
    }

    #[test]
    fn test_clamping() {
        // Test that out-of-range values are clamped
        let samples: Vec<f64> = vec![-2.0, -1.5, 1.5, 2.0];
        let i16_samples = SampleFormatConverter::f64_to_i16(&samples);

        assert_eq!(i16_samples[0], -32767); // Clamped to -1.0
        assert_eq!(i16_samples[1], -32767); // Clamped to -1.0
        assert_eq!(i16_samples[2], 32767); // Clamped to 1.0
        assert_eq!(i16_samples[3], 32767); // Clamped to 1.0
    }

    #[test]
    fn test_precision_preservation() {
        // Test that we maintain precision through conversion
        let original: Vec<f64> = vec![0.123456789, -0.987654321, 0.5, -0.25];
        let i32_samples = SampleFormatConverter::f64_to_i32(&original);
        let back_to_f64: Vec<f64> = i32_samples.as_slice().to_f64();

        for (orig, converted) in original.iter().zip(back_to_f64.iter()) {
            assert!((orig - converted).abs() < 1e-9); // Very high precision with i32
        }
    }

    #[test]
    fn test_volume_control() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        // Test initial volume
        assert_eq!(processor.volume(), 1.0);

        // Test setting volume
        processor.set_volume(0.5);
        assert_eq!(processor.volume(), 0.5);

        // Test volume clamping
        processor.set_volume(1.5);
        assert_eq!(processor.volume(), 1.0);

        processor.set_volume(-0.5);
        assert_eq!(processor.volume(), 0.0);
    }

    #[test]
    fn test_apply_volume() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        let mut samples = vec![1.0, -1.0, 0.5, -0.5];
        processor.set_volume(0.5);
        processor.apply_volume(&mut samples);

        assert!((samples[0] - 0.5).abs() < 1e-10);
        assert!((samples[1] - (-0.5)).abs() < 1e-10);
        assert!((samples[2] - 0.25).abs() < 1e-10);
        assert!((samples[3] - (-0.25)).abs() < 1e-10);
    }

    #[test]
    fn test_apply_volume_static() {
        let mut samples = vec![1.0, -1.0, 0.5, -0.5];
        AudioProcessor::apply_volume_static(&mut samples, 0.5);

        assert!((samples[0] - 0.5).abs() < 1e-10);
        assert!((samples[1] - (-0.5)).abs() < 1e-10);
        assert!((samples[2] - 0.25).abs() < 1e-10);
        assert!((samples[3] - (-0.25)).abs() < 1e-10);
    }

    #[test]
    fn test_volume_ramping() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        // Set initial volume
        processor.set_volume(0.0);
        assert_eq!(processor.volume(), 0.0);

        // Set target volume with ramping (10ms ramp)
        processor.set_volume_ramped(1.0, 10.0);

        // Process samples - volume should gradually increase
        let mut samples = vec![1.0; 441]; // 10ms at 44.1kHz = 441 samples
        processor.apply_volume(&mut samples);

        // After processing, volume should be close to target
        assert!((processor.volume() - 1.0).abs() < 0.01);

        // First sample should have lower volume than last
        assert!(samples[0] < samples[samples.len() - 1]);
    }

    #[test]
    fn test_volume_ramping_immediate() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        processor.set_volume(0.0);

        // Zero duration should apply immediately
        processor.set_volume_ramped(1.0, 0.0);
        assert_eq!(processor.volume(), 1.0);
    }

    #[test]
    fn test_volume_ramping_down() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        // Start at full volume
        processor.set_volume(1.0);

        // Ramp down to 0
        processor.set_volume_ramped(0.0, 10.0);

        let mut samples = vec![1.0; 441];
        processor.apply_volume(&mut samples);

        // Volume should be close to 0
        assert!(processor.volume() < 0.01);

        // First sample should have higher volume than last
        assert!(samples[0] > samples[samples.len() - 1]);
    }

    #[test]
    fn test_db_to_linear_conversion() {
        // Test common dB values
        assert!((AudioProcessor::db_to_linear(0.0) - 1.0).abs() < 1e-10); // 0 dB = 1.0
        assert!((AudioProcessor::db_to_linear(-6.0) - 0.501187).abs() < 0.001); // -6 dB ≈ 0.5
        assert!((AudioProcessor::db_to_linear(-20.0) - 0.1).abs() < 0.001); // -20 dB = 0.1
        assert_eq!(AudioProcessor::db_to_linear(-60.0), 0.0); // -60 dB = silence
        assert_eq!(AudioProcessor::db_to_linear(-100.0), 0.0); // Below threshold
    }

    #[test]
    fn test_linear_to_db_conversion() {
        // Test common linear values
        assert!((AudioProcessor::linear_to_db(1.0) - 0.0).abs() < 1e-10); // 1.0 = 0 dB
        assert!((AudioProcessor::linear_to_db(0.5) - (-6.02)).abs() < 0.1); // 0.5 ≈ -6 dB
        assert!((AudioProcessor::linear_to_db(0.1) - (-20.0)).abs() < 0.1); // 0.1 = -20 dB
        assert_eq!(AudioProcessor::linear_to_db(0.0), -60.0); // 0.0 = -60 dB (silence)
    }

    #[test]
    fn test_db_linear_roundtrip() {
        let test_values = vec![0.0, 0.1, 0.25, 0.5, 0.75, 1.0];

        for &linear in &test_values {
            let db = AudioProcessor::linear_to_db(linear);
            let back_to_linear = AudioProcessor::db_to_linear(db);
            assert!((linear - back_to_linear).abs() < 0.001);
        }
    }

    #[test]
    fn test_volume_with_zero_samples() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        processor.set_volume(0.5);
        let mut samples: Vec<f64> = vec![];
        processor.apply_volume(&mut samples);

        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_volume_ramping_precision() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let mut processor = AudioProcessor::new(format);

        processor.set_volume(0.0);
        processor.set_volume_ramped(1.0, 100.0); // 100ms ramp

        // Process in chunks
        for _ in 0..10 {
            let mut samples = vec![1.0; 441]; // 10ms chunks
            processor.apply_volume(&mut samples);
        }

        // After 100ms, should be at target
        assert!((processor.volume() - 1.0).abs() < 0.01);
    }
}
