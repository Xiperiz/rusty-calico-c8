use sdl2::audio::AudioCallback;

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = match self.phase {
                x if x > 0.0 && x < 0.5 => self.volume,
                _ => -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

impl SquareWave {
    pub fn new(phase_inc: f32, phase: f32, volume: f32) -> SquareWave {
        SquareWave {
            phase_inc,
            phase,
            volume
        }
    }
}
