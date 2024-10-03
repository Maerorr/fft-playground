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