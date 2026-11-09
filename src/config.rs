//! Reguły balansu odtworzone z `Emojis vs Zombies` (Unity/Mono, Assembly-CSharp.dll).
//!
//! Każda funkcja ma w komentarzu metodę źródłową, z której wynika. Wartości
//! oznaczone `[D]` są decyzjami tej rekonstrukcji: oryginał trzyma je
//! w ScriptableObjectach w `data.unity3d`, a nie w kodzie C#, więc nie da się
//! ich odczytać analizą statyczną assembly.

pub const WIN_W: usize = 1280;
pub const WIN_H: usize = 720;

pub const WALL_X: f32 = 176.0;
pub const SPAWN_X: f32 = 1010.0;
pub const WAVE_SECONDS: f32 = 30.0;
pub const BOSS_WAVE: u32 = 5;

/// Układ ekranu: wysokość górnego paska i początek dolnego panelu.
pub const HUD_HEIGHT: f32 = 100.0;
pub const PANEL_Y: f32 = 548.0;

/// Constants::.cctor
pub const ENEMY_HEALTH_GROWTH: f32 = 1.15;

pub struct Rarity {
    pub name: &'static str,
    pub color: u32,
    /// Constants::GetStatMultiplier — switch 0..5.
    pub stat_mult: f32,
}

/// Kolejność z RarityType (Common..Mythic); kolory z Constants::GetRarityColor.
pub const RARITIES: [Rarity; 6] = [
    Rarity { name: "Common",    color: 0x92a1b9, stat_mult: 0.0 },
    Rarity { name: "Uncommon",  color: 0xe69c69, stat_mult: 2.0 },
    Rarity { name: "Rare",      color: 0x5ac54f, stat_mult: 5.0 },
    Rarity { name: "Epic",      color: 0xdb3ffd, stat_mult: 10.0 },
    Rarity { name: "Legendary", color: 0xed7614, stat_mult: 15.0 },
    Rarity { name: "Mythic",    color: 0xffc825, stat_mult: 20.0 },
];

/// Constants::GetDropDuplicateStatMultiplier — bonus za duplikat, per rzadkość.
pub const DUPLICATE_BONUS: [f32; 6] = [0.001, 0.005, 0.02, 0.04, 0.10, 0.15];

pub struct DefenderDef {
    pub name: &'static str,
    /// WeaponType — Constants::GetWeaponDamageMultiplier.
    pub weapon: &'static str,
    pub damage: f32,
    pub aps: f32,
    pub range: f32,
    pub cost: f64,
    pub color: u32,
    pub splash: f32,
}

/// [D] Roster obrońców. Bronie i ich mnożniki pochodzą z WeaponType.
pub const DEFENDERS: [DefenderDef; 7] = [
    DefenderDef { name: "CAT",    weapon: "pistol",  damage: 4.0,  aps: 1.20, range: 330.0, cost: 25.0,  color: 0xe8b06a, splash: 0.0 },
    DefenderDef { name: "DOG",    weapon: "shotgun", damage: 7.0,  aps: 0.80, range: 230.0, cost: 45.0,  color: 0xb08f6a, splash: 0.0 },
    DefenderDef { name: "FOX",    weapon: "rifle",   damage: 6.0,  aps: 1.10, range: 360.0, cost: 60.0,  color: 0xe06a3c, splash: 0.0 },
    DefenderDef { name: "BEAR",   weapon: "sniper",  damage: 12.0, aps: 0.45, range: 470.0, cost: 90.0,  color: 0x8a6a4a, splash: 0.0 },
    DefenderDef { name: "RABBIT", weapon: "smg",     damage: 3.0,  aps: 2.20, range: 250.0, cost: 75.0,  color: 0xc8c8d0, splash: 0.0 },
    DefenderDef { name: "BOAR",   weapon: "rocket",  damage: 18.0, aps: 0.30, range: 400.0, cost: 140.0, color: 0x6e5a48, splash: 62.0 },
    DefenderDef { name: "OWL",    weapon: "hmg",     damage: 4.0,  aps: 1.80, range: 340.0, cost: 110.0, color: 0x8a7ab0, splash: 0.0 },
];

pub struct UpgradeDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub base: f64,
    pub growth: f64,
}

/// UpgradeUI: cena = round(base * growth^(level-1) * 5^floor((level-1)/25)).
pub const UPGRADES: [UpgradeDef; 6] = [
    UpgradeDef { name: "FIREPOWER", desc: "+15% DAMAGE",  base: 30.0,  growth: 1.35 },
    UpgradeDef { name: "RATE",      desc: "+10% SPEED",   base: 40.0,  growth: 1.40 },
    UpgradeDef { name: "LOOT",      desc: "+20% CASH",    base: 55.0,  growth: 1.45 },
    UpgradeDef { name: "WALL",      desc: "+100 HP",     base: 50.0,  growth: 1.38 },
    UpgradeDef { name: "PLATFORM",  desc: "+1 SLOT",      base: 120.0, growth: 2.10 },
    UpgradeDef { name: "COLLECTOR", desc: "+1 SCOUT",     base: 90.0,  growth: 1.60 },
];

/// Indeks ulepszenia zbieraczy w tablicy UPGRADES.
pub const COLLECTOR_UPGRADE: usize = 5;

pub struct PackDef {
    pub tier: usize,
    pub cost: f64,
}

/// UnlockableType: UndergroundPack..ApocalypsePack.
pub const PACKS: [PackDef; 5] = [
    PackDef { tier: 0, cost: 150.0 },
    PackDef { tier: 2, cost: 450.0 },
    PackDef { tier: 4, cost: 1200.0 },
    PackDef { tier: 6, cost: 3000.0 },
    PackDef { tier: 8, cost: 7500.0 },
];

/// PackData::GetRarityChance — pełne tabele 10 tierów + domyślny (>=10).
pub const PACK_TABLES: [[f32; 6]; 11] = [
    [0.94, 0.06, 0.00, 0.0000, 0.000, 0.0000],
    [0.84, 0.14, 0.02, 0.0000, 0.000, 0.0000],
    [0.74, 0.20, 0.06, 0.0000, 0.000, 0.0000],
    [0.66, 0.24, 0.09, 0.0100, 0.000, 0.0000],
    [0.60, 0.25, 0.12, 0.0280, 0.002, 0.0000],
    [0.54, 0.25, 0.15, 0.0535, 0.006, 0.0005],
    [0.49, 0.25, 0.17, 0.0705, 0.018, 0.0010],
    [0.45, 0.24, 0.185, 0.0920, 0.031, 0.0020],
    [0.41, 0.23, 0.205, 0.1125, 0.039, 0.0035],
    [0.38, 0.22, 0.220, 0.1300, 0.045, 0.0050],
    [0.34, 0.21, 0.235, 0.1500, 0.057, 0.0080],
];

pub struct RebirthDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub base: u32,
    pub growth: f64,
}

pub const REBIRTH_UPGRADES: [RebirthDef; 4] = [
    RebirthDef { name: "ANCIENT MIGHT", desc: "+25% DMG",  base: 2, growth: 1.8 },
    RebirthDef { name: "INSTINCT",      desc: "+15% RATE", base: 3, growth: 1.8 },
    RebirthDef { name: "GREED",         desc: "+30% CASH", base: 2, growth: 1.7 },
    RebirthDef { name: "ETERNAL WALL",  desc: "+250 WALL", base: 2, growth: 1.6 },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Basic,
    Fast,
    Fat,
    Boss,
}

pub struct KindDef {
    pub hp: f32,
    pub speed: f32,
    pub size: f32,
    pub cash: f64,
    pub wall_damage_mult: f32,
}

/// [D] Mnożniki wariantów wroga; sam wybór wariantu pochodzi z kodu.
pub fn kind_def(kind: Kind) -> KindDef {
    match kind {
        Kind::Basic => KindDef { hp: 1.0, speed: 26.0, size: 1.00, cash: 1.0, wall_damage_mult: 0.35 },
        Kind::Fast => KindDef { hp: 0.6, speed: 46.0, size: 0.85, cash: 0.8, wall_damage_mult: 0.25 },
        Kind::Fat => KindDef { hp: 2.4, speed: 17.0, size: 1.35, cash: 1.6, wall_damage_mult: 0.60 },
        Kind::Boss => KindDef { hp: 16.0, speed: 15.0, size: 1.85, cash: 9.0, wall_damage_mult: 3.0 },
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Constants::GetEnemyHealth(wave, baseHealth) — pełna formuła z IL.
pub fn enemy_health(wave: u32, base: f32) -> f32 {
    let w = wave as f32;
    let t = if w > 6.0 {
        lerp(0.5, 1.0, clamp01((w - 6.0) / 6.0))
    } else {
        0.5
    };
    let x = if w > 10.0 { 0.035 * w / 10.0 } else { 0.0 };
    let y = if w >= 20.0 { 2.0 + (w - 10.0) / 10.0 } else { 2.0 };
    (y * ENEMY_HEALTH_GROWTH.powf(w + x) * base * t).floor()
}

/// GameManager::IsBossWave — fala % BossWave == 0.
pub fn is_boss_wave(wave: u32) -> bool {
    wave % BOSS_WAVE == 0
}

/// LevelSystem::GetExperienceForLevel — floor(20 * 2^level).
pub fn xp_for_level(level: u32) -> f64 {
    (20.0 * 2f64.powi(level as i32)).floor()
}

/// UpgradeUI::GetMilestoneMultiplier — 5^floor((level-1)/25).
pub fn milestone_multiplier(level: u32) -> f64 {
    5f64.powi(((level.saturating_sub(1)) / 25) as i32)
}

/// UpgradeUI::GetNewPrice — round((base * growth^(level-1)) * milestone).
pub fn upgrade_price(base: f64, growth: f64, level: u32) -> f64 {
    (base * growth.powi(level as i32 - 1) * milestone_multiplier(level)).round()
}

/// RebirthManager::UpdateBossDefeated — floor(spawnWave / 5).
pub fn rebirth_points_for_boss(spawn_wave: u32) -> u32 {
    spawn_wave / BOSS_WAVE
}

/// RebirthManager::CalculateRebirthPoints — bossowie + 2*(i+1) za każde 10 fal.
pub fn calculate_rebirth_points(highest_wave: u32, bosses_defeated: u32) -> u32 {
    let mut points = bosses_defeated;
    let cycles = highest_wave / 10;
    for i in 0..cycles {
        points += (i + 1) * 2;
    }
    points
}

/// Constants::GetWeaponDamageMultiplier — mnożnik broni * (1 + 0.05 * rarityMult).
pub fn weapon_damage_multiplier(weapon: &str, rarity: usize) -> f32 {
    let base = match weapon {
        "pistol" => 1.25,
        "shotgun" => 0.75,
        "rifle" => 1.0,
        "sniper" => 2.0,
        "smg" => 0.5,
        "rocket" => 3.0,
        "hmg" => 0.75,
        _ => 0.0,
    };
    base * (1.0 + 0.05 * RARITIES[rarity.min(5)].stat_mult)
}

/// PackUI::GetXPForLevel — round(5 * PackGrowthValue^level + 5 * level * packXPBase)
/// z PackGrowthValue = 1.5 (Constants::.cctor).
pub fn pack_xp_for_level(level: u32, xp_base: f64) -> u32 {
    (5.0 * 1.5f64.powi(level as i32) + 5.0 * level as f64 * xp_base).round() as u32
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Weapon,
    Gear,
}

pub struct WeaponItem {
    pub name: &'static str,
    pub weapon: &'static str,
}

/// [D] Nazwy dropów broni; typ broni z WeaponType, mnożnik z Constants.
pub const WEAPON_ITEMS: [WeaponItem; 6] = [
    WeaponItem { name: "REVOLVER", weapon: "pistol" },
    WeaponItem { name: "BURST", weapon: "rifle" },
    WeaponItem { name: "SCATTER", weapon: "shotgun" },
    WeaponItem { name: "LONGSHOT", weapon: "sniper" },
    WeaponItem { name: "BUZZ", weapon: "smg" },
    WeaponItem { name: "BOOMER", weapon: "rocket" },
];

pub struct GearItem {
    pub name: &'static str,
    pub damage: f32,
    pub aps: f32,
}

/// [D] Sprzęt: dodatki procentowe do obrażeń i szybkości ataku obrońcy.
pub const GEAR_ITEMS: [GearItem; 5] = [
    GearItem { name: "SCOPE", damage: 0.10, aps: 0.00 },
    GearItem { name: "MAGAZINE", damage: 0.00, aps: 0.08 },
    GearItem { name: "GRIP", damage: 0.05, aps: 0.05 },
    GearItem { name: "BARREL", damage: 0.18, aps: -0.04 },
    GearItem { name: "TRIGGER", damage: -0.06, aps: 0.15 },
];

/// StatType: CoinCollectorSpeed (3001), CoinCollectorCapacity (3002).
pub const COLLECTOR_BASE_SPEED: f32 = 62.0;
pub const COLLECTOR_BASE_CAPACITY: u32 = 6;
pub const COLLECTOR_FORT_X: f32 = 210.0;

pub struct QuestDef {
    pub kind: QuestKind,
    pub label: &'static str,
    pub target: u32,
    pub reward: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuestKind {
    Kills,
    Wave,
    Packs,
}

/// QuestType: BossQuest / FirstInventory / OpenFirstPack w oryginale; tu zestaw celów.
pub const QUESTS: [QuestDef; 4] = [
    QuestDef { kind: QuestKind::Kills, label: "KILL ZOMBIES", target: 25, reward: 120.0 },
    QuestDef { kind: QuestKind::Wave, label: "REACH WAVE", target: 8, reward: 220.0 },
    QuestDef { kind: QuestKind::Packs, label: "OPEN PACKS", target: 1, reward: 90.0 },
    QuestDef { kind: QuestKind::Kills, label: "KILL ZOMBIES", target: 120, reward: 650.0 },
];

/// Rzut rzadkości z tabeli PackData dla danego tieru.
pub fn roll_rarity(tier: usize, r: f32) -> usize {
    let table = &PACK_TABLES[tier.min(PACK_TABLES.len() - 1)];
    let mut acc = 0.0;
    for (i, chance) in table.iter().enumerate() {
        acc += chance;
        if r <= acc {
            return i;
        }
    }
    table.len() - 1
}
