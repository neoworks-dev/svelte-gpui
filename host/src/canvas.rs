//! `g-canvas`: pixels the app paints into. Each canvas keeps a buffer at the window's
//! scale, and the app's drawing commands are rasterised into it by tiny-skia as they
//! arrive, so a stroke costs the same however much is already on the canvas. The
//! buffer is handed to GPUI as an image whenever it changed.

use std::sync::Arc;

use gpui::RenderImage;
use serde_json::Value;
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Stroke, Transform};

use crate::style::parse_colour;

/// Where the stroke being drawn has got to.
struct Pen {
    x: f32,
    y: f32,
    colour: tiny_skia::Color,
    width: f32,
}

#[derive(Default)]
pub struct Surface {
    pixmap: Option<Pixmap>,
    scale: f32,
    /// Commands that arrived before the canvas had been laid out, and so had no pixels.
    queued: Vec<Vec<Value>>,
    pen: Option<Pen>,
    changed: bool,
    image: Option<Arc<RenderImage>>,
    /// Images that were drawn and are now out of date, for the window to free.
    pub stale: Vec<Arc<RenderImage>>,
}

impl Surface {
    /// Runs one drawing command, or keeps it until the canvas has a size.
    pub fn command(&mut self, parts: &[Value]) {
        if self.pixmap.is_none() {
            self.queued.push(parts.to_vec());
            return;
        }
        self.run(parts);
    }

    /// Gives the canvas its size in device pixels. What was drawn is kept, from the top left,
    /// and scaled when the window moved to a screen of another scale.
    pub fn resize(&mut self, width: u32, height: u32, scale: f32) {
        let same = self
            .pixmap
            .as_ref()
            .is_some_and(|pixmap| pixmap.width() == width && pixmap.height() == height);
        if (same && self.scale == scale) || width == 0 || height == 0 {
            return;
        }
        let Some(mut fresh) = Pixmap::new(width, height) else {
            return;
        };
        if let Some(old) = &self.pixmap {
            // Moving to a screen of another scale keeps the drawing where it was, resampled.
            let factor = if self.scale > 0.0 { scale / self.scale } else { 1.0 };
            let paint = PixmapPaint { quality: tiny_skia::FilterQuality::Bilinear, ..PixmapPaint::default() };
            fresh.draw_pixmap(0, 0, old.as_ref(), &paint, Transform::from_scale(factor, factor), None);
        }
        self.pixmap = Some(fresh);
        self.scale = scale;
        self.changed = true;
        for parts in std::mem::take(&mut self.queued) {
            self.run(&parts);
        }
    }

    /// The pixels as a GPUI image, made again only when they changed.
    pub fn image(&mut self) -> Option<Arc<RenderImage>> {
        if !self.changed {
            return self.image.clone();
        }
        let pixmap = self.pixmap.as_ref()?;
        // GPUI takes BGRA with straight alpha; tiny-skia keeps RGBA premultiplied.
        let mut bytes = Vec::with_capacity(pixmap.data().len());
        for pixel in pixmap.pixels() {
            let straight = pixel.demultiply();
            bytes.extend_from_slice(&[straight.blue(), straight.green(), straight.red(), straight.alpha()]);
        }
        let buffer = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), bytes)?;
        let fresh = Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(buffer)]));
        if let Some(old) = self.image.replace(fresh.clone()) {
            self.stale.push(old);
        }
        self.changed = false;
        Some(fresh)
    }

    /// Every image this canvas made, for the window to free once the canvas is gone.
    pub fn release(&mut self) -> Vec<Arc<RenderImage>> {
        let mut all = std::mem::take(&mut self.stale);
        all.extend(self.image.take());
        all
    }

    fn run(&mut self, parts: &[Value]) {
        let number = |index: usize| parts.get(index).and_then(Value::as_f64).unwrap_or(0.0) as f32;
        let colour = |index: usize| {
            let rgba = parts
                .get(index)
                .and_then(Value::as_str)
                .and_then(parse_colour)
                .unwrap_or(gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
            tiny_skia::Color::from_rgba(rgba.r, rgba.g, rgba.b, rgba.a).unwrap_or(tiny_skia::Color::WHITE)
        };
        match parts.first().and_then(Value::as_str).unwrap_or("") {
            "clear" => {
                let fill = match parts.get(1) {
                    Some(Value::String(_)) => colour(1),
                    _ => tiny_skia::Color::TRANSPARENT,
                };
                if let Some(pixmap) = &mut self.pixmap {
                    pixmap.fill(fill);
                }
                self.pen = None;
            }
            "begin" => {
                let pen = Pen { x: number(1), y: number(2), colour: colour(3), width: number(4).max(0.1) };
                self.dot(pen.x, pen.y, pen.width / 2.0, pen.colour);
                self.pen = Some(pen);
            }
            "to" => {
                let (x, y) = (number(1), number(2));
                if let Some(pen) = &self.pen {
                    let (from_x, from_y, colour, width) = (pen.x, pen.y, pen.colour, pen.width);
                    self.segment(from_x, from_y, x, y, colour, width);
                }
                if let Some(pen) = &mut self.pen {
                    pen.x = x;
                    pen.y = y;
                }
            }
            "end" => self.pen = None,
            "line" => self.segment(number(1), number(2), number(3), number(4), colour(5), number(6)),
            "rect" => {
                let scale = self.scale;
                if let (Some(pixmap), Some(rect)) = (
                    &mut self.pixmap,
                    Rect::from_xywh(number(1) * scale, number(2) * scale, number(3) * scale, number(4) * scale),
                ) {
                    pixmap.fill_rect(rect, &paint(colour(5)), Transform::identity(), None);
                }
            }
            "circle" => self.dot(number(1), number(2), number(3), colour(4)),
            other => eprintln!("svelte-gpui: unknown canvas command {other:?}"),
        }
        self.changed = true;
    }

    fn dot(&mut self, x: f32, y: f32, radius: f32, colour: tiny_skia::Color) {
        let scale = self.scale;
        let Some(pixmap) = &mut self.pixmap else {
            return;
        };
        if let Some(path) = PathBuilder::from_circle(x * scale, y * scale, (radius * scale).max(0.5)) {
            pixmap.fill_path(&path, &paint(colour), FillRule::Winding, Transform::identity(), None);
        }
    }

    fn segment(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, colour: tiny_skia::Color, width: f32) {
        let scale = self.scale;
        let Some(pixmap) = &mut self.pixmap else {
            return;
        };
        let mut builder = PathBuilder::new();
        builder.move_to(x0 * scale, y0 * scale);
        builder.line_to(x1 * scale, y1 * scale);
        let Some(path) = builder.finish() else {
            return;
        };
        let stroke = Stroke {
            width: width * scale,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        pixmap.stroke_path(&path, &paint(colour), &stroke, Transform::identity(), None);
    }
}

fn paint(colour: tiny_skia::Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color(colour);
    paint.anti_alias = true;
    paint
}
