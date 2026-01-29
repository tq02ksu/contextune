//! Lock-free ring buffer implementation
//!
//! Provides zero-copy audio data flow between decoder and output

use crate::audio::format::AudioFormat;
use std::sync::Arc;

/// Audio buffer for storing decoded audio data
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Audio data stored as 64-bit floats for maximum precision
    data: Arc<Vec<f64>>,
    /// Audio format specification
    format: AudioFormat,
    /// Number of frames (samples per channel)
    frames: usize,
}

impl AudioBuffer {
    /// Create a new audio buffer
    pub fn new(format: AudioFormat, frames: usize) -> Self {
        let total_samples = frames * format.channels as usize;
        Self {
            data: Arc::new(vec![0.0; total_samples]),
            format,
            frames,
        }
    }
    
    /// Create a buffer with existing data
    pub fn with_data(format: AudioFormat, data: Vec<f64>) -> Self {
        let frames = data.len() / format.channels as usize;
        Self {
            data: Arc::new(data),
            format,
            frames,
        }
    }
    
    /// Get the audio format
    pub fn format(&self) -> &AudioFormat {
        &self.format
    }
    
    /// Get the number of frames
    pub fn frames(&self) -> usize {
        self.frames
    }
    
    /// Get the total number of samples
    pub fn samples(&self) -> usize {
        self.data.len()
    }
    
    /// Get the duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        self.frames as f64 / self.format.sample_rate as f64
    }
    
    /// Get a reference to the audio data
    pub fn data(&self) -> &[f64] {
        &self.data
    }
    
    /// Get audio data for a specific channel
    pub fn channel_data(&self, channel: usize) -> Option<Vec<f64>> {
        if channel >= self.format.channels as usize {
            return None;
        }
        
        let mut channel_data = Vec::with_capacity(self.frames);
        for frame in 0..self.frames {
            let sample_index = frame * self.format.channels as usize + channel;
            if sample_index < self.data.len() {
                channel_data.push(self.data[sample_index]);
            }
        }
        
        Some(channel_data)
    }
    
    /// Convert to i16 samples
    pub fn to_i16_samples(&self) -> Vec<i16> {
        self.data.iter().map(|&sample| {
            (sample.clamp(-1.0, 1.0) * i16::MAX as f64) as i16
        }).collect()
    }
    
    /// Convert to i32 samples
    pub fn to_i32_samples(&self) -> Vec<i32> {
        self.data.iter().map(|&sample| {
            (sample.clamp(-1.0, 1.0) * i32::MAX as f64) as i32
        }).collect()
    }
    
    /// Convert to f32 samples
    pub fn to_f32_samples(&self) -> Vec<f32> {
        self.data.iter().map(|&sample| sample as f32).collect()
    }
    
    /// Convert to f64 samples (clone)
    pub fn to_f64_samples(&self) -> Vec<f64> {
        self.data.as_ref().clone()
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Get a slice of frames
    pub fn slice(&self, start_frame: usize, frame_count: usize) -> Option<AudioBuffer> {
        if start_frame >= self.frames || start_frame + frame_count > self.frames {
            return None;
        }
        
        let start_sample = start_frame * self.format.channels as usize;
        let sample_count = frame_count * self.format.channels as usize;
        let end_sample = start_sample + sample_count;
        
        if end_sample <= self.data.len() {
            let slice_data = self.data[start_sample..end_sample].to_vec();
            Some(AudioBuffer::with_data(self.format.clone(), slice_data))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::format::SampleFormat;

    #[test]
    fn test_buffer_creation() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let buffer = AudioBuffer::new(format.clone(), 1024);
        
        assert_eq!(buffer.format(), &format);
        assert_eq!(buffer.frames(), 1024);
        assert_eq!(buffer.samples(), 2048); // 1024 frames * 2 channels
    }
    
    #[test]
    fn test_buffer_with_data() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let data = vec![0.1, 0.2, 0.3, 0.4]; // 2 frames, 2 channels
        let buffer = AudioBuffer::with_data(format, data.clone());
        
        assert_eq!(buffer.frames(), 2);
        assert_eq!(buffer.samples(), 4);
        assert_eq!(buffer.data(), &data);
    }
    
    #[test]
    fn test_sample_format_conversion() {
        let format = AudioFormat::new(44100, 1, SampleFormat::F32);
        let data = vec![0.5, -0.5, 1.0, -1.0];
        let buffer = AudioBuffer::with_data(format, data);
        
        // Convert to i16
        let i16_samples = buffer.to_i16_samples();
        assert_eq!(i16_samples.len(), 4);
        assert_eq!(i16_samples[0], (0.5 * i16::MAX as f64) as i16);
        
        // Convert to f32
        let f32_samples = buffer.to_f32_samples();
        assert_eq!(f32_samples.len(), 4);
        assert!((f32_samples[0] - 0.5).abs() < 0.001);
    }
}