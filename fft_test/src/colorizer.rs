// C C# D D# E F F# G G# A A# B
const NOTES_FREQ: [[f32; 9]; 12] = [
        [16.35, 32.70, 65.41, 130.81, 261.63, 523.25, 1046.50, 2093.00, 4186.01],
        [17.32, 34.65, 69.30, 138.59, 277.18, 554.37, 1108.73, 2217.46, 4434.92],
        [18.35, 36.71, 73.42, 146.83, 293.66, 587.33, 1174.66, 2349.32, 4698.64],
        [19.45, 38.89, 77.78, 155.56, 311.13, 622.25, 1244.51, 2489.02, 4978.03],
        [20.60, 41.20, 82.41, 164.81, 329.63, 659.25, 1318.51, 2637.02, 5274.04],
        [21.83, 43.65, 87.31, 174.61, 349.23, 698.46, 1396.91, 2793.83, 5587.65],
        [23.12, 46.25, 92.50, 185.00, 369.99, 739.99, 1479.98, 2959.96, 5919.91],
        [24.50, 49.00, 98.00, 196.00, 392.00, 783.99, 1567.98, 3135.96, 6271.93],
        [25.96, 51.91, 103.83, 207.65, 415.30, 830.61, 1661.22, 3322.44, 6644.88],
        [27.50, 55.00, 110.00, 220.00, 440.00, 880.00, 1760.00, 3520.00, 7040.00],
        [29.14, 58.27, 116.54, 233.08, 466.16, 932.33, 1864.66, 3729.31, 7458.62],
        [30.87, 61.74, 123.47, 246.94, 493.88, 987.77, 1975.53, 3951.07, 7902.13]
    ];

pub struct Colorizer {
    notes: [bool; 12],
    low_freq_cutoff: f32,
    high_freq_cutoff: f32,
    dry_gain: f32,
    bin_width: f32,
}

impl Colorizer {
    pub fn new(notes: [bool; 12], lfc: f32, hfc: f32, dry: f32, bin_width: f32) -> Self {
        // for note in notes.iter() {
        //     print!("{} ", note);
        // }
        Self {
            notes: notes,
            low_freq_cutoff: lfc,
            high_freq_cutoff: hfc,
            dry_gain: dry,
            bin_width: bin_width,
        }
    }

    fn is_freq_in_notes(&self, freq: f32) -> bool {
        let mut flag = false;
        
        for (note_num, used_note) in self.notes.iter().enumerate() {
            if *used_note {
                for note_freq in NOTES_FREQ[note_num] {
                    if (note_freq - freq).abs() < 2f32 * self.bin_width {
                        flag = true;
                        return flag;
                    }
                }
            }
        }

        flag
    }

    pub fn process_spectrum(&self, spectrum: &Vec<f32>, frequencies: &Vec<f32>, db: &Vec<f32>) -> Vec<f32> {
        let mut output: Vec<f32> = vec![0f32; spectrum.len()];

        for (i, (mag, freq)) in spectrum.iter().zip(frequencies).enumerate() {

            if i == 0 || i == spectrum.len() - 1 {
                output[i] = *mag;
                continue;
            }

            if  *freq > self.high_freq_cutoff {
                output[i] = *mag * 0.5;
                continue;
            }

            if *freq < self.low_freq_cutoff {
                output[i] = *mag * 0.45;
                continue;
            }

            if self.is_freq_in_notes(*freq) {
                output[i] = *mag;
            } else {
                output[i] = *mag * self.dry_gain;
            }


            // if i == 0 || i == spectrum.len() - 1 || *freq < self.low_freq_cutoff || *freq > self.high_freq_cutoff {
            //     output[i] = *mag;
            //     continue;
            // }
            // // if (freq - 440.0).abs() < self.bin_width {
            // //     println!("{}Hz value: {}, db: {}", freq, mag, db[i]);
            // // }
            
            // if self.is_freq_in_notes(*freq) {
            //     output[i] = *mag;
            // } else {
            //     output[i] = *mag * self.dry_gain;
            // }
        }

        output
    }
}