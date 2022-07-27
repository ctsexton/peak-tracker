struct Peak {
    frequency: f32,
    amplitude: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FrequencyDistance {
    a: usize,      // index of peak in array A
    b: usize,      // index of peak in array B
    distance: f32, // distance in frequency
}

const NUM_PEAKS: usize = 20;

/// Finds the frequency distance between all peaks in a and all peaks in b
fn calculate_peak_distances(a: &[Peak], b: &[Peak], output: &mut [Option<FrequencyDistance>]) {
    assert_eq!(a.len() * b.len(), output.len());

    for (index_a, item_a) in a.iter().enumerate() {
        for (index_b, item_b) in b.iter().enumerate() {
            let distance = (item_a.frequency - item_b.frequency).abs();
            let index = index_a * a.len() + index_b;
            output[index] = Some(FrequencyDistance {
                a: index_a,
                b: index_b,
                distance,
            });
        }
    }
}

const MAX_PEAKS: usize = 20;
const MAX_DISTANCE: f32 = 187.5;

fn match_closest_peaks<const NPEAKS: usize>(
    a: &[Peak; NPEAKS],
    b: &[Peak; NPEAKS],
) -> [Option<(usize, usize)>; NPEAKS]
where
    [(); NPEAKS * NPEAKS]: Sized,
{
    let mut distances: [Option<FrequencyDistance>; NPEAKS * NPEAKS] = [None; NPEAKS * NPEAKS];
    calculate_peak_distances(a, b, &mut distances);
    distances.sort_unstable_by(|a, b| {
        a.unwrap()
            .distance
            .partial_cmp(&b.unwrap().distance)
            .unwrap()
    });
    let mut matches: [Option<(usize, usize)>; NPEAKS] = [None; NPEAKS];
    let mut taken_from_a = [false; NPEAKS];
    let mut taken_from_b = [false; NPEAKS];
    let mut match_index = 0;
    for item in distances.iter() {
        let item = item.unwrap();
        if match_index >= matches.len() || item.distance > MAX_DISTANCE {
            break;
        }
        if !taken_from_a[item.a] && !taken_from_b[item.b] {
            taken_from_a[item.a] = true;
            taken_from_b[item.b] = true;
            matches[match_index] = Some((item.a, item.b));
            match_index += 1;
        }
    }
    matches
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_peak_distances() {
        let a = [
            Peak {
                frequency: 20.0,
                amplitude: 1.0,
            },
            Peak {
                frequency: 30.0,
                amplitude: 1.0,
            },
        ];
        let b = [
            Peak {
                frequency: 21.0,
                amplitude: 1.0,
            },
            Peak {
                frequency: 25.0,
                amplitude: 1.0,
            },
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

    #[test]
    fn sort_peak_distances() {
        let mut distances = [(0, 1, 5.0), (0, 0, 1.0), (1, 0, 9.0), (1, 1, 5.0)];
        distances.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        let expected_sorting = [(0, 0, 1.0), (0, 1, 5.0), (1, 1, 5.0), (1, 0, 9.0)];
        assert_eq!(distances, expected_sorting);
    }
}
