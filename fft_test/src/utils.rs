use crate::FFT_SIZE;

pub fn multiply_vectors(a: &Vec<f32>, b: &Vec<f32>) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).collect::<Vec<f32>>()
}

pub fn f32_to_db(x: f32) -> f32 {
    20.0 * x.log10()
}

pub fn f32_to_normalized_db(x: f32) -> f32 {
    let db = f32_to_db(x);
    //dbNormalized = db - 20 * log10(fftLength * pow(2,N)/2)
    db - 20.0 * (FFT_SIZE as f32 * 2.0f32.powi(16) / 2.0).log10()
}

#[cfg(test)]
mod tests {
    use super::multiply_vectors;

    #[test]
    fn vector_mult_test() {
        let expected = vec![1.0, 4.0, 6.0, 50.0];
        let a = vec![1.0, 2.0, 3.0, 5.0];
        let b = vec![1.0, 2.0, 2.0, 10.0];

        let mult = multiply_vectors(&a, &b);

        assert_eq!(expected, mult);
    }
}