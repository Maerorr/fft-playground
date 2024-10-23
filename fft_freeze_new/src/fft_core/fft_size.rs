use nih_plug::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FFTSize {
    _512 = 512,
    _1024 = 1024,
    _2048 = 2048,
    _4096 = 4096,
}

impl Enum for FFTSize {
    fn variants() -> &'static [&'static str] {
        &["512", "1024", "2048", "4096"]
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&["512", "1024", "2048", "4096"])
    }

    fn to_index(self) -> usize {
        match self {
            FFTSize::_512 => 0,
            FFTSize::_1024 => 1,
            FFTSize::_2048 => 2,
            FFTSize::_4096 => 3,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => FFTSize::_512,
            1 => FFTSize::_1024,
            2 => FFTSize::_2048,
            3 => FFTSize::_4096,
            _ => panic!("Invalid index!"),
        }
    }
}

impl FFTSize {
    pub fn num_bins(&self) -> usize {
        *self as usize / 2 + 1
    }
}