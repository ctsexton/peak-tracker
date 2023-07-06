use crate::peak::Peak;
use assert_no_alloc::assert_no_alloc;

#[derive(Clone, Copy, Debug, PartialEq)]
struct FrequencyDistance {
    a: usize,      // index of peak in array A
    b: usize,      // index of peak in array B
    distance: f32, // distance in frequency
}

/// Finds the frequency distance between all peaks in a and all peaks in b
fn calculate_peak_distances(
    a: &[Option<Peak>],
    b: &[Option<Peak>],
    output: &mut [Option<FrequencyDistance>],
) {
    assert_eq!(a.len() * b.len(), output.len());

    for (index_a, item_a) in a.iter().enumerate() {
        for (index_b, item_b) in b.iter().enumerate() {
            let index = index_a * a.len() + index_b;
            if let (Some(item_a), Some(item_b)) = (item_a, item_b) {
                let distance = (item_a.frequency - item_b.frequency).abs();
                output[index] = Some(FrequencyDistance {
                    a: index_a,
                    b: index_b,
                    distance,
                });
            } else {
                output[index] = None;
            }
        }
    }
}

const MAX_PEAKS: usize = 20;
const MAX_DISTANCE: f32 = 187.5;

fn match_closest_peaks<const NPEAKS: usize>(
    a: &[Option<Peak>; NPEAKS],
    b: &[Option<Peak>; NPEAKS],
) -> [Option<usize>; NPEAKS]
where
    [(); NPEAKS * NPEAKS]: Sized,
{
    let mut distances: [Option<FrequencyDistance>; NPEAKS * NPEAKS] = [None; NPEAKS * NPEAKS];
    calculate_peak_distances(a, b, &mut distances);
    distances.sort_unstable_by(|a, b| {
        if let (Some(a), Some(b)) = (a, b) {
            return a.distance.partial_cmp(&b.distance).unwrap();
        } else if let Some(_a) = a {
            return std::cmp::Ordering::Less;
        } else {
            return std::cmp::Ordering::Greater;
        }
    });
    let mut matches: [Option<usize>; NPEAKS] = [None; NPEAKS];
    let mut taken_from_a = [false; NPEAKS];
    let mut taken_from_b = [false; NPEAKS];
    let mut match_index = 0;
    for item in distances.iter() {
        if let Some(item) = item {
            if match_index >= matches.len() || item.distance > MAX_DISTANCE {
                break;
            }
            if !taken_from_a[item.a] && !taken_from_b[item.b] {
                taken_from_a[item.a] = true;
                taken_from_b[item.b] = true;
                matches[item.a] = Some(item.b);
                match_index += 1;
            }
        }
    }
    matches
}

pub struct PeakTracker {
    peaks: [Option<Peak>; MAX_PEAKS],
}

impl PeakTracker {
    pub fn new() -> Self {
        let peaks = [None; MAX_PEAKS];
        Self { peaks }
    }

    pub fn update_peaks(&mut self, mut batch: [Option<Peak>; MAX_PEAKS]) {
        assert_no_alloc(|| {
            let matches = match_closest_peaks(&self.peaks, &batch);
            let mut new_peaks: [Option<Peak>; MAX_PEAKS] = [None; MAX_PEAKS];
            for (index, item) in matches.iter().enumerate() {
                if let Some(target) = *item {
                    new_peaks[index] = batch[target].take();
                }
            }
            let mut unmapped_peaks = batch.iter_mut().flatten();
            for new_peak in new_peaks.iter_mut().filter(|p| p.is_none()) {
                if let Some(peak) = unmapped_peaks.next() {
                    *new_peak = Some(*peak);
                } else {
                    break;
                }
            }
            self.peaks = new_peaks;
        })
    }

    pub fn latest(&self) -> &[Option<Peak>; MAX_PEAKS] {
        &self.peaks
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_peak_distances() {
        let a = [
            Some(Peak {
                frequency: 20.0,
                amplitude: 1.0,
            }),
            Some(Peak {
                frequency: 30.0,
                amplitude: 1.0,
            }),
        ];
        let b = [
            Some(Peak {
                frequency: 21.0,
                amplitude: 1.0,
            }),
            Some(Peak {
                frequency: 25.0,
                amplitude: 1.0,
            }),
        ];
        let mut result: [Option<FrequencyDistance>; 4] = [None; 4];
        calculate_peak_distances(&a, &b, &mut result);
        let expected_result: Vec<Option<FrequencyDistance>> =
            [(0, 0, 1.0), (0, 1, 5.0), (1, 0, 9.0), (1, 1, 5.0)]
                .into_iter()
                .map(|(a, b, distance)| Some(FrequencyDistance { a, b, distance }))
                .collect();
        assert_eq!(result.as_slice(), expected_result.as_slice());
    }
}
