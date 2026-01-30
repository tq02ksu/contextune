//! Lock-free ring buffer for audio data
//!
//! Provides zero-copy audio data flow between decoder and output with minimal latency

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crate::audio::format::AudioFormat;

/// Lock-free ring buffer for audio samples
pub struct AudioRingBuffer {
    /// Buffer data stored as f64 samples for maximum precision
    buffer: Vec<f64>,
    /// Buffer capacity in samples
    capacity: usize,
    /// Write position (producer)
    write_pos: AtomicUsize,
    /// Read position (consumer)
    read_pos: AtomicUsize,
    /// Audio format
    format: AudioFormat,
}

/// Producer interface for writing audio data to the ring buffer
pub struct RingBufferProducer {
    /// Shared reference to the ring buffer
    buffer: Arc<AudioRingBuffer>,
}

/// Consumer interface for reading audio data from the ring buffer
pub struct RingBufferConsumer {
    /// Shared reference to the ring buffer
    buffer: Arc<AudioRingBuffer>,
}

/// Configuration for ring buffer creation
#[derive(Debug, Clone)]
pub struct RingBufferConfig {
    /// Buffer size in seconds (recommended: 2-5 seconds)
    pub buffer_duration_seconds: f64,
    /// Audio format
    pub format: AudioFormat,
    /// Whether to allow overwriting old data when buffer is full
    pub allow_overwrite: bool,
}

impl RingBufferConfig {
    /// Create a new ring buffer configuration with validation
    pub fn new(buffer_duration_seconds: f64, format: AudioFormat, allow_overwrite: bool) -> Result<Self, String> {
        if buffer_duration_seconds < 0.1 {
            return Err("Buffer duration must be at least 0.1 seconds".to_string());
        }
        if buffer_duration_seconds > 30.0 {
            return Err("Buffer duration must not exceed 30 seconds".to_string());
        }
        
        Ok(Self {
            buffer_duration_seconds,
            format,
            allow_overwrite,
        })
    }
    
    /// Create a configuration optimized for low latency (0.5-1 second)
    pub fn low_latency(format: AudioFormat) -> Self {
        Self {
            buffer_duration_seconds: 0.5,
            format,
            allow_overwrite: false,
        }
    }
    
    /// Create a configuration optimized for standard playback (2-3 seconds)
    pub fn standard(format: AudioFormat) -> Self {
        Self {
            buffer_duration_seconds: 2.5,
            format,
            allow_overwrite: false,
        }
    }
    
    /// Create a configuration optimized for high latency tolerance (4-5 seconds)
    pub fn high_latency(format: AudioFormat) -> Self {
        Self {
            buffer_duration_seconds: 4.5,
            format,
            allow_overwrite: false,
        }
    }
    
    /// Get the total buffer size in samples
    pub fn total_samples(&self) -> usize {
        (self.buffer_duration_seconds * self.format.sample_rate as f64) as usize
            * self.format.channels as usize
    }
    
    /// Get the buffer size in bytes (assuming f64 samples)
    pub fn buffer_size_bytes(&self) -> usize {
        self.total_samples() * std::mem::size_of::<f64>()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.buffer_duration_seconds < 0.1 {
            return Err("Buffer duration must be at least 0.1 seconds".to_string());
        }
        if self.buffer_duration_seconds > 30.0 {
            return Err("Buffer duration must not exceed 30 seconds".to_string());
        }
        if self.format.sample_rate == 0 {
            return Err("Sample rate must be greater than 0".to_string());
        }
        if self.format.channels == 0 {
            return Err("Channel count must be greater than 0".to_string());
        }
        
        // Check for reasonable memory usage (limit to ~100MB)
        let max_bytes = 100 * 1024 * 1024; // 100MB
        if self.buffer_size_bytes() > max_bytes {
            return Err(format!(
                "Buffer size ({} MB) exceeds maximum allowed (100 MB)",
                self.buffer_size_bytes() / (1024 * 1024)
            ));
        }
        
        Ok(())
    }
}

impl AudioRingBuffer {
    /// Create a new ring buffer with the specified configuration
    pub fn new(config: RingBufferConfig) -> Result<(RingBufferProducer, RingBufferConsumer), String> {
        // Validate configuration
        config.validate()?;
        
        let total_samples = config.total_samples();
        
        let buffer = Arc::new(AudioRingBuffer {
            buffer: vec![0.0; total_samples],
            capacity: total_samples,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            format: config.format,
        });
        
        let producer = RingBufferProducer {
            buffer: buffer.clone(),
        };
        
        let consumer = RingBufferConsumer {
            buffer,
        };
        
        Ok((producer, consumer))
    }
    
    /// Get the buffer capacity in samples
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Get the audio format
    pub fn format(&self) -> &AudioFormat {
        &self.format
    }
    
    /// Get the number of samples available for reading
    pub fn available_read(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        
        if write_pos >= read_pos {
            write_pos - read_pos
        } else {
            self.capacity - read_pos + write_pos
        }
    }
    
    /// Get the number of samples available for writing
    pub fn available_write(&self) -> usize {
        self.capacity - self.available_read() - 1 // Leave one sample gap to distinguish full from empty
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.available_read() == 0
    }
    
    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.available_write() == 0
    }
    
    /// Get buffer utilization as a percentage (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        self.available_read() as f64 / self.capacity as f64
    }
}

impl RingBufferProducer {
    /// Write samples to the ring buffer
    /// Returns the number of samples actually written
    pub fn write(&self, samples: &[f64]) -> usize {
        let available = self.buffer.available_write();
        let to_write = samples.len().min(available);
        
        if to_write == 0 {
            return 0;
        }
        
        let write_pos = self.buffer.write_pos.load(Ordering::Acquire);
        let capacity = self.buffer.capacity;
        
        // Handle wrap-around
        if write_pos + to_write <= capacity {
            // No wrap-around needed
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr() as *mut f64;
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    buffer_ptr.add(write_pos),
                    to_write,
                );
            }
        } else {
            // Need to wrap around
            let first_chunk = capacity - write_pos;
            let second_chunk = to_write - first_chunk;
            
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr() as *mut f64;
                
                // Write first chunk
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    buffer_ptr.add(write_pos),
                    first_chunk,
                );
                
                // Write second chunk at the beginning
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr().add(first_chunk),
                    buffer_ptr,
                    second_chunk,
                );
            }
        }
        
        // Update write position
        let new_write_pos = (write_pos + to_write) % capacity;
        self.buffer.write_pos.store(new_write_pos, Ordering::Release);
        
        to_write
    }
    
    /// Write samples with potential overwrite if buffer is full
    /// Returns the number of samples actually written
    pub fn write_overwrite(&self, samples: &[f64]) -> usize {
        let capacity = self.buffer.capacity;
        let to_write = samples.len().min(capacity - 1); // Leave one sample gap
        
        if to_write == 0 {
            return 0;
        }
        
        let write_pos = self.buffer.write_pos.load(Ordering::Acquire);
        
        // Handle wrap-around
        if write_pos + to_write <= capacity {
            // No wrap-around needed
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr() as *mut f64;
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    buffer_ptr.add(write_pos),
                    to_write,
                );
            }
        } else {
            // Need to wrap around
            let first_chunk = capacity - write_pos;
            let second_chunk = to_write - first_chunk;
            
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr() as *mut f64;
                
                // Write first chunk
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    buffer_ptr.add(write_pos),
                    first_chunk,
                );
                
                // Write second chunk at the beginning
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr().add(first_chunk),
                    buffer_ptr,
                    second_chunk,
                );
            }
        }
        
        // Update write position
        let new_write_pos = (write_pos + to_write) % capacity;
        self.buffer.write_pos.store(new_write_pos, Ordering::Release);
        
        // If we overwrote data, advance read position
        let available_after_write = self.buffer.available_read();
        if available_after_write >= capacity - 1 {
            // We overwrote some data, advance read position
            let read_pos = self.buffer.read_pos.load(Ordering::Acquire);
            let new_read_pos = (new_write_pos + 1) % capacity;
            self.buffer.read_pos.store(new_read_pos, Ordering::Release);
        }
        
        to_write
    }
    
    /// Get the number of samples that can be written without blocking
    pub fn available_write(&self) -> usize {
        self.buffer.available_write()
    }
    
    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.buffer.is_full()
    }
}

impl RingBufferConsumer {
    /// Read samples from the ring buffer
    /// Returns the number of samples actually read
    pub fn read(&self, output: &mut [f64]) -> usize {
        let available = self.buffer.available_read();
        let to_read = output.len().min(available);
        
        if to_read == 0 {
            return 0;
        }
        
        let read_pos = self.buffer.read_pos.load(Ordering::Acquire);
        let capacity = self.buffer.capacity;
        
        // Handle wrap-around
        if read_pos + to_read <= capacity {
            // No wrap-around needed
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr();
                std::ptr::copy_nonoverlapping(
                    buffer_ptr.add(read_pos),
                    output.as_mut_ptr(),
                    to_read,
                );
            }
        } else {
            // Need to wrap around
            let first_chunk = capacity - read_pos;
            let second_chunk = to_read - first_chunk;
            
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr();
                
                // Read first chunk
                std::ptr::copy_nonoverlapping(
                    buffer_ptr.add(read_pos),
                    output.as_mut_ptr(),
                    first_chunk,
                );
                
                // Read second chunk from the beginning
                std::ptr::copy_nonoverlapping(
                    buffer_ptr,
                    output.as_mut_ptr().add(first_chunk),
                    second_chunk,
                );
            }
        }
        
        // Update read position
        let new_read_pos = (read_pos + to_read) % capacity;
        self.buffer.read_pos.store(new_read_pos, Ordering::Release);
        
        to_read
    }
    
    /// Get the buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    
    /// Read samples and fill with silence if not enough data available
    pub fn read_with_silence(&self, output: &mut [f64]) -> usize {
        let read_count = self.read(output);
        
        // Fill remaining with silence
        if read_count < output.len() {
            for sample in &mut output[read_count..] {
                *sample = 0.0;
            }
        }
        
        output.len()
    }
    
    /// Peek at samples without consuming them
    pub fn peek(&self, output: &mut [f64]) -> usize {
        let available = self.buffer.available_read();
        let to_peek = output.len().min(available);
        
        if to_peek == 0 {
            return 0;
        }
        
        let read_pos = self.buffer.read_pos.load(Ordering::Acquire);
        let capacity = self.buffer.capacity;
        
        // Handle wrap-around
        if read_pos + to_peek <= capacity {
            // No wrap-around needed
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr();
                std::ptr::copy_nonoverlapping(
                    buffer_ptr.add(read_pos),
                    output.as_mut_ptr(),
                    to_peek,
                );
            }
        } else {
            // Need to wrap around
            let first_chunk = capacity - read_pos;
            let second_chunk = to_peek - first_chunk;
            
            unsafe {
                let buffer_ptr = self.buffer.buffer.as_ptr();
                
                // Peek first chunk
                std::ptr::copy_nonoverlapping(
                    buffer_ptr.add(read_pos),
                    output.as_mut_ptr(),
                    first_chunk,
                );
                
                // Peek second chunk from the beginning
                std::ptr::copy_nonoverlapping(
                    buffer_ptr,
                    output.as_mut_ptr().add(first_chunk),
                    second_chunk,
                );
            }
        }
        
        to_peek
    }
    
    /// Skip samples without reading them
    pub fn skip(&self, count: usize) -> usize {
        let available = self.buffer.available_read();
        let to_skip = count.min(available);
        
        if to_skip == 0 {
            return 0;
        }
        
        let read_pos = self.buffer.read_pos.load(Ordering::Acquire);
        let new_read_pos = (read_pos + to_skip) % self.buffer.capacity;
        self.buffer.read_pos.store(new_read_pos, Ordering::Release);
        
        to_skip
    }
    
    /// Get the number of samples available for reading
    pub fn available_read(&self) -> usize {
        self.buffer.available_read()
    }
    
    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for RingBufferConfig {
    fn default() -> Self {
        Self::standard(crate::audio::format::AudioFormat::default())
    }
}

// Ensure the ring buffer components are Send + Sync for multi-threading
unsafe impl Send for AudioRingBuffer {}
unsafe impl Sync for AudioRingBuffer {}
unsafe impl Send for RingBufferProducer {}
unsafe impl Sync for RingBufferProducer {}
unsafe impl Send for RingBufferConsumer {}
unsafe impl Sync for RingBufferConsumer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::format::{AudioFormat, SampleFormat};

    #[test]
    fn test_ring_buffer_creation() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 1.0,
            format: format.clone(),
            allow_overwrite: false,
        };
        
        let result = AudioRingBuffer::new(config);
        assert!(result.is_ok());
        
        let (producer, consumer) = result.unwrap();
        
        assert_eq!(producer.available_write(), 44100 * 2 - 1); // -1 for gap
        assert_eq!(consumer.available_read(), 0);
        assert!(consumer.is_empty());
        assert!(!producer.is_full());
    }

    #[test]
    fn test_basic_write_read() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 1.0,
            format,
            allow_overwrite: false,
        };
        
        let (producer, consumer) = AudioRingBuffer::new(config).unwrap();
        
        // Write some samples
        let input_samples = vec![1.0, 2.0, 3.0, 4.0];
        let written = producer.write(&input_samples);
        assert_eq!(written, 4);
        
        // Read the samples back
        let mut output_samples = vec![0.0; 4];
        let read = consumer.read(&mut output_samples);
        assert_eq!(read, 4);
        assert_eq!(output_samples, input_samples);
    }

    #[test]
    fn test_wrap_around() {
        let format = AudioFormat::new(44100, 1, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 0.1, // Small buffer for testing, but above minimum
            format,
            allow_overwrite: false,
        };
        
        let (producer, consumer) = AudioRingBuffer::new(config).unwrap();
        let capacity = producer.buffer.capacity;
        
        // Fill most of the buffer
        let samples1 = vec![1.0; capacity - 10];
        let written1 = producer.write(&samples1);
        assert_eq!(written1, capacity - 10);
        
        // Read some samples to make space
        let mut output = vec![0.0; 5];
        let read1 = consumer.read(&mut output);
        assert_eq!(read1, 5);
        
        // Write more samples that will wrap around
        let samples2 = vec![2.0; 10];
        let written2 = producer.write(&samples2);
        assert_eq!(written2, 10);
        
        // Read remaining samples
        let mut output2 = vec![0.0; capacity - 10];
        let read2 = consumer.read(&mut output2);
        assert_eq!(read2, capacity - 10);
        
        // The first part should be the remaining 1.0 samples (capacity - 10 - 5 = capacity - 15)
        // The last 10 samples should be the 2.0 samples that wrapped around
        let remaining_ones = capacity - 15; // We wrote capacity-10, read 5, so capacity-15 remain
        for i in 0..remaining_ones {
            assert_eq!(output2[i], 1.0);
        }
        for i in remaining_ones..output2.len() {
            assert_eq!(output2[i], 2.0);
        }
    }

    #[test]
    fn test_overwrite_mode() {
        let format = AudioFormat::new(44100, 1, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 0.1, // Small buffer, but above minimum
            format,
            allow_overwrite: true,
        };
        
        let (producer, consumer) = AudioRingBuffer::new(config).unwrap();
        let capacity = producer.buffer.capacity;
        
        // Fill the entire buffer with overwrite
        let samples = vec![1.0; capacity * 2]; // More than capacity
        let written = producer.write_overwrite(&samples);
        assert_eq!(written, capacity - 1); // Should write capacity - 1
        
        // Buffer should be nearly full
        assert!(producer.buffer.available_read() > 0);
    }

    #[test]
    fn test_peek_and_skip() {
        let format = AudioFormat::new(44100, 1, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 1.0,
            format,
            allow_overwrite: false,
        };
        
        let (producer, consumer) = AudioRingBuffer::new(config).unwrap();
        
        // Write some samples
        let input_samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        producer.write(&input_samples);
        
        // Peek at samples
        let mut peek_output = vec![0.0; 3];
        let peeked = consumer.peek(&mut peek_output);
        assert_eq!(peeked, 3);
        assert_eq!(peek_output, vec![1.0, 2.0, 3.0]);
        
        // Available read should not change after peek
        assert_eq!(consumer.available_read(), 5);
        
        // Skip some samples
        let skipped = consumer.skip(2);
        assert_eq!(skipped, 2);
        assert_eq!(consumer.available_read(), 3);
        
        // Read remaining samples
        let mut output = vec![0.0; 3];
        let read = consumer.read(&mut output);
        assert_eq!(read, 3);
        assert_eq!(output, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_read_with_silence() {
        let format = AudioFormat::new(44100, 1, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 1.0,
            format,
            allow_overwrite: false,
        };
        
        let (producer, consumer) = AudioRingBuffer::new(config).unwrap();
        
        // Write fewer samples than we'll try to read
        let input_samples = vec![1.0, 2.0];
        producer.write(&input_samples);
        
        // Try to read more samples than available
        let mut output = vec![99.0; 5]; // Initialize with non-zero values
        let read = consumer.read_with_silence(&mut output);
        assert_eq!(read, 5); // Should always return requested amount
        
        // First two should be the actual data, rest should be silence
        assert_eq!(output[0], 1.0);
        assert_eq!(output[1], 2.0);
        assert_eq!(output[2], 0.0);
        assert_eq!(output[3], 0.0);
        assert_eq!(output[4], 0.0);
    }

    #[test]
    fn test_buffer_utilization() {
        let format = AudioFormat::new(44100, 1, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 1.0,
            format,
            allow_overwrite: false,
        };
        
        let (producer, consumer) = AudioRingBuffer::new(config).unwrap();
        let capacity = producer.buffer.capacity;
        
        // Initially empty
        assert_eq!(producer.buffer.utilization(), 0.0);
        
        // Fill half the buffer
        let samples = vec![1.0; capacity / 2];
        producer.write(&samples);
        
        let utilization = producer.buffer.utilization();
        assert!((utilization - 0.5).abs() < 0.01); // Should be approximately 50%
        
        // Fill completely
        let remaining_samples = vec![1.0; capacity / 2 - 1];
        producer.write(&remaining_samples);
        
        let full_utilization = producer.buffer.utilization();
        assert!(full_utilization > 0.99); // Should be nearly 100%
    }

    #[test]
    fn test_ring_buffer_config_validation() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F64);
        
        // Test valid configuration
        let valid_config = RingBufferConfig::new(2.5, format.clone(), false);
        assert!(valid_config.is_ok());
        
        // Test too short duration
        let short_config = RingBufferConfig::new(0.05, format.clone(), false);
        assert!(short_config.is_err());
        
        // Test too long duration
        let long_config = RingBufferConfig::new(35.0, format.clone(), false);
        assert!(long_config.is_err());
        
        // Test validation method
        let config = RingBufferConfig {
            buffer_duration_seconds: 2.0,
            format: format.clone(),
            allow_overwrite: false,
        };
        assert!(config.validate().is_ok());
        
        let invalid_config = RingBufferConfig {
            buffer_duration_seconds: 0.01, // Too short
            format: format.clone(),
            allow_overwrite: false,
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_ring_buffer_config_presets() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F64);
        
        // Test low latency preset
        let low_latency = RingBufferConfig::low_latency(format.clone());
        assert_eq!(low_latency.buffer_duration_seconds, 0.5);
        assert!(!low_latency.allow_overwrite);
        assert!(low_latency.validate().is_ok());
        
        // Test standard preset
        let standard = RingBufferConfig::standard(format.clone());
        assert_eq!(standard.buffer_duration_seconds, 2.5);
        assert!(!standard.allow_overwrite);
        assert!(standard.validate().is_ok());
        
        // Test high latency preset
        let high_latency = RingBufferConfig::high_latency(format.clone());
        assert_eq!(high_latency.buffer_duration_seconds, 4.5);
        assert!(!high_latency.allow_overwrite);
        assert!(high_latency.validate().is_ok());
        
        // Test default
        let default = RingBufferConfig::default();
        assert_eq!(default.buffer_duration_seconds, 2.5);
        assert!(default.validate().is_ok());
    }

    #[test]
    fn test_ring_buffer_config_calculations() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F64);
        let config = RingBufferConfig {
            buffer_duration_seconds: 2.0,
            format: format.clone(),
            allow_overwrite: false,
        };
        
        // Test total samples calculation
        let expected_samples = (2.0 * 44100.0) as usize * 2; // 2 seconds * sample rate * channels
        assert_eq!(config.total_samples(), expected_samples);
        
        // Test buffer size in bytes
        let expected_bytes = expected_samples * std::mem::size_of::<f64>();
        assert_eq!(config.buffer_size_bytes(), expected_bytes);
        
        // Test that buffer is created with correct size
        let (producer, _consumer) = AudioRingBuffer::new(config).unwrap();
        assert_eq!(producer.buffer.capacity(), expected_samples);
    }

    #[test]
    fn test_ring_buffer_config_memory_limit() {
        let format = AudioFormat::new(192000, 8, SampleFormat::F64); // High sample rate, many channels
        
        // Test that very large buffers are rejected
        let huge_config = RingBufferConfig {
            buffer_duration_seconds: 30.0, // 30 seconds of 192kHz 8-channel audio
            format,
            allow_overwrite: false,
        };
        
        let validation_result = huge_config.validate();
        // This should fail due to memory limit (would be ~350MB)
        assert!(validation_result.is_err());
        
        let creation_result = AudioRingBuffer::new(huge_config);
        assert!(creation_result.is_err());
    }
}