use core::fmt;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref MINION_HP_TABLE: Vec<MobHP> = initialize_mob_hp_tables(MobClass::Minion);
}

#[derive(Debug)]
pub enum MobClass {
    Minion,
    Lieutenant,
    Boss,
    EliteBoss,
    Archvillian,
    Hero,
}

impl fmt::Display for MobClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MobClass::Minion => write!(f, "minion"),
            MobClass::Lieutenant => write!(f, "lieutenant"),
            MobClass::Boss => write!(f, "boss"),
            MobClass::EliteBoss => write!(f, "eliteboss"),
            MobClass::Archvillian => write!(f, "archvillian"),
            MobClass::Hero => write!(f, "hero"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MobHP {
    pub level: i32,
    pub hp: i32,
}

pub fn initialize_mob_hp_tables(mob_class: MobClass) -> Vec<MobHP> {
    csv::Reader::from_path(format!(".\\resources\\{}_hp_table.csv", mob_class))
        .unwrap()
        .deserialize()
        .map(|r| r.unwrap()).collect()
}

pub fn get_mob_hp(level: &String ) -> i32 {
    let l = i32::from_str_radix(level, 10).unwrap();
    MINION_HP_TABLE.iter().find(|d| l == d.level).unwrap().hp
}
