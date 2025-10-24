use hound::{WavSpec, WavWriter};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;

const SAMPLE_RATE: u32 = 44100;
const AMPLITUDE: i16 = 16383;

pub struct AudioEngine {
    sink: Sink,
    _stream: OutputStream,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        Ok(AudioEngine {
            sink,
            _stream,
        })
    }

    pub fn play_sound(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;
        self.sink.append(source);
        Ok(())
    }

    pub fn stop(&self) {
        self.sink.stop();
    }
}

// Generate 8-bit style background music
pub fn generate_background_music() -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("../../assets/audio/syn_bg_music.wav", spec)?;
    
    // Create a simple 8-bit style melody
    let melody_notes = [
        (440.0, 0.5), // A4
        (523.25, 0.5), // C5
        (659.25, 0.5), // E5
        (523.25, 0.5), // C5
        (440.0, 0.5), // A4
        (392.0, 0.5), // G4
        (440.0, 1.0), // A4
    ];

    // Generate the melody with 8-bit style square waves
    for (frequency, duration) in melody_notes.iter() {
        let samples = (SAMPLE_RATE as f32 * duration) as usize;
        for i in 0..samples {
            let t = i as f32 / SAMPLE_RATE as f32;
            let sample = if (t * frequency * 2.0 * std::f32::consts::PI).sin() > 0.0 {
                AMPLITUDE
            } else {
                -AMPLITUDE
            };
            writer.write_sample(sample)?;
        }
    }

    writer.finalize()?;
    Ok(())
}

// Generate sound effects
pub fn generate_sound_effects() -> Result<(), Box<dyn std::error::Error>> {
    // Create audio directory
    std::fs::create_dir_all("../../assets/audio")?;

    // Console message sound (short beep)
    generate_beep("../../assets/audio/console_beep.wav", 800.0, 0.1)?;
    
    // Alert sound (higher pitch beep)
    generate_beep("../../assets/audio/alert.wav", 1200.0, 0.2)?;
    
    // Victory sound (ascending notes)
    generate_victory_sound("../../assets/audio/victory.wav")?;
    
    // Rocket launch sound (descending tone)
    generate_rocket_sound("../../assets/audio/rocket.wav")?;

    Ok(())
}

fn generate_beep(file_path: &str, frequency: f32, duration: f32) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(file_path, spec)?;
    let samples = (SAMPLE_RATE as f32 * duration) as usize;
    
    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin() * AMPLITUDE as f32;
        writer.write_sample(sample as i16)?;
    }

    writer.finalize()?;
    Ok(())
}

fn generate_victory_sound(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(file_path, spec)?;
    
    // Victory fanfare: ascending notes
    let notes = [523.25, 659.25, 783.99, 1046.5]; // C5, E5, G5, C6
    let note_duration = 0.3;
    
    for frequency in notes.iter() {
        let samples = (SAMPLE_RATE as f32 * note_duration) as usize;
        for i in 0..samples {
            let t = i as f32 / SAMPLE_RATE as f32;
            let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin() * AMPLITUDE as f32;
            writer.write_sample(sample as i16)?;
        }
    }

    writer.finalize()?;
    Ok(())
}

fn generate_rocket_sound(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(file_path, spec)?;
    
    // Rocket launch: descending tone with noise
    let duration = 0.8;
    let samples = (SAMPLE_RATE as f32 * duration) as usize;
    
    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = t / duration;
        
        // Descending frequency from 800Hz to 200Hz
        let frequency = 800.0 - (600.0 * progress);
        
        // Add some noise for rocket effect
        let noise = (rand::random::<f32>() - 0.5) * 0.1;
        let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin() * (1.0 + noise) * AMPLITUDE as f32;
        writer.write_sample(sample as i16)?;
    }

    writer.finalize()?;
    Ok(())
}
