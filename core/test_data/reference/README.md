# Reference Audio Test Files

This directory contains reference audio test files used for audio quality testing and verification.

## File Organization

Files are organized by sample rate and bit depth:
- `44100Hz_16bit/` - CD quality
- `48000Hz_16bit/` - Professional audio standard
- `96000Hz_16bit/` - High-resolution audio
- `192000Hz_16bit/` - Ultra high-resolution audio
- `44100Hz_24bit/` - CD sample rate with 24-bit depth
- `48000Hz_24bit/` - Professional audio with 24-bit depth
- `96000Hz_24bit/` - High-resolution with 24-bit depth
- `192000Hz_24bit/` - Ultra high-resolution with 24-bit depth
- `special/` - Special test signals

## Test Signals

### Sine Waves
Pure sine waves at standard test frequencies:
- `sine_100Hz.wav` - Low frequency test
- `sine_440Hz.wav` - A4 reference pitch
- `sine_1kHz.wav` - Standard reference frequency
- `sine_5kHz.wav` - Mid-high frequency test
- `sine_10kHz.wav` - High frequency test

### Noise Signals
- `white_noise.wav` - White noise for noise floor testing

### Impulse Signals
- `impulse.wav` - Single sample impulse for transient response testing

### Silence
- `silence.wav` - Digital silence for noise floor verification

### Special Test Files
- `special/full_scale_1kHz.wav` - Full-scale sine wave for clipping tests
- `special/short_impulse.wav` - Very short impulse for transient response
- `special/sweep_20Hz_20kHz.wav` - Logarithmic frequency sweep for frequency response analysis

## Generating Test Files

To regenerate these test files, run:

```bash
cargo run --example generate_test_audio
```

Or use the Python script:

```bash
python3 scripts/generate_test_audio.py
```

## Usage in Tests

These files are used in:
- Bit-perfect playback verification
- Frequency response analysis
- THD+N (Total Harmonic Distortion + Noise) measurements
- Audio checksum validation
- Format conversion accuracy tests

## File Specifications

All files are:
- Mono (1 channel)
- WAV format (uncompressed PCM or float)
- 5 seconds duration (except special files)
- Amplitude: 0.8 (80% of full scale) for sine waves, except full-scale test
- Amplitude: 0.5 (50% of full scale) for white noise

## Notes

- These files should NOT be committed to version control (they are in .gitignore)
- Generate them locally as needed for testing
- Total size: approximately 500MB when all files are generated
