//! Dolny panel: zakladki i karty wszystkich ekranow.

use super::*;

const TAB_NAMES: [&str; 4] = ["DEFENDERS", "UPGRADES", "REBIRTH", "OPTIONS"];

pub(crate) fn draw_panel(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    c.copy_region(&a.chrome, (PANEL_Y - 2.0) as usize, WIN_H);
    for (i, r) in l.tabs.iter().enumerate() {
        button(c, a, r, TAB_NAMES[i], g.tab == i);
    }
    // delikatne podświetlenie elementu pod kursorem
    if let Some((mx, my)) = l.mouse {
        for (i, r) in l.tabs.iter().enumerate() {
            if g.tab != i && r.contains(mx, my) {
                c.rect_outline(r.x, r.y, r.w, r.h, 0x7a6a9a, 1);
            }
        }
        for r in &l.cards {
            if r.contains(mx, my) {
                c.rect_outline(r.x, r.y, r.w, r.h, 0x8a7ab0, 1);
            }
        }
    }
    match g.tab {
        0 => draw_defender_cards(c, g, l, a),
        1 => draw_upgrade_cards(c, g, l, a),
        2 => draw_rebirth_panel(c, g, l, a),
        _ => draw_options_panel(c, g, l, a),
    }
}

pub(crate) fn card(c: &mut Canvas, a: &Assets, r: &Rect, accent: u32, enabled: bool, selected: bool) {
    let tint = if selected { 0x2e2244 } else { CARD_TINT };
    c.nine(&a.frame2, r.x, r.y, r.w, r.h, 12, Some(tint));
    if selected {
        c.rect_outline(r.x, r.y, r.w, r.h, accent, 2);
    }
    if !enabled {
        c.rect_alpha(r.x, r.y, r.w, r.h, 0x05030a, 0.45);
    }
}

pub(crate) fn draw_defender_cards(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    for (i, def) in DEFENDERS.iter().enumerate() {
        let r = l.cards[i];
        let owned = g.unlocked[i];
        let affordable = g.cash >= def.cost;
        let selected = g.selected == Some(i);
        card(c, a, &r, def.color, affordable, selected);
        let (col, row) = DEFENDER_FACES[i % DEFENDER_FACES.len()];
        c.blit_cell(&a.emoji, col, row, 16, r.x + 8.0, r.y + 8.0, 34.0);
        c.blit_fit(a.gun(def.weapon), r.x + 46.0, r.y + 16.0, 90.0, 20.0);
        c.text(r.x + 8.0, r.y + 48.0, def.name, INK, 2);
        let dps = def.damage * def.aps * g.damage_mult() * weapon_damage_multiplier(def.weapon, 0);
        c.text(r.x + 8.0, r.y + 64.0, &format!("{} DPS", fmt_num(dps as f64)), MUTED, 2);
        // cena za KAŻDĄ kopię, nie tylko za pierwsze odblokowanie
        let price_color = if affordable { GOLD } else { 0xc42430 };
        c.text(r.x + 96.0, r.y + 64.0, &format!("${}", fmt_num(def.cost)), price_color, 2);
        let rarity = &RARITIES[g.char_rarity[i]];
        let placed = g.defenders.iter().filter(|d| d.def == i).count();
        let label = if owned { rarity.name } else { "LOCKED" };
        let label_color = if owned { rarity.color } else { DIM };
        c.text(r.x + 8.0, r.y + 80.0, label, label_color, 2);
        if placed > 0 {
            c.text(r.x + 96.0, r.y + 80.0, &format!("X{}", placed), 0x5ac54f, 2);
        }
        let action = if owned { "CLICK TO PLACE" } else { "CLICK TO BUY" };
        c.text(r.x + 8.0, r.y + 98.0, action, if affordable { 0x5ac54f } else { DIM }, 2);
    }
}

pub(crate) fn draw_upgrade_cards(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    for (i, up) in UPGRADES.iter().enumerate() {
        let r = l.cards[i];
        let cost = g.upgrade_cost(i);
        let affordable = g.cash >= cost;
        card(c, a, &r, 0x5ac54f, affordable, false);
        c.text(r.x + 8.0, r.y + 8.0, up.name, INK, 2);
        c.text(r.x + 8.0, r.y + 24.0, up.desc, MUTED, 2);
        c.text(r.x + 8.0, r.y + 42.0, &format!("LVL {}", g.upgrades[i]), 0x5ac54f, 3);
        c.text(r.x + 8.0, r.y + 68.0, &format!("${}", fmt_num(cost)), if affordable { GOLD } else { 0xc42430 }, 3);
        let lvl = g.upgrades[i];
        let prog = (lvl % 25) as f32 / 25.0;
        bar(c, a, r.x + 8.0, r.y + 94.0, r.w - 16.0, 12.0, prog, 0x5ac54f);
        c.text(r.x + 8.0, r.y + 108.0, &format!("X5 AT LVL {}", (lvl / 25 + 1) * 25), DIM, 2);
    }
    if let Some(r) = l.cards.get(6) {
        c.text(r.x + 8.0, r.y + 8.0, "PRICE RULE", 0xe69c69, 2);
        c.text(r.x + 8.0, r.y + 26.0, "BASE * GROWTH", MUTED, 2);
        c.text(r.x + 8.0, r.y + 42.0, "^(LVL-1) * X5", MUTED, 2);
        c.text(r.x + 8.0, r.y + 58.0, "EVERY 25 LVL", MUTED, 2);
        c.text(r.x + 8.0, r.y + 82.0, "SHARED RULE", DIM, 2);
        c.text(r.x + 8.0, r.y + 98.0, "ALL 6 CARDS", DIM, 2);
    }
}

pub(crate) fn draw_rebirth_panel(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    for (i, rb) in REBIRTH_UPGRADES.iter().enumerate() {
        let r = l.cards[i];
        let cost = g.rebirth_upgrade_cost(i);
        let affordable = g.rebirth_points >= cost;
        card(c, a, &r, 0xdb3ffd, affordable, false);
        c.text(r.x + 8.0, r.y + 8.0, rb.name, INK, 2);
        c.text(r.x + 8.0, r.y + 24.0, rb.desc, MUTED, 2);
        c.text(r.x + 8.0, r.y + 42.0, &format!("LVL {}", g.rebirth_upgrades[i]), 0xdb3ffd, 3);
        c.text(r.x + 8.0, r.y + 70.0, &format!("{} PTS", cost), if affordable { 0xdb3ffd } else { 0xc42430 }, 2);
    }
    let r = l.cards[4];
    card(c, a, &r, GOLD, g.can_rebirth(), false);
    c.text(r.x + 8.0, r.y + 6.0, "REBIRTH", GOLD, 3);
    c.text(r.x + 8.0, r.y + 30.0, "RESET RUN", MUTED, 2);
    c.text(r.x + 8.0, r.y + 46.0, "KEEP DROPS", MUTED, 2);
    c.text(r.x + 8.0, r.y + 68.0, &format!("GAIN {}", g.rebirth_gain()), 0xdb3ffd, 2);
    c.text(r.x + 8.0, r.y + 86.0, &format!("BEST {}", g.highest_wave), DIM, 2);
    let (label, color) = if g.can_rebirth() {
        ("CLICK = REBIRTH", 0x5ac54f)
    } else {
        ("NEED WAVE 10", 0xc42430)
    };
    c.text(r.x + 8.0, r.y + 102.0, label, color, 2);

    let r = l.cards[5];
    c.text(r.x + 8.0, r.y + 8.0, "REBIRTH RULE", 0xe69c69, 2);
    c.text(r.x + 8.0, r.y + 26.0, "BOSS = FLOOR", MUTED, 2);
    c.text(r.x + 8.0, r.y + 42.0, "(WAVE/5) PTS", MUTED, 2);
    c.text(r.x + 8.0, r.y + 62.0, "+2*(I+1) PER", DIM, 2);
    c.text(r.x + 8.0, r.y + 78.0, "10 WAVES", DIM, 2);

}

pub(crate) fn draw_options_panel(c: &mut Canvas, g: &Game, l: &Layout, a: &Assets) {
    let r = l.cards[0];
    card(c, a, &r, 0x5ac54f, true, false);
    c.text(r.x + 8.0, r.y + 8.0, "GAME SPEED", INK, 2);
    c.text(r.x + 8.0, r.y + 34.0, &format!("X{}", g.speed as i32), 0x5ac54f, 5);
    c.text(r.x + 8.0, r.y + 98.0, "CLICK / F", DIM, 2);

    let r = l.cards[1];
    card(c, a, &r, GOLD, true, false);
    c.text(r.x + 8.0, r.y + 8.0, "SAVE", INK, 2);
    c.text(r.x + 8.0, r.y + 30.0, "WRITES FILE", MUTED, 2);
    c.text(r.x + 8.0, r.y + 56.0, "F5", GOLD, 3);

    let r = l.cards[2];
    card(c, a, &r, 0xc42430, true, false);
    c.text(r.x + 8.0, r.y + 8.0, "FULL RESET", 0xc42430, 2);
    c.text(r.x + 8.0, r.y + 30.0, "CLICK TWICE", MUTED, 2);
    if g.reset_armed {
        c.text(r.x + 8.0, r.y + 50.0, "SURE?", 0xc42430, 3);
    }
    c.text(r.x + 8.0, r.y + 78.0, "WIPES SAVE", MUTED, 2);
    c.text(r.x + 8.0, r.y + 94.0, "AND ALL DATA", MUTED, 2);


    let r = l.cards[3];
    card(c, a, &r, 0x92a1b9, true, false);
    c.text(r.x + 8.0, r.y + 8.0, "COLLECTORS", INK, 2);
    c.text(r.x + 8.0, r.y + 30.0, &format!("COUNT {}", g.collectors.len()), MUTED, 2);
    c.text(r.x + 8.0, r.y + 46.0, &format!("CARRY {}", g.collector_capacity()), MUTED, 2);
    c.text(r.x + 8.0, r.y + 62.0, &format!("SPEED {}", g.collector_speed() as i32), MUTED, 2);

    let r = l.cards[4];
    card(c, a, &r, 0xed7614, true, false);
    c.text(r.x + 8.0, r.y + 8.0, "ASSETS", 0xe69c69, 2);
    c.text(r.x + 8.0, r.y + 28.0, "UI FLAT PACK", MUTED, 2);
    c.text(r.x + 8.0, r.y + 44.0, "GUNS PNG", MUTED, 2);
    c.text(r.x + 8.0, r.y + 60.0, "16X16 EMOJI", MUTED, 2);
    c.blit_fit(&a.ammo, r.x + 8.0, r.y + 80.0, 120.0, 30.0);
}
