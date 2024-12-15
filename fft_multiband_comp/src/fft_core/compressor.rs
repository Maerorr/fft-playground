use crate::utils;
use nih_plug::nih_log;

#[derive(Clone, Copy)]
pub struct Compressor {
    pub th: f32,
    pub r: f32,
    pub w: f32,
    pub att: f32,
    pub rel: f32,

    env: f32,
    reduced: f32,
}

impl Compressor {
    pub fn new(th: f32, r: f32, w: f32, att_coeff: f32, rel_coeff: f32) -> Self {
        Self {
            th,
            r,
            w,
            att: att_coeff,
            rel: rel_coeff,
            env: -100.0, // working with db so -100dB = 0.0 gain
            reduced: 0.0,
        }
    }

    pub fn set_params(&mut self, th: f32, r: f32, w: f32, att_coeff: f32, rel_coeff: f32) {
        self.th = th;
        self.r = r;
        self.w = w;
        self.att = att_coeff;
        self.rel = rel_coeff;
    }

    // returns by how much to CHANGE the signal. Not the final output value
    pub fn process_db(&mut self, x_db: f32) -> f32 {        
        // first, update envelope follower
        if x_db > self.env {
            self.env = self.att * (self.env - x_db) + x_db;
        } else {
            self.env = self.rel * (self.env - x_db) + x_db;
        }

        // compressor stage
        if 2.0 * (self.env - self.th) < -self.w {
            self.reduced = self.env;
        } else if 2.0 * (self.env - self.th).abs() <= self.w {
            self.reduced = self.env + ((1.0 / self.r - 1.0) * (self.env - self.th + self.w / 2.0).powi(2)) / (2.0 * self.w);
        } else {
            self.reduced = self.th + (self.env - self.th) / self.r;
        }

        // input * reduction, but in db
        // x_db + (output - self.env)

        // delta
        self.reduced - self.env
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let x_db = utils::gain_to_db(x.abs());
        // first, update envelope follower
        if x_db > self.env {
            self.env = self.att * (self.env - x_db) + x_db;
        } else {
            self.env = self.rel * (self.env - x_db) + x_db;
        }

        // compressor stage
        if 2.0 * (self.env - self.th) < -self.w {
            self.reduced = self.env;
        } else if 2.0 * (self.env - self.th).abs() <= self.w {
            self.reduced = self.env + ((1.0 / self.r - 1.0) * (self.env - self.th + self.w / 2.0).powi(2)) / (2.0 * self.w);
        } else {
            self.reduced = self.th + (self.env - self.th) / self.r;
        }

        let out_val = x * utils::db_to_gain(self.reduced - self.env);
        out_val
    }
}