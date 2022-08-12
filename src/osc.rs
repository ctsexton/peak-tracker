use std::f32::consts::PI;

#[derive(Debug)]
pub struct SinOsc {
    frequency: f32, // radians per sample
    amplitude: f32,
    phase: f32,
}

impl SinOsc {
    pub fn new(frequency: f32, amplitude: f32, phase: f32) -> Self {
        Self {
            frequency,
            amplitude,
            phase,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn set_frequency_hz(&mut self, hz: f32, sample_rate: f32) {
        self.frequency = 2.0 * PI * hz / sample_rate;
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    pub fn next(&mut self) -> f32 {
        let phase = self.phase;
        self.phase = (self.phase + self.frequency) % (2.0 * PI);
        f32::sin(phase) * self.amplitude
    }

    pub fn current_value(&self) -> f32 {
        f32::sin(self.phase) * self.amplitude
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_osc() {
        let mut osc = SinOsc::new(0.25 * PI, 1.0, 0.0);
        assert!(osc.next().abs() < 0.0001);
        assert!((osc.next() - 0.7071).abs() < 0.0001);
        assert!((osc.next() - 1.0).abs() < 0.0001);
        assert!((osc.next() - 0.7071).abs() < 0.0001);
        assert!((osc.next() - 0.0).abs() < 0.0001);
        assert!((osc.next() - -0.7071).abs() < 0.0001);
        assert!((osc.next() - -1.0).abs() < 0.0001);

        osc.set_frequency_hz(1.0, 48000.0);
        assert!((osc.frequency - 0.000130).abs() < 0.00001);
        osc.set_frequency_hz(24000.0, 48000.0);
        assert!((osc.frequency - PI).abs() < 0.00001);
        osc.set_frequency_hz(12000.0, 48000.0);
        assert!((osc.frequency - PI / 2.0).abs() < 0.00001);
    }
}
