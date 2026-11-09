//! Symulacja: fale, wrogowie, obrońcy, pociski, monety, paczki, prestiż.

use crate::config::*;

mod combat;
mod economy;
mod quests;
pub mod waves;


/// Sloty nie mają sztywnego limitu: każdy poziom PLATFORMY dokłada cztery
/// pola w nowej kolumnie. Siatka 12 kolumn wystarcza na 48 pól.
pub const SLOT_COLUMNS: usize = 12;
pub const MAX_SLOTS: usize = SLOT_COLUMNS * 4;

/// Pozycja slotu w siatce 4 wierszy — rośnie w prawo z każdą kolumną.
pub fn slot_pos(i: usize) -> (f32, f32) {
    let col = i / 4;
    let row = i % 4;
    (238.0 + col as f32 * 78.0, 232.0 + row as f32 * 88.0)
}

pub const LANES: [f32; 4] = [232.0, 320.0, 408.0, 496.0];


pub struct Enemy {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub kind: Kind,
    pub size: f32,
    pub cash: f64,
    pub spawn_wave: u32,
    pub hit: f32,
    pub phase: f32,
}

pub struct Defender {
    pub slot: usize,
    pub def: usize,
    pub rarity: usize,
    pub cd: f32,
    pub flash: f32,
    /// Założona broń z ekwipunku: (indeks w WEAPON_ITEMS, rzadkość).
    pub weapon: Option<(usize, usize)>,
    /// Założony sprzęt: (indeks w GEAR_ITEMS, rzadkość).
    pub gear: Option<(usize, usize)>,
}

#[derive(Clone, Copy)]
pub struct InvItem {
    pub kind: ItemKind,
    pub idx: usize,
    pub rarity: usize,
    pub count: u32,
}

pub struct Collector {
    pub x: f32,
    pub y: f32,
    pub taken: u32,
    pub carried: f64,
    pub emptying: bool,
}

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub lx: f32,
    pub ly: f32,
    pub target: u64,
    pub dmg: f32,
    pub splash: f32,
    pub crit: bool,
}

pub struct Coin {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub value: f64,
    pub t: f32,
}

pub struct Floater {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub t: f32,
    pub color: u32,
}

pub struct Game {
    pub cash: f64,
    pub wave: u32,
    /// Parametry proceduralne bieżącej fali (archetyp, tempo, mnożniki).
    pub plan: waves::WavePlan,
    pub highest_wave: u32,
    pub wave_time: f32,
    pub spawn_timer: f32,
    pub wall_hp: f64,
    pub level: u32,
    pub xp: f64,
    pub kills: u32,
    pub bosses_defeated: u32,
    pub packs_opened: u32,
    pub rebirth_points: u32,
    pub upgrades: [u32; 6],
    pub rebirth_upgrades: [u32; 4],
    pub unlocked: [bool; 7],
    pub dup_count: [u32; 7],
    /// Najlepsza wylosowana rzadkość dla każdej postaci (roster).
    pub char_rarity: [usize; 7],
    pub weapon_bonus: f32,
    pub item_bonus: f32,
    pub enemies: Vec<Enemy>,
    pub defenders: Vec<Defender>,
    pub projectiles: Vec<Projectile>,
    pub coins: Vec<Coin>,
    pub floaters: Vec<Floater>,
    pub selected: Option<usize>,
    pub game_over: bool,
    pub paused: bool,
    pub tab: usize,
    pub time: f32,
    pub autosave_timer: f32,
    pub message: String,
    pub message_t: f32,
    pub total_earned: f64,
    pub pack_level: [u32; 5],
    pub pack_xp: [f64; 5],
    pub inventory: Vec<InvItem>,
    pub collectors: Vec<Collector>,
    pub speed: f32,
    pub dps: f32,
    pub damage_window: Vec<(f32, f32)>,
    pub quest_idx: usize,
    pub quest_progress: u32,
    pub quest_done: u32,
    pub reset_armed: bool,
    next_id: u64,
    rng: u64,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            cash: 40.0,
            wave: 1,
            plan: waves::plan(1),
            highest_wave: 1,
            wave_time: WAVE_SECONDS,
            spawn_timer: 0.0,
            wall_hp: 500.0,
            level: 0,
            xp: 0.0,
            kills: 0,
            bosses_defeated: 0,
            packs_opened: 0,
            rebirth_points: 0,
            upgrades: [0; 6],
            rebirth_upgrades: [0; 4],
            unlocked: [false; 7],
            dup_count: [0; 7],
            char_rarity: [0; 7],
            weapon_bonus: 0.0,
            item_bonus: 0.0,
            enemies: Vec::new(),
            defenders: Vec::new(),
            projectiles: Vec::new(),
            coins: Vec::new(),
            floaters: Vec::new(),
            selected: None,
            game_over: false,
            paused: false,
            tab: 0,
            time: 0.0,
            autosave_timer: 0.0,
            message: String::new(),
            message_t: 0.0,
            total_earned: 0.0,
            pack_level: [0; 5],
            pack_xp: [0.0; 5],
            inventory: Vec::new(),
            collectors: Vec::new(),
            speed: 1.0,
            dps: 0.0,
            damage_window: Vec::new(),
            quest_idx: 0,
            quest_progress: 0,
            quest_done: 0,
            reset_armed: false,
            next_id: 1,
            rng: 0x9e3779b97f4a7c15,
        };
        g.unlocked[0] = true;
        g.wall_hp = g.wall_max();
        g.collectors.push(Collector { x: 300.0, y: 520.0, taken: 0, carried: 0.0, emptying: false });
        g
    }

    pub fn collector_speed(&self) -> f32 {
        COLLECTOR_BASE_SPEED + self.upgrades[COLLECTOR_UPGRADE] as f32 * 8.0
    }

    pub fn collector_capacity(&self) -> u32 {
        COLLECTOR_BASE_CAPACITY + self.upgrades[COLLECTOR_UPGRADE] * 2
    }

    fn spawn_collector(&mut self) {
        let x = COLLECTOR_FORT_X + 40.0;
        self.collectors.push(Collector {
            x,
            y: 520.0,
            taken: 0,
            carried: 0.0,
            emptying: false,
        });
    }

    /// Dodaje drop do ekwipunku, łącząc duplikaty tego samego typu i rzadkości.
    pub fn add_item(&mut self, kind: ItemKind, idx: usize, rarity: usize) {
        if let Some(slot) = self
            .inventory
            .iter_mut()
            .find(|i| i.kind == kind && i.idx == idx && i.rarity == rarity)
        {
            slot.count += 1;
            return;
        }
        self.inventory.push(InvItem { kind, idx, rarity, count: 1 });
    }

    /// Zakłada przedmiot z ekwipunku na obrońcę; zwraca opis do logu.
    pub fn equip_item(&mut self, def_index: usize, inv_index: usize) -> bool {
        if inv_index >= self.inventory.len() || def_index >= self.defenders.len() {
            self.log("SELECT A DEFENDER FIRST");
            return false;
        }
        let item = self.inventory[inv_index];
        let (kind, idx, rarity) = (item.kind, item.idx, item.rarity);
        match kind {
            ItemKind::Weapon => self.defenders[def_index].weapon = Some((idx, rarity)),
            ItemKind::Gear => self.defenders[def_index].gear = Some((idx, rarity)),
        }
        let name = match kind {
            ItemKind::Weapon => WEAPON_ITEMS[idx.min(WEAPON_ITEMS.len() - 1)].name,
            ItemKind::Gear => GEAR_ITEMS[idx.min(GEAR_ITEMS.len() - 1)].name,
        };
        self.inventory[inv_index].count -= 1;
        if self.inventory[inv_index].count == 0 {
            self.inventory.remove(inv_index);
        }
        self.log(format!("EQUIPPED {} ON {}", name, DEFENDERS[self.defenders[def_index].def].name));
        true
    }



    pub fn rand(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        let v = x.wrapping_mul(0x2545F4914F6CDD1D);
        ((v >> 40) as f32) / ((1u32 << 24) as f32)
    }

    fn rand_range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.rand()
    }

    // ---- statystyki pochodne ----
    pub fn damage_mult(&self) -> f32 {
        1.0 + self.upgrades[0] as f32 * 0.15
            + self.rebirth_upgrades[0] as f32 * 0.25
            + self.weapon_bonus
            + self.level as f32 * 0.01
    }

    pub fn speed_mult(&self) -> f32 {
        1.0 + self.upgrades[1] as f32 * 0.10 + self.rebirth_upgrades[1] as f32 * 0.15 + self.item_bonus
    }

    pub fn cash_mult(&self) -> f32 {
        1.0 + self.upgrades[2] as f32 * 0.20 + self.rebirth_upgrades[2] as f32 * 0.30
    }

    pub fn wall_max(&self) -> f64 {
        500.0 + self.upgrades[3] as f64 * 100.0 + self.rebirth_upgrades[3] as f64 * 250.0
    }

    /// Bez górnego limitu — kolejne poziomy platformy dokładają pola.
    pub fn max_slots(&self) -> usize {
        4 + self.upgrades[4] as usize
    }

    pub fn upgrade_cost(&self, i: usize) -> f64 {
        upgrade_price(UPGRADES[i].base, UPGRADES[i].growth, self.upgrades[i] + 1)
    }

    pub fn rebirth_upgrade_cost(&self, i: usize) -> u32 {
        (REBIRTH_UPGRADES[i].base as f64 * REBIRTH_UPGRADES[i].growth.powi(self.rebirth_upgrades[i] as i32))
            .floor() as u32
    }

    pub fn slot_occupied(&self, slot: usize) -> bool {
        self.defenders.iter().any(|d| d.slot == slot)
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.message_t = 2.8;
    }
}
