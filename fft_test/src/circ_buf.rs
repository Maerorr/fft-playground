use crate::FFT_SIZE;

pub struct CircBuf {
    buf: Vec<f32>,
    idx: usize,
    len: usize,
    has_been_filled_at_least_once: bool,
}

impl CircBuf {
    pub fn new(len: usize) -> Self {
        Self {
            buf: vec![0f32; len],
            idx: 0,
            len: len,
            has_been_filled_at_least_once: false,
        }
    }

    pub fn add_sample(&mut self, sample: f32) {
        self.buf[self.idx] = sample;
        self.idx += 1;
        
        if self.idx == self.len {
            self.idx = 0;
            
            self.has_been_filled_at_least_once = true;
        }
    }

    pub fn was_filled_at_least_once(&self) -> bool {
        self.has_been_filled_at_least_once
    }

    pub fn get_slice_as_vec(&self) -> Vec<f32> {
        let mut out_slice: Vec<f32> = vec![0f32; self.len];

        // because we're using a ring buffer we need to copy in two parts
        // [....NEW DATA.... | ....... OLD DATA ..........................]
        //                   ^idx
        // into a time-continuous slice
        // [....... OLD DATA ..........................|....NEW DATA....]

        out_slice[0..(self.len - self.idx)].copy_from_slice(&self.buf[self.idx..self.len]);
        out_slice[(self.len - self.idx)..self.len].copy_from_slice(&self.buf[0..self.idx]);

        out_slice
    }

    pub fn to_string(&self) -> String {
        let mut s: String = String::new();
        s += "[";
        for i in self.buf.iter() {
            s += format!("{}, ", i).as_str();
        }
        s += "]";

        s
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_slice_test() {
        let expected = [3.0, 4.0, 5.0, 6.0, 1.0, 2.0];
        let mut ringbuf = CircBuf::new(6);
        ringbuf.add_sample(9.0);
        ringbuf.add_sample(8.0);
        // from now on this should be the output. in the same order
        ringbuf.add_sample(3.0);
        ringbuf.add_sample(4.0);
        ringbuf.add_sample(5.0);
        ringbuf.add_sample(6.0);
        ringbuf.add_sample(1.0);
        ringbuf.add_sample(2.0);

        let slice = ringbuf.get_slice_as_vec();

        assert_eq!(slice.as_slice(), expected);
    }

    #[test]
    fn filled_test() {
        let mut ringbuf = CircBuf::new(6);
        ringbuf.add_sample(9.0);
        ringbuf.add_sample(8.0);
        ringbuf.add_sample(3.0);
        ringbuf.add_sample(4.0);
        ringbuf.add_sample(5.0);
        assert_eq!(ringbuf.has_been_filled_at_least_once, false);
        ringbuf.add_sample(6.0);
        assert_eq!(ringbuf.has_been_filled_at_least_once, true);
        ringbuf.add_sample(6.0);
        assert_eq!(ringbuf.has_been_filled_at_least_once, true);
    }
}