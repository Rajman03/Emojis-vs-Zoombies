//! Proceduralna grafika pixel-art: monety, naboje, błysk lufy.
//!
//! Każdy sprite to kilka wierszy znaków i paleta — wzór zmienia się w jednym
//! miejscu, bez ruszania logiki rysowania. Skala jest całkowita, więc piksele
//! pozostają ostre (blit_rect nie interpoluje).

use crate::gfx::{Canvas, Sprite};

const TRANSPARENT: u32 = 0;

fn rgb(v: u32) -> u32 {
    0xff00_0000 | v
}

fn sprite(rows: &[&str], palette: &[(char, u32)]) -> Sprite {
    let h = rows.len();
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let mut px = vec![TRANSPARENT; w * h];
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some((_, color)) = palette.iter().find(|(c, _)| *c == ch) {
                px[y * w + x] = rgb(*color);
            }
        }
    }
    Sprite { w, h, px }
}

pub struct PixelArt {
    /// Trzy klatki obrotu monety (pełna, wąska, krawędź).
    coin: [Sprite; 3],
    bullet: Sprite,
    bullet_crit: Sprite,
    flash: Sprite,
}

impl PixelArt {
    pub fn new() -> Self {
        let gold = [
            ('#', 0x6b4a08),
            ('o', 0xffc825),
            ('h', 0xfff2b0),
        ];
        let full = [
            "..#####..",
            ".#ooooo#.",
            "#oohhooo#",
            "#oohhhoo#",
            "#ooohhoo#",
            "#ooohhoo#",
            "#ooooooo#",
            ".#ooooo#.",
            "..#####..",
        ];
        let narrow = [
            "...###...",
            "..#ooo#..",
            ".#ohhoo#.",
            ".#ohhoo#.",
            ".#ohhoo#.",
            ".#ohhoo#.",
            "..#ooo#..",
            "...###...",
            ".........",
        ];
        let edge = [
            "...###...",
            "..##o##..",
            "..#oho#..",
            "..#oho#..",
            "..#oho#..",
            "..#oho#..",
            "..#oho#..",
            "..##o##..",
            "...###...",
        ];
        let bullet_palette = [('#', 0x8a5a00), ('y', 0xffc825), ('h', 0xfff6d0)];
        let crit_palette = [('#', 0xa03000), ('y', 0xff7a1e), ('h', 0xfff0a0)];
        Self {
            coin: [sprite(&full, &gold), sprite(&narrow, &gold), sprite(&edge, &gold)],
            bullet: sprite(
                &[".hhh.", "hyyyh", ".hhh."],
                &bullet_palette,
            ),
            bullet_crit: sprite(
                &[".hhh.", "hyyyh", ".hhh."],
                &crit_palette,
            ),
            flash: sprite(
                &["..h..", ".hhh.", "hhyhh", ".hhh.", "..h.."],
                &[('h', 0xfff6d0), ('y', 0xffc825)],
            ),
        }
    }

    /// Moneta 2x w skali, animowana trzema klatkami.
    pub fn coin(&self, c: &mut Canvas, x: f32, y: f32, age: f32) {
        let frame = ((age * 3.0) as usize) % self.coin.len();
        let s = &self.coin[frame];
        let (w, h) = (s.w as f32 * 2.0, s.h as f32 * 2.0);
        c.blit_rect(s, 0, 0, s.w, s.h, x - w * 0.5, y - h * 0.5, w, h, None, 1.0);
    }

    pub fn bullet(&self, c: &mut Canvas, x: f32, y: f32, crit: bool) {
        let s = if crit { &self.bullet_crit } else { &self.bullet };
        let (w, h) = (s.w as f32 * 2.0, s.h as f32 * 2.0);
        c.blit_rect(s, 0, 0, s.w, s.h, x - w * 0.5, y - h * 0.5, w, h, None, 1.0);
    }

    pub fn flash(&self, c: &mut Canvas, x: f32, y: f32, strength: f32) {
        let s = &self.flash;
        let (w, h) = (s.w as f32 * 2.0, s.h as f32 * 2.0);
        c.blit_rect(s, 0, 0, s.w, s.h, x - w * 0.5, y - h * 0.5, w, h, None, strength.clamp(0.0, 1.0));
    }
}
