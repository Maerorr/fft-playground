use crate::nih_log;

pub struct CircBuf {
    pub buf: Vec<f32>,
    read_idx: usize,
    pub write_idx: usize,
    pub len: usize,
    // for the same of incrementing read_idx without allocating new temp variable
    read_idx_2: usize,
    has_been_filled_at_least_once: bool,
    
}

impl CircBuf {
    pub fn new(len: usize) -> Self {
        Self {
            buf: vec![0f32; len],
            read_idx: 0,
            read_idx_2: 0,
            write_idx: 0,
            len: len,
            has_been_filled_at_least_once: false,
        }
    }

    pub fn write(&mut self, sample: f32) {
        self.buf[self.write_idx] = sample;
        self.write_idx += 1;
        if self.write_idx == self.len {
            self.write_idx = 0;
            self.has_been_filled_at_least_once = true;
        }
    }

    pub fn was_filled_at_least_once(&self) -> bool {
        self.has_been_filled_at_least_once
    }

    pub fn read(&mut self, x: usize) -> f32 {
        
        self.read_idx_2 = self.read_idx;
        self.read_idx = CircBuf::idx_wrap((self.read_idx + 1) as isize, self.len);
        if (x == 0) {
            //nih_log!("read_idx {}", self.read_idx_2);
        }
        //
        let out = self.buf[self.read_idx_2];
        self.buf[self.read_idx_2] = 0.0f32;
        out
    }

    pub fn get_slice_as_vec(&self) -> Vec<f32> {
        let mut out_slice: Vec<f32> = vec![0f32; self.len];

        // because we're using a ring buffer we need to copy in two parts
        // [....NEW DATA.... | ....... OLD DATA ..........................]
        //                   ^idx
        // into a time-continuous slice
        // [....... OLD DATA ..........................|....NEW DATA....]

        out_slice[0..(self.len - self.write_idx)].copy_from_slice(&self.buf[self.write_idx..self.len]);
        out_slice[(self.len - self.write_idx)..self.len].copy_from_slice(&self.buf[0..self.write_idx]);

        out_slice
    }

    pub fn set_write_idx(&mut self, idx: usize) {
        self.write_idx = CircBuf::idx_wrap(idx as isize, self.len);
    }
    pub fn set_read_idx(&mut self, idx: isize) {
        self.read_idx = CircBuf::idx_wrap(idx as isize, self.len);
    }

    pub fn write_with_overlap(&mut self, data: &Vec<f32>, overlap: isize) {
        for (i, sample) in data.iter().enumerate() {           
            if i < overlap as usize {
                self.buf[CircBuf::idx_wrap(self.write_idx as isize - overlap + i as isize, self.len)] += *sample;
            } else {
                self.write(*sample);
            }
        }
    }

    pub fn idx_wrap(v: isize, len: usize) -> usize {
        (((v + len as isize) + len as isize) % len as isize ) as usize
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
    use std::process::id;

    use crate::{FFT_SIZE, HOP_SIZE};

    use super::*;

    #[test]
    fn get_slice_test() {
        let expected = [3.0, 4.0, 5.0, 6.0, 1.0, 2.0];
        let mut ringbuf = CircBuf::new(6);
        ringbuf.write(9.0);
        ringbuf.write(8.0);
        // from now on this should be the output. in the same order
        ringbuf.write(3.0);
        ringbuf.write(4.0);
        ringbuf.write(5.0);
        ringbuf.write(6.0);
        ringbuf.write(1.0);
        ringbuf.write(2.0);

        let slice = ringbuf.get_slice_as_vec();

        assert_eq!(slice.as_slice(), expected);

        // for (i, j) in (0..(self.input_buffer.len - self.input_buffer.write_idx)).into_iter().zip(self.input_buffer.write_idx..self.input_buffer.len) {
        //     self.fft_in[i] = self.input_buffer.buf[j];
        // }

        // for (i, j) in ((self.input_buffer.len - self.input_buffer.write_idx)..self.input_buffer.len).into_iter().zip(0..self.input_buffer.write_idx) {
        //     self.fft_in[i] = self.input_buffer.buf[j];
        // }

        let mut test = vec![0.0f32;6];
        for (i,  j) in (0..(ringbuf.len - ringbuf.write_idx)).into_iter().zip(ringbuf.write_idx..ringbuf.len) {
            test[i] = ringbuf.buf[j];
        }
        for (i,  j) in ((ringbuf.len - ringbuf.write_idx)..(ringbuf.len)).into_iter().zip(0..ringbuf.write_idx) {
            test[i] = ringbuf.buf[j];
        }

        assert_eq!(test.as_slice(), expected);
    }

    #[test]
    fn read_test() {
        let mut ringbuf = CircBuf::new(6);
        ringbuf.write(3.0);
        ringbuf.write(4.0);
        ringbuf.write(5.0);
        ringbuf.write(6.0);
        ringbuf.write(1.0);
        ringbuf.write(2.0);
        // assert_eq!(ringbuf.read(), 3.0);
        // assert_eq!(ringbuf.read(), 4.0);
        // assert_eq!(ringbuf.read(), 5.0);
    }

    #[test]
    fn filled_test() {
        let mut ringbuf = CircBuf::new(6);
        ringbuf.write(9.0);
        ringbuf.write(8.0);
        ringbuf.write(3.0);
        ringbuf.write(4.0);
        ringbuf.write(5.0);
        assert_eq!(ringbuf.has_been_filled_at_least_once, false);
        ringbuf.write(6.0);
        assert_eq!(ringbuf.has_been_filled_at_least_once, true);
        ringbuf.write(6.0);
        assert_eq!(ringbuf.has_been_filled_at_least_once, true);
    }

    #[test]
    fn to_idx_test() {
        let mut idx = -1;
        let len = 5;
        let expected = 4;

        assert_eq!(len + idx, expected);
    }

    #[test]
    fn write_with_overlap_test() {
        let mut ringbuf = CircBuf::new(10);
        for _ in 0..10 {
            ringbuf.write(1.0);
        }
        assert_eq!(ringbuf.write_idx, 0);

        ringbuf.write_with_overlap(&vec![2.0f32; 5], 2);

        assert_eq!(ringbuf.buf[7], 1f32);
        assert_eq!(ringbuf.buf[9], 3f32);
        assert_eq!(ringbuf.buf[8], 3f32);

        assert_eq!(ringbuf.buf[0], 2f32);
        assert_eq!(ringbuf.buf[1], 2f32);
        assert_eq!(ringbuf.buf[2], 2f32);
        assert_eq!(ringbuf.buf[3], 1f32);
    }

    #[test] 
    fn overlap_test() {
        let mut ringbuf = CircBuf::new(FFT_SIZE);
        ringbuf.set_write_idx(HOP_SIZE);
        for _ in 0..FFT_SIZE {
            ringbuf.write(1.0f32);
        }

        ringbuf.write_with_overlap(&vec![2.0f32; HOP_SIZE * 2], HOP_SIZE as isize);
        assert_eq!(ringbuf.buf[ringbuf.buf.len() - 1], 1.0f32);
        assert_eq!(ringbuf.buf[0], 3.0f32);
    }
}