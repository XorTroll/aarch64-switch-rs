use nx::gpu;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use font8x8::UnicodeFonts;

const X_MASK: u32 = !0x7B4;
const Y_MASK: u32 = 0x7B4;

#[derive(Copy, Clone)]
pub struct RGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA8 {
    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r: r, g: g, b: b, a: a }
    }

    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r: r, g: g, b: b, a: 0xFF }
    }

    const fn decode(raw: u32) -> (u8, u8, u8, u8) {
        let a = (raw & 0xFF) as u8;
        let b = ((raw >> 8) & 0xFF) as u8;
        let c = ((raw >> 16) & 0xFF) as u8;
        let d = ((raw >> 24) & 0xFF) as u8;
        (a, b, c, d)
    }

    pub const fn from_rgba(raw: u32) -> Self {
        let (a, b, g, r) = Self::decode(raw);
        Self::new_rgba(r, g, b, a)
    }

    pub const fn from_abgr(raw: u32) -> Self {
        let (r, g, b, a) = Self::decode(raw);
        Self::new_rgba(r, g, b, a)
    }

    const fn encode(a: u8, b: u8, c: u8, d: u8) -> u32 {
        ((a as u32 & 0xFF) | ((b as u32 & 0xFF) << 8) | ((c as u32 & 0xFF) << 16) | ((d as u32 & 0xFF) << 24))
    }

    pub const fn encode_rgba(&self) -> u32 {
        Self::encode(self.a, self.b, self.g, self.r)
    }

    pub const fn encode_abgr(&self) -> u32 {
        Self::encode(self.r, self.g, self.b, self.a)
    }

    const fn blend_color_impl(src: u32, dst: u32, alpha: u32) -> u8 {
        let one_minus_a = 0xFF - alpha;
        ((dst * alpha + src * one_minus_a) / 0xFF) as u8
    }

    pub const fn blend_with(&self, other: Self) -> Self {
        let r = Self::blend_color_impl(other.r as u32, self.r as u32, self.a as u32);
        let g = Self::blend_color_impl(other.g as u32, self.g as u32, self.a as u32);
        let b = Self::blend_color_impl(other.b as u32, self.b as u32, self.a as u32);
        Self::new_rgb(r, g, b)
    }
}

pub struct SurfaceBuffer {
    buf: *mut u32,
    buf_size: usize,
    width: u32,
    height: u32,
    color_fmt: gpu::ColorFormat
}

impl SurfaceBuffer {
    fn swizzle_mask(mask: u32, mut value: u32) -> u32 {
        let mut out: u32 = 0;
        for i in 0..32 {
            let bit = bit!(i);
            if (mask & bit) != 0 {
                if (value & 1) != 0 {
                    out |= bit;
                }
                value >>= 1;
            }
        }
        out
    }
    
    const fn clamp(coord: i32, max: u32) -> u32 {
        if coord < 0 {
            return 0;
        }
        if coord > max as i32 {
            return max;
        }
        coord as u32
    }

    fn is_valid(&self) -> bool {
        !self.buf.is_null() && (self.buf_size > 0) && (self.width > 0) && (self.height > 0)
    }

    pub const fn from(buf: *mut u8, buf_size: usize, width: u32, height: u32, color_fmt: gpu::ColorFormat) -> Self {
        Self { buf: buf as *mut u32, buf_size: buf_size, width: width, height: height, color_fmt: color_fmt }
    }

    pub fn clear(&mut self, color: RGBA8) {
        if self.is_valid() {
            unsafe {
                let buf_size_32 = self.buf_size / 4;
                for i in 0..buf_size_32 {
                    let cur = self.buf.offset(i as isize);
                    *cur = color.encode_abgr();
                }
            }
        }
    }

    pub fn blit_with_color(&mut self, x: i32, y: i32, width: i32, height: i32, color: RGBA8) {
        if self.is_valid() {
            unsafe {
                let x0 = Self::clamp(x, self.width);
                let x1 = Self::clamp(x + width, self.width);
                let y0 = Self::clamp(y, self.height);
                let y1 = Self::clamp(y + height, self.height);

                let bpp = gpu::calculate_bpp(self.color_fmt);
                let aligned_width = gpu::align_width(bpp, self.width);
                let y_increment = Self::swizzle_mask(X_MASK, aligned_width);

                let mut x0_offset = Self::swizzle_mask(X_MASK, x0) + y_increment * (y0 / gpu::BLOCK_HEIGHT);
                let mut y0_offset = Self::swizzle_mask(Y_MASK, y0);

                for _ in y0..y1 {
                    let buf_line = self.buf.offset(y0_offset as isize);
                    let mut x_offset = x0_offset;
                    for _ in x0..x1 {
                        let cur = buf_line.offset(x_offset as isize);
                        let old_color = RGBA8::from_abgr(*cur);
                        let new_color = color.blend_with(old_color);
                        *cur = new_color.encode_abgr();
                        x_offset = (x_offset - X_MASK) & X_MASK;
                    }
                    y0_offset = (y0_offset - Y_MASK) & Y_MASK;
                    if y0_offset == 0 {
                        x0_offset += y_increment;
                    }
                }
            }
        }
    }

    fn draw_font_text_impl(&mut self, font: &rusttype::Font, text: &str, color: RGBA8, size: f32, x: i32, y: i32) {
        let scale = rusttype::Scale::uniform(size);
        let v_metrics = font.v_metrics(scale);
        
        let glyphs: Vec<_> = font.layout(&text[..], rusttype::Scale::uniform(size), rusttype::point(x as f32, y as f32 + v_metrics.ascent)).collect();
        for glyph in &glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|g_x, g_y, g_v| {
                    let mut pix_color = color;
                    // Different alpha depending on the pixel
                    pix_color.a = (g_v * 255.0) as u8;
                    self.blit_with_color(g_x as i32 + bounding_box.min.x as i32, g_y as i32 + bounding_box.min.y as i32, 1, 1, pix_color);
                });
            }
        }
    }
    
    pub fn draw_font_text(&mut self, font: &rusttype::Font, text: String, color: RGBA8, size: f32, x: i32, y: i32) {
        let scale = rusttype::Scale::uniform(size);
        let v_metrics = font.v_metrics(scale);
        
        let mut tmp_y = y;
        for semi_text in text.lines() {
            self.draw_font_text_impl(font, semi_text, color, size, x, tmp_y);
            tmp_y += v_metrics.ascent as i32;
        }
    }

    pub fn draw_bitmap_text(&mut self, text: String, color: RGBA8, scale: i32, x: i32, y: i32) {
        let mut tmp_x = x;
        let mut tmp_y = y;
        for c in text.chars() {
            match c {
                '\n' | '\r' => {
                    tmp_y += 8 * scale;
                    tmp_x = x;
                },
                _ => {
                    if let Some(glyph) = font8x8::BASIC_FONTS.get(c) {
                        let char_tmp_x = tmp_x;
                        let char_tmp_y = tmp_y;
                        for gx in &glyph {
                            for bit in 0..8 {
                                match *gx & 1 << bit {
                                    0 => {},
                                    _ => {
                                        self.blit_with_color(tmp_x, tmp_y, scale, scale, color);
                                    },
                                }
                                tmp_x += scale;
                            }
                            tmp_y += scale;
                            tmp_x = char_tmp_x;
                        }
                        tmp_x += 8 * scale;
                        tmp_y = char_tmp_y;
                    }
                }
            }
        }
    }
}

// Needed by rusttype

pub trait FloatExt {
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn fract(self) -> Self;
    fn trunc(self) -> Self;
    fn round(self) -> Self;
    fn abs(self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn floor(self) -> Self {
        libm::floorf(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        libm::ceilf(self)
    }
    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }
    #[inline]
    fn trunc(self) -> Self {
        libm::truncf(self)
    }
    #[inline]
    fn round(self) -> Self {
        libm::roundf(self)
    }
    #[inline]
    fn abs(self) -> Self {
        libm::fabsf(self)
    }
}