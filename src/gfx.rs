//! Rysowanie: własny framebuffer, prymitywy i miniaturowy font 3x5.
//! Zero assetów i zero zależności graficznych — wszystko liczone w kodzie.

/// Obrazek z RGBA (kanał alfa w górnym bajcie).
pub struct Sprite {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u32>,
}

impl Sprite {
    pub fn from_png(bytes: &[u8]) -> Option<Sprite> {
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let mut px = Vec::with_capacity((w * h) as usize);
        for p in rgba.pixels() {
            let [r, g, b, a] = p.0;
            px.push(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32);
        }
        Some(Sprite { w: w as usize, h: h as usize, px })
    }
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub buf: Vec<u32>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, buf: vec![0; w * h] }
    }

    /// Kopiuje poziomy pas `[y0, y1)` z innego bufora tego samego rozmiaru.
    pub fn copy_region(&mut self, src: &[u32], y0: usize, y1: usize) {
        if src.len() != self.buf.len() || y0 >= y1 || y1 > self.h {
            return;
        }
        self.buf[y0 * self.w..y1 * self.w].copy_from_slice(&src[y0 * self.w..y1 * self.w]);
    }

    /// Pionowy gradient — tło nieba.
    pub fn vgradient(&mut self, top: u32, bottom: u32) {
        for y in 0..self.h {
            let t = y as f32 / (self.h - 1).max(1) as f32;
            let c = mix(top, bottom, t);
            for x in 0..self.w {
                self.buf[y * self.w + x] = c;
            }
        }
    }

    #[inline]
    pub fn px(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        self.buf[y as usize * self.w + x as usize] = color;
    }

    #[inline]
    pub fn px_alpha(&mut self, x: i32, y: i32, color: u32, alpha: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let i = y as usize * self.w + x as usize;
        self.buf[i] = mix(self.buf[i], color, alpha.clamp(0.0, 1.0));
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        let x0 = x.floor().max(0.0) as usize;
        let y0 = y.floor().max(0.0) as usize;
        let x1 = ((x + w).ceil() as i64).clamp(0, self.w as i64) as usize;
        let y1 = ((y + h).ceil() as i64).clamp(0, self.h as i64) as usize;
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        for py in y0..y1 {
            let row = py * self.w;
            self.buf[row + x0..row + x1].fill(color);
        }
    }

    pub fn rect_alpha(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32, alpha: f32) {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = (x + w).ceil() as i32;
        let y1 = (y + h).ceil() as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                self.px_alpha(px, py, color, alpha);
            }
        }
    }

    pub fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32, thick: i32) {
        for t in 0..thick {
            let o = t as f32;
            self.rect(x + o, y + o, w - o * 2.0, 1.0, color);
            self.rect(x + o, y + h - 1.0 - o, w - o * 2.0, 1.0, color);
            self.rect(x + o, y + o, 1.0, h - o * 2.0, color);
            self.rect(x + w - 1.0 - o, y + o, 1.0, h - o * 2.0, color);
        }
    }

    /// Wypełniony okrąg (dyski rysowane skanline).
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, color: u32) {
        if r <= 0.0 {
            return;
        }
        let y0 = (cy - r).floor() as i32;
        let y1 = (cy + r).ceil() as i32;
        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            let sq = r * r - dy * dy;
            if sq <= 0.0 {
                continue;
            }
            let half = sq.sqrt();
            let x0 = (cx - half).round() as i32;
            let x1 = (cx + half).round() as i32;
            for x in x0..x1 {
                self.px(x, y, color);
            }
        }
    }

    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, thick: f32, color: u32) {
        let outer = r + thick;
        let y0 = (cy - outer).floor() as i32;
        let y1 = (cy + outer).ceil() as i32;
        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            let sq = outer * outer - dy * dy;
            if sq <= 0.0 {
                continue;
            }
            let half = sq.sqrt();
            let inner = if (r * r - dy * dy) > 0.0 { (r * r - dy * dy).sqrt() } else { 0.0 };
            let x0 = (cx - half).round() as i32;
            let x1 = (cx + half).round() as i32;
            for x in x0..x1 {
                let dx = (x as f32 + 0.5 - cx).abs();
                if dx > inner {
                    self.px(x, y, color);
                }
            }
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let steps = dx.abs().max(dy.abs()).max(1.0).round() as i32;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.px((x0 + dx * t).round() as i32, (y0 + dy * t).round() as i32, color);
        }
    }

    /// Kopiuje prostokąt sprite'a, skalując go najbliższym sąsiadem.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_rect(
        &mut self,
        s: &Sprite,
        sx: usize,
        sy: usize,
        sw: usize,
        sh: usize,
        dx: f32,
        dy: f32,
        dw: f32,
        dh: f32,
        tint: Option<u32>,
        alpha_mul: f32,
    ) {
        if sw == 0 || sh == 0 || dw <= 0.0 || dh <= 0.0 {
            return;
        }
        let x0 = dx.round() as i32;
        let y0 = dy.round() as i32;
        let pw = dw.max(1.0).round() as i32;
        let ph = dh.max(1.0).round() as i32;
        for oy in 0..ph {
            let src_y = sy + (oy as usize * sh) / ph.max(1) as usize;
            if src_y >= s.h {
                continue;
            }
            for ox in 0..pw {
                let src_x = sx + (ox as usize * sw) / pw.max(1) as usize;
                if src_x >= s.w {
                    continue;
                }
                let p = s.px[src_y * s.w + src_x];
                let a = (((p >> 24) & 0xff) as f32 / 255.0) * alpha_mul;
                if a <= 0.004 {
                    continue;
                }
                let mut r = (p >> 16) & 0xff;
                let mut g = (p >> 8) & 0xff;
                let mut b = p & 0xff;
                if let Some(t) = tint {
                    r = r * ((t >> 16) & 0xff) / 255;
                    g = g * ((t >> 8) & 0xff) / 255;
                    b = b * (t & 0xff) / 255;
                }
                let color = (r << 16) | (g << 8) | b;
                if a >= 0.999 {
                    self.px(x0 + ox, y0 + oy, color);
                } else {
                    self.px_alpha(x0 + ox, y0 + oy, color, a);
                }
            }
        }
    }

    /// Skaluje sprite tak, by zmiescil sie w prostokacie (z zachowaniem proporcji).
    pub fn blit_fit(&mut self, s: &Sprite, dx: f32, dy: f32, box_w: f32, box_h: f32) {
        let sw = s.w as f32;
        let sh = s.h as f32;
        let k = (box_w / sw).min(box_h / sh);
        let w = sw * k;
        let h = sh * k;
        self.blit_rect(
            s,
            0,
            0,
            s.w,
            s.h,
            dx + (box_w - w) * 0.5,
            dy + (box_h - h) * 0.5,
            w,
            h,
            None,
            1.0,
        );
    }

    /// Wycinek arkusza (np. emoji 16x16) przeskalowany do rozmiaru docelowego.
    pub fn blit_cell(&mut self, s: &Sprite, col: usize, row: usize, cell: usize, dx: f32, dy: f32, size: f32) {
        self.blit_rect(s, col * cell, row * cell, cell, cell, dx, dy, size, size, None, 1.0);
    }

    /// Ramka 9-slice: narożniki 1:1, krawędzie i środek rozciągnięte.
    pub fn nine(
        &mut self,
        s: &Sprite,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        border: usize,
        tint: Option<u32>,
    ) {
        let b = border.min(s.w / 2).min(s.h / 2);
        if b == 0 {
            self.blit_rect(s, 0, 0, s.w, s.h, x, y, w, h, tint, 1.0);
            return;
        }
        let bf = b as f32;
        let sw = s.w;
        let sh = s.h;
        // narożniki
        self.blit_rect(s, 0, 0, b, b, x, y, bf, bf, tint, 1.0);
        self.blit_rect(s, sw - b, 0, b, b, x + w - bf, y, bf, bf, tint, 1.0);
        self.blit_rect(s, 0, sh - b, b, b, x, y + h - bf, bf, bf, tint, 1.0);
        self.blit_rect(s, sw - b, sh - b, b, b, x + w - bf, y + h - bf, bf, bf, tint, 1.0);
        // krawędzie
        self.blit_rect(s, b, 0, sw - 2 * b, b, x + bf, y, w - 2.0 * bf, bf, tint, 1.0);
        self.blit_rect(s, b, sh - b, sw - 2 * b, b, x + bf, y + h - bf, w - 2.0 * bf, bf, tint, 1.0);
        self.blit_rect(s, 0, b, b, sh - 2 * b, x, y + bf, bf, h - 2.0 * bf, tint, 1.0);
        self.blit_rect(s, sw - b, b, b, sh - 2 * b, x + w - bf, y + bf, bf, h - 2.0 * bf, tint, 1.0);
        // środek
        self.blit_rect(s, b, b, sw - 2 * b, sh - 2 * b, x + bf, y + bf, w - 2.0 * bf, h - 2.0 * bf, tint, 1.0);
    }

    pub fn text(&mut self, x: f32, y: f32, s: &str, color: u32, scale: i32) {
        let sc = scale.max(1) as i32;
        let step = 4.0 * sc as f32;
        let mut cx = x;
        for ch in s.chars() {
            let up = ch.to_ascii_uppercase();
            if up != ' ' {
                if let Some(rows) = glyph(up) {
                    // blok glyphu zapisujemy wierszami, bez wywołania na piksel
                    for (ry, row) in rows.iter().enumerate() {
                        let py = y as i32 + ry as i32 * sc;
                        if py < 0 || py as usize >= self.h {
                            continue;
                        }
                        for rx in 0..3 {
                            if row & (1 << (2 - rx)) == 0 {
                                continue;
                            }
                            let px = (cx as i32 + rx * sc).max(0);
                            let x0 = px as usize;
                            let x1 = ((px + sc) as usize).min(self.w);
                            if x1 <= x0 {
                                continue;
                            }
                            for dy in 0..sc {
                                let r = (py as usize + dy as usize) * self.w;
                                for i in x0..x1 {
                                    self.buf[r + i] = color;
                                }
                            }
                        }
                    }
                }
            }
            cx += step;
        }
    }

    pub fn text_width(s: &str, scale: i32) -> f32 {
        s.chars().count() as f32 * 4.0 * scale as f32
    }

    pub fn text_center(&mut self, cx: f32, y: f32, s: &str, color: u32, scale: i32) {
        let w = Self::text_width(s, scale);
        self.text(cx - w * 0.5 + scale as f32 * 0.5, y, s, color, scale);
    }
}

/// Mieszanie dwóch kolorów 0xRRGGBB.
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ar = ((a >> 16) & 0xff) as f32;
    let ag = ((a >> 8) & 0xff) as f32;
    let ab = (a & 0xff) as f32;
    let br = ((b >> 16) & 0xff) as f32;
    let bg = ((b >> 8) & 0xff) as f32;
    let bb = (b & 0xff) as f32;
    let r = (ar + (br - ar) * t) as u32;
    let g = (ag + (bg - ag) * t) as u32;
    let bl = (ab + (bb - ab) * t) as u32;
    (r << 16) | (g << 8) | bl
}

pub fn shade(c: u32, factor: f32) -> u32 {
    let r = (((c >> 16) & 0xff) as f32 * factor).min(255.0) as u32;
    let g = (((c >> 8) & 0xff) as f32 * factor).min(255.0) as u32;
    let b = ((c & 0xff) as f32 * factor).min(255.0) as u32;
    (r << 16) | (g << 8) | b
}

/// Font 3x5: każdy glif to 5 wierszy po 3 bity (bit 2 = lewa kolumna).
fn glyph(ch: char) -> Option<[u8; 5]> {
    let g = match ch {
        'A' => [0b111, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b111, 0b100, 0b101, 0b101, 0b111],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b111],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
        'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
        'Q' => [0b111, 0b101, 0b101, 0b111, 0b011],
        'R' => [0b111, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b001, 0b001],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b010],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        ';' => [0b000, 0b010, 0b000, 0b010, 0b010],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '?' => [0b111, 0b001, 0b011, 0b000, 0b010],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        '(' => [0b001, 0b010, 0b010, 0b010, 0b001],
        ')' => [0b100, 0b010, 0b010, 0b010, 0b100],
        '[' => [0b011, 0b010, 0b010, 0b010, 0b011],
        ']' => [0b110, 0b010, 0b010, 0b010, 0b110],
        '<' => [0b001, 0b010, 0b100, 0b010, 0b001],
        '>' => [0b100, 0b010, 0b001, 0b010, 0b100],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
        '*' => [0b101, 0b010, 0b111, 0b010, 0b101],
        '\'' => [0b010, 0b010, 0b000, 0b000, 0b000],
        '"' => [0b101, 0b101, 0b000, 0b000, 0b000],
        '#' => [0b101, 0b111, 0b101, 0b111, 0b101],
        '_' => [0b000, 0b000, 0b000, 0b000, 0b111],
        _ => return None,
    };
    Some(g)
}
