//! HUD, quest, podpowiedzi i nakladki.

use super::*;

pub(crate) fn overlay(c: &mut Canvas, a: &Assets, title: &str, sub: &str, color: u32) {
    c.rect_alpha(0.0, 300.0, WIN_W as f32, 210.0, 0x05030a, 0.86);
    panel(c, a, 330.0, 310.0, 620.0, 120.0, 0x1a1226);
    c.text_center(640.0, 336.0, title, color, 6);
    c.text_center(640.0, 386.0, sub, INK, 2);
}


pub(crate) fn draw_swap_hint(c: &mut Canvas, g: &Game, a: &Assets) {
    if g.tab != 0 {
        return;
    }
    let (x, y, w, h) = (944.0, 452.0, 328.0, 92.0);
    panel(c, a, x, y, w, h, 0x1a1329);
    c.text(x + 10.0, y + 8.0, "HOW TO SWAP", 0xe69c69, 2);
    c.text(x + 10.0, y + 28.0, "1 CLICK CARD", MUTED, 2);
    c.text(x + 10.0, y + 44.0, "2 CLICK SLOT", MUTED, 2);
    c.text(x + 10.0, y + 62.0, "REPLACES CURRENT", 0x5ac54f, 2);
    c.text(x + 10.0, y + 78.0, "RIGHT CLICK = REMOVE", 0xc42430, 2);
}

pub(crate) fn draw_quest(c: &mut Canvas, g: &Game, a: &Assets) {
    let q = &QUESTS[g.quest_idx];
    let done = g.quest_progress.min(q.target);
    panel(c, a, 8.0, 92.0, 310.0, 50.0, 0x1a1329);
    c.text(18.0, 100.0, &format!("QUEST: {}", q.label), GOLD, 2);
    c.text(18.0, 118.0, &format!("{}/{}", done, q.target), INK, 2);
    c.text(96.0, 118.0, &format!("+${:.0}", q.reward), 0x5ac54f, 2);
}

pub(crate) fn draw_hud(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    c.copy_region(&a.chrome, 0, HUD_HEIGHT as usize);
    // gotówka
    a.art.coin(c, 34.0, 32.0, 0.0);
    c.text(58.0, 22.0, &fmt_num(g.cash), INK, 3);
    if g.rebirth_points > 0 {
        c.text(58.0, 52.0, &format!("{} PTS", g.rebirth_points), 0xdb3ffd, 2);
    }
    // fala
    c.text(240.0, 14.0, &format!("WAVE {}", g.wave), INK, 3);
    c.text(370.0, 20.0, g.plan.name, 0xe69c69, 2);
    bar(c, a, 240.0, 42.0, 220.0, 16.0, g.wave_time / WAVE_SECONDS, if is_boss_wave(g.wave) { 0xed7614 } else { 0x5ac54f });
    if is_boss_wave(g.wave) {
        c.text(240.0, 64.0, "BOSS WAVE", 0xed7614, 2);
    }
    // poziom
    c.text(500.0, 14.0, &format!("LVL {}", g.level), 0x5ac54f, 3);
    let need = xp_for_level(g.level);
    bar(c, a, 500.0, 42.0, 150.0, 16.0, (g.xp / need) as f32, 0x5ac54f);
    // statystyki
    c.text(668.0, 16.0, &format!("DMG {}", fmt_pct(g.damage_mult() - 1.0)), 0xe69c69, 2);
    c.text(668.0, 34.0, &format!("RATE {}", fmt_pct(g.speed_mult() - 1.0)), 0x5ac54f, 2);
    c.text(668.0, 54.0, &format!("CASH {}", fmt_pct(g.cash_mult() - 1.0)), GOLD, 2);
    c.text(800.0, 16.0, &format!("DPS {}", fmt_num(g.dps as f64)), 0xdb3ffd, 2);
    c.text(800.0, 34.0, &format!("SPEED X{}", g.speed as i32), 0x5ac54f, 2);
    c.text(800.0, 54.0, &format!("COINS {}", g.coins.len()), GOLD, 2);
    // przyciski
    button(c, a, &l.btn_pause, if g.paused { "PLAY" } else { "PAUSE" }, false);
    button(c, a, &l.btn_save, "SAVE", false);
}

