//! Walka: spawn wrogow, petla symulacji, obrazenia.

use super::*;

impl Game {
    fn spawn_enemy(&mut self, kind: Kind, reward_mult: f32) {
        let kd = kind_def(kind);
        let lane = if kind == Kind::Boss {
            1
        } else {
            (self.rand() * LANES.len() as f32) as usize % LANES.len()
        };
        let y = if kind == Kind::Boss {
            364.0
        } else {
            LANES[lane] + self.rand_range(-10.0, 10.0)
        };
        let base = if kind == Kind::Boss { 14.0 } else { 10.0 };
        let (hp_mult, speed_mult, cash_mult) =
            (self.plan.hp_mult, self.plan.speed_mult, self.plan.cash_mult);
        let hp = enemy_health(self.wave, base) * kd.hp * hp_mult;
        let cash = (hp as f64 * 0.18 + 2.0)
            * kd.cash
            * reward_mult as f64
            * self.cash_mult() as f64
            * cash_mult as f64;
        let id = self.next_id;
        self.next_id += 1;
        let spawn_x = SPAWN_X + self.rand_range(0.0, 40.0);
        let speed = (kd.speed + self.rand_range(-2.0, 2.0)) * speed_mult;
        let phase = self.rand() * 6.28;
        self.enemies.push(Enemy {
            id,
            x: spawn_x,
            y,
            hp,
            max_hp: hp,
            speed,
            kind,
            size: kd.size,
            cash,
            spawn_wave: self.wave,
            hit: 0.0,
            phase,
        });
    }

    pub fn update(&mut self, dt: f32) {
        if self.game_over || self.paused {
            return;
        }
        let dt = dt.min(0.05) * self.speed.max(0.0);
        self.time += dt;
        self.message_t = (self.message_t - dt).max(0.0);

        // licznik DPS z okna 3 s (UIManager::UpdateDPS)
        let now = self.time;
        self.damage_window.retain(|(t, _)| now - t <= 3.0);
        let dealt: f32 = self.damage_window.iter().map(|(_, d)| *d).sum();
        self.dps = dealt / 3.0;

        // --- fale: GameManager::Update ---
        self.spawn_timer += dt;
        let mut interval = self.plan.interval;
        if is_boss_wave(self.wave) {
            interval *= 1.5;
        }
        if self.spawn_timer >= interval && self.wave != BOSS_WAVE {
            let n = self.plan.batch;
            for _ in 0..n {
                let r = self.rand();
                let kind = waves::roll(&self.plan, r);
                self.spawn_enemy(kind, 1.0 / n as f32);
            }
            self.spawn_timer = 0.0;
        }
        self.wave_time -= dt;
        if self.wave_time <= 0.0 {
            self.wave += 1;
            self.plan = waves::plan(self.wave);
            self.wave_time = WAVE_SECONDS;
            if is_boss_wave(self.wave) {
                self.spawn_enemy(Kind::Boss, 1.0);
            }
            if self.wave > self.highest_wave {
                self.highest_wave = self.wave;
            }
            self.quest_tick(QuestKind::Wave, 1);
        }

        // --- wrogowie ---
        let mut wall_damage = 0.0f64;
        let mut reached: Vec<usize> = Vec::new();
        for (i, e) in self.enemies.iter_mut().enumerate() {
            e.phase += dt * 6.0;
            e.x -= e.speed * dt;
            e.hit = (e.hit - dt * 5.0).max(0.0);
            if e.x <= WALL_X + 12.0 {
                let kd = kind_def(e.kind);
                wall_damage += (e.max_hp as f64 * kd.wall_damage_mult as f64).max(5.0).floor();
                reached.push(i);
            }
        }
        // usuwanie od końca, żeby indeksy pozostały ważne
        for i in reached.into_iter().rev() {
            self.enemies.remove(i);
        }
        if wall_damage > 0.0 {
            self.wall_hp -= wall_damage;
            self.floaters.push(Floater {
                x: WALL_X + 30.0,
                y: 300.0,
                text: format!("-{}", wall_damage as i64),
                t: 1.0,
                color: 0xc42430,
            });
            if self.wall_hp <= 0.0 {
                self.wall_hp = 0.0;
                self.game_over = true;
            }
        }

        // --- obrońcy ---
        // Pętla po indeksach: potrzebujemy &mut tylko na jednym polu naraz,
        // a w środku losujemy i czytamy globalne mnożniki.
        let mut shots: Vec<(f32, f32, u64, f32, f32, bool)> = Vec::new();
        let speed_mult = self.speed_mult();
        let damage_mult = self.damage_mult();
        for i in 0..self.defenders.len() {
            let (slot, def_idx, rarity, cd, weapon, gear) = {
                let d = &self.defenders[i];
                (d.slot, d.def, d.rarity, d.cd, d.weapon, d.gear)
            };
            let mut new_cd = cd - dt;
            let mut fired = false;
            if slot < MAX_SLOTS && new_cd <= 0.0 {
                let (sx, sy) = slot_pos(slot);
                let range = DEFENDERS[def_idx].range;
                let mut best: Option<(u64, f32)> = None;
                for e in self.enemies.iter() {
                    let dx = e.x - sx;
                    let dy = e.y - sy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= range {
                        match best {
                            Some((_, bd)) if bd <= dx => {}
                            _ => best = Some((e.id, dx)),
                        }
                    }
                }
                if let Some((id, _)) = best {
                    let def = &DEFENDERS[def_idx];
                    // Założona broń nadpisuje broń bazową (mnożnik z WeaponType).
                    let (weapon_type, weapon_rarity) = match weapon {
                        Some((idx, r)) => (WEAPON_ITEMS[idx.min(WEAPON_ITEMS.len() - 1)].weapon, r),
                        None => (def.weapon, rarity),
                    };
                    let (gear_dmg, gear_aps) = match gear {
                        Some((idx, _)) => {
                            let gi = &GEAR_ITEMS[idx.min(GEAR_ITEMS.len() - 1)];
                            (gi.damage, gi.aps)
                        }
                        None => (0.0, 0.0),
                    };
                    let mut dmg = def.damage
                        * (1.0 + 0.05 * RARITIES[rarity].stat_mult)
                        * (1.0 + self.dup_count[def_idx] as f32 * DUPLICATE_BONUS[rarity])
                        * damage_mult
                        * weapon_damage_multiplier(weapon_type, weapon_rarity)
                        * (1.0 + gear_dmg);
                    let crit_chance = 0.03 + 0.01 * RARITIES[rarity].stat_mult;
                    let crit = self.rand() < crit_chance;
                    if crit {
                        dmg *= 2.0;
                    }
                    new_cd = 1.0 / (def.aps * speed_mult * (1.0 + gear_aps)).max(0.05);
                    fired = true;
                    shots.push((sx, sy, id, dmg.max(1.0), def.splash, crit));
                }
            }
            let d = &mut self.defenders[i];
            d.cd = new_cd;
            d.flash = if fired { 1.0 } else { (d.flash - dt * 4.0).max(0.0) };
        }
        for (sx, sy, id, dmg, splash, crit) in shots {
            self.projectiles.push(Projectile { x: sx, y: sy, lx: sx, ly: sy, target: id, dmg, splash, crit });
        }

        // --- pociski ---
        let mut hits: Vec<(u64, f32, f32, bool)> = Vec::new();
        let mut dead_proj: Vec<usize> = Vec::new();
        for (i, p) in self.projectiles.iter_mut().enumerate() {
            let target = self.enemies.iter().find(|e| e.id == p.target);
            let (tx, ty) = match target {
                Some(e) => {
                    p.lx = e.x;
                    p.ly = e.y;
                    (e.x, e.y)
                }
                None => (p.lx, p.ly),
            };
            let dx = tx - p.x;
            let dy = ty - p.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let step = 900.0 * dt;
            if dist <= step.max(6.0) {
                hits.push((p.target, p.dmg, p.splash, p.crit));
                dead_proj.push(i);
            } else {
                p.x += dx / dist * step;
                p.y += dy / dist * step;
            }
        }
        for i in dead_proj.into_iter().rev() {
            self.projectiles.remove(i);
        }

        // --- obrażenia ---
        for (id, dmg, splash, crit) in hits {
            if splash > 0.0 {
                let origin = self.enemies.iter().find(|e| e.id == id).map(|e| (e.x, e.y));
                if let Some((ox, oy)) = origin {
                    let mut struck: Vec<(u64, f32)> = Vec::new();
                    for e in self.enemies.iter() {
                        let dx = e.x - ox;
                        let dy = e.y - oy;
                        if (dx * dx + dy * dy).sqrt() <= splash {
                            let falloff = if e.id == id { 1.0 } else { 0.6 };
                            struck.push((e.id, dmg * falloff));
                        }
                    }
                    for (sid, sdmg) in struck {
                        self.apply_damage(sid, sdmg, crit);
                    }
                }
            } else {
                self.apply_damage(id, dmg, crit);
            }
        }

        // --- monety na ziemi: leżą i czekają na zbieracza ---
        let mut dead_coins: Vec<usize> = Vec::new();
        for (i, c) in self.coins.iter_mut().enumerate() {
            c.t += dt;
            if c.t < 0.45 {
                c.x += c.vx * dt;
                c.y += c.vy * dt;
                c.vy += 620.0 * dt;
                if c.y > 556.0 {
                    c.y = 556.0;
                    c.vy *= -0.55;
                    c.vx *= 0.7;
                }
            } else if c.t > 12.0 {
                dead_coins.push(i);
            }
        }
        for i in dead_coins.into_iter().rev() {
            self.coins.remove(i);
        }

        // --- zbieracze monet (CoinCollector::Update) ---
        let col_speed = self.collector_speed();
        let capacity = self.collector_capacity();
        let mut cash_gain = 0.0f64;
        for ci in 0..self.collectors.len() {
            let (mut cx, mut cy, mut taken, mut carried, mut emptying) = {
                let c = &self.collectors[ci];
                (c.x, c.y, c.taken, c.carried, c.emptying)
            };
            if taken >= capacity {
                emptying = true;
            }
            if emptying {
                let dx = COLLECTOR_FORT_X - cx;
                if dx.abs() > 8.0 {
                    cx += dx.signum() * col_speed * dt;
                    cy += (516.0 - cy) * dt * 3.0;
                } else {
                    cash_gain += carried;
                    self.floaters.push(Floater {
                        x: cx,
                        y: cy - 26.0,
                        text: format!("+${:.0}", carried),
                        t: 1.0,
                        color: 0xffc825,
                    });
                    carried = 0.0;
                    taken = 0;
                    emptying = false;
                }
            } else {
                let mut best: Option<(usize, f32, f32)> = None;
                for (i, coin) in self.coins.iter().enumerate() {
                    let dx = coin.x - cx;
                    let dy = coin.y - cy;
                    let d2 = dx * dx + dy * dy;
                    if best.map_or(true, |(_, _, bd)| d2 < bd) {
                        best = Some((i, dx, dy));
                    }
                }
                if let Some((idx, dx, dy)) = best {
                    let d = (dx * dx + dy * dy).sqrt().max(1.0);
                    if d < 16.0 {
                        let coin = self.coins.remove(idx);
                        taken += 1;
                        carried += coin.value;
                    } else {
                        cx += dx / d * col_speed * dt;
                        cy += dy / d * col_speed * dt;
                    }
                } else {
                    cy += (500.0 - cy) * dt * 2.0;
                }
            }
            let c = &mut self.collectors[ci];
            c.x = cx;
            c.y = cy;
            c.taken = taken;
            c.carried = carried;
            c.emptying = emptying;
        }
        if cash_gain > 0.0 {
            self.cash += cash_gain;
            self.total_earned += cash_gain;
        }

        // --- floaters ---
        for f in self.floaters.iter_mut() {
            f.t -= dt;
            f.y -= 42.0 * dt;
        }
        self.floaters.retain(|f| f.t > 0.0);

        // --- autosave (GameManager::Update: co 5 s) ---
        self.autosave_timer += dt;
    }

    fn apply_damage(&mut self, id: u64, dmg: f32, crit: bool) {
        let mut killed: Option<(f32, f32, f64, Kind, u32)> = None;
        let mut floater: Option<Floater> = None;
        let mut dealt = 0.0f32;
        if let Some(e) = self.enemies.iter_mut().find(|e| e.id == id) {
            dealt = dmg.min(e.hp.max(0.0));
            e.hp -= dmg;
            e.hit = 1.0;
            if e.hp <= 0.0 {
                killed = Some((e.x, e.y, e.cash, e.kind, e.spawn_wave));
            } else {
                floater = Some(Floater {
                    x: e.x,
                    y: e.y - 26.0,
                    text: format!("{}{}", if crit { "!" } else { "" }, dmg.round() as i64),
                    t: 0.7,
                    color: if crit { 0xffc825 } else { 0xffffff },
                });
            }
        }
        if let Some(f) = floater {
            self.floaters.push(f);
        }
        // do DPS liczymy tylko realnie zadane obrażenia (bez overkilla)
        self.damage_window.push((self.time, dealt));
        if let Some((x, y, cash, kind, spawn_wave)) = killed {
            self.enemies.retain(|e| e.id != id);
            self.kills += 1;
            self.quest_tick(QuestKind::Kills, 1);
            if kind == Kind::Boss {
                self.bosses_defeated += 1;
                let pts = rebirth_points_for_boss(spawn_wave);
                self.floaters.push(Floater {
                    x,
                    y: y - 50.0,
                    text: format!("BOSS +{}P", pts),
                    t: 1.4,
                    color: 0xffc825,
                });
            }
            // monety: rozbicie na 1..4 (Enemy::AddRewards)
            let count = 1 + (self.rand() * 3.0) as u32;
            let each = cash / count as f64;
            for _ in 0..count {
                let vx = self.rand_range(-70.0, 70.0);
                let vy = self.rand_range(-190.0, -90.0);
                self.coins.push(Coin {
                    x,
                    y,
                    vx,
                    vy,
                    value: each,
                    t: 0.0,
                });
            }
            // doświadczenie
            self.xp += 6.0 + spawn_wave as f64 * 1.5;
            loop {
                let need = xp_for_level(self.level);
                if self.xp >= need {
                    self.xp -= need;
                    self.level += 1;
                    self.floaters.push(Floater {
                        x: WALL_X + 40.0,
                        y: 200.0,
                        text: format!("LEVEL {}", self.level),
                        t: 1.6,
                        color: 0x5ac54f,
                    });
                } else {
                    break;
                }
            }
        }
    }
}
