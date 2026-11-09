//! Ekonomia: ekwipunek, obroncy, ulepszenia, paczki, prestiz.

use super::*;

impl Game {
    pub(crate) fn return_equipment(&mut self, d: &Defender) {
        if let Some((idx, r)) = d.weapon {
            self.add_item(ItemKind::Weapon, idx, r);
        }
        if let Some((idx, r)) = d.gear {
            self.add_item(ItemKind::Gear, idx, r);
        }
    }

    /// Stawia postać w slocie. Każda postawiona kopia kosztuje tyle, ile wynosi
    /// cena postaci — posiadanie postaci nie daje darmowych kopii.
    /// Jeśli slot jest zajęty, podmienia obrońcę (stary oddaje sprzęt).
    pub fn place_defender(&mut self, slot: usize, def_idx: usize) -> bool {
        if slot >= self.max_slots() {
            self.log("SLOT LOCKED");
            return false;
        }
        if def_idx >= DEFENDERS.len() {
            return false;
        }
        let cost = DEFENDERS[def_idx].cost;
        if self.cash < cost {
            self.log("NOT ENOUGH CASH");
            return false;
        }
        self.cash -= cost;
        self.unlocked[def_idx] = true;
        if let Some(pos) = self.defenders.iter().position(|d| d.slot == slot) {
            let old = self.defenders.remove(pos);
            self.return_equipment(&old);
        }
        let rarity = self.char_rarity[def_idx];
        self.defenders.push(Defender {
            slot,
            def: def_idx,
            rarity,
            cd: 0.0,
            flash: 0.0,
            weapon: None,
            gear: None,
        });
        self.selected = Some(def_idx);
        self.log(format!("{} DEPLOYED -${}", DEFENDERS[def_idx].name, cost as i64));
        true
    }

    /// Usuwa obrońcę ze slotu, zwracając sprzęt do ekwipunku.
    pub fn remove_defender(&mut self, slot: usize) -> bool {
        if let Some(pos) = self.defenders.iter().position(|d| d.slot == slot) {
            let d = self.defenders.remove(pos);
            self.return_equipment(&d);
            self.log(format!("{} REMOVED", DEFENDERS[d.def].name));
            true
        } else {
            false
        }
    }

    /// Zdejmuje broń albo sprzęt z obrońcy (wraca do ekwipunku).
    pub fn unequip(&mut self, def_index: usize, is_weapon: bool) -> bool {
        if def_index >= self.defenders.len() {
            return false;
        }
        let taken = if is_weapon {
            self.defenders[def_index].weapon.take()
        } else {
            self.defenders[def_index].gear.take()
        };
        match taken {
            Some((idx, r)) => {
                if is_weapon {
                    self.add_item(ItemKind::Weapon, idx, r);
                } else {
                    self.add_item(ItemKind::Gear, idx, r);
                }
                self.log("UNEQUIPPED");
                true
            }
            None => false,
        }
    }

    pub fn buy_upgrade(&mut self, i: usize) -> bool {
        let cost = self.upgrade_cost(i);
        if self.cash < cost {
            self.log("NOT ENOUGH CASH");
            return false;
        }
        self.cash -= cost;
        self.upgrades[i] += 1;
        if i == 3 {
            self.wall_hp = self.wall_max();
        }
        if i == COLLECTOR_UPGRADE {
            self.spawn_collector();
        }
        self.log(format!("{} LVL {}", UPGRADES[i].name, self.upgrades[i]));
        true
    }

    pub fn buy_pack(&mut self, i: usize) -> bool {
        let pack = &PACKS[i];
        if self.cash < pack.cost {
            self.log("NOT ENOUGH CASH");
            return false;
        }
        self.cash -= pack.cost;
        self.packs_opened += 1;

        // PackUI::AddXP + GetXPForLevel: paczka sama leveluje i podnosi tier tabeli.
        let xp_base = 10.0;
        let gained = 12.0 + self.rand() as f64 * 6.0;
        self.pack_xp[i] += gained;
        loop {
            let need = pack_xp_for_level(self.pack_level[i], xp_base) as f64;
            if self.pack_xp[i] >= need && need > 0.0 {
                self.pack_xp[i] -= need;
                self.pack_level[i] += 1;
            } else {
                break;
            }
        }

        let tier = (pack.tier + self.pack_level[i] as usize).min(PACK_TABLES.len() - 1);
        let rarity = roll_rarity(tier, self.rand());
        // CategoryType: None (character) / Unique (weapon x3) / Special (item x6)
        let cat_roll = self.rand();

        if cat_roll < 0.70 {
            let target = (self.rand() * DEFENDERS.len() as f32) as usize;
            if self.unlocked[target] {
                self.dup_count[target] += 1;
                let bonus = DUPLICATE_BONUS[rarity] * 100.0;
                self.log(format!("DUP {} +{:.1}%", DEFENDERS[target].name, bonus));
            } else {
                self.unlocked[target] = true;
                self.log(format!("NEW {} {}", RARITIES[rarity].name.to_uppercase(), DEFENDERS[target].name));
            }
            // roster trzyma najlepszą rzadkość; postawieni obrońcy dziedziczą ją
            if rarity > self.char_rarity[target] {
                self.char_rarity[target] = rarity;
            }
            let cr = self.char_rarity[target];
            for d in self.defenders.iter_mut() {
                if d.def == target {
                    d.rarity = cr;
                }
            }
        } else if cat_roll < 0.90 {
            let idx = (self.rand() * WEAPON_ITEMS.len() as f32) as usize;
            self.add_item(ItemKind::Weapon, idx, rarity);
            self.log(format!("{} {}", RARITIES[rarity].name.to_uppercase(), WEAPON_ITEMS[idx].name));
        } else {
            let idx = (self.rand() * GEAR_ITEMS.len() as f32) as usize;
            self.add_item(ItemKind::Gear, idx, rarity);
            self.log(format!("{} {}", RARITIES[rarity].name.to_uppercase(), GEAR_ITEMS[idx].name));
        }
        self.quest_tick(QuestKind::Packs, 1);
        true
    }

    pub fn can_rebirth(&self) -> bool {
        self.highest_wave >= 10
    }

    pub fn rebirth_gain(&self) -> u32 {
        calculate_rebirth_points(self.highest_wave, self.bosses_defeated)
    }

    pub fn do_rebirth(&mut self) -> bool {
        if !self.can_rebirth() {
            self.log("REACH WAVE 10 FIRST");
            return false;
        }
        let gained = self.rebirth_gain();
        self.rebirth_points += gained;
        self.reset_run();
        self.rebirth_upgrades = [0; 4];
        self.log(format!("REBIRTH +{} PTS", gained));
        true
    }

    pub fn buy_rebirth_upgrade(&mut self, i: usize) -> bool {
        let cost = self.rebirth_upgrade_cost(i);
        if self.rebirth_points < cost {
            self.log("NOT ENOUGH POINTS");
            return false;
        }
        self.rebirth_points -= cost;
        self.rebirth_upgrades[i] += 1;
        self.wall_hp = self.wall_max();
        self.log(format!("{} LVL {}", REBIRTH_UPGRADES[i].name, self.rebirth_upgrades[i]));
        true
    }

    /// Restart po śmierci / po prestiżu: fale od nowa, zachowane ulepszenia.
    pub fn reset_run(&mut self) {
        self.wave = 1;
        self.plan = waves::plan(1);
        self.wave_time = WAVE_SECONDS;
        self.spawn_timer = 0.0;
        self.enemies.clear();
        self.projectiles.clear();
        self.coins.clear();
        self.wall_hp = self.wall_max();
        self.game_over = false;
        self.bosses_defeated = 0;
        self.dps = 0.0;
        self.damage_window.clear();
        self.floaters.clear();
        self.quest_progress = 0;
        for c in self.collectors.iter_mut() {
            c.x = 300.0;
            c.y = 520.0;
            c.taken = 0;
            c.carried = 0.0;
            c.emptying = false;
        }
    }

    /// Kasuje caly postep: postacie, ekwipunek, ulepszenia, prestiz i statystyki.
    pub fn hard_reset(&mut self) {
        *self = Game::new();
    }

}
