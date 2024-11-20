use std::f32::consts::PI;

pub const MINUS_INF_DB: f32 = -100f32;
pub const MINUS_INF_GAIN: f32 = 1e-5;

// https://www.musicdsp.org/en/latest/Filters/257-1-pole-lpf-for-smooth-parameter-changes.html
pub struct SimpleLPF {
    pub a: f32,
    b: f32,
    z: f32,
}

impl SimpleLPF {
    pub fn new(a: f32) -> Self {
        Self {
            a,
            b: 1.0f32 - a,
            z: 0.0f32
        }
    }

    pub fn set_a(&mut self, a: f32) {
        self.a = a;
        self.b = 1.0 - a;
    }

    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        self.z = (sample * self.b) + (self.z * self.a);
        self.z
    }
}

pub fn multiply_vectors(a: &Vec<f32>, b: &Vec<f32>) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).collect::<Vec<f32>>()
}

pub fn multiply_vectors_in_place(a: &mut Vec<f32>, b: &Vec<f32>) {
    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x *= *y;
    }
}

#[inline]
pub fn gain_to_db(x: f32) -> f32 {
    f32::max(x, MINUS_INF_GAIN).log10() * 20.0
}

#[inline]
pub fn db_to_gain(x: f32) -> f32 {
    if x > MINUS_INF_DB {
        10.0f32.powf(x * 0.05f32)
    } else {
        0.0
    }
}

#[inline]
pub fn fft_size_to_bins(size: usize) -> usize {
    (size / 2) + 1
} 

#[inline]
pub fn gauss(x: f32, sig: f32) -> f32 {
    let sig_clamped = sig.max(0.001f32);
    (1.0 / (sig_clamped * (2.0 * PI).sqrt())) *
    (-0.5 * (x * x)/(sig_clamped * sig_clamped)).exp()
}

#[inline]
pub fn calculate_peakness(x: f32, p: f32, one_over_p: f32) -> f32 {
    //x.clamp(0.0, 1.0)//.powi(2)
    (1.0f32 - (1.0f32 - x.clamp(0.0, 1.0)).powf(p)).powf(one_over_p)
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0f32 - t) * a + t * b
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

    #[test]
    fn gauss_test() {
        let expected = 1.9947f32;
        let val = gauss(0.0, 0.2f32);
        print!("expected: {}, value: {}", expected, val);
        assert!((val - expected).abs() < 0.001);
    }
}