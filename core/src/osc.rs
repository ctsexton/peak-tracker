use std::f32::consts::PI;

#[derive(Debug)]
pub struct SinOsc {
    frequency: f32, // radians per sample
    amplitude: f32,
    phase: f32,
    lowpass_amp: f32, // multiplier to prevent aliasing
}

fn get_lowpass_amp(hz: f32) -> f32 {
    if hz > 18000.0 {
        (-0.00025 * hz + 5.5).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

impl SinOsc {
    pub fn new(frequency: f32, amplitude: f32, phase: f32) -> Self {
        Self {
            frequency,
            amplitude,
            phase,
            lowpass_amp: 1.0,
        }
    }

    pub fn set_frequency_hz(&mut self, hz: f32, sample_rate: f32) {
        let hz = hz.clamp(20.0, sample_rate * 0.5);
        self.frequency = 2.0 * PI * hz / sample_rate;
        self.lowpass_amp = get_lowpass_amp(hz);
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f32 {
        let phase = self.phase;
        self.phase = (self.phase + self.frequency) % (2.0 * PI);
        f32::sin(phase) * self.amplitude * self.lowpass_amp
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
        assert!((osc.next() - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
        assert!((osc.next() - 1.0).abs() < 0.0001);
        assert!((osc.next() - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
        assert!((osc.next() - 0.0).abs() < 0.0001);
        assert!((osc.next() - -std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);
        assert!((osc.next() - -1.0).abs() < 0.0001);

        osc.set_frequency_hz(20.0, 48000.0);
        assert!((osc.frequency - 0.002618).abs() < 0.00001);
        osc.set_frequency_hz(24000.0, 48000.0);
        assert!((osc.frequency - PI).abs() < 0.00001);
        osc.set_frequency_hz(12000.0, 48000.0);
        assert!((osc.frequency - PI / 2.0).abs() < 0.00001);
    }
}
