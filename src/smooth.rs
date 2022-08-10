#[derive(Debug)]
pub struct SmoothedValue {
    value: f32,
    target: f32,
    remaining_steps_to_target: usize,
    smooth_length: usize,
}

impl SmoothedValue {
    pub fn new(initial: f32, smooth_length: usize) -> Self {
        Self {
            value: initial,
            target: initial,
            remaining_steps_to_target: 0,
            smooth_length,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
        self.remaining_steps_to_target = self.smooth_length;
    }

    pub fn next(&mut self) -> f32 {
        if self.remaining_steps_to_target <= 0 {
            return self.value;
        }
        let step = (self.target - self.value) / self.remaining_steps_to_target as f32;
        self.value = self.value + step;
        self.remaining_steps_to_target -= 1;
        return self.value;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_smoothed_value() {
        let mut smoothed_value = SmoothedValue::new(0.0, 4);
        assert!((smoothed_value.next() - 0.0).abs() < f32::EPSILON);
        smoothed_value.set_target(1.0);
        assert!((smoothed_value.next() - 0.25).abs() < f32::EPSILON);
        assert!((smoothed_value.next() - 0.5).abs() < f32::EPSILON);
        assert!((smoothed_value.next() - 0.75).abs() < f32::EPSILON);
        assert!((smoothed_value.next() - 1.0).abs() < f32::EPSILON);
    }
}
