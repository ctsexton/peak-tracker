use std::f32::consts::PI;

/// Returns the size of each bin when dividing the frequency range from
/// 0 to the nyquist frequency by num_bins;
/// ```
/// # use core::utils::frequency_per_bin;
/// assert_eq!(frequency_per_bin(48000.0, 10), 2400.0);
/// ```
pub fn frequency_per_bin(sample_rate: f32, num_bins: usize) -> f32 {
    let nyquist = 0.5 * sample_rate;
    nyquist / num_bins as f32
}

/// Returns the largest unsigned integer x such that 2**x <= n
/// ```
/// # use core::utils::ilog2;
/// assert_eq!(ilog2(6), Some(2));
/// ```
pub fn ilog2(n: usize) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let mut result = 0;
    let mut n = n;
    while n > 0 {
        n >>= 1;
        result += 1;
    }
    Some(result - 1)
}

/// Rounds n down to the next "floor" that is a power of 2.
/// ```
/// # use core::utils::floor_to_power_of_2;
/// assert_eq!(floor_to_power_of_2(31), Some(16));
/// assert_eq!(floor_to_power_of_2(33), Some(32));
/// ```
pub fn floor_to_power_of_2(n: usize) -> Option<usize> {
    Some(1 << ilog2(n)?)
}

pub fn build_sample(partials: &[(f32, f32, f32)], size: usize, sample_rate: f32) -> Vec<f32> {
    let mut sample = vec![0_f32; size];
    for (frequency, amplitude, phase) in partials {
        for (index, x) in sample.iter_mut().enumerate() {
            *x += amplitude
                    * (phase * 2. * PI + index as f32 * 2. * PI * frequency / sample_rate).sin()
        }
    }
    sample
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_frequency_per_bin() {
        let fperbin = frequency_per_bin(48000.0, 10);
        assert_eq!(fperbin, 2400.0);
    }

    #[test]
    fn test_ilog2() {
        assert_eq!(ilog2(0), None);
        assert_eq!(ilog2(1), Some(0));
        assert_eq!(ilog2(3), Some(1));
        assert_eq!(ilog2(6), Some(2));
        assert_eq!(ilog2(8), Some(3));
        assert_eq!(ilog2(17), Some(4));
    }

    #[test]
    fn test_floor_to_power_of_2() {
        assert_eq!(floor_to_power_of_2(0), None);
        assert_eq!(floor_to_power_of_2(1), Some(1));
        assert_eq!(floor_to_power_of_2(3), Some(2));
        assert_eq!(floor_to_power_of_2(1023), Some(512));
        assert_eq!(floor_to_power_of_2(1025), Some(1024));
    }

    #[test]
    fn test_build_sample() {
        let partials = [(2.0, 1.0, 0.0)];
        let sample = build_sample(&partials, 8, 8.0);
        let expected_sample = [0.0, 1.0, 0.0, -1.0];
        for (item, expected) in sample.iter().zip(expected_sample.iter()) {
            let diff = (*item - *expected).abs();
            assert!(diff < 0.001);
        }
    }
}
