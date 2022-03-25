use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::AudioSubsystem;
use nfd::{Response,Result as NFDResult};

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = self.volume;

            if self.phase > 0.5 {
                *x = -*x;
            }

            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub fn create_audio_device(audio_subsystem: &AudioSubsystem) -> Result<AudioDevice<SquareWave>, String> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        }
    })
}

pub fn prompt_file() -> NFDResult<Option<String>> {
    let result = nfd::open_file_dialog(None, None)?;

    Ok(match result {
        Response::Okay(filename) => Some(filename),
        Response::OkayMultiple(files) => Some(files[0].clone()),
        Response::Cancel => None,
    })
}