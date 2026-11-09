//! Scena: tlo, mur, sloty, obroncy, wrogowie, pociski, monety.

use super::*;

pub(crate) fn draw_background(c: &mut Canvas, g: &Game, a: &Assets) {
    // tło jest gotowe w Assets — kopiujemy bufor zamiast liczyć gradient co klatkę
    c.buf.copy_from_slice(&a.backdrop);
    for i in 0..70 {
        let x = ((i * 197) % 1240 + 20) as f32;
        let y = ((i * 83) % 300 + 20) as f32;
        let tw = ((g.time * 0.8 + i as f32).sin() * 0.5 + 0.5) * 0.6 + 0.4;
        c.px_alpha(x as i32, y as i32, 0xffffff, tw * 0.7);
    }
}

pub(crate) fn draw_wall(c: &mut Canvas, g: &Game) {
    let base_x = WALL_X - 88.0;
    let top = 150.0;
    let height = 392.0;
    let ratio = (g.wall_hp / g.wall_max()).clamp(0.0, 1.0) as f32;
    c.rect(base_x, top, 88.0, height, 0x4a4054);
    let mut row = 0;
    let mut y = top;
    while y < top + height {
        let off = if row % 2 == 0 { 0.0 } else { 22.0 };
        let mut x = base_x - off;
        while x < base_x + 88.0 {
            c.rect(x + 1.0, y + 1.0, 42.0, 20.0, 0x574c63);
            x += 44.0;
        }
        y += 22.0;
        row += 1;
    }
    let cracks = ((1.0 - ratio) * 10.0) as i32;
    for i in 0..cracks {
        let cx = base_x + 10.0 + ((i * 29) % 68) as f32;
        let cy = top + 40.0 + ((i * 71) % 300) as f32;
        c.line(cx, cy, cx + 16.0, cy + 26.0, 0x241c2c);
        c.line(cx + 16.0, cy + 26.0, cx + 4.0, cy + 48.0, 0x241c2c);
    }
    c.rect(base_x, top - 30.0, 88.0, 14.0, 0x1a1226);
    let hp_color = mix(0xc42430, 0x5ac54f, ratio);
    c.rect(base_x, top - 30.0, 88.0 * ratio, 14.0, hp_color);
    c.rect_outline(base_x, top - 30.0, 88.0, 14.0, 0x0b0714, 1);
    c.text_center(base_x + 44.0, top - 48.0, &fmt_num(g.wall_hp), INK, 2);
}

pub(crate) fn draw_slots(c: &mut Canvas, g: &Game, a: &Assets) {
    let visible = g.max_slots().min(MAX_SLOTS);
    for i in 0..visible {
        let (cx, cy) = slot_pos(i);
        let occupied = g.slot_occupied(i);
        if occupied {
            // zajęty slot: kafel z assetu jako podstawa pod postać
            c.nine(&a.slot, cx - 30.0, cy - 30.0, 60.0, 60.0, 10, Some(0x332748));
            let is_selected = g
                .selected
                .is_some_and(|sel| g.defenders.iter().any(|d| d.slot == i && d.def == sel));
            if is_selected {
                // pulsująca ramka wokół pola wybranego obrońcy
                let pulse = (g.time * 4.0).sin() * 0.5 + 0.5;
                let glow = mix(0xffc825, 0xfff2b0, pulse);
                c.rect_outline(cx - 34.0, cy - 34.0, 68.0, 68.0, glow, 2);
                c.rect_outline(cx - 30.0, cy - 30.0, 60.0, 60.0, 0x5a4a72, 1);
            } else {
                c.rect_outline(cx - 30.0, cy - 30.0, 60.0, 60.0, 0x8a7ab0, 1);
            }
        } else {
            // pusty slot: sam pierścień, żeby nie zasłaniać tła
            let col = if g.selected.is_some() {
                let pulse = (g.time * 2.0 + i as f32).sin() * 0.5 + 0.5;
                mix(0x3a2c50, 0xffc825, pulse * 0.7)
            } else {
                0x4a3a62
            };
            c.ring(cx, cy, 26.0, 2.0, col);
        }
    }
}

pub(crate) fn draw_defenders(c: &mut Canvas, g: &Game, a: &Assets) {
    for d in &g.defenders {
        if d.slot >= MAX_SLOTS {
            continue;
        }
        let (x, y) = slot_pos(d.slot);
        let def = &DEFENDERS[d.def];
        let rarity = &RARITIES[d.rarity];
        // pierścień rzadkości pod postacią
        c.ring(x, y, 30.0, 2.0, shade(rarity.color, 0.75));
        let (col, row) = DEFENDER_FACES[d.def % DEFENDER_FACES.len()];
        c.blit_cell(&a.emoji, col, row, 16, x - 24.0, y - 24.0, 48.0);
        // broń: założona z ekwipunku albo bazowa dla typu
        let gun: &Sprite = match d.weapon {
            Some((idx, _)) => a.gun_for_item(idx),
            None => a.gun(def.weapon),
        };
        let gy = y - 2.0;
        c.blit_fit(gun, x + 14.0, gy, 54.0, 24.0);
        if d.flash > 0.0 {
            a.art.flash(c, x + 68.0, gy + 12.0, d.flash);
        }
    }
}

pub(crate) fn draw_enemies(c: &mut Canvas, g: &Game, a: &Assets) {
    for e in &g.enemies {
        let s = e.size;
        let bob = e.phase.sin() * 2.5 * s;
        let y = e.y + bob;
        let size = 16.0 * 2.7 * s;
        let (col, row) = enemy_face(e.kind);
        // cień
        c.blit_cell(&a.emoji, col, row, 16, e.x - size * 0.5, y - size * 0.5, size);
        if e.hit > 0.0 {
            c.rect_alpha(e.x - size * 0.5, y - size * 0.5, size, size, 0xffffff, 0.35 * e.hit);
        }
        let w = 44.0 * s;
        let ratio = (e.hp / e.max_hp).clamp(0.0, 1.0);
        c.rect(e.x - w * 0.5, y - size * 0.62, w, 6.0, 0x1a1226);
        c.rect(e.x - w * 0.5, y - size * 0.62, w * ratio, 6.0, mix(0xc42430, 0x5ac54f, ratio));
    }
}

pub(crate) fn draw_projectiles(c: &mut Canvas, g: &Game, a: &Assets) {
    for p in &g.projectiles {
        // smuga za pociskiem: dwa bloki w kierunku celu
        let (mut dx, mut dy) = (p.lx - p.x, p.ly - p.y);
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        dx /= len;
        dy /= len;
        let trail = if p.crit { 0xff9a3c } else { 0xffe08a };
        for step in 0..4 {
            let d = 7.0 + step as f32 * 3.0;
            let alpha = 0.5 - step as f32 * 0.1;
            c.rect_alpha(p.x - dx * d - 1.0, p.y - dy * d - 1.0, 2.0, 2.0, trail, alpha);
        }
        a.art.bullet(c, p.x, p.y, p.crit);
    }
}

pub(crate) fn draw_coins(c: &mut Canvas, g: &Game, a: &Assets) {
    for coin in &g.coins {
        a.art.coin(c, coin.x, coin.y, g.time + coin.x * 0.01);
    }
}

pub(crate) fn draw_collectors(c: &mut Canvas, g: &Game) {
    for col in &g.collectors {
        c.disc(col.x, col.y, 13.0, 0x6a7a8a);
        c.disc(col.x - 3.0, col.y - 4.0, 8.0, 0x8a9aaa);
        c.rect(col.x - 8.0, col.y - 6.0, 12.0, 6.0, 0x141018);
        c.px((col.x - 2.0) as i32, (col.y - 4.0) as i32, 0x5ac54f);
        c.rect(col.x - 9.0, col.y + 10.0, 5.0, 6.0, 0x4a5a6a);
        c.rect(col.x + 4.0, col.y + 10.0, 5.0, 6.0, 0x4a5a6a);
        if col.taken > 0 {
            c.disc(col.x + 15.0, col.y - 9.0, 7.0, GOLD);
            c.text_center(col.x + 15.0, col.y + 1.0, &format!("{}", col.taken), 0x3a2a00, 2);
        }
    }
}

pub(crate) fn draw_floaters(c: &mut Canvas, g: &Game) {
    for f in &g.floaters {
        let alpha = f.t.min(1.0);
        let col = mix(0x0b0714, f.color, alpha);
        c.text_center(f.x, f.y, &f.text, col, 2);
    }
}
