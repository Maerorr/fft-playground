pub fn multiply_vectors(a: &Vec<f32>, b: &Vec<f32>) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).collect::<Vec<f32>>()
}

pub fn multiply_vectors_in_place(a: &mut Vec<f32>, b: &Vec<f32>) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x *= *y;
    }
}

pub fn f32_to_db(x: f32) -> f32 {
    20.0 * x.log10()
}

#[inline]
pub fn fft_size_to_bins(size: usize) -> usize {
    (size / 2) + 1
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_mult_test() {
        let expected = vec![1.0, 4.0, 6.0, 50.0];
        let a = vec![1.0, 2.0, 3.0, 5.0];
        let b = vec![1.0, 2.0, 2.0, 10.0];

        let mult = multiply_vectors(&a, &b);

        assert_eq!(expected, mult);
    }

    #[test]
    fn multiply_vectors_in_place_test() {
        let expected = vec![1.0, 4.0, 6.0, 50.0];
        let mut a = vec![1.0, 2.0, 3.0, 5.0];
        let b = vec![1.0, 2.0, 2.0, 10.0];

        multiply_vectors_in_place(&mut a, &b);

        assert_eq!(expected, a);
    }
}