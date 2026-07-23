pub struct Summon {
    pub name: &'static str,
    pub quest: &'static str,
    pub astral: bool,
    pub skill_pool: &'static str,
    pub equip_pool: &'static str,
    pub skills: &'static [(&'static str, &'static str)],
}

pub struct Bonus {
    pub key: &'static str,
    pub name: &'static str,
    pub normal_id: &'static str,
    pub normal_max: &'static str,
    pub astral_id: &'static str,
    pub astral_max: &'static str,
}

impl Bonus {
    pub fn id(&self, astral: bool) -> &'static str {
        if astral { self.astral_id } else { self.normal_id }
    }

    pub fn max(&self, astral: bool, half: bool) -> &'static str {
        if astral && !half { self.astral_max } else { self.normal_max }
    }
}

// caps go 20,25,30,35,40,45,50,60,80,100 and the curve is lvl 5-9,
// so lvl 6 lands on normal tier values and lvl 9 on the astral max
pub const ASTRAL_CURVE_MAX: &str = "4E547493";
pub const ASTRAL_CURVE_HALF: &str = "5A370B86";

pub const BONUSES: &[Bonus] = &[
    Bonus { key: "normatkcap",  name: "Normal Attack Damage Cap Up", normal_id: "A66241C9", normal_max: "50%",   astral_id: "9245DFA4", astral_max: "100%" },
    Bonus { key: "skilldmgcap", name: "Skill Damage Cap Up",         normal_id: "2FFB509F", normal_max: "50%",   astral_id: "CE70C58A", astral_max: "100%" },
    Bonus { key: "skyartcap",   name: "Skybound Art Damage Cap Up",  normal_id: "A3539FBB", normal_max: "50%",   astral_id: "5A1D2C89", astral_max: "100%" },
    Bonus { key: "healcap",     name: "Healing Cap Up",              normal_id: "2270BC40", normal_max: "50%",   astral_id: "2EA9CA80", astral_max: "75%"  },
    Bonus { key: "stun",        name: "Stun Power Up",               normal_id: "F7B0316F", normal_max: "15",    astral_id: "F0F77BC1", astral_max: "20"   },
    Bonus { key: "atk",         name: "Attack Power Up",             normal_id: "A8900C80", normal_max: "+2000", astral_id: "A3E537B1", astral_max: "+3000"},
    Bonus { key: "hp",          name: "Health Up",                   normal_id: "BC4E92CB", normal_max: "+2000", astral_id: "F2256E44", astral_max: "+5000"},
    Bonus { key: "crit",        name: "Critical Hit Rate Up",        normal_id: "00D171E0", normal_max: "20%",   astral_id: "A074A967", astral_max: "30%"  },
    Bonus { key: "skilldmg",    name: "Skill Damage Up",             normal_id: "664E8E32", normal_max: "20%",   astral_id: "AF39A3FB", astral_max: "30%"  },
    Bonus { key: "skyart",      name: "Skybound Art Damage Up",      normal_id: "75138D21", normal_max: "20%",   astral_id: "33C0D50C", astral_max: "30%"  },
    Bonus { key: "chainburst",  name: "Chain Burst Damage Up",       normal_id: "5A39D81B", normal_max: "50%",   astral_id: "54B09A37", astral_max: "100%" },
];

// raw hashes out of summon_lot.tbl, signature skill first
pub const SUMMONS: &[Summon] = &[
    Summon { name: "Behemoth III", quest: "The Myths Are Real", astral: false, skill_pool: "DF902143", equip_pool: "393EF1D8", skills: &[
        ("Stout Heart", "A1A8E39D"), ("Supplementary DMG", "57AB5B10"), ("Uplift", "B5FF9FD3"),
        ("Celestial Ventus", "73220725"), ("Less Is More", "82CE278D"), ("Critical Hit Rate", "8D78A19B") ] },
    Summon { name: "Wee Pincer III", quest: "On the Threshold of Provenance", astral: false, skill_pool: "2340DCED", equip_pool: "C39F7144", skills: &[
        ("Crabvestment Returns", "1B0D9897"), ("DMG Cap", "DC584F60"), ("Cascade", "05F2ECDC"), ("Steel Nerves", "1470F860") ] },
    Summon { name: "Albacore III", quest: "Strifes way: Bizzare (Conflux)", astral: false, skill_pool: "1250EFD2", equip_pool: "E31F7BAD", skills: &[
        ("Natural Defenses", "0EAD65E0"), ("Path to Mastery", "5E422AE5"), ("Rupie Tycoon", "C86F3082"), ("Fast Learner", "F687C5EF") ] },
    Summon { name: "Furycane Nihilla", quest: "The Eternal Grind", astral: false, skill_pool: "384C3333", equip_pool: "FD5DACF0", skills: &[
        ("Celestial Aqua", "A898E283"), ("Fatebreaker", "D029FE08"), ("Potion Hoarder", "24883AF3"),
        ("Guts", "E69A4694"), ("Blight Resistance", "9702860F") ] },
    Summon { name: "Lucilius", quest: "On the Threshold of Finality", astral: true, skill_pool: "CC0B0EF1", equip_pool: "F63793E4", skills: &[
        ("Alpha", "DBE1D775"), ("Beta", "8D2ADB6E"), ("Gamma", "5C862E13"), ("Berserker Echo", "EE85CD1F"),
        ("Tyranny", "71F11A9B"), ("Celestial Terra", "9232DC17") ] },
    Summon { name: "Beelzebub", quest: "On the Threshold of Chaos", astral: true, skill_pool: "BD452CC9", equip_pool: "B7D9379B", skills: &[
        ("Spartan Echo", "3D8153A1"), ("Supplementary DMG", "57AB5B10"), ("DMG Cap", "DC584F60"),
        ("Drain", "7CCFF74F"), ("Celestial Lumen", "A7726190"), ("Improved Guard", "0AA20846") ] },
    Summon { name: "Rolan", quest: "On the Threshold of The World", astral: true, skill_pool: "26428274", equip_pool: "A35D7A5C", skills: &[
        ("War Elemental", "4C588C27"), ("Uplift", "B5FF9FD3"), ("Autorevive", "95F3FA86"),
        ("Quick Cooldown", "318D12E9"), ("Drain", "7CCFF74F"), ("Aegis", "E0ABFDFE") ] },
    Summon { name: "Lilith", quest: "On the Threshold of Destruction", astral: true, skill_pool: "05205336", equip_pool: "DCB2B22B", skills: &[
        ("War Elemental", "4C588C27"), ("Uplift", "B5FF9FD3"), ("Potion Hoarder", "24883AF3"),
        ("Tyranny", "71F11A9B"), ("Linked Together", "3FEC5F80"), ("Improved Healing", "9389CC06") ] },
];
