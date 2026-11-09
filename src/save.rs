//! Zapis stanu do pliku w katalogu użytkownika.
//! Format jest celowo prosty (key=value), żeby obejść się bez serde.

use crate::config::{DEFENDERS, GEAR_ITEMS, ItemKind, WEAPON_ITEMS};
use crate::game::{Defender, Game, InvItem, MAX_SLOTS};
use std::fs;
use std::path::PathBuf;

fn save_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share/emojis_vs_zombies/save.txt")
}

pub fn save(g: &Game) -> std::io::Result<PathBuf> {
    let path = save_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut out = String::new();
    out.push_str("version=1\n");
    out.push_str(&format!("cash={}\n", g.cash));
    out.push_str(&format!("wave={}\n", g.wave));
    out.push_str(&format!("highest_wave={}\n", g.highest_wave));
    out.push_str(&format!("wall_hp={}\n", g.wall_hp));
    out.push_str(&format!("level={}\n", g.level));
    out.push_str(&format!("xp={}\n", g.xp));
    out.push_str(&format!("kills={}\n", g.kills));
    out.push_str(&format!("bosses_defeated={}\n", g.bosses_defeated));
    out.push_str(&format!("packs_opened={}\n", g.packs_opened));
    out.push_str(&format!("rebirth_points={}\n", g.rebirth_points));
    out.push_str(&format!("weapon_bonus={}\n", g.weapon_bonus));
    out.push_str(&format!("item_bonus={}\n", g.item_bonus));
    out.push_str(&format!("total_earned={}\n", g.total_earned));
    out.push_str(&format!("speed={}\n", g.speed));
    out.push_str(&format!(
        "upgrades={}\n",
        g.upgrades.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    ));
    out.push_str(&format!(
        "rebirth_upgrades={}\n",
        g.rebirth_upgrades.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    ));
    out.push_str(&format!(
        "unlocked={}\n",
        g.unlocked.iter().map(|v| if *v { "1" } else { "0" }).collect::<Vec<_>>().join("")
    ));
    out.push_str(&format!(
        "dups={}\n",
        g.dup_count.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    ));
    out.push_str(&format!(
        "char_rarity={}\n",
        g.char_rarity.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    ));
    out.push_str(&format!(
        "pack_level={}\n",
        g.pack_level.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    ));
    out.push_str(&format!(
        "pack_xp={}\n",
        g.pack_xp.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    ));
    out.push_str(&format!(
        "inventory={}\n",
        g.inventory
            .iter()
            .map(|i| format!(
                "{}:{}:{}:{}",
                match i.kind {
                    ItemKind::Weapon => "W",
                    ItemKind::Gear => "G",
                },
                i.idx,
                i.rarity,
                i.count
            ))
            .collect::<Vec<_>>()
            .join(";")
    ));
    out.push_str(&format!(
        "defenders={}\n",
        g.defenders
            .iter()
            .map(|d| format!(
                "{}:{}:{}:{}:{}:{}:{}",
                d.slot,
                d.def,
                d.rarity,
                d.weapon.map(|(i, _)| i as i64).unwrap_or(-1),
                d.weapon.map(|(_, r)| r as i64).unwrap_or(-1),
                d.gear.map(|(i, _)| i as i64).unwrap_or(-1),
                d.gear.map(|(_, r)| r as i64).unwrap_or(-1)
            ))
            .collect::<Vec<_>>()
            .join(";")
    ));
    fs::write(&path, out)?;
    Ok(path)
}

pub fn load() -> Option<Game> {
    let text = fs::read_to_string(save_path()).ok()?;
    let mut g = Game::new();
    for line in text.lines() {
        let (k, v) = match line.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        match k {
            "cash" => g.cash = v.parse().unwrap_or(g.cash),
            "wave" => g.wave = v.parse().unwrap_or(g.wave),
            "highest_wave" => g.highest_wave = v.parse().unwrap_or(g.highest_wave),
            "wall_hp" => g.wall_hp = v.parse().unwrap_or(g.wall_hp),
            "level" => g.level = v.parse().unwrap_or(g.level),
            "xp" => g.xp = v.parse().unwrap_or(g.xp),
            "kills" => g.kills = v.parse().unwrap_or(g.kills),
            "bosses_defeated" => g.bosses_defeated = v.parse().unwrap_or(g.bosses_defeated),
            "packs_opened" => g.packs_opened = v.parse().unwrap_or(g.packs_opened),
            "rebirth_points" => g.rebirth_points = v.parse().unwrap_or(g.rebirth_points),
            "weapon_bonus" => g.weapon_bonus = v.parse().unwrap_or(g.weapon_bonus),
            "item_bonus" => g.item_bonus = v.parse().unwrap_or(g.item_bonus),
            "total_earned" => g.total_earned = v.parse().unwrap_or(g.total_earned),
            "speed" => g.speed = v.parse::<f32>().unwrap_or(1.0).clamp(1.0, 3.0),
            "upgrades" => {
                for (i, part) in v.split(',').enumerate().take(6) {
                    g.upgrades[i] = part.parse().unwrap_or(0).min(999);
                }
            }
            "rebirth_upgrades" => {
                for (i, part) in v.split(',').enumerate().take(4) {
                    g.rebirth_upgrades[i] = part.parse().unwrap_or(0);
                }
            }
            "unlocked" => {
                for (i, ch) in v.chars().enumerate().take(7) {
                    g.unlocked[i] = ch == '1';
                }
            }
            "dups" => {
                for (i, part) in v.split(',').enumerate().take(7) {
                    g.dup_count[i] = part.parse().unwrap_or(0);
                }
            }
            "char_rarity" => {
                for (i, part) in v.split(',').enumerate().take(7) {
                    g.char_rarity[i] = part.parse().unwrap_or(0).min(5);
                }
            }
            "pack_level" => {
                for (i, part) in v.split(',').enumerate().take(5) {
                    g.pack_level[i] = part.parse().unwrap_or(0);
                }
            }
            "pack_xp" => {
                for (i, part) in v.split(',').enumerate().take(5) {
                    g.pack_xp[i] = part.parse().unwrap_or(0.0);
                }
            }
            "inventory" => {
                g.inventory.clear();
                for entry in v.split(';') {
                    let parts: Vec<&str> = entry.split(':').collect();
                    if parts.len() != 4 {
                        continue;
                    }
                    let kind = match parts[0] {
                        "W" => ItemKind::Weapon,
                        "G" => ItemKind::Gear,
                        _ => continue,
                    };
                    let idx: usize = parts[1].parse().unwrap_or(0);
                    let limit = match kind {
                        ItemKind::Weapon => WEAPON_ITEMS.len(),
                        ItemKind::Gear => GEAR_ITEMS.len(),
                    };
                    if idx >= limit {
                        continue;
                    }
                    let count: u32 = parts[3].parse().unwrap_or(1).max(1);
                    g.inventory.push(InvItem {
                        kind,
                        idx,
                        rarity: parts[2].parse().unwrap_or(0).min(5),
                        count,
                    });
                }
            }
            "defenders" => {
                g.defenders.clear();
                for entry in v.split(';') {
                    if entry.is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = entry.split(':').collect();
                    if parts.len() < 3 {
                        continue;
                    }
                    let slot: usize = parts[0].parse().unwrap_or(usize::MAX);
                    if slot >= MAX_SLOTS {
                        continue; // stare zapisy trzymały niepostawione postacie
                    }
                    let def: usize = parts[1].parse().unwrap_or(0);
                    if def >= DEFENDERS.len() {
                        continue;
                    }
                    let rarity: usize = parts[2].parse().unwrap_or(0).min(5);
                    let num = |i: usize| -> i64 {
                        parts.get(i).and_then(|v| v.parse::<i64>().ok()).unwrap_or(-1)
                    };
                    let weapon = if num(3) >= 0 && num(4) >= 0 && (num(3) as usize) < WEAPON_ITEMS.len() {
                        Some((num(3) as usize, (num(4) as usize).min(5)))
                    } else {
                        None
                    };
                    let gear = if num(5) >= 0 && num(6) >= 0 && (num(5) as usize) < GEAR_ITEMS.len() {
                        Some((num(5) as usize, (num(6) as usize).min(5)))
                    } else {
                        None
                    };
                    g.defenders.push(Defender {
                        slot,
                        def,
                        rarity,
                        cd: 0.0,
                        flash: 0.0,
                        weapon,
                        gear,
                    });
                }
            }
            _ => {}
        }
    }
    g.unlocked[0] = true;
    g.wall_hp = g.wall_hp.min(g.wall_max()).max(0.0);
    Some(g)
}
