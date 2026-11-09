//! Proceduralny generator fal.
//!
//! Numer fali wyznacza archetyp (rój, elita, szturm, mieszana), tempo spawnu,
//! liczebność partii i mnożniki wrogów. Losowość jest deterministyczna —
//! ta sama fala zawsze wygląda tak samo, ale kolejne fale różnią się od siebie,
//! a trudność rośnie wraz z numerem.

use crate::config::Kind;

#[derive(Clone, Copy)]
pub struct WavePlan {
    pub name: &'static str,
    pub interval: f32,
    pub batch: u32,
    pub hp_mult: f32,
    pub speed_mult: f32,
    pub cash_mult: f32,
    /// Szanse na [basic, fast, fat] — sumują się do 1.
    pub weights: [f32; 3],
}

fn hash(v: u32) -> u32 {
    let mut x = v.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2545_F491);
    x ^= x >> 13;
    x
}

/// Losowy dryf w zakresie [-amp, amp] zależny od fali i soli.
fn jitter(wave: u32, salt: u32, amp: f32) -> f32 {
    let v = (hash(wave ^ salt) >> 8 & 0xffff) as f32 / 65535.0;
    (v * 2.0 - 1.0) * amp
}

pub const ARCHETYPES: [&str; 4] = ["MIXED", "SWARM", "ELITE", "RUSH"];

pub fn plan(wave: u32) -> WavePlan {
    let w = wave as f32;
    let kind = (hash(wave) % 4) as usize;

    // bazowe tempo i liczebność rosną z falą, archetyp je modyfikuje
    let base_interval = (5.0 - w * 0.045).clamp(0.9, 5.0);
    let base_batch = 1 + (w / 12.0) as u32;

    let (interval, batch, hp, speed, tag) = match kind {
        // rój: dużo słabszych, szybciej
        1 => (base_interval * 0.65, base_batch + 2, 0.78, 1.0, ARCHETYPES[1]),
        // elita: mniej, ale mocniejsi i wolniejsi
        2 => (base_interval * 1.35, (base_batch / 2).max(1), 1.55, 0.92, ARCHETYPES[2]),
        // szturm: szybcy i liczni
        3 => (base_interval * 0.55, base_batch + 1, 0.95, 1.35, ARCHETYPES[3]),
        _ => (base_interval, base_batch, 1.0, 1.0, ARCHETYPES[0]),
    };

    // skład wrogów: nowe typy wchodzą progresywnie i wypierają podstawowych
    let fast = ((w - 4.0) / 26.0).clamp(0.0, 0.45);
    let fat = ((w - 8.0) / 34.0).clamp(0.0, 0.4);
    let basic = (1.0 - fast - fat).max(0.05);

    WavePlan {
        name: tag,
        interval: (interval * (1.0 + jitter(wave, 11, 0.12))).max(0.45),
        batch: batch.max(1),
        hp_mult: (hp * (1.0 + w * 0.012) * (1.0 + jitter(wave, 23, 0.10))).max(0.3),
        speed_mult: (speed * (1.0 + jitter(wave, 37, 0.08))).max(0.5),
        cash_mult: 1.0 + w * 0.01,
        weights: [basic, fast, fat],
    }
}

/// Wybiera typ wroga zgodnie z wagami fali (r z zakresu 0..1).
pub fn roll(plan: &WavePlan, r: f32) -> Kind {
    let [basic, fast, _fat] = plan.weights;
    if r < basic {
        Kind::Basic
    } else if r < basic + fast {
        Kind::Fast
    } else {
        Kind::Fat
    }
}
