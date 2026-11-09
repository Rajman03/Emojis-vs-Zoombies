//! Ładowanie assetów graficznych wkompilowanych w binarkę.
//!
//! Źródła (skopiowane do `assets/`, licencje obok):
//! - UI: Complete UI Essential Pack (Flat Theme) — ramki, przyciski, paski, sloty
//! - Bronie: guns_gameassets (PNG)
//! - Twarze: 16x16 Emoji Pack v1.1 (arkusz 8x8, kafle 16 px)

use crate::gfx::Sprite;

pub struct Assets {
    pub button: Vec<Sprite>,
    pub frame: Sprite,
    pub frame2: Sprite,
    pub slot: Sprite,
    pub bar: Sprite,
    pub fill: Sprite,
    pub emoji: Sprite,
    /// Proceduralna grafika pixel-art (monety, naboje, błysk).
    pub art: crate::art::PixelArt,
    /// Statyczne tło (niebo, księżyc, ziemia) — rysowane raz, potem kopiowane.
    pub backdrop: Vec<u32>,
    /// Statyczna oprawa UI: pasek HUD i dolny panel — rysowane raz.
    pub chrome: Vec<u32>,
    pub gun_pistol: Sprite,
    pub gun_rifle: Sprite,
    pub gun_shotgun: Sprite,
    pub gun_sniper: Sprite,
    pub gun_smg: Sprite,
    pub gun_rocket: Sprite,
    pub ammo: Sprite,
}

impl Assets {
    pub fn load() -> Assets {
        macro_rules! spr {
            ($p:expr) => {
                Sprite::from_png(include_bytes!($p)).expect(concat!("nie mozna zdekodowac ", $p))
            };
        }
        let mut assets = Assets {
            button: vec![
                spr!("../assets/ui/UI_Flat_Button01a_1.png"),
                spr!("../assets/ui/UI_Flat_Button01a_2.png"),
                spr!("../assets/ui/UI_Flat_Button01a_3.png"),
            ],
            frame: spr!("../assets/ui/UI_Flat_Frame01a.png"),
            frame2: spr!("../assets/ui/UI_Flat_Frame02a.png"),
            slot: spr!("../assets/ui/UI_Flat_FrameSlot01a.png"),
            bar: spr!("../assets/ui/UI_Flat_Bar01a.png"),
            fill: spr!("../assets/ui/UI_Flat_BarFill01a.png"),
            emoji: spr!("../assets/emoji.png"),
            art: crate::art::PixelArt::new(),
            backdrop: build_backdrop(),
            chrome: Vec::new(),
            gun_pistol: spr!("../assets/guns/pistol.png"),
            gun_rifle: spr!("../assets/guns/assaultrifle.png"),
            gun_shotgun: spr!("../assets/guns/shotgun.png"),
            gun_sniper: spr!("../assets/guns/sniper.png"),
            gun_smg: spr!("../assets/guns/smg2.png"),
            gun_rocket: spr!("../assets/guns/grenade.png"),
            ammo: spr!("../assets/guns/ammobox.png"),
        };
        assets.chrome = build_chrome(&assets.frame);
        assets
    }

    /// Ikona broni wg WeaponType.
    pub fn gun(&self, weapon: &str) -> &Sprite {
        match weapon {
            "shotgun" => &self.gun_shotgun,
            "rifle" => &self.gun_rifle,
            "sniper" => &self.gun_sniper,
            "smg" => &self.gun_smg,
            "rocket" => &self.gun_rocket,
            "hmg" => &self.gun_rifle,
            _ => &self.gun_pistol,
        }
    }

    /// Ikona broni wg indeksu WEAPON_ITEMS.
    pub fn gun_for_item(&self, idx: usize) -> &Sprite {
        match idx {
            0 => &self.gun_pistol,
            1 => &self.gun_rifle,
            2 => &self.gun_shotgun,
            3 => &self.gun_sniper,
            4 => &self.gun_smg,
            5 => &self.gun_rocket,
            _ => &self.gun_pistol,
        }
    }
}

/// Twarze obrońców w arkuszu emoji (kolumna, wiersz).
pub const DEFENDER_FACES: [(usize, usize); 7] = [
    (0, 0), // cat
    (1, 0), // dog
    (3, 0), // fox
    (0, 5), // bear — okulary
    (2, 2), // rabbit — język
    (7, 2), // boar — wściekły
    (6, 6), // owl — czapka
];

/// Twarze wrogów: (kolumna, wiersz) egzemplarza z arkusza.
pub fn enemy_face(kind: crate::config::Kind) -> (usize, usize) {
    use crate::config::Kind;
    match kind {
        Kind::Basic => (6, 7),  // zielony
        Kind::Fast => (3, 7),   // niebieski
        Kind::Fat => (0, 7),    // czerwony
        Kind::Boss => (5, 7),   // fioletowy róg
    }
}

/// Buduje statyczne tło raz przy starcie: niebo, księżyc, ziemia, trawa.
fn build_backdrop() -> Vec<u32> {
    use crate::config::{WIN_H, WIN_W};
    use crate::gfx::Canvas;
    let mut c = Canvas::new(WIN_W, WIN_H);
    c.vgradient(0x120a1e, 0x2b1c3a);
    c.disc(1080.0, 120.0, 42.0, 0xf2ecc9);
    c.disc(1064.0, 108.0, 34.0, 0x1a1028);
    c.rect(0.0, 520.0, WIN_W as f32, 40.0, 0x241634);
    c.rect(0.0, 536.0, WIN_W as f32, 30.0, 0x1a1026);
    for i in 0..40 {
        let x = (i as f32 * 33.0) % WIN_W as f32;
        let h = 8.0 + ((i * 37) % 14) as f32;
        c.rect(x, 528.0 - h, 3.0, h, 0x2f2040);
    }
    c.rect_alpha(0.0, 470.0, 620.0, 90.0, 0x6a4a8a, 0.10);
    c.buf
}

/// Statyczna oprawa UI: górny pasek i dolny panel. Rysowana raz przy starcie,
/// a w każdej klatce tylko kopiowana — 9-slice z alpha było najdroższą częścią.
fn build_chrome(frame: &Sprite) -> Vec<u32> {
    use crate::config::{HUD_HEIGHT, PANEL_Y, WIN_H, WIN_W};
    use crate::gfx::Canvas;
    let tint = 0x171022;
    let mut c = Canvas::new(WIN_W, WIN_H);
    c.nine(frame, -8.0, -14.0, WIN_W as f32 + 16.0, HUD_HEIGHT, 12, Some(tint));
    c.nine(
        frame,
        -8.0,
        PANEL_Y - 2.0,
        WIN_W as f32 + 16.0,
        WIN_H as f32 - PANEL_Y + 10.0,
        12,
        Some(tint),
    );
    c.buf
}
