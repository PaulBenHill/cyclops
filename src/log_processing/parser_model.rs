use chrono::{self, DateTime, Local, NaiveDateTime};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Serialize, Clone)]
pub struct HitOrMiss {
    pub target: String,
    pub power_name: String,
    pub chance_to_hit: f32,
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

#[derive(Debug, Serialize, Clone)]
pub struct DataPosition {
    pub line_number: u32,
    #[serde(serialize_with = "date_to_string")]
    pub date: DateTime<Local>,
}

impl DataPosition {
    pub fn new(ln: u32, date_str: &str) -> Self {
        DataPosition {
            line_number: ln,
            date: Self::extract_date(date_str),
        }
    }

    pub fn extract_date(date_str: &str) -> DateTime<Local> {
        NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    Unique,
    Unique1,
    Unique2,
    Special,
    Quantum,
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
            "Unique" => Self::Unique,
            "Unique1" => Self::Unique1,
            "Unique2" => Self::Unique2,
            "Special" => Self::Special,
            "Quantum" => Self::Quantum,
            _ => panic!("Unable to parse damage type {}", damage_type),
        }
    }
}

impl fmt::Display for DamageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamageDealt {
    pub target: String,
    pub power_name: String,
    pub damage: f32,
    pub damage_type: DamageType,
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub struct HealEnduranceAction {
    source: String,
    target: String,
    power_name: String,
    amount: f32,
}

impl HealEnduranceAction {
    pub fn new(source: &str, target: &str, power_name: &str, amount: &str) -> Self {
        let value: f32 = match amount.parse() {
            Ok(number) => number,
            Err(error) => {
                println!("Unparsable heal or endurance number {}:{:?}", amount, error);
                12345.0
            }
        };
        HealEnduranceAction {
            source: String::from(source),
            target: String::from(target),
            power_name: String::from(power_name),
            amount: value,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum FileDataPoint {
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
    OtherAutoHit {
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
    PlayerBlindDebuff {
        data_position: DataPosition,
        target: String,
        power_name: String,
    },
    PlayerDirectDamage {
        data_position: DataPosition,
        damage_dealt: DamageDealt,
    },
    PlayerCriticalDamage {
        data_position: DataPosition,
        damage_dealt: DamageDealt,
        critical_type: String,
    },
    PlayerDamageDoT {
        data_position: DataPosition,
        damage_dealt: DamageDealt,
    },
    PlayerHealOther {
        data_position: DataPosition,
        heal_action: HealEnduranceAction,
    },
    PlayerHealed {
        data_position: DataPosition,
        heal_action: HealEnduranceAction,
    },
    PlayerEnduranceOther {
        data_position: DataPosition,
        heal_action: HealEnduranceAction,
    },
    PlayerEndurance {
        data_position: DataPosition,
        heal_action: HealEnduranceAction,
    },
    PlayerHealDoT {
        data_position: DataPosition,
        heal_action: HealEnduranceAction,
    },
    PlayerHit {
        data_position: DataPosition,
        action_result: HitOrMiss,
    },
    PlayerStreakbreakerHit {
        data_position: DataPosition,
        action_result: HitOrMiss,
    },
    PlayerMiss {
        data_position: DataPosition,
        action_result: HitOrMiss,
    },
    PlayerKnockback {
        data_position: DataPosition,
        target: String,
        power_name: String,
    },
    PlayerPowerRecharged {
        data_position: DataPosition,
        power_name: String,
    },
    PlayerFulcrumShift {
        data_position: DataPosition,
    },
    PlayerTerrifyProc {
        data_position: DataPosition,
        target: String,
        power_name: String,
    },
    PlayerReadyingPower {
        data_position: DataPosition,
        power_name: String,
    },
    PlayerPowerActivation {
        data_position: DataPosition,
        power_name: String,
    },
    PseudoPetControl {
        data_position: DataPosition,
        pet_name: String,
        control_type: ControlPower,
    },
    PseudoPetHit {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    PseudoPetStreakbreakerHit {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    PsuedoPetMiss {
        data_position: DataPosition,
        name: String,
        action_result: HitOrMiss,
    },
    PseudoPetDirectDamage {
        data_position: DataPosition,
        pet_name: String,
        damage_dealt: DamageDealt,
    },
    PsuedoPetCriticalDamage {
        data_position: DataPosition,
        pet_name: String,
        damage_dealt: DamageDealt,
        critical_type: String,
    },
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
    PseudoPetResistDebuff {
        data_position: DataPosition,
        pet_name: String,
        target: String,
        power_name: String,
    },
    PseudoPetSleepDebuff {
        data_position: DataPosition,
        pet_name: String,
        target: String,
        power_name: String,
    },
    SessionMarker {
        data_position: DataPosition,
        player_name: String,
    },
    PlayerVictory {
        data_position: DataPosition,
        target: String,
    },
    OtherVictory {
        data_position: DataPosition,
        source: String,
        target: String,
    },
    AutohitPower {
        data_position: DataPosition,
        source: String,
        target: String,
        power_name: String,
    },
    AutohitPowerIgnore {
        data_position: DataPosition,
        source: String,
        target: String,
        power_name: String,
    },
    ResistanceDebuff {
        data_position: DataPosition,
    },
    TargetUnaffected {
        data_position: DataPosition,
    },
    ChatMessage {
        data_position: DataPosition,
        category: String,
        message: String,
    },
    Unparsed {
        #[serde(flatten)]
        data_position: DataPosition,
        content: String,
    },
}

fn date_to_string<S>(date: &DateTime<Local>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{}", date.format(r"%Y-%M-%d %H:%M:%S")))
}
