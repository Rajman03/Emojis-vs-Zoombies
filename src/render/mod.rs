//! Rysowanie sceny i interfejsu z użyciem assetów (UI kit, bronie, emoji).
//! Layout jest liczony raz i zwracany, żeby pętla główna trafiała myszą
//! w te same prostokąty, które widzi gracz.

use crate::assets::{enemy_face, Assets, DEFENDER_FACES};
use crate::config::*;
use crate::game::{slot_pos, Game, MAX_SLOTS};
use crate::gfx::{mix, shade, Canvas, Sprite};

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

pub struct Layout {
    pub tabs: Vec<Rect>,
    pub cards: Vec<Rect>,
    pub slots: Vec<Rect>,
    pub btn_pause: Rect,
    pub btn_save: Rect,
    pub btn_restart: Rect,
    /// Pozycja kursora z bieżącej klatki (do podświetlania pod myszą).
    pub mouse: Option<(f32, f32)>,
    /// Zakładka, dla której policzono karty.
    pub tab: usize,
}

const CARD_Y: f32 = 590.0;
const CARD_H: f32 = 122.0;
const INK: u32 = 0xf4efe6;
const MUTED: u32 = 0x9a8ab0;
const DIM: u32 = 0x5f5177;
const CARD_TINT: u32 = 0x211832;
const GOLD: u32 = 0xffc825;

/// Ile kart rysuje każda zakładka (reszta to karty informacyjne).
const CARD_COUNTS: [usize; 4] = [7, 7, 6, 5];

const CARD_W: f32 = 144.0;
const CARD_PITCH: f32 = 152.0;

/// Wyśrodkowany rząd prostokątów o stałym rytmie — panel nie zostawia
/// pustego pasa z prawej strony przy mniejszej liczbie kart.
fn row(count: usize, y: f32, h: f32) -> Vec<Rect> {
    let span = count as f32 * CARD_PITCH - (CARD_PITCH - CARD_W);
    let x0 = (WIN_W as f32 - span) * 0.5;
    (0..count)
        .map(|i| Rect { x: x0 + i as f32 * CARD_PITCH, y, w: CARD_W, h })
        .collect()
}

pub fn layout(tab: usize) -> Layout {
    let tabs = row(CARD_COUNTS.len(), PANEL_Y + 6.0, 28.0);
    let cards = row(CARD_COUNTS[tab.min(CARD_COUNTS.len() - 1)], CARD_Y, CARD_H);
    let slots: Vec<Rect> = (0..MAX_SLOTS)
        .map(|i| {
            let (x, y) = slot_pos(i);
            Rect { x: x - 34.0, y: y - 34.0, w: 68.0, h: 68.0 }
        })
        .collect();
    Layout {
        tabs,
        cards,
        slots,
        btn_pause: Rect { x: 1112.0, y: 16.0, w: 74.0, h: 32.0 },
        btn_save: Rect { x: 1194.0, y: 16.0, w: 74.0, h: 32.0 },
        btn_restart: Rect { x: 470.0, y: 430.0, w: 340.0, h: 52.0 },
        mouse: None,
        tab,
    }
}

fn fmt_num(v: f64) -> String {
    let v = v.max(0.0);
    if v >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v >= 10_000.0 {
        format!("{:.1}K", v / 1000.0)
    } else {
        format!("{:.0}", v)
    }
}

fn fmt_pct(v: f32) -> String {
    format!("{:.0}%", v * 100.0)
}

fn panel(c: &mut Canvas, a: &Assets, x: f32, y: f32, w: f32, h: f32, tint: u32) {
    c.nine(&a.frame, x, y, w, h, 12, Some(tint));
}

fn button(c: &mut Canvas, a: &Assets, r: &Rect, label: &str, active: bool) {
    let tint = if active { 0x4a3568 } else { 0x241a38 };
    c.nine(&a.button[active as usize], r.x, r.y, r.w, r.h, 10, Some(tint));
    c.text_center(r.x + r.w * 0.5, r.y + r.h * 0.5 - 6.0, label, if active { INK } else { MUTED }, 2);
}

fn bar(c: &mut Canvas, a: &Assets, x: f32, y: f32, w: f32, h: f32, ratio: f32, color: u32) {
    c.nine(&a.bar, x, y, w, h, 3, Some(0x2a2038));
    let fw = (w - 4.0) * ratio.clamp(0.0, 1.0);
    if fw > 1.0 {
        c.blit_rect(&a.fill, 0, 0, a.fill.w, a.fill.h, x + 2.0, y + 2.0, fw, h - 4.0, Some(color), 1.0);
    }
}


mod field;
mod hud;
mod panel;

use field::*;
use hud::*;
use panel::*;

pub fn draw(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    draw_background(c, g, a);
    draw_wall(c, g);
    draw_slots(c, g, a);
    draw_defenders(c, g, a);
    draw_enemies(c, g, a);
    draw_projectiles(c, g, a);
    draw_coins(c, g, a);
    draw_collectors(c, g);
    draw_floaters(c, g);
    draw_quest(c, g, a);
    draw_swap_hint(c, g, a);
    draw_hud(c, g, l, a);
    draw_panel(c, g, l, a);

    if g.message_t > 0.0 && !g.game_over {
        let alpha = (g.message_t / 0.6).min(1.0);
        let w = Canvas::text_width(&g.message, 3) + 44.0;
        c.rect_alpha(640.0 - w * 0.5, 104.0, w, 40.0, 0x0b0714, 0.8 * alpha);
        panel(c, a, 640.0 - w * 0.5, 104.0, w, 40.0, 0x241a38);
        c.text_center(640.0, 116.0, &g.message, GOLD, 3);
    }
    if g.paused && !g.game_over {
        overlay(c, a, "PAUSED", "PRESS SPACE TO RESUME", 0x5ac54f);
    }
    if g.game_over {
        overlay(c, a, "THE WALL FELL", "", 0xc42430);
        c.text_center(640.0, 386.0, &format!("REACHED WAVE {}", g.highest_wave), GOLD, 3);
        let r = l.btn_restart;
        c.nine(&a.button[0], r.x, r.y, r.w, r.h, 10, Some(0x4a2030));
        c.text_center(r.x + r.w * 0.5, r.y + 18.0, "RESTART RUN", INK, 3);
    }
}

