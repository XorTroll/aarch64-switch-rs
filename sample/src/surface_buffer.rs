use nx::gpu;

const X_MASK: u32 = !0x7B4;
const Y_MASK: u32 = 0x7B4;

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

    pub fn clear(&mut self, color: u32) {
        if self.is_valid() {
            unsafe {
                let buf_size_32 = self.buf_size / 4;
                for i in 0..buf_size_32 {
                    let cur = self.buf.offset(i as isize);
                    *cur = color;
                }
            }
        }
    }

    pub fn blit_with_color(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
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
                        *cur = color;
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
}