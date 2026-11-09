//! Emojis vs Zombies — rekonstrukcja w Rust.
//!
//! Uruchomienie:  cargo run --release
//! Autotest:      cargo run --release -- --selftest

mod art;
mod assets;
mod config;
mod game;
mod gfx;
mod render;
mod save;

use assets::Assets;
use config::*;
use game::Game;
use gfx::Canvas;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use render::layout;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--selftest") {
        std::process::exit(selftest());
    }
    if args.iter().any(|a| a == "--diagnose") {
        std::process::exit(diagnose());
    }
    if args.iter().any(|a| a == "--bench") {
        bench();
        return;
    }
    if let Some(pos) = args.iter().position(|a| a == "--shot") {
        let path = args.get(pos + 1).cloned().unwrap_or_else(|| "frame.ppm".to_string());
        let tab = args
            .iter()
            .position(|a| a == "--tab")
            .and_then(|p| args.get(p + 1))
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
            .min(5);
        match shot(&path, tab) {
            Ok(()) => {
                println!("zapisano klatke: {path}");
                return;
            }
            Err(e) => {
                eprintln!("nie mozna zapisac klatki: {e}");
                std::process::exit(1);
            }
        }
    }
    if let Err(err) = run() {
        eprintln!("nie mozna uruchomic okna: {err}");
        eprintln!("sprawdz DISPLAY / WAYLAND_DISPLAY i biblioteki X11");
        std::process::exit(1);
    }
}

fn run() -> Result<(), minifb::Error> {
    let opts = WindowOptions {
        resize: false,
        scale: Scale::X1,
        ..Default::default()
    };
    let mut window = Window::new("Emojis vs Zombies — Rust reconstruction", WIN_W, WIN_H, opts)?;
    window.limit_update_rate(Some(std::time::Duration::from_micros(16_667)));

    let assets = Assets::load();
    let mut canvas = Canvas::new(WIN_W, WIN_H);
    let mut game = save::load().unwrap_or_else(Game::new);
    let mut lay = layout(game.tab);
    let mut last = Instant::now();
    let mut prev_mouse = false;
    let mut prev_right = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32();
        last = now;

        for key in window.get_keys_pressed(KeyRepeat::No) {
            match key {
                Key::Key1 => game.tab = 0,
                Key::Key2 => game.tab = 1,
                Key::Key3 => game.tab = 2,
                Key::Key4 => game.tab = 3,
                Key::F => game.speed = if game.speed >= 3.0 { 1.0 } else { game.speed + 1.0 },
                Key::Space => game.paused = !game.paused,
                Key::F5 => {
                    let _ = save::save(&game);
                    game.log("SAVED");
                }
                Key::B => {
                    game.do_rebirth();
                }
                Key::R => {
                    if game.game_over {
                        game.reset_run();
                        game.log("RUN RESTARTED");
                    }
                }
                _ => {}
            }
        }

        let down = window.get_mouse_down(MouseButton::Left);
        if down && !prev_mouse {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard) {
                handle_click(&mut game, &lay, mx, my);
            }
        }
        prev_mouse = down;

        let right = window.get_mouse_down(MouseButton::Right);
        if right && !prev_right {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard) {
                handle_right_click(&mut game, &lay, mx, my);
            }
        }
        prev_right = right;
        // karty zależą od zakładki — przeliczamy tylko przy zmianie
        if lay.tab != game.tab {
            lay = layout(game.tab);
        }
        lay.mouse = window.get_mouse_pos(MouseMode::Discard);

        game.update(dt);
        if game.autosave_timer >= 5.0 {
            game.autosave_timer = 0.0;
            let _ = save::save(&game);
        }

        render::draw(&mut canvas, &game, &lay, &assets);

        window.update_with_buffer(&canvas.buf, WIN_W, WIN_H)?;
    }

    let _ = save::save(&game);
    Ok(())
}

/// Mierzy czas rysowania klatki — punkt odniesienia dla optymalizacji.
fn bench() {
    let mut g = Game::new();
    g.cash = 900.0;
    for slot in 0..4 {
        g.place_defender(slot, slot);
    }
    let dt = 1.0 / 60.0;
    for _ in 0..900 {
        g.update(dt);
    }
    let assets = Assets::load();
    let lay = layout(0);
    let mut canvas = Canvas::new(WIN_W, WIN_H);
    let frames = 300;
    let start = std::time::Instant::now();
    for _ in 0..frames {
        render::draw(&mut canvas, &g, &lay, &assets);
    }
    let ms = start.elapsed().as_secs_f64() * 1000.0 / frames as f64;
    println!("bench: {:.2} ms/klatke ({frames} klatek, wrogowie={})", ms, g.enemies.len());
}

/// Renderuje deterministyczną klatkę rozgrywki do pliku PPM (P6).
/// Pozwala obejrzeć oprawę bez otwierania okna i bez zrzutów ekranu.
fn shot(path: &str, tab: usize) -> std::io::Result<()> {
    let mut g = Game::new();
    g.cash = 400.0;
    g.tab = tab;
    g.rebirth_points = 12;
    g.highest_wave = 14;
    g.upgrades = [3, 2, 1, 2, 3, 0];
    for slot in 0..4 {
        g.place_defender(slot, slot % DEFENDERS.len());
    }
    let dt = 1.0 / 60.0;
    for _ in 0..(14.0 / dt) as usize {
        g.update(dt);
    }
    g.selected = Some(1);
    g.cash = 512.0;
    // przykladowe monety i pociski, zeby podglad pokazywal nowa grafike
    for (i, (x, y)) in [(700.0f32, 470.0f32), (760.0, 500.0), (820.0, 466.0)].iter().enumerate() {
        g.coins.push(crate::game::Coin {
            x: *x,
            y: *y,
            vx: 0.0,
            vy: 0.0,
            value: 12.0 + i as f64,
            t: 1.0 + i as f32 * 0.4,
        });
    }
    g.projectiles.push(crate::game::Projectile {
        x: 470.0,
        y: 250.0,
        lx: 560.0,
        ly: 244.0,
        target: 0,
        dmg: 10.0,
        splash: 0.0,
        crit: false,
    });
    // przykladowe drop-y, zeby zakladka GEAR miala co pokazac
    g.add_item(ItemKind::Weapon, 3, 2);
    g.add_item(ItemKind::Gear, 1, 1);
    g.add_item(ItemKind::Weapon, 0, 4);

    let assets = Assets::load();
    let mut canvas = Canvas::new(WIN_W, WIN_H);
    let lay = layout(tab);
    render::draw(&mut canvas, &g, &lay, &assets);

    let mut out: Vec<u8> = Vec::with_capacity(WIN_W * WIN_H * 3 + 32);
    out.extend_from_slice(format!("P6\n{} {}\n255\n", WIN_W, WIN_H).as_bytes());
    for px in &canvas.buf {
        out.push(((px >> 16) & 0xff) as u8);
        out.push(((px >> 8) & 0xff) as u8);
        out.push((px & 0xff) as u8);
    }
    std::fs::write(path, out)
}

/// Losowe akcje gracza + kontrola niezmienników po każdej klatce.
fn diagnose() -> i32 {
    let mut issues: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let report = |issues: &mut Vec<String>, seen: &mut std::collections::HashSet<String>, msg: String| {
        let key = msg.split(' ').take(3).collect::<Vec<_>>().join(" ");
        if seen.insert(key) && issues.len() < 40 {
            issues.push(msg);
        }
    };

    let mut g = Game::new();
    g.cash = 2500.0;
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    let mut seed: u64 = 0xC0FFEE_1234_5678;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut swaps = 0;
    let mut removals = 0;
    let mut equips = 0;
    let mut packs = 0;

    let mut frame = 0u32;
    while t < 900.0 {
        frame += 1;
        // --- losowe akcje gracza, ~dwie akcje na sekunde (nie co klatka) ---
        let act = frame % 30 == 0;
        let roll = if act { next() % 100 } else { 100 };
        if roll < 12 && g.cash > 200.0 {
            g.buy_pack((next() as usize) % PACKS.len());
            packs += 1;
        } else if roll < 30 {
            let slot = (next() as usize) % g.max_slots();
            let def = (next() as usize) % DEFENDERS.len();
            if g.place_defender(slot, def) {
                swaps += 1;
            }
        } else if roll < 36 {
            let slot = (next() as usize) % g.max_slots();
            if g.remove_defender(slot) {
                removals += 1;
            }
        } else if roll < 48 && g.cash > 300.0 {
            g.buy_upgrade((next() as usize) % UPGRADES.len());
        } else if roll < 58 && !g.inventory.is_empty() {
            if let Some(di) = g.defenders.iter().position(|d| d.slot != usize::MAX) {
                let inv = (next() as usize) % g.inventory.len();
                if g.equip_item(di, inv) {
                    equips += 1;
                }
            }
        } else if roll < 62 {
            if let Some(di) = g.defenders.iter().position(|d| d.slot != usize::MAX) {
                g.unequip(di, next() % 2 == 0);
            }
        } else if roll < 64 && g.can_rebirth() {
            g.do_rebirth();
        }

        g.speed = 3.0;
        g.update(dt);
        if g.game_over {
            g.reset_run();
        }
        t += dt;

        // --- niezmienniki ---
        let at = format!("t={:.1}s wave={}", t, g.wave);
        if !g.cash.is_finite() || g.cash < 0.0 {
            report(&mut issues, &mut seen, format!("gotowka nieprawidlowa ({}) {}", g.cash, at));
        }
        if !g.wall_hp.is_finite() || g.wall_hp < 0.0 || g.wall_hp > g.wall_max() + 0.001 {
            report(&mut issues, &mut seen, format!("HP muru poza zakresem ({}) {}", g.wall_hp, at));
        }
        if !g.dps.is_finite() || g.dps < 0.0 {
            report(&mut issues, &mut seen, format!("DPS nieprawidlowy ({}) {}", g.dps, at));
        }
        if g.defenders.iter().any(|d| d.slot >= g.max_slots() || d.slot >= crate::game::MAX_SLOTS) {
            report(&mut issues, &mut seen, format!("obronca w zablokowanym slocie {}", at));
        }
        let mut slots: Vec<usize> = g.defenders.iter().map(|d| d.slot).collect();
        slots.sort_unstable();
        let before = slots.len();
        slots.dedup();
        if slots.len() != before {
            report(&mut issues, &mut seen, format!("duplikat slotu obroncow {}", at));
        }
        if g.defenders.iter().any(|d| d.def >= DEFENDERS.len() || d.rarity > 5) {
            report(&mut issues, &mut seen, format!("obronca z nieprawidlowa postacia/rzadkoscia {}", at));
        }
        if g.char_rarity.iter().any(|r| *r > 5) {
            report(&mut issues, &mut seen, format!("rzadkosc postaci poza zakresem {}", at));
        }
        if g.inventory.iter().any(|i| i.count == 0 || i.rarity > 5) {
            report(&mut issues, &mut seen, format!("zly wpis ekwipunku {}", at));
        }
        if g.inventory.iter().any(|i| {
            let max = match i.kind {
                ItemKind::Weapon => WEAPON_ITEMS.len(),
                ItemKind::Gear => GEAR_ITEMS.len(),
            };
            i.idx >= max
        }) {
            report(&mut issues, &mut seen, format!("indeks przedmiotu poza tabela {}", at));
        }
        if g.enemies.iter().any(|e| {
            !e.x.is_finite() || !e.hp.is_finite() || e.hp > e.max_hp + 0.01 || e.max_hp <= 0.0
        }) {
            report(&mut issues, &mut seen, format!("wróg z nieprawidlowym stanem {}", at));
        }
        if g.coins.iter().any(|c| !c.value.is_finite() || c.value < 0.0) {
            report(&mut issues, &mut seen, format!("moneta z nieprawidlowa wartoscia {}", at));
        }
        if g.collectors.iter().any(|c| !c.x.is_finite() || !c.carried.is_finite() || c.carried < 0.0) {
            report(&mut issues, &mut seen, format!("zbieracz z nieprawidlowym stanem {}", at));
        }
        if g.wave == 0 {
            report(&mut issues, &mut seen, format!("fala zerowa {}", at));
        }
        if g.pack_level.iter().any(|l| *l > 40) {
            report(&mut issues, &mut seen, format!("poziom paczki poza skala {}", at));
        }
    }

    // --- scenariusze graniczne ---
    let mut edge = Game::new();
    edge.cash = 0.0;
    let no_cash_place = edge.place_defender(0, 3);
    if no_cash_place || !edge.defenders.is_empty() || edge.cash < 0.0 {
        report(&mut issues, &mut seen, "postawienie bez gotowki lamie ekonomie".to_string());
    }
    let mut edge2 = Game::new();
    edge2.cash = 100000.0;
    edge2.upgrades[4] = 99;
    let slots_cap = edge2.max_slots();
    let (last_x, last_y) = crate::game::slot_pos(slots_cap - 1);
    if slots_cap != 4 + 99 || !last_x.is_finite() || !last_y.is_finite() {
        report(&mut issues, &mut seen, format!("sloty nie rosna z ulepszeniem ({})", slots_cap));
    }
    // ekwipunek: nie da sie zalozyc dwa razy tego samego przedmiotu
    edge2.add_item(ItemKind::Weapon, 0, 0);
    edge2.place_defender(0, 0);
    let first = edge2.equip_item(0, 0);
    let second = edge2.inventory.is_empty();
    if !first || !second {
        report(&mut issues, &mut seen, "zakladanie przedmiotu gubi albo duplikuje wpis".to_string());
    }

    println!("diagnostyka: symulacja {:.0}s, akcje: podmiany={} usuniecia={} zalozenia={} paczki={}", t, swaps, removals, equips, packs);
    println!("diagnostyka: fala={} najwyzsza={} zabojstwa={} poziom={}", g.wave, g.highest_wave, g.kills, g.level);
    if issues.is_empty() {
        println!("DIAGNOSTYKA OK - brak naruszen niezmiennikow");
        0
    } else {
        println!("DIAGNOSTYKA: {} problemow", issues.len());
        for i in &issues {
            println!("  - {}", i);
        }
        1
    }
}

fn handle_click(g: &mut Game, lay: &render::Layout, mx: f32, my: f32) {
    // pasek górny
    if lay.btn_pause.contains(mx, my) {
        g.paused = !g.paused;
        return;
    }
    if lay.btn_save.contains(mx, my) {
        let _ = save::save(g);
        g.log("SAVED");
        return;
    }

    // zakładki
    for (i, r) in lay.tabs.iter().enumerate() {
        if r.contains(mx, my) {
            g.tab = i;
            return;
        }
    }

    if g.game_over {
        if lay.btn_restart.contains(mx, my) {
            g.reset_run();
            g.log("RUN RESTARTED");
        }
        return;
    }

    // karty panelu
    for (i, r) in lay.cards.iter().enumerate() {
        if !r.contains(mx, my) {
            continue;
        }
        match g.tab {
            0 => {
                if i < DEFENDERS.len() {
                    g.selected = if g.selected == Some(i) { None } else { Some(i) };
                }
            }
            1 => {
                if i < UPGRADES.len() {
                    g.buy_upgrade(i);
                }
            }
            2 => {
                if i < REBIRTH_UPGRADES.len() {
                    g.buy_rebirth_upgrade(i);
                } else if i == 4 {
                    g.do_rebirth();
                }
            }
            _ => match i {
                0 => g.speed = if g.speed >= 3.0 { 1.0 } else { g.speed + 1.0 },
                1 => {
                    let _ = save::save(g);
                    g.log("SAVED");
                }
                2 => {
                    if g.reset_armed {
                        g.hard_reset();
                        let _ = save::save(g);
                        g.log("GAME RESET");
                        g.reset_armed = false;
                    } else {
                        g.reset_armed = true;
                        g.log("CLICK AGAIN TO RESET");
                    }
                }
                _ => {}
            },
        }
        return;
    }

    // sloty na polu: klik z wyborem = postaw/podmień, klik bez wyboru = wybierz obsadzonego
    for (i, r) in lay.slots.iter().enumerate() {
        if i >= g.max_slots() {
            continue;
        }
        if r.contains(mx, my) {
            match g.selected {
                Some(def_idx) => {
                    g.place_defender(i, def_idx);
                }
                None => {
                    if let Some(d) = g.defenders.iter().find(|d| d.slot == i) {
                        g.selected = Some(d.def);
                        g.log("RIGHT CLICK SLOT TO REMOVE");
                    }
                }
            }
            return;
        }
    }
}

fn handle_right_click(g: &mut Game, lay: &render::Layout, mx: f32, my: f32) {
    for (i, r) in lay.slots.iter().enumerate() {
        if i < g.max_slots() && r.contains(mx, my) {
            g.remove_defender(i);
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Autotest formuł i symulacji
// ---------------------------------------------------------------------------

fn approx(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() <= eps
}

fn selftest() -> i32 {
    let mut failed = 0;
    let mut check = |name: &str, ok: bool| {
        println!("[{}] {}", if ok { " OK " } else { "FAIL" }, name);
        if !ok {
            failed += 1;
        }
    };

    check("enemy_health(wave 1, base 10) == 11", approx(enemy_health(1, 10.0), 11.0, 0.001));
    // 2 * 1.15^5 * 10 * 0.5 = 20.11 -> floor 20
    check("enemy_health(wave 5, base 10) == 20", approx(enemy_health(5, 10.0), 20.0, 0.001));
    check(
        "enemy_health rosnie z fala",
        enemy_health(30, 10.0) > enemy_health(20, 10.0) && enemy_health(20, 10.0) > enemy_health(10, 10.0),
    );
    // proceduralne fale: deterministyczne, sensowne i trudniejsze z numerem
    let p1 = crate::game::waves::plan(1);
    let p1_again = crate::game::waves::plan(1);
    check(
        "plan fali jest deterministyczny",
        (p1.hp_mult - p1_again.hp_mult).abs() < 1e-6 && (p1.interval - p1_again.interval).abs() < 1e-6,
    );
    check(
        "plany 1..200 maja sens",
        (1..200).all(|w| {
            let p = crate::game::waves::plan(w);
            let sum: f32 = p.weights.iter().sum();
            p.interval > 0.0 && p.batch >= 1 && p.hp_mult > 0.0 && p.speed_mult > 0.0 && (sum - 1.0).abs() < 0.02
        }),
    );
    check(
        "fala 30 trudniejsza niz 3",
        enemy_health(30, 10.0) * crate::game::waves::plan(30).hp_mult
            > enemy_health(3, 10.0) * crate::game::waves::plan(3).hp_mult * 10.0,
    );
    check(
        "sklad wrogow zmienia sie z fala",
        crate::game::waves::plan(30).weights[1] > crate::game::waves::plan(3).weights[1],
    );
    check(
        "archetypy fal sie roznia",
        (1..9).map(|w| crate::game::waves::plan(w).name).collect::<std::collections::HashSet<_>>().len() > 1,
    );
    check("is_boss_wave 5/10, nie 6", is_boss_wave(5) && is_boss_wave(10) && !is_boss_wave(6));
    check("xp_for_level 0/3", xp_for_level(0) == 20.0 && xp_for_level(3) == 160.0);
    check(
        "milestone 25/26 (x1 / x5)",
        approx(milestone_multiplier(25) as f32, 1.0, 0.001) && approx(milestone_multiplier(26) as f32, 5.0, 0.001),
    );
    check("upgrade_price(base 30, lvl 1) == 30", upgrade_price(30.0, 1.35, 1) == 30.0);
    check(
        "upgrade_price skacze x5 na 26 poziomie",
        upgrade_price(30.0, 1.35, 26) >= upgrade_price(30.0, 1.35, 25) * 4.9,
    );
    check("rebirth_points_for_boss(25) == 5", rebirth_points_for_boss(25) == 5);
    check(
        "calculate_rebirth_points",
        calculate_rebirth_points(20, 3) == 3 + 2 + 4,
    );
    check("weapon mult sniper/common == 2.0", approx(weapon_damage_multiplier("sniper", 0), 2.0, 0.001));
    check(
        "weapon mult sniper/mythic == 4.0",
        approx(weapon_damage_multiplier("sniper", 5), 4.0, 0.001),
    );
    check("tabele paczek sumuja sie do 1", PACK_TABLES.iter().all(|t| approx(t.iter().sum::<f32>(), 1.0, 0.01)));
    check("roll_rarity dol/gore", roll_rarity(0, 0.0) == 0 && roll_rarity(9, 0.999) == 5);

    // symulacja 600 s z automatycznym rozmieszczaniem obroncow
    let mut g = Game::new();
    g.cash = 500.0;
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    let mut next_def = 0usize;
    let mut restarts = 0;
    while t < 600.0 {
        for slot in 0..g.max_slots() {
            if !g.slot_occupied(slot) {
                let idx = next_def % DEFENDERS.len();
                if g.place_defender(slot, idx) {
                    next_def += 1;
                }
            }
        }
        if g.cash > g.upgrade_cost(0) * 1.5 {
            g.buy_upgrade(0);
        }
        if g.game_over {
            g.reset_run();
            restarts += 1;
        }
        g.update(dt);
        t += dt;
    }
    check("symulacja: sa zabojstwa", g.kills > 0);
    check("symulacja: fale postepuja", g.highest_wave > 1);
    check("symulacja: mur ma skonczone HP", g.wall_hp.is_finite() && g.wall_hp >= 0.0);
    check("symulacja: gotowka skonczona", g.cash.is_finite());

    // kazda postawiona kopia kosztuje tyle, ile wynosi cena postaci
    let mut paid = Game::new();
    paid.cash = 1000.0;
    let price = DEFENDERS[0].cost;
    paid.place_defender(0, 0);
    let after_one = paid.cash;
    paid.place_defender(1, 0);
    let after_two = paid.cash;
    check(
        "kazda kopia obroncy kosztuje",
        (after_one - (1000.0 - price)).abs() < 0.001
            && (after_two - (1000.0 - price * 2.0)).abs() < 0.001,
    );
    let mut broke = Game::new();
    broke.cash = price - 1.0;
    check(
        "brak gotowki blokuje postawienie",
        !broke.place_defender(0, 0) && broke.defenders.is_empty() && (broke.cash - (price - 1.0)).abs() < 0.001,
    );

    // podmiana i usuwanie obrońców (zgloszony blad: brak wymiany slabych emoji)
    let mut g2 = Game::new();
    g2.cash = 1000.0;
    g2.place_defender(0, 0);
    let swapped = g2.place_defender(0, 1);
    check(
        "podmiana obrońcy w tym samym slocie",
        swapped && g2.defenders.len() == 1 && g2.defenders[0].def == 1,
    );
    g2.add_item(ItemKind::Weapon, 2, 1);
    let before = g2.inventory.len();
    g2.equip_item(0, 0);
    let equipped = g2.defenders[0].weapon.is_some();
    let returned = g2.remove_defender(0);
    check(
        "usuniecie obrońcy zwraca sprzęt",
        returned && g2.defenders.is_empty() && equipped && g2.inventory.len() == before,
    );
    let unequipped = {
        g2.place_defender(0, 0);
        g2.equip_item(0, 0);
        g2.unequip(0, true) && g2.defenders[0].weapon.is_none() && !g2.inventory.is_empty()
    };
    check("zdejmowanie broni wraca do ekwipunku", unequipped);

    // ulepszenie zbieraczy: wiecej zbieraczy, wieksza pojemność
    let mut cg = Game::new();
    cg.cash = 100_000.0;
    let c0 = cg.collectors.len();
    let cap0 = cg.collector_capacity();
    let spd0 = cg.collector_speed();
    cg.buy_upgrade(COLLECTOR_UPGRADE);
    check("ulepszenie COLLECTOR dodaje zbieracza", cg.collectors.len() == c0 + 1);
    check(
        "ulepszenie COLLECTOR zwieksza pojemnosc i tempo",
        cg.collector_capacity() > cap0 && cg.collector_speed() > spd0,
    );
    for _ in 0..10 {
        cg.buy_upgrade(COLLECTOR_UPGRADE);
    }
    check("zbieracze rosna bez limitu", cg.collectors.len() == c0 + 11);

    // sloty tez nie maja sufitu
    let mut sg = Game::new();
    sg.upgrades[4] = 12;
    check("sloty rosna z ulepszeniem", sg.max_slots() == 16);

    // pelny reset nie moze zostawic zadnego sladu poprzedniego stanu
    let mut rg = Game::new();
    rg.cash = 5000.0;
    rg.wave = 12;
    rg.highest_wave = 30;
    rg.rebirth_points = 9;
    rg.rebirth_upgrades = [2, 1, 0, 3];
    rg.upgrades = [1, 2, 3, 4, 5, 6];
    rg.pack_level = [4, 3, 2, 1, 0];
    rg.unlocked = [true; 7];
    rg.char_rarity = [5; 7];
    rg.dup_count = [3; 7];
    rg.add_item(ItemKind::Weapon, 0, 0);
    rg.place_defender(0, 1);
    rg.hard_reset();
    let fresh = Game::new();
    check(
        "pelny reset kasuje caly postep",
        rg.cash == fresh.cash
            && rg.wave == 1
            && rg.highest_wave == 1
            && rg.rebirth_points == 0
            && rg.upgrades == [0; 6]
            && rg.rebirth_upgrades == [0; 4]
            && rg.pack_level == [0; 5]
            && rg.unlocked[0]
            && !rg.unlocked[1..].iter().any(|u| *u)
            && rg.char_rarity == [0; 7]
            && rg.dup_count == [0; 7]
            && rg.inventory.is_empty()
            && rg.defenders.is_empty()
            && rg.collectors.len() == 1
            && !rg.game_over,
    );

    // paczki nie panikuja i trzymaja zakres rzadkosci
    let mut ok_packs = true;
    g.cash = 1_000_000.0;
    for i in 0..PACKS.len() {
        for _ in 0..40 {
            g.buy_pack(i);
        }
        if g.packs_opened == 0 {
            ok_packs = false;
        }
    }
    check("paczki otwieraja sie bez bledu", ok_packs && g.packs_opened > 0);

    // zapis / odczyt
    let tmp = std::env::temp_dir().join("evz_selftest_home");
    let _ = std::fs::create_dir_all(&tmp);
    std::env::set_var("HOME", &tmp);
    let saved_cash = g.cash;
    g.speed = 3.0;
    let wrote = save::save(&g).is_ok();
    let loaded = save::load();
    check("zapis i odczyt stanu", wrote && loaded.is_some());
    if let Some(lg) = loaded {
        check("odczyt zachowuje gotowke", (lg.cash - saved_cash).abs() < 0.001);
        check("odczyt zachowuje obroncow", lg.defenders.len() == g.defenders.len());
        check("odczyt zachowuje tempo gry", (lg.speed - 3.0).abs() < 0.001);
    }

    // uszkodzony zapis nie moze wysypac gry ani wprowadzic smieci
    let dir = tmp.join(".local/share/emojis_vs_zombies");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(
        dir.join("save.txt"),
        "version=1\nspeed=99\nchar_rarity=99,99,99,99,99,99,99\nupgrades=1,2,3,4,5,999\n\
         defenders=0:0:99;9:2:1;0:99:1\ninventory=W:99:99:1;G:0:7:2\ndups=1,2,3\n",
    );
    let ok_corrupt = match save::load() {
        Some(cg2) => {
            cg2.char_rarity.iter().all(|r| *r <= 5)
                && cg2.speed <= 3.0
                && cg2.defenders.iter().all(|d| {
                    d.def < DEFENDERS.len() && d.rarity <= 5 && d.slot < crate::game::MAX_SLOTS
                })
                && cg2.inventory.iter().all(|i| {
                    i.rarity <= 5
                        && match i.kind {
                            ItemKind::Weapon => i.idx < WEAPON_ITEMS.len(),
                            ItemKind::Gear => i.idx < GEAR_ITEMS.len(),
                        }
                })
        }
        None => false,
    };
    check("uszkodzony zapis przycinany bez paniki", ok_corrupt);

    // reset zapisany na dysk nie moze wrocic po restarcie
    let mut pg = Game::new();
    pg.cash = 900.0;
    pg.unlocked = [true; 7];
    pg.hard_reset();
    let _ = save::save(&pg);
    let survived = save::load()
        .map(|r| r.cash == pg.cash && r.unlocked[1..].iter().all(|u| !*u))
        .unwrap_or(false);
    check("reset przezywa restart gry", survived);

    println!();
    println!(
        "symulacja: fala={} zabojstwa={} gotowka={:.0} restarty={} paczki={}",
        g.highest_wave, g.kills, g.cash, restarts, g.packs_opened
    );
    if failed == 0 {
        println!("SELFTEST OK");
        0
    } else {
        println!("SELFTEST FAILED: {} problemow", failed);
        1
    }
}
