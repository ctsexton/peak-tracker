#[cfg(test)]
mod test {
    use realfft::num_complex::ComplexFloat;
    use realfft::RealFftPlanner;
    #[test]
    fn test_fft() {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(4);
        let mut output = fft.make_output_vec();
        let mut input = vec![1.0, 0.0, -0.0, 0.0];
        fft.process(&mut input, &mut output);
        println!("Raw FFT: {:?}", output);
        println!(
            "Mag FFT: {:?}",
            output.iter().map(|c| c.norm()).collect::<Vec<f32>>()
        );
        println!(
            "Phase FFT: {:?}",
            output.iter().map(|c| c.arg()).collect::<Vec<f32>>()
        );
    }
}
