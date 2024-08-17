use std::sync::Arc;

use vizia::prelude::*;
use vizia::vg;

use crate::app_data::AppEvent;
use crate::app_data::UnitsMode;
use crate::app_data::ZoomMode;
use crate::waveform::to_f32;
use crate::waveform::Waveform;
use crate::waveform::SAMPLES_PER_PIXEL;

pub struct Waveview<
    L1: Lens<Target = Arc<Waveform>>,
    L2: Lens<Target = usize>,
    L3: Lens<Target = usize>,
> {
    waveform_lens: L1,
    zoom_level_lens: L2,
    start_lens: L3,
    units_mode: UnitsMode,
}

impl<L1, L2, L3> Waveview<L1, L2, L3>
where
    L1: Lens<Target = Arc<Waveform>>,
    L2: Lens<Target = usize>,
    L3: Lens<Target = usize>,
{
    pub fn new(
        cx: &mut Context,
        waveform_lens: L1,
        zoom_level_lens: L2,
        start_lens: L3,
    ) -> Handle<Self> {
        Self { waveform_lens, zoom_level_lens, start_lens, units_mode: UnitsMode::Decibel }
            .build(cx, |cx| {})
            .bind(waveform_lens, |mut handle, _| handle.needs_redraw())
    }
}

impl<L1, L2, L3> View for Waveview<L1, L2, L3>
where
    L1: Lens<Target = Arc<Waveform>>,
    L2: Lens<Target = usize>,
    L3: Lens<Target = usize>,
{
    fn element(&self) -> Option<&'static str> {
        Some("waveview")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _| match window_event {
            WindowEvent::MouseScroll(x, y) => {
                // println!("scroll {} {}", x, y);
                // if *y > 0.0 {
                //     if cx.modifiers().contains(Modifiers::CTRL) {
                //         // Zoom In
                //         cx.emit(AppEvent::ZoomIn);
                //     } else {
                //     }
                // }
                // cx.emit(AppEvent::Pan(*x));
            }

            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        if let Some(waveform) = self.waveform_lens.get_ref(cx) {
            let bounds = cx.bounds();

            let mut path1 = vg::Path::new();
            let mut path2 = vg::Path::new();

            path1.move_to((bounds.x, bounds.center().1));
            path2.move_to((bounds.x, bounds.center().1));

            let x = bounds.x;
            let w = bounds.w;
            let y = bounds.y;
            let h = bounds.h;

            let zoom_level = self.zoom_level_lens.get(cx);

            let start = self.start_lens.get(cx);

            if let Some(waveform_data) = waveform.get_data(zoom_level) {
                for pixel in 0..w as usize {
                    if start + pixel >= waveform_data.len() {
                        break;
                    }

                    let v_min = waveform_data[start + pixel].0;
                    let v_max = waveform_data[start + pixel].1;

                    match self.units_mode {
                        UnitsMode::Decibel => {
                            let v_min_db = 1.0 + (20.0 * v_min.abs().log10()).max(-60.0) / 60.0;
                            let v_max_db = 1.0 + (20.0 * v_max.abs().log10()).max(-60.0) / 60.0;

                            let v_min_db = if v_min < 0.0 { -v_min_db } else { v_min_db };

                            let v_max_db = if v_max < 0.0 { -v_max_db } else { v_max_db };

                            path1.line_to((x + (pixel as f32), y + h / 2.0 - v_min_db * h / 2.0));
                            path1.line_to((x + (pixel as f32), y + h / 2.0 - v_max_db * h / 2.0));
                        }

                        UnitsMode::Linear => {
                            path1.line_to((x + (pixel as f32), y + h / 2.0 - v_min * h / 2.0));
                            path1.line_to((x + (pixel as f32), y + h / 2.0 - v_max * h / 2.0));
                        }
                    }
                }

                // Draw min/max paths
                let mut paint = vg::Paint::default();
                paint.set_color(Color::rgba(50, 50, 255, 255));
                paint.set_stroke_width(1.0);
                paint.set_anti_alias(true);
                paint.set_style(vg::PaintStyle::Stroke);
                canvas.draw_path(&mut path1, &paint);
            }
        }
    }
}
