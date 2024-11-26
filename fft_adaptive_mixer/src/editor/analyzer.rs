use std::sync::{Arc, Mutex};

use nih_plug::{nih_debug_assert, nih_log, prelude::AtomicF32};
use nih_plug_vizia::vizia::{image::Pixel, prelude::*, vg};
use std::sync::atomic::Ordering;
use crate::analyzer_data::{self, AnalyzerData};

use super::EQ_FREQS;

const MIN_F: f32 = 20.0f32;
const MAX_F: f32 = 20_000.0f32;

const MIN_F_LN: f32 = 2.995732; // ln(20.0)
const MAX_F_LN: f32 = 9.9034f32;// ln(20_000.0)

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
    where
        LAnalyzerData: Lens<Target = Arc<Mutex<triple_buffer::Output<AnalyzerData>>>>,
        LRate: Lens<Target = Arc<AtomicF32>>,
    {
        Self {
            analyzer_data: analyzer_data.get(cx),
            sample_rate: sample_rate.get(cx),
        }
        .build(cx, |_cx| ())
    }
}

impl View for Analyzer {
    fn element(&self) -> Option<&'static str> {
        Some("Analyzer")
    }

    fn draw(
        &self,
        cx: &mut nih_plug_vizia::vizia::context::DrawContext,
        canvas: &mut nih_plug_vizia::vizia::view::Canvas,
    ) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let mut analyzer_data = self.analyzer_data.lock().unwrap();
        let analyzer_data = analyzer_data.read();
        let sr = self.sample_rate.load(Ordering::Relaxed);
        let nyquist = sr / 2.0;
        draw_spectrum_guides(cx, canvas, analyzer_data);
        draw_spectrum(cx, canvas, analyzer_data, nyquist, sr);
        draw_reduction(cx, canvas, analyzer_data, nyquist, sr);
        draw_cutoffs(cx, canvas, analyzer_data, nyquist, sr);
        draw_eq(cx, canvas, analyzer_data);

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

        let paint = vg::Paint::color(border_color)
        .with_line_width(border_width);
        canvas.stroke_path(&path, &paint);
    }
}

#[inline]
fn db_to_height(db_value: f32) -> f32 {
    ((db_value + 80.0) / 100.0).clamp(0.0f32, 1.0f32)
}

#[inline]
fn eq_db_to_height(db_value: f32) -> f32 {
    (db_value / 30.0).clamp(-1.0f32, 1.0f32)
}

#[inline]
fn reduction_db_to_height(db_value: f32) -> f32 {
    ((db_value) / 80.0).clamp(0.0f32, 1.0f32)
}

#[inline]
fn freq_to_x(f: f32) -> f32 {
    //((f.clamp(MIN_F, MAX_F).ln() - MIN_F_LN) / (MAX_F_LN - MIN_F_LN)).clamp(0.0f32, 1.0f32)
    //((f / REF_FREQ).ln() / (MAX_F / REF_FREQ).ln()).clamp(0.0f32, 1.0f32)
    ((f.clamp(MIN_F, MAX_F).ln() - MIN_F_LN) / (MAX_F_LN - MIN_F_LN)).clamp(0.0f32, 1.0f32)
}

fn draw_spectrum(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    analyzer_data: &AnalyzerData,
    nyquist_hz: f32,
    sr: f32,
) {
    let bounds = cx.bounds();
    let border_width = cx.border_width();

    // let mut shape_path = vg::Path::new();
    // shape_path.move_to(bounds.x + (bounds.w * freq_to_x(200.0)), bounds.y + (bounds.h * (1.0 - 0.8)));
    // shape_path.line_to(bounds.x + (bounds.w * freq_to_x(2000.0)), bounds.y + (bounds.h * (1.0 - 0.8)));
    // shape_path.line_to(bounds.x + (bounds.w * freq_to_x(2000.0)), bounds.y + (bounds.h * (1.0 - 0.3)));
    // //shape_path.line_to(bounds.x + (bounds.w * freq_to_x(200.0)), bounds.y + (bounds.h * (1.0 - 0.3)));
    // //shape_path.line_to(bounds.x + (bounds.w * freq_to_x(200.0)), bounds.y + (bounds.h * (1.0 - 0.8)));
    // shape_path.close();
    // let bars_paint = vg::Paint::color(vg::Color::rgb(230, 230, 250)).with_line_width(1.0);
    // canvas.fill_path(&shape_path, &bars_paint);
    //let mut outline_path = vg::Path::new();
    let mut bars_path = vg::Path::new();
    bars_path.move_to(bounds.x + border_width / 2f32, bounds.y + bounds.h * 0.5);
    //outline_path.move_to(bounds.x + border_width / 2f32, bounds.y + bounds.h * 0.5);

    for (magnitude, f) in analyzer_data
        .magnitudes
        .iter().zip(analyzer_data.frequencies.iter())
        .take(analyzer_data.num_bins - 1)
        .skip(1)
    {
        let x = freq_to_x(*f);
        
        let physical_x_coord = bounds.x + (bounds.w * x).clamp(border_width, bounds.w - border_width);
        
        let height = db_to_height(*magnitude);

        bars_path.line_to(physical_x_coord, bounds.y + (bounds.h * (0.5 - height / 2.0)));
        //outline_path.line_to(physical_x_coord, bounds.y + (bounds.h * (0.5 - height / 2.0)));
        //bars_path.move_to(physical_x_coord, bounds.y + (bounds.h * (1.0 - height)));
    }

    bars_path.line_to(bounds.x + bounds.w, bounds.y + bounds.h * 0.5);
    bars_path.close();

    let bars_paint = vg::Paint::color(vg::Color::rgb(199, 207, 221)).with_line_width(0.0);
    canvas.fill_path(&bars_path, &bars_paint);

    //let outline_paint = vg::Paint::color(vg::Color::rgb(230, 50, 253)).with_line_width(2.0);
    //canvas.stroke_path(&outline_path, &outline_paint);
}

pub fn draw_spectrum_guides(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    analyzer_data: &AnalyzerData,
) {
    let bounds = cx.bounds();
    let border_width = cx.border_width();

    for i in 1..20 {
        let mut bars_path = vg::Path::new();
        let x = freq_to_x(i as f32 * 1000.0);
        bars_path.move_to(bounds.x + (bounds.w * x), bounds.y + (bounds.h));
        bars_path.line_to(bounds.x + (bounds.w * x), bounds.y);
        let bars_paint = vg::Paint::color(vg::Color::rgb(70, 70, 70)).with_line_width(1.0);
        canvas.stroke_path(&bars_path, &bars_paint);
    }
}

pub fn draw_reduction(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    analyzer_data: &AnalyzerData,
    nyquist_hz: f32,
    sr: f32,
) {
let bounds = cx.bounds();
    let border_width = cx.border_width();

    // let mut shape_path = vg::Path::new();
    // shape_path.move_to(bounds.x + (bounds.w * freq_to_x(200.0)), bounds.y + (bounds.h * (1.0 - 0.8)));
    // shape_path.line_to(bounds.x + (bounds.w * freq_to_x(2000.0)), bounds.y + (bounds.h * (1.0 - 0.8)));
    // shape_path.line_to(bounds.x + (bounds.w * freq_to_x(2000.0)), bounds.y + (bounds.h * (1.0 - 0.3)));
    // //shape_path.line_to(bounds.x + (bounds.w * freq_to_x(200.0)), bounds.y + (bounds.h * (1.0 - 0.3)));
    // //shape_path.line_to(bounds.x + (bounds.w * freq_to_x(200.0)), bounds.y + (bounds.h * (1.0 - 0.8)));
    // shape_path.close();
    // let bars_paint = vg::Paint::color(vg::Color::rgb(230, 230, 250)).with_line_width(1.0);
    // canvas.fill_path(&shape_path, &bars_paint);

    let mut bars_path = vg::Path::new();
    bars_path.move_to(bounds.x + border_width / 2f32, bounds.y + bounds.h * 0.5);

    for (reduction, f) in analyzer_data
        .reduction
        .iter().zip(analyzer_data.frequencies.iter())
        .take(analyzer_data.num_bins - 1)
        .skip(1)
    {
        let x = freq_to_x(*f);
        
        let physical_x_coord = bounds.x + (bounds.w * x).clamp(border_width, bounds.w - border_width);

        //let height = reduction_db_to_height(*reduction);
        let height = reduction_db_to_height(*reduction);

        bars_path.line_to(physical_x_coord, bounds.y + (bounds.h * (0.5 + height / 2.0)));
        //bars_path.move_to(physical_x_coord, bounds.y + (bounds.h * (1.0 - height)));
    }

    bars_path.line_to(bounds.x + bounds.w, bounds.y + bounds.h * 0.5);
    bars_path.close();

    let bars_paint = vg::Paint::color(vg::Color::rgb(122, 9, 250)).with_line_width(0.0);
    canvas.fill_path(&bars_path, &bars_paint);
}

pub fn draw_cutoffs(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    analyzer_data: &AnalyzerData,
    nyquist_hz: f32,
    sr: f32,
) {
    let bounds = cx.bounds();
    let border_width = cx.border_width();
    
    let mut bars_path = vg::Path::new();
    //nih_log!("{} + ({} * {} + {})", bounds.x, bounds.w, freq_to_x(analyzer_data.lowcut), border_width);
    //nih_log!("low {}, high {}, low_x {}, high_x {}", analyzer_data.lowcut, analyzer_data.highcut, freq_to_x(analyzer_data.lowcut), freq_to_x(analyzer_data.highcut));
    let lowcut_x =
        (bounds.x + (bounds.w * freq_to_x(analyzer_data.lowcut)) + border_width) * 0.99f32;
    bars_path.move_to(lowcut_x, bounds.y + (bounds.h));
    bars_path.line_to(lowcut_x, bounds.y);
    let bars_paint = vg::Paint::color(vg::Color::rgb(219, 63, 253)).with_line_width(1.0);
    canvas.stroke_path(&bars_path, &bars_paint);

    let mut bars_path = vg::Path::new();
    let highcut_x =
        (bounds.x + (bounds.w * freq_to_x(analyzer_data.highcut)) + border_width) * 0.99f32;
    bars_path.move_to(highcut_x, bounds.y + (bounds.h));
    bars_path.line_to(highcut_x, bounds.y);
    let bars_paint = vg::Paint::color(vg::Color::rgb(219, 63, 253)).with_line_width(1.0);
    canvas.stroke_path(&bars_path, &bars_paint);
}

pub fn draw_eq(
    cx: &mut DrawContext,
    canvas: &mut Canvas,
    analyzer_data: &AnalyzerData,
) {
    let bounds = cx.bounds();
    let border_width = cx.border_width();

    let mut path = vg::Path::new();
    path.move_to(bounds.x + border_width / 2.0, bounds.y + bounds.h / 2.0);

    for (i, gain) in analyzer_data.eq.iter().enumerate() {
        path.line_to(
            bounds.x + border_width + (bounds.w * freq_to_x(EQ_FREQS[i])), 
            bounds.y + (bounds.h * (0.5 - eq_db_to_height(*gain) / 2.0))
        );
        path.line_to(
            bounds.x + border_width + (bounds.w * freq_to_x(EQ_FREQS[i + 1])), 
            bounds.y + (bounds.h * (0.5 - eq_db_to_height(*gain) / 2.0))
        );
        path.line_to(
            bounds.x + border_width + (bounds.w * freq_to_x(EQ_FREQS[i + 1])), 
            bounds.y + (bounds.h * (0.5 - eq_db_to_height(*gain) / 2.0))
        );
        
    }
    let bars_paint = vg::Paint::color(vg::Color::rgb(153, 230, 95)).with_line_width(1.0);
    canvas.stroke_path(&path, &bars_paint);
}