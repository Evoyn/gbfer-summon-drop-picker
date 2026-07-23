use crate::data::{ASTRAL_CURVE_HALF, ASTRAL_CURVE_MAX, BONUSES, SUMMONS};

const BASE_SUMMON_LOT: &[u8] = include_bytes!("../base/summon_lot.tbl");

// same every pick, just copied out next to the patched summon_lot
pub const BASE_TABLES: [(&str, &[u8]); 3] = [
    ("summon.tbl", include_bytes!("../base/summon.tbl")),
    ("reward_summon_lot.tbl", include_bytes!("../base/reward_summon_lot.tbl")),
    ("summon_curve.tbl", include_bytes!("../base/summon_curve.tbl")),
];

pub const VANILLA: [(&str, &[u8]); 4] = [
    ("summon_lot.tbl", include_bytes!("../vanilla/summon_lot.tbl")),
    ("summon.tbl", include_bytes!("../vanilla/summon.tbl")),
    ("reward_summon_lot.tbl", include_bytes!("../vanilla/reward_summon_lot.tbl")),
    ("summon_curve.tbl", include_bytes!("../vanilla/summon_curve.tbl")),
];

#[derive(Clone, Copy)]
pub struct Pick {
    pub skill: usize,
    pub bonus: usize,
    pub half_cap: bool,
}

impl Default for Pick {
    fn default() -> Self {
        Pick { skill: 0, bonus: 0, half_cap: false }
    }
}

// 8b header = row count, then 20b rows, little endian:
//   +0 key  +4 skill/baseparam  +8 curve  +12 weight(i32)  +16 unk
const ROW: usize = 20;
const HEADER: usize = 8;

fn rows(b: &[u8]) -> usize {
    i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
}

fn field(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap_or(0)
}

fn each_row_of(b: &[u8], pool: &str) -> Vec<usize> {
    let pool = hex(pool);
    (0..rows(b))
        .map(|r| HEADER + r * ROW)
        .filter(|&off| off + ROW <= b.len() && field(b, off) == pool)
        .collect()
}

// the rest go to 1 not 0, keep + rest still rolls, just heavily rigged
fn force(b: &mut [u8], pool: &str, keep: &str) {
    let keep = hex(keep);
    for off in each_row_of(b, pool) {
        if field(b, off + 4) != keep {
            b[off + 12..off + 16].copy_from_slice(&1i32.to_le_bytes());
        }
    }
}

fn set_curve(b: &mut [u8], pool: &str, curve: &str) {
    let curve = hex(curve).to_le_bytes();
    for off in each_row_of(b, pool) {
        b[off + 8..off + 12].copy_from_slice(&curve);
    }
}

pub fn patched_summon_lot(picks: &[Pick]) -> Vec<u8> {
    let mut lot = BASE_SUMMON_LOT.to_vec();
    for (s, p) in SUMMONS.iter().zip(picks) {
        force(&mut lot, s.skill_pool, s.skills[p.skill].1);
        force(&mut lot, s.equip_pool, BONUSES[p.bonus].id(s.astral));
        if s.astral {
            let curve = if p.half_cap { ASTRAL_CURVE_HALF } else { ASTRAL_CURVE_MAX };
            set_curve(&mut lot, s.equip_pool, curve);
        }
    }
    lot
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(b: &[u8], key: &str) -> Vec<(u32, u32, i32)> {
        each_row_of(b, key)
            .into_iter()
            .map(|off| {
                let w = i32::from_le_bytes([b[off + 12], b[off + 13], b[off + 14], b[off + 15]]);
                (field(b, off + 4), field(b, off + 8), w)
            })
            .collect()
    }

    fn kept(b: &[u8], key: &str) -> Vec<u32> {
        pool(b, key).iter().filter(|r| r.2 > 1).map(|r| r.0).collect()
    }

    fn index_of(name: &str) -> usize {
        SUMMONS.iter().position(|s| s.name == name).unwrap()
    }

    #[test]
    fn base_table_shape() {
        let expected = 726 + SUMMONS.len() * 11;
        assert_eq!(rows(BASE_SUMMON_LOT), expected);
        assert_eq!(BASE_SUMMON_LOT.len(), HEADER + expected * ROW);
    }

    #[test]
    fn vanilla_is_unmodified() {
        let lot = VANILLA.iter().find(|(n, _)| *n == "summon_lot.tbl").unwrap().1;
        assert_eq!(rows(lot), 726);
    }

    #[test]
    fn every_summon_has_its_pools_and_skills() {
        for s in SUMMONS {
            assert_eq!(pool(BASE_SUMMON_LOT, s.equip_pool).len(), 11, "{}", s.name);
            let ids: Vec<u32> = pool(BASE_SUMMON_LOT, s.skill_pool).iter().map(|r| r.0).collect();
            for (name, h) in s.skills {
                assert!(ids.contains(&hex(h)), "{} missing {}", s.name, name);
            }
        }
    }

    #[test]
    fn forces_one_row_per_pool() {
        let rolan = index_of("Rolan");
        let mut picks = vec![Pick::default(); SUMMONS.len()];
        picks[0] = Pick { skill: 3, bonus: 1, half_cap: false };
        picks[rolan] = Pick { skill: 5, bonus: 4, half_cap: false };
        let lot = patched_summon_lot(&picks);

        assert_eq!(kept(&lot, "DF902143"), vec![hex("73220725")]);
        assert_eq!(kept(&lot, "393EF1D8"), vec![hex("2FFB509F")]);
        assert_eq!(kept(&lot, "26428274"), vec![hex("E0ABFDFE")]);
        assert_eq!(kept(&lot, "A35D7A5C"), vec![hex("F0F77BC1")]);
    }

    #[test]
    fn half_cap_only_moves_that_summon() {
        let mut picks = vec![Pick::default(); SUMMONS.len()];
        picks[index_of("Lucilius")].half_cap = true;
        let lot = patched_summon_lot(&picks);

        assert!(pool(&lot, "F63793E4").iter().all(|r| r.1 == hex(ASTRAL_CURVE_HALF)));
        assert!(pool(&lot, "B7D9379B").iter().all(|r| r.1 == hex(ASTRAL_CURVE_MAX)));
        assert!(pool(&lot, "393EF1D8").iter().all(|r| r.1 == hex("2E2C5483")));
    }
}
