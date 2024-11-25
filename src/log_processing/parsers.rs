use lazy_static::lazy_static;
use regex::Regex;

use crate::log_processing::parser_model::*;

lazy_static! {
    static ref TEST_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) ").unwrap();
    static ref TEST_HIT_MATCHER: Regex = Regex::new(r"^HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();

    static ref SESSION_MARKER_MATCHER_1: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[Local\] (.+): (?:(STARTPARSE|ENDPARSE).*).*").unwrap();
    static ref SESSION_MARKER_MATCHER_2: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (?:Now entering the Rogue Isles|Welcome to City of Heroes), (.+)!").unwrap();

    //static ref EXP_INF_GAIN_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You gain ([0-9,]+) experience and ([0-9,]+) inf.+").unwrap();
    static ref EXP_INF_GAIN_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You gain (?:(?P<exp>[0-9,]+) experience)?(?: and )?(?:(?:(?P<inf>[0-9,]+)) (?:influence|infamy))?.").unwrap();
    static ref LOOT_DROP_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You received (.+)[.]").unwrap();

    static ref MOB_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) HITS you! (.+) power had a (.+)% chance to hit and rolled a (.+)[.]").unwrap();
    static ref MOB_MISS_MATCHER: Regex 	= Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) MISSES! (.+) power had a (.+)% chance to hit, but rolled a (.+)[.]").unwrap();
    static ref MOB_PSEDUOPET_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) HITS you! (.+) power had a (.+)% chance to hit and rolled a (.+)[.]").unwrap();
    static ref MOB_PSEDUOPET_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) MISSES! (.+) power had a (.+)% chance to hit, but rolled a (.+)[.]").unwrap();
    static ref MOB_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) for ([0-9.]+) points of\s?(?:unresistable)?\s?(.+) damage.*[.]").unwrap();
    static ref MOB_DAMAGE_DOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) for ([0-9.]+) points of\s?(?:unresistable)?\s?(.+) damage over time[.]").unwrap();
    static ref MOB_PSEUDO_PET_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  (.+) hits you with their (.+) for ([0-9.]+) points of\s?(?:unresistable)?\s?(.+) damage.*[.]").unwrap();
    static ref MOB_PSEUDO_PET_DAMAGE_DOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  (.+) hits you with their (.+) for ([0-9.]+) points of\s?(?:unresistable)?\s?(.+) damage over time[.]").unwrap();

    static ref PLAYER_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PLAYER_HIT_STREAKBREAKER_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power was forced to hit by streakbreaker.").unwrap();
    static ref PLAYER_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PLAYER_KNOCKBACK_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You knock (.+) off their feet with your (.+)!").unwrap();
    static ref PLAYER_BLIND_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You blind (.+) with your (.+), reducing their perception and chance to hit!$").unwrap();
    static ref PLAYER_TERRIFY_PROC_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You terrify (.+) with your (.+) causing them to have reduced damage.$").unwrap();
    static ref PLAYER_READYING_POWER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Readying (.+)\.").unwrap();
    static ref FULCRUM_SHIFT_POWER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) The Fulcrum Shift has increased .+!").unwrap();

    static ref ACTIVATION_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You activated the (.+) power.$").unwrap();

    static ref PSEDUO_PET_KNOCKBACK_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.*):  You knock (.+) off their feet with your (.+)!").unwrap();
    static ref PSEUDO_PET_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PSEUDO_PET_STREAKBREAKER_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  HIT (.+)! Your (.+) power was forced to hit by streakbreaker.").unwrap();
    static ref PSEUDO_PET_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PSEDUO_PET_RESIST_DEBUFF: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  Your (.+) reduces the resistances of (.+).$").unwrap();
    static ref PSEDUO_PET_SLEEP_DEBUFF: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You put (.+) to sleep with your (.+).$").unwrap();

    static ref PLAYER_VICTORY_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You have defeated (.+)").unwrap();
    static ref OTHER_VICTORY_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) has defeated (.+)").unwrap();
    static ref POWER_RECHARGED_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) is recharged.$").unwrap();

    static ref PLAYER_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You (Stun|Hold|Immobilize|Confuse|Taunt|Terrify?) (.+?) with your (.+)[.]").unwrap();
    static ref PSEUDO_PET_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You (Stun|Hold|Immobilize|Confuse|Taunt|Terrify?) (.+) with your (.+)[.]").unwrap();
    static ref MOB_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?) (Stuns|Holds|Immobilizes|Confuses|Taunts|Terrifies?) you with their (.+)[.]").unwrap();
    static ref MOB_PSEUDO_PET_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+?) (Stuns|Holds|Immobilizes|Confuses|Taunts|Terrifies?) you with their ([a-zA-Z ]+?)[.]").unwrap();

    static ref AUTOHIT_ONE_MATCHER: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power is autohit.").unwrap();
    static ref AUTOHIT_TWO_MATCHER: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) HITS you! Your (.+) power was autohit.").unwrap();
    static ref AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_ONE: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  HIT (.+)! Your (.+) power is autohit.").unwrap();
    static ref AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_TWO: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) HITS you! (.+) power was autohit.").unwrap();
    static ref OTHER_AUTO_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) HITS you! (.+) power was autohit\.").unwrap();

    static ref CHAT_MESSAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[(.*?)\] (.*)").unwrap();

    static ref UNPARSED_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+)").unwrap();


    static ref PLAYER_HEAL_OTHER_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You heal (.+) with (.+) for (.+) health points(.*)[.]$").unwrap();
    static ref PLAYER_HEALED_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) heals you with their (.+) for (.+) health points").unwrap();

    static ref PLAYER_ENDURANCE_OTHER_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with (.+) granting them (.*) points of endurance[.]$").unwrap();
    static ref PLAYER_ENDURANCE_BUFF_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) granting you (.+) points of endurance").unwrap();

    static ref RESISTANCE_DEBUFF: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+).*reduc.*resistance.*").unwrap();
    static ref TARGET_UNAFFECTED: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Target is unaffected by.*").unwrap();

    // Combined regex
    static ref PLAYER_ATTACK_DAMAGE: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for ([0-9.]+) points of\s?(?:unresistable)?\s?(.+) damage\s*(?P<dot>over time){0,1}(?P<critical>.+){0,1}[.]").unwrap();
    static ref PSEUDO_PET_ATTACK_DAMAGE: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You hit (.+) with your (.+) for (.+) points of\s?(?:unresistable)?\s?(.+) damage\s*(?P<dot>over time){0,1}(?P<critical>.+){0,1}[.]").unwrap();
    static ref PLAYER_PET_ATTACK_DAMAGE: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+)(?:(?P<pet>.+){0,1}:){0,1}\s+You hit (.+) with your (.+) for (.+) points of\s?(?:unresistable)?\s?(.+) damage\s*(?P<dot>over time){0,1}(?P<critical>.+){0,1}[.]").unwrap();

    // todo next
    //static ref PLAYER_HEAL_HOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) heals you with their (.+) for (.+) health points over time.").unwrap();
    //static ref PSEUDO_PET_HEAL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You heal (.+) with (.+) for (.+) health points(.*)[.]$").unwrap();
    //static ref PSEUDO_PET_HEAL_HOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) heals you with their (.+) for (.+) health points over time.").unwrap();
    /*
    public static final String PATTERN_PSEUDOHEAL	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You heal (.+) with (.+) for (.+) health points.";
    public static final String PATTERN_END			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) granting them (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_END2			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Your (.+) grants you (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_PSEUDOEND	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You hit (.+) with your (.+) granting them (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_END_DAM		= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for (.+) points of their endurance(.*)[.]$";
    public static final String PATTERN_PSEUDOEND_DAM= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You hit (.+) with your (.+) for (.+) points of their endurance(.*)[.]$";
    public static final String PATTERN_DROP_COMMON	= "^Invention:.+";
    public static final String COMBAT 			= "combat_";
    public static final String GLOBAL_PREFIX 	= "global_";
    public static final String SUMMARY 			= "summary_";
    */
}

pub fn initialize_matchers() -> Vec<fn(u32, &String) -> Option<FileDataPoint>> {
    // Order matters!
    vec![
        extract_session_marker_1,
        extract_session_marker_2,
        pseudo_pet_attack_damage,
        extract_pseudo_pet_miss,
        extract_pseudo_pet_hit,
        extract_pseudo_pet_streakbreaker_hit,
        player_pet_attack_damage,
        extract_power_recharged,
        extract_exp_inf_gain,
        extract_other_victory,
        extract_player_activation,
        extract_chat_message,
        extract_mob_pseudopet_hit,
        extract_mob_pseudopet_miss,
        extract_player_terrify_proc,
        extract_fulcrum_shift,
        extract_player_hit,
        extract_player_endurance,
        extract_player_endurance_other,
        extract_player_healed,
        extract_player_heal_other,
        extract_autohit_one,
        extract_autohit_two,
        extract_autohit_pseudo_pet_one,
        extract_autohit_pseudo_pet_two,
        extract_pseudo_pet_knockback,
        extract_pseudo_pet_resist_debuff,
        extract_pseudo_pet_sleep_debuff,
        extract_player_knockback,
        extract_player_readying_power,
        extract_player_blind,
        extract_player_streakbreaker_hit,
        extract_player_miss,
        extract_loot_drop,
        extract_mob_hit,
        extract_mob_miss,
        extract_player_victory,
        extract_mob_pseudo_pet_control,
        extract_mob_control,
        extract_pseudo_pet_control,
        extract_player_control,
        extract_mob_pseudo_pet_damage_dot,
        extract_mob_pseudo_pet_damage,
        extract_mob_damage_dot,
        extract_mob_damage,
        extract_resistance_debuff,
        extract_other_auto_hit,
        extract_target_unaffected,
        //extract_recipe_drop,
        //extract_player_knockback,
        //extract_pet_knockback,
        extract_unparsed, //This should always be the last matcher, as it matches everything
    ]
}

pub fn extract_session_marker_1(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = SESSION_MARKER_MATCHER_1.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::SessionMarker {
            data_position: DataPosition::new(line_number, &data[1]),
            player_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_session_marker_2(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = SESSION_MARKER_MATCHER_2.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::SessionMarker {
            data_position: DataPosition::new(line_number, &data[1]),
            player_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn player_pet_attack_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_ATTACK_DAMAGE.captures(line);

    match caps {
        Some(data) => {
            if data.name("critical").is_some() {
                Some(FileDataPoint::PlayerCriticalDamage {
                    data_position: DataPosition::new(line_number, &data[1]),
                    damage_dealt: DamageDealt::new(&data[2], &data[3], &data[4], &data[5]),
                    critical_type: String::from(&data[7]),
                })
            } else if data.name("dot").is_some() {
                Some(FileDataPoint::PlayerDamageDoT {
                    data_position: DataPosition::new(line_number, &data[1]),
                    damage_dealt: DamageDealt::new(&data[2], &data[3], &data[4], &data[5]),
                })
            } else {
                Some(FileDataPoint::PlayerDirectDamage {
                    data_position: DataPosition::new(line_number, &data[1]),
                    damage_dealt: DamageDealt::new(&data[2], &data[3], &data[4], &data[5]),
                })
            }
        }
        None => None,
    }
}

pub fn pseudo_pet_attack_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_ATTACK_DAMAGE.captures(line);

    match caps {
        Some(data) => {
            if data.name("critical").is_some() {
                Some(FileDataPoint::PsuedoPetCriticalDamage {
                    data_position: DataPosition::new(line_number, &data[1]),
                    pet_name: String::from(&data[2]),
                    damage_dealt: DamageDealt::new(
                        &data[3],
                        &format!("{}: {}", &data[2], &data[4]),
                        &data[5],
                        &data[6],
                    ),
                    critical_type: String::from(&data[8]),
                })
            } else if data.name("dot").is_some() {
                Some(FileDataPoint::PsuedoPetDamageDoT {
                    data_position: DataPosition::new(line_number, &data[1]),
                    pet_name: String::from(&data[2]),
                    damage_dealt: DamageDealt::new(
                        &data[3],
                        &format!("{}: {}", &data[2], &data[4]),
                        &data[5],
                        &data[6],
                    ),
                })
            } else {
                Some(FileDataPoint::PseudoPetDirectDamage {
                    data_position: DataPosition::new(line_number, &data[1]),
                    pet_name: String::from(&data[2]),
                    damage_dealt: DamageDealt::new(
                        &data[3],
                        &format!("{}: {}", &data[2], &data[4]),
                        &data[5],
                        &data[6],
                    ),
                })
            }
        }
        None => None,
    }
}

pub fn extract_exp_inf_gain(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = EXP_INF_GAIN_MATCHER.captures(line);

    match caps {
        Some(data) => {
            let mut experience = 0;
            let mut influence = 0;
            if let Some(value) = data.name("exp") {
                experience = value.as_str().replace(',', "").parse().unwrap();
            }
            if let Some(value) = data.name("inf") {
                influence = value.as_str().replace(',', "").parse().unwrap();
            }
            Some(FileDataPoint::ExpAndInfGain {
                data_position: DataPosition::new(line_number, &data[1]),
                exp: experience,
                inf: influence,
            })
        }
        None => None,
    }
}

pub fn extract_loot_drop(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = LOOT_DROP_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::LootMisc {
            data_position: DataPosition::new(line_number, &data[1]),
            loot: String::from(&data[2]),
        }),
        None => None,
    }
}

/*
todo: subtypes of loot
LootRecipe
static ref LOOT_RECIPE_MATCHER: Regex = Regex::new(r"You received .+\(Recipe\).*").unwrap();
pub fn extract_recipe_drop(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = LOOT_RECIPE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::LootRecipe {
            data_position: DataPosition::new(line_number, &data[1]),
            recipe: String::from(&data[2]),
        }),
        None => None,
    }
}
*/

pub fn extract_mob_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobHit {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new("Player", &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_other_auto_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = OTHER_AUTO_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::OtherAutoHit {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new("Player", &data[3], "100"),
        }),
        None => None,
    }
}

pub fn extract_mob_miss(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_MISS_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobMiss {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new("Player", &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_mob_pseudopet_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_PSEDUOPET_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobPseudoPetHit {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[3]),
            action_result: HitOrMiss::new(&data[2], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_mob_pseudopet_miss(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_PSEDUOPET_MISS_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobPseudoPetMiss {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[3]),
            action_result: HitOrMiss::new(&data[2], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_player_activation(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = ACTIVATION_MATCHER.captures(line);
    match caps {
        Some(data) => Some(FileDataPoint::PlayerPowerActivation {
            data_position: DataPosition::new(line_number, &data[1]),
            power_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_player_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerHit {
            data_position: DataPosition::new(line_number, &data[1]),
            action_result: HitOrMiss::new(&data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_streakbreaker_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HIT_STREAKBREAKER_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerStreakbreakerHit {
            data_position: DataPosition::new(line_number, &data[1]),
            action_result: HitOrMiss::new(&data[2], &data[3], &"100"),
        }),
        None => None,
    }
}

pub fn extract_player_miss(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_MISS_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerMiss {
            data_position: DataPosition::new(line_number, &data[1]),
            action_result: HitOrMiss::new(&data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

fn extract_fulcrum_shift(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = FULCRUM_SHIFT_POWER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerFulcrumShift {
            data_position: DataPosition::new(line_number, &data[1]),
        }),
        None => None,
    }
}

fn extract_player_blind(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_BLIND_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerBlindDebuff {
            data_position: DataPosition::new(line_number, &data[1]),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

fn extract_player_terrify_proc(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_TERRIFY_PROC_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerTerrifyProc {
            data_position: DataPosition::new(line_number, &data[1]),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

fn extract_player_readying_power(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_READYING_POWER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerReadyingPower {
            data_position: DataPosition::new(line_number, &data[1]),
            power_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_player_healed(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HEALED_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerHealed {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new(&data[2], "Player", &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_heal_other(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HEAL_OTHER_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerHealOther {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new("Player", &data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_endurance(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_ENDURANCE_BUFF_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerEndurance {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new(&data[2], "Player", &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_endurance_other(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_ENDURANCE_OTHER_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerEnduranceOther {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new("Player", &data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PseudoPetHit {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new(
                &data[3],
                &format!("{}: {}", &data[2], &data[4]),
                &data[5],
            ),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_streakbreaker_hit(
    line_number: u32,
    line: &String,
) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_STREAKBREAKER_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PseudoPetStreakbreakerHit {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new(&data[3], &format!("{}: {}", &data[2], &data[4]), &"100"),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_miss(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_MISS_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetMiss {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new(
                &data[3],
                &format!("{}: {}", &data[2], &data[4]),
                &data[5],
            ),
        }),
        None => None,
    }
}

pub fn extract_player_knockback(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_KNOCKBACK_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerKnockback {
            data_position: DataPosition::new(line_number, &data[1]),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_knockback(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEDUO_PET_KNOCKBACK_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetKnockdown {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            target: String::from(&data[3]),
            power_name: String::from(&data[4]),
        }),
        None => None,
    }
}

fn extract_pseudo_pet_resist_debuff(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEDUO_PET_RESIST_DEBUFF.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PseudoPetResistDebuff {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            target: String::from(&data[4]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

fn extract_pseudo_pet_sleep_debuff(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEDUO_PET_SLEEP_DEBUFF.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PseudoPetSleepDebuff {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            target: String::from(&data[4]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_player_victory(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_VICTORY_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerVictory {
            data_position: DataPosition::new(line_number, &data[1]),
            target: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_other_victory(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = OTHER_VICTORY_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::OtherVictory {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from(&data[2]),
            target: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_power_recharged(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = POWER_RECHARGED_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PowerRecharged {
            data_position: DataPosition::new(line_number, &data[1]),
            power_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_control(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_CONTROL_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PseudoPetControl {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            control_type: ControlPower::new(&data[3], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_mob_control(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_CONTROL_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobControl {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            control_type: ControlPower::new(&data[3], "Player", &data[4]),
        }),
        None => None,
    }
}

pub fn extract_mob_pseudo_pet_control(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_PSEUDO_PET_CONTROL_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobPseudoPetControl {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[3]),
            control_type: ControlPower::new(&data[4], &data[2], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_player_control(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_CONTROL_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerControl {
            data_position: DataPosition::new(line_number, &data[1]),
            control_type: ControlPower::new(&data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_mob_pseudo_pet_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_PSEUDO_PET_DAMAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobPseudoPetDamage {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[3]),
            damage_dealt: DamageDealt::new(&data[2], &data[4], &data[5], &data[6]),
        }),
        None => None,
    }
}

pub fn extract_mob_pseudo_pet_damage_dot(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_PSEUDO_PET_DAMAGE_DOT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobPseudoPetDamageDoT {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            damage_dealt: DamageDealt::new(&data[3], &data[4], &data[5], &data[6]),
        }),
        None => None,
    }
}

pub fn extract_mob_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_DAMAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobDamage {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            damage_dealt: DamageDealt::new("Player", &data[3], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_mob_damage_dot(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = MOB_DAMAGE_DOT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::MobDamageDoT {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            damage_dealt: DamageDealt::new("Player", &data[3], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_autohit_pseudo_pet_one(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_ONE.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPowerIgnore {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from("Player"),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_autohit_pseudo_pet_two(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_TWO.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPowerIgnore {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from(&data[2]),
            target: String::from(&data[3]),
            power_name: String::from(&data[4]),
        }),
        None => None,
    }
}

pub fn extract_autohit_one(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_ONE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPower {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from("Player"),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_autohit_two(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_TWO_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPowerIgnore {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from("Player"),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_chat_message(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = CHAT_MESSAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::ChatMessage {
            data_position: DataPosition::new(line_number, &data[1]),
            category: String::from(&data[2]),
            message: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_resistance_debuff(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = RESISTANCE_DEBUFF.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::ResistanceDebuff { 
            data_position: DataPosition::new(line_number, &data[1])
        }),
        None => None,
    }
}

pub fn extract_target_unaffected(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = TARGET_UNAFFECTED.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::TargetUnaffected { 
            data_position: DataPosition::new(line_number, &data[1])
        }),
        None => None,
    }
}

pub fn extract_unparsed(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = UNPARSED_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::Unparsed {
            data_position: DataPosition::new(line_number, &data[1]),
            content: String::from(&data[2]),
        }),
        None => None,
    }
}
