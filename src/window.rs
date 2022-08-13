use dasp::frame::Mono;
use dasp::signal::window::{hanning};


pub fn apply_hanning_window<const SIZE: usize>(frame: &[f32; SIZE]) -> [f32; SIZE] {
    let window = hanning::<Mono<f32>>(SIZE);
    let mut output = [0_f32; SIZE];
    for ((index, x), [w]) in frame.iter().enumerate().zip(window) {
        output[index] = *x * w;
    }
    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_apply_hanning() {
        let frame = [2_f32; 5];
        let windowed = apply_hanning_window::<5>(&frame);
        let expected = [0.0, 1.0, 2.0, 1.0, 0.0];
        for (expected, actual) in windowed.iter().zip(expected.iter()) {
            assert!((expected - actual).abs() < f32::EPSILON);
        }
    }
}
