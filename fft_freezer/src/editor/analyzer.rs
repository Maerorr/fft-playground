use std::{sync::{Arc, Mutex}};

use nih_plug::{nih_debug_assert, nih_log, prelude::AtomicF32};
use nih_plug_vizia::vizia::{image::Pixel, prelude::*, vg};
use std::sync::atomic::Ordering;

use crate::analyzer_data::{self, AnalyzerData};

const LN_FREQ_RANGE_START_HZ: f32 = 3.4011974; // 30.0f32.ln();
const LN_FREQ_RANGE_END_HZ: f32 = 10.05; // 22_000.0f32.ln();
const LN_FREQ_RANGE: f32 = LN_FREQ_RANGE_END_HZ - LN_FREQ_RANGE_START_HZ;

#[derive(Debug, Clone)]
pub struct Analyzer {
    analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    sample_rate: Arc<AtomicF32>,
}

impl Analyzer {
    pub fn new<LAnalyzerData, LRate>(
        cx: &mut Context,
        analyzer_data: LAnalyzerData,
        sample_rate: LRate,
    ) -> Handle<Self>
    where LAnalyzerData: Lens<Target = Arc<Mutex<triple_buffer::Output<AnalyzerData>>>>,
    LRate: Lens<Target = Arc<AtomicF32>>,
    {
        Self {
            analyzer_data: analyzer_data.get(cx),
            sample_rate: sample_rate.get(cx),
        }.build(
            cx,
            |_cx| (),
        )
    }
}

impl View for Analyzer {
    fn element(&self) -> Option<&'static str> {
        Some("Analyzer")
    }

    fn draw(&self, cx: &mut nih_plug_vizia::vizia::context::DrawContext, canvas: &mut nih_plug_vizia::vizia::view::Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let mut analyzer_data = self.analyzer_data.lock().unwrap();
        let analyzer_data = analyzer_data.read();
        let nyquist = self.sample_rate.load(Ordering::Relaxed) / 2.0;

        draw_spectrum(cx, canvas, analyzer_data, nyquist);

        // draw border
        let border_width = cx.border_width();
        let border_color: vg::Color = cx.border_color().into();

        let mut path = vg::Path::new();
        {
            let x = bounds.x + border_width / 2.0;
            let y = bounds.y + border_width / 2.0;
            let w = bounds.w - border_width;
            let h = bounds.h - border_width;
            path.move_to(x, y);
            path.line_to(x, y + h);
            path.line_to(x + w, y + h);
            path.line_to(x + w, y);
            path.close();
        }

        let paint = vg::Paint::color(border_color).with_line_width(border_width);
        canvas.stroke_path(&path, &paint);
    }
}

#[inline]
fn db_to_unclamped_height(db_value: f32) -> f32 {
    (db_value + 80.0) / 100.0
}

fn draw_spectrum(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    analyzer_data: &AnalyzerData,
    nyquist_hz: f32,
) {
    let bounds = cx.bounds();

    let bin_frequency = |bin_idx: f32| (bin_idx / analyzer_data.num_bins as f32) * nyquist_hz;
    // A `[0, 1]` value indicating at which relative x-coordinate a bin should be drawn at
    let bin_x =
        |bin_idx: f32| (bin_frequency(bin_idx).ln() - LN_FREQ_RANGE_START_HZ) / LN_FREQ_RANGE;

    let magnitude_height = |magnitude: f32| {
        //nih_debug_assert!(magnitude >= 0.0, "mag: {}", magnitude);
        //let magnitude_db = nih_plug::util::gain_to_db(magnitude);
        db_to_unclamped_height(magnitude).clamp(0.0, 1.0)
    };

    let mut bars_path = vg::Path::new();
    bars_path.move_to(bounds.x, bounds.y + bounds.h);
    
    //nih_log!("{}", analyzer_data.num_bins);
    //nih_log!("x for 0: {}Hz -> {}, x for 512: {}Hz -> {}", bin_frequency(0.0), bin_x(bin_frequency(0.0)), bin_frequency(512.0), bin_x(bin_frequency(512.0)));

    for (bin_idx, magnitude) in analyzer_data
        .magnitudes
        .iter()
        .enumerate()
        .take(analyzer_data.num_bins - 1)
        .skip(1)
    {
        let x = bin_x(bin_idx as f32);
        
        let physical_x_coord = bounds.x + (bounds.w * x);

        // Scale this so that 1.0/0 dBFS magnitude is at 80% of the height, the bars begin
        // at -80 dBFS, and that the scaling is linear. This is the same scaling used in
        // Diopser's spectrum analyzer.
        let height = magnitude_height(*magnitude);

        bars_path.line_to(physical_x_coord, bounds.y + (bounds.h * (1.0 - height)));
        bars_path.move_to(physical_x_coord, bounds.y + (bounds.h * (1.0 - height)));
    }
    // for i in 0..5 {
        
    //     let physical_x_coord = bounds.x + (bounds.w * (i as f32 / 5.0));
    //     if i == 0 {
    //         bars_path.move_to(physical_x_coord, bounds.y + bounds.h);
    //     }
    //     bars_path.line_to(physical_x_coord, bounds.y + (bounds.h * 0.5));
    //     bars_path.move_to(physical_x_coord, bounds.y + (bounds.h * 0.5));
    // }
    let bars_paint = vg::Paint::color(vg::Color::rgb(25, 25, 25)).with_line_width(1.0);
    canvas.stroke_path(&bars_path, &bars_paint);

}