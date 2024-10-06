use fft_processor::FFTProcessor;
use hound;
use realfft::num_complex::{Complex, ComplexFloat};
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use std::f32::consts::PI;
use std::{mem, usize};
use std::{fs::File, thread};
use image::{RgbImage, Rgb};
use rand::{thread_rng, Rng, RngCore};
use rand::seq::SliceRandom;

mod fft_processor;
mod circ_buf;
mod utils;
mod colorizer;

const FFT_SIZE: usize = 4096;
const NUM_BINS: usize = FFT_SIZE / 2;
const OVERLAP: usize = 4;
const HOP_SIZE: usize = FFT_SIZE / OVERLAP;
const WINDOW_CORRECTION: f32 = 2.0 / 3.0;

fn main() {


    let wav_path = "res/noisysaw.wav";
    let mut reader = hound::WavReader::open(wav_path).expect("can't open file");
    let spec = reader.spec();
    
    let samples: Vec<f32> =
        reader.samples::<i16>()
            .map(|s| (s.unwrap() as f32) / i16::MAX as f32)
            .collect();

    println!("input sample length: {}", samples.len());

    let mut fft_processor = FFTProcessor::new(spec.sample_rate);

    for sample in samples {
        fft_processor.process_sample(sample);
    }

    let audio = fft_processor.get_audio();

    println!("output sample length: {}", audio.len());

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: spec.sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("output.wav", spec).expect("can't make a wav writer");

    for sample in audio {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16).unwrap();
    }
    writer.finalize().unwrap();

    // for (i, input_block) in fft_processor.audio_in.iter().enumerate() {
    //     let filename = format!("{}{}.wav", "res/inbins/block", i);

    //     let spec = hound::WavSpec {
    //         channels: 1,
    //         sample_rate: spec.sample_rate as u32,
    //         bits_per_sample: 16,
    //         sample_format: hound::SampleFormat::Int,
    //     };
    //     let mut writer = hound::WavWriter::create(filename, spec).expect("can't make a wav writer");

    //     for sample in input_block {
    //         let sample_i16 = (sample * i16::MAX as f32) as i16;
    //         writer.write_sample(sample_i16).unwrap();
    //     }
    //     writer.finalize().unwrap();
    // }

    // save to audio
    
}
