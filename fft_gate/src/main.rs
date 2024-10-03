use fft_gate::FFTGate;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<FFTGate>();
}