// MIT License

// Copyright (c) 2022 AnonmousDapper

use line_drawing::{Bresenham, BresenhamCircle};

use half::f16;

use crate::{FB_HEIGHT, FB_WIDTH};

const BUF_LEN: usize = FB_WIDTH as usize * FB_HEIGHT as usize * 8;

const GAMMA: f32 = 2.2;

pub type Size = i32;
pub type Point = (Size, Size);

type Buffer = Vec<u8>;

#[inline]
fn as_idx(x: Size, y: Size) -> usize {
    (y * 8 * FB_WIDTH + x * 8) as usize
}

pub trait Surface {
    type Pixel;

    fn set_pixel(&mut self, x: Size, y: Size, data: &Self::Pixel);

    fn set_row(&mut self, y: Size, data: &Self::Pixel);

    fn set_col(&mut self, x: Size, data: &Self::Pixel);

    fn fill(&mut self, data: &Self::Pixel);
}

impl Surface for Buffer {
    type Pixel = [u8; 8];

    #[inline]
    fn set_pixel(&mut self, x: Size, y: Size, data: &Self::Pixel) {
        let idx = as_idx(x.rem_euclid(FB_WIDTH), y.rem_euclid(FB_HEIGHT));
        self[idx..idx + 8].copy_from_slice(data);
    }

    #[inline]
    fn set_row(&mut self, y: Size, data: &Self::Pixel) {
        for x in 0..FB_WIDTH {
            self.set_pixel(x, y, data);
        }
    }

    #[inline]
    fn set_col(&mut self, x: Size, data: &Self::Pixel) {
        for y in 0..FB_HEIGHT {
            self.set_pixel(x, y, data);
        }
    }

    #[inline]
    fn fill(&mut self, data: &Self::Pixel) {
        for pixel in self.chunks_exact_mut(8) {
            pixel.copy_from_slice(data);
        }
    }
}

pub trait Render {
    type Color;

    fn pixel(&mut self, point: Point, color: Self::Color);

    fn line(&mut self, origin: Point, end: Point, color: Self::Color);

    fn square(&mut self, origin: Point, length: Size, color: Self::Color);

    fn circle(&mut self, origin: Point, radius: Size, color: Self::Color);

    fn fill(&mut self, color: Self::Color);

    fn clear(&mut self);

    // higher-level methods
    fn particle(&mut self, point: Point, size: Size, color: Self::Color);
}

pub struct Canvas {
    frame: Buffer,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            frame: vec![0; BUF_LEN],
        }
    }

    #[inline]
    pub fn get_frame(&self) -> &Buffer {
        &self.frame
    }

    #[inline]
    pub fn render_to(&self, frame: &mut [u8]) {
        let buf = self.get_frame();

        frame.copy_from_slice(buf);
    }
}

impl Render for Canvas {
    type Color = RgbaF16;

    #[inline]
    fn pixel(&mut self, (x, y): Point, color: Self::Color) {
        self.frame.set_pixel(x, y, &color.as_bytes())
    }

    #[inline]
    fn line(&mut self, origin: Point, end: Point, color: Self::Color) {
        for point in Bresenham::new(origin, end) {
            self.pixel(point, color);
        }
    }

    #[inline]
    fn square(&mut self, (x, y): Point, length: Size, color: Self::Color) {
        for dx in x..(x + length) {
            for dy in y..(y + length) {
                self.pixel((dx, dy), color)
            }
        }
    }

    #[inline]
    fn circle(&mut self, (x, y): Point, radius: Size, color: Self::Color) {
        let coords = BresenhamCircle::new(x, y, radius)
            .into_iter()
            .collect::<Vec<Point>>();

        let q1 = coords.iter().step_by(4).skip(1);
        let q2 = coords[1..].iter().step_by(4).skip(1).rev();
        let q3 = coords[2..].iter().step_by(4).skip(1).rev();
        let q4 = coords[3..].iter().step_by(4).skip(1);

        for ((ox, oy), (ex, _ey)) in q1.zip(q2) {
            for lx in *ex..(*ox) {
                self.pixel((lx, *oy - 1), color);
            }
        }

        for ((ex, _ey), (ox, oy)) in q3.zip(q4) {
            for lx in *ex..(*ox) {
                self.pixel((lx, *oy), color);
            }
        }
    }

    #[inline]
    fn fill(&mut self, color: Self::Color) {
        self.frame.fill(&color.as_bytes())
    }

    #[inline]
    fn clear(&mut self) {
        self.frame.fill(&[0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[inline]
    fn particle(&mut self, (x, y): Point, size: Size, color: Self::Color) {
        match size {
            0 => {}
            1 => self.pixel((x, y), color),
            2 => self.square((x, y), size, color),
            3 => {
                self.line((x, y + 1), (x, y - 1), color);
                self.line((x + 1, y), (x - 1, y), color);
            }
            _ => self.circle((x, y), size / 2, color),
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RgbaF16 {
    r: f16,
    g: f16,
    b: f16,
    a: f16,
}

impl RgbaF16 {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self::new_raw(r.powf(GAMMA), g.powf(GAMMA), b.powf(GAMMA), 1.0)
    }

    pub fn rgb(r: u16, g: u16, b: u16) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    pub fn new_raw(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: f16::from_f32(r),
            g: f16::from_f32(g),
            b: f16::from_f32(b),
            a: f16::from_f32(a),
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> [u8; 8] {
        let [rh, rl] = self.r.to_le_bytes();
        let [gh, gl] = self.g.to_le_bytes();
        let [bh, bl] = self.b.to_le_bytes();
        let [ah, al] = self.a.to_le_bytes();

        [rh, rl, gh, gl, bh, bl, ah, al]
    }
}
