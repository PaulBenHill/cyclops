use chrono::{self, DateTime, Local, NaiveDateTime};

#[derive(Debug)]
pub struct HitOrMiss {
    target: String,
    power_name: String,
    chance_to_hit: f32,
}

impl HitOrMiss {
    pub fn new(target: &str, power_name: &str, chance_str: &str) -> Self {
        HitOrMiss {
            target: String::from(target),
            power_name: String::from(power_name),
            chance_to_hit: chance_str.parse().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct DataPosition {
    line_number: u32,
    date: DateTime<Local>,
}

impl DataPosition {
    pub fn new(line_number: u32, date_str: &str) -> Self {
        DataPosition {
            line_number: line_number,
            date: Self::extract_date(date_str),
        }
    }

    fn extract_date(date_str: &str) -> DateTime<Local> {
        NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
    }
}

#[derive(Debug)]
pub enum DamageType {
    Smashing,
    Lethal,
    Fire,
    Cold,
    Energy,
    NegativeEnergy,
    Psionic,
    Toxic,
    Electrolytic,
    Prismatic,
    Unique2,
}

impl DamageType {
    fn from_str(damage_type: &str) -> DamageType {
        match damage_type {
            "Smashing" => Self::Smashing,
            "Lethal" => Self::Lethal,
            "Fire" => Self::Fire,
            "Cold" => Self::Cold,
            "Energy" => Self::Energy,
            "Negative Energy" => Self::NegativeEnergy,
            "Psionic" => Self::Psionic,
            "Toxic" => Self::Toxic,
            "Electrolytic" => Self::Electrolytic,
            "Prismatic" => Self::Prismatic,
            "Unique2" => Self::Unique2,
            _ => panic!("Unable to parse damage type {}", damage_type),
        }
    }
}

#[derive(Debug)]
pub struct DamageDealt {
    target: String,
    power_name: String,
    damage: f32,
    damage_type: DamageType,
}

impl DamageDealt {
    pub fn new(target: &str, power_name: &str, damage_str: &str, damage_type: &str) -> Self {
        DamageDealt {
            target: String::from(target),
            power_name: String::from(power_name),
            damage: damage_str.parse().unwrap(),
            damage_type: DamageType::from_str(damage_type),
        }
    }
}

#[derive(Debug)]
pub enum ControlType {
    Stun,
    Hold,
    Immobilize,
    Confuse,
    Taunt,
    Terrify,
    Sleep,
}

impl ControlType {
    fn from_str(control_type: &str) -> ControlType {
        match control_type {
            "Stun" => Self::Stun,
            "Stuns" => Self::Stun,
            "Hold" => Self::Hold,
            "Holds" => Self::Hold,
            "Immobilize" => Self::Immobilize,
            "Immobilizes" => Self::Immobilize,
            "Confuse" => Self::Confuse,
            "Confuses" => Self::Confuse,
            "Taunt" => Self::Taunt,
            "Taunts" => Self::Taunt,
            "Terrify" => Self::Terrify,
            "Terrifies" => Self::Terrify,
            "Sleep" => Self::Sleep,
            "Sleeps" => Self::Sleep,
            _ => panic!("Unable to parse control type {}", control_type),
        }
    }
}

#[derive(Debug)]
pub struct ControlPower {
    control_type: ControlType,
    target: String,
    power_name: String,
}
impl ControlPower {
    pub fn new(control_type: &str, target: &str, power_name: &str) -> Self {
        ControlPower {
            control_type: ControlType::from_str(control_type),
            target: String::from(target),
            power_name: String::from(power_name),
        }
    }
}

#[derive(Debug)]
pub enum FileDataPoint {
    EndParse {
        data_position: DataPosition,
        player_name: String,
    },
    ExpAndInfGain {
        data_position: DataPosition,
        exp: u32,
        inf: u32,
    },
    LootMisc {
        data_position: DataPosition,
        loot: String,
    },
    MobControl {
        data_position: DataPosition,
        name: String,
        control_type: ControlPower,
    },
    MobDamage {
        data_position: DataPosition,
        name: String,
        damage_dealt: DamageDealt,
    },
    MobDamageDoT {
        data_position: DataPosition,
        name: String,
        damage_dealt: DamageDealt,
    },
    MobPseudoPetDamage {
        data_position: DataPosition,
        name: String,
        damage_dealt: DamageDealt,
    },
    MobPseudoPetDamageDoT {
        data_position: DataPosition,
        name: String,
        damage_dealt: DamageDealt,
    },
    MobPseudoPetControl {
        data_position: DataPosition,
        name: String,
        control_type: ControlPower,
    },
    MobHit {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    MobMiss {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    MobPseudoPetHit {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    MobPseudoPetMiss {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    ProcDamage,
    ProcDebuff,
    ProcHeal,
    ProcEnd,
    PlayerControl {
        data_position: DataPosition,
        control_type: ControlPower,
    },
    PlayerDamage {
        data_position: DataPosition,
        damage_dealt: DamageDealt,
    },
    PlayerDamageCritical,
    PlayerDamageDoT {
        data_position: DataPosition,
        damage_dealt: DamageDealt,
    },
    PlayerHeal,
    PlayerHealDoT,
    PlayerHit {
        data_position: DataPosition,
        action_result: HitOrMiss,
    },
    PlayerKnockback {
        data_position: DataPosition,
        target: String,
        power_name: String,
    },
    PlayerMiss,
    PlayerPowerActivation {
        data_position: DataPosition,
        power_name: String,
    },
    PseudoPetControl {
        data_position: DataPosition,
        pet_name: String,
        control_type: ControlPower,
    },
    PsuedoPetHit {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    PsuedoPetMiss {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    PsuedoPetDamage {
        data_position: DataPosition,
        pet_name: String,
        damage_dealt: DamageDealt,
    },
    PsuedoPetDamageCritical,
    PsuedoPetDamageDoT {
        data_position: DataPosition,
        pet_name: String,
        damage_dealt: DamageDealt,
    },
    PsuedoPetKnockdown {
        data_position: DataPosition,
        pet_name: String,
        target: String,
        power_name: String,
    },
    StartParse,
    WelcomeMessageHero,
    WelcomeMessageVillian,
    PlayerVictory {
        data_position: DataPosition,
        target: String,
    },
    OtherVictory {
        data_position: DataPosition,
        source: String,
        target: String,
    },
    PowerRecharged {
        data_position: DataPosition,
        power_name: String,
    },
    AutohitPower {
        data_position: DataPosition,
        source: String,
        target: String,
        power_name: String,
    },
    ChatMessage {
        data_position: DataPosition,
        category: String,
        message: String,
    },
    Unparsed {
        data_position: DataPosition,
        content: String,
    },
}
