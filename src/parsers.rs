use std::fs::File;

use lazy_static::lazy_static;
use regex::Regex;

use crate::parser_model::*;

lazy_static! {
    static ref END_PARSE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[Local\] (.+): ENDPARSE (.+)").unwrap();

    static ref EXP_INF_GAIN_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You gain ([0-9,]+) experience and ([0-9,]+) inf.+").unwrap();
    static ref LOOT_DROP_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You received (.+)[.]").unwrap();

    static ref MOB_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) HITS you! (.+) power had a (.+)% chance to hit and rolled a (.+)[.]").unwrap();
    static ref MOB_MISS_MATCHER: Regex 	= Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) MISSES! (.+) power had a (.+)% chance to hit, but rolled a (.+)[.]").unwrap();
    static ref MOB_PSEDUOPET_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) HITS you! (.+) power had a (.+)% chance to hit and rolled a (.+)[.]").unwrap();
    static ref MOB_PSEDUOPET_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) MISSES! (.+) power had a (.+)% chance to hit, but rolled a (.+)[.]").unwrap();
    static ref MOB_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) for ([0-9.]+) points of (.+) damage.*[.]").unwrap();
    static ref MOB_DAMAGE_DOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) for ([0-9.]+) points of (.+) damage over time[.]").unwrap();
    static ref MOB_PSEUDO_PET_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  (.+) hits you with their (.+) for ([0-9.]+) points of (.+) damage.*[.]").unwrap();
    static ref MOB_PSEUDO_PET_DAMAGE_DOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  (.+) hits you with their (.+) for ([0-9.]+) points of (.+) damage over time[.]").unwrap();

    //static ref DAM_PROC_MATCHER: Regex = Regex::new(r"(.+: Chance for .+)|(.+/Chance for .+)").unwrap();
    static ref PLAYER_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for ([0-9.]+) points of (.+) damage[.]").unwrap();
    static ref PLAYER_CRITICAL_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for ([0-9.]+) points of\s?(?:unresistable)?\s?(.+) damage \((.*)\)[.]").unwrap();
    static ref PLAYER_DOT_MATCHER:Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for (.+) points of (.+) damage over time.").unwrap();
    static ref PLAYER_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PLAYER_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PLAYER_KNOCKBACK_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You knock (.+) off their feet with your (.+)!").unwrap();
    static ref PLAYER_BLIND_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You blind (.+) with your (.+), reducing their perception and chance to hit!$").unwrap();
    static ref PLAYER_TERRIFY_PROC_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You terrify (.+) with your (.+) causing them to have reduced damage.$").unwrap();
    static ref PLAYER_READYING_POWER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Readying (.+)\.").unwrap();

    static ref ACTIVATION_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You activated the (.+) power.$").unwrap();
    static ref START_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[Local\] (.+): STARTPARSE (.+)").unwrap();

    static ref PSEDUO_PET_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You hit (.+) with your (.+) for (.+) points of (.+) damage.$").unwrap();
    static ref PSEDUO_PET_CRITICAL_DAMAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You hit (.+) with your (.+) for (.+) points of\s?(?:unresistable)?\s?(.+) damage \((.*)\).$").unwrap();
    static ref PSEDUO_PET_DAMAGE_DOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You hit (.+) with your (.+) for (.+) points of (.+) damage over time.").unwrap();
    static ref PSEDUO_PET_KNOCKBACK_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.*):  You knock (.+) off their feet with your (.+)!").unwrap();
    static ref PSEUDO_PET_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PSEUDO_PET_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PSEDUO_PET_RESIST_DEBUFF: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  Your (.+) reduces the resistances of (.+).$").unwrap();
    static ref PSEDUO_PET_SLEEP_DEBUFF: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You put (.+) to sleep with your (.+).$").unwrap();

    static ref PLAYER_VICTORY_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You have defeated (.+)").unwrap();
    static ref OTHER_VICTORY_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) has defeated (.+)").unwrap();
    static ref POWER_RECHARGED_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) is recharged.$").unwrap();

    static ref PLAYER_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You (Stun|Hold|Immobilize|Confuse|Taunt|Terrify?) (.+?) with your ([a-zA-Z ]+?)[.]").unwrap();
    static ref PSEUDO_PET_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You (Stun|Hold|Immobilize|Confuse|Taunt|Terrify?) (.+) with your ([a-zA-Z ]+)[.]").unwrap();
    static ref MOB_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?) (Stuns|Holds|Immobilizes|Confuses|Taunts|Terrifies?) you with their ([a-zA-Z ]+?)[.]").unwrap();
    static ref MOB_PSEUDO_PET_CONTROL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+?) (Stuns|Holds|Immobilizes|Confuses|Taunts|Terrifies?) you with their ([a-zA-Z ]+?)[.]").unwrap();

    static ref AUTOHIT_ONE_MATCHER: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power is autohit.").unwrap();
    static ref AUTOHIT_TWO_MATCHER: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) HITS you! Your (.+) power is autohit.").unwrap();
    static ref AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_ONE: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) HITS you! (.+) power was autohit.").unwrap();
    static ref AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_TWO: Regex  = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  HIT (.+)! Your (.+) power is autohit.").unwrap();

    static ref CHAT_MESSAGE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[(.*?)\] (.*)").unwrap();

    static ref UNPARSED_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+)").unwrap();


    static ref PLAYER_HEAL_OTHER_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You heal (.+) with (.+) for (.+) health points(.*)[.]$").unwrap();
    static ref PLAYER_HEALED_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) heals you with their (.+) for (.+) health points").unwrap();

    static ref PLAYER_ENDURANCE_OTHER_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with (.+) granting them (.*) points of endurance[.]$").unwrap();
    static ref PLAYER_ENDURANCE_BUFF: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) granting you (.+) points of endurance").unwrap();

    // todo next
    //static ref PLAYER_HEAL_HOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) heals you with their (.+) for (.+) health points over time.").unwrap();
    //static ref PSEUDO_PET_HEAL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You heal (.+) with (.+) for (.+) health points(.*)[.]$").unwrap();
    //static ref PSEUDO_PET_HEAL_HOT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  (.+) heals you with their (.+) for (.+) health points over time.").unwrap();
    /*
    public static final String PATTERN_WELCOME_VILLIAN	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Now entering the Rogue Isles, (.+)!";
    public static final String PATTERN_MISS 	 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).";
    public static final String PATTERN_DAM_CRIT		= ".+\\[CRITICAL\\].+";
    public static final String PATTERN_PSEUDOHEAL	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You heal (.+) with (.+) for (.+) health points.";
    public static final String PATTERN_END			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) granting them (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_END2			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Your (.+) grants you (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_PSEUDOEND	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You hit (.+) with your (.+) granting them (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_END_DAM		= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for (.+) points of their endurance(.*)[.]$";
    public static final String PATTERN_PSEUDOEND_DAM= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You hit (.+) with your (.+) for (.+) points of their endurance(.*)[.]$";
    public static final String PATTERN_XP_INCARN	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You have defeated (.+)";
    public static final String PATTERN_DROP_COMMON	= "^Invention:.+";
    public static final String COMBAT 			= "combat_";
    public static final String GLOBAL_PREFIX 	= "global_";
    public static final String SUMMARY 			= "summary_";
    */
}

pub fn initialize_matcher() -> Vec<fn(u32, &String) -> Option<FileDataPoint>> {
    // Order matters!
    vec![
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
        extract_player_terrify_proc,
        extract_player_knockback,
        extract_player_readying_power,
        extract_mob_pseudopet_hit,
        extract_mob_pseudopet_miss,
        extract_activation,
        extract_player_blind,
        extract_player_hit,
        extract_player_miss,
        extract_player_damage_dot,
        extract_player_damage,
        extract_player_critical_damage,
        extract_pseudo_pet_damage_dot,
        extract_pseudo_pet_damage,
        extract_pseudo_pet_hit,
        extract_pseudo_pet_critical_damage,
        extract_pseudo_pet_miss,
        extract_end_parse,
        extract_exp_inf_gain,
        extract_loot_drop,
        extract_mob_hit,
        extract_mob_miss,
        extract_player_victory,
        extract_other_victory,
        extract_power_recharged,
        extract_mob_pseudo_pet_control,
        extract_mob_control,
        extract_pseudo_pet_control,
        extract_player_control,
        extract_mob_pseudo_pet_damage_dot,
        extract_mob_pseudo_pet_damage,
        extract_mob_damage_dot,
        extract_mob_damage,
        extract_chat_message,
        //extract_recipe_drop,
        //extract_player_knockback,
        //extract_pet_knockback,
        extract_unparsed, //This should always be the last matcher, as it matches everything
    ]
}

pub fn extract_end_parse(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = END_PARSE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::EndParse {
            data_position: DataPosition::new(line_number, &data[1]),
            player_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_exp_inf_gain(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = EXP_INF_GAIN_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::ExpAndInfGain {
            data_position: DataPosition::new(line_number, &data[1]),
            exp: data[2].replace(",", "").parse().unwrap(),
            inf: data[3].replace(",", "").parse().unwrap(),
        }),
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
            action_result: HitOrMiss::new("player", &data[3], &data[4]),
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
            action_result: HitOrMiss::new("player", &data[3], &data[4]),
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

pub fn extract_activation(line_number: u32, line: &String) -> Option<FileDataPoint> {
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

pub fn extract_player_critical_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_CRITICAL_DAMAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerCriticalDamage {
            data_position: DataPosition::new(line_number, &data[1]),
            damage_dealt: DamageDealt::new(&data[2], &data[3], &data[4], &data[5]),
            critical_type: String::from(&data[6]),
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

pub fn extract_player_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_DAMAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerDamage {
            data_position: DataPosition::new(line_number, &data[1]),
            damage_dealt: DamageDealt::new(&data[2], &data[3], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_player_damage_dot(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_DOT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerDamageDoT {
            data_position: DataPosition::new(line_number, &data[1]),
            damage_dealt: DamageDealt::new(&data[2], &data[3], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_player_healed(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HEALED_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerHealed {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new(&data[2], "player", &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_heal_other(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HEAL_OTHER_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerHealOther {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new("player", &data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_endurance(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_ENDURANCE_BUFF.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerEndurance {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new(&data[2], "player", &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_player_endurance_other(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_ENDURANCE_OTHER_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerEnduranceOther {
            data_position: DataPosition::new(line_number, &data[1]),
            heal_action: HealEnduranceAction::new("player", &data[2], &data[3], &data[4]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_damage(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEDUO_PET_DAMAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetDamage {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            damage_dealt: DamageDealt::new(&data[3], &data[4], &data[5], &data[6]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_critical_damage(
    line_number: u32,
    line: &String,
) -> Option<FileDataPoint> {
    let caps = PSEDUO_PET_CRITICAL_DAMAGE_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetCriticalDamage {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            damage_dealt: DamageDealt::new(&data[3], &data[4], &data[5], &data[6]),
            critical_type: String::from(&data[7]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_damage_dot(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEDUO_PET_DAMAGE_DOT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetDamageDoT {
            data_position: DataPosition::new(line_number, &data[1]),
            pet_name: String::from(&data[2]),
            damage_dealt: DamageDealt::new(&data[3], &data[4], &data[5], &data[6]),
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetHit {
            data_position: DataPosition::new(line_number, &data[1]),
            name: String::from(&data[2]),
            action_result: HitOrMiss::new(&data[3], &data[4], &data[5]),
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
            action_result: HitOrMiss::new(&data[3], &data[4], &data[5]),
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
            control_type: ControlPower::new(&data[3], "player", &data[4]),
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
            damage_dealt: DamageDealt::new("player", &data[3], &data[4], &data[5]),
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
            damage_dealt: DamageDealt::new("player", &data[3], &data[4], &data[5]),
        }),
        None => None,
    }
}

pub fn extract_autohit_pseudo_pet_one(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_ONE.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPower {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from("player"),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_autohit_pseudo_pet_two(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_MATCHER_PSEUDO_PET_MATCHER_TWO.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPower {
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
            source: String::from("player"),
            target: String::from(&data[2]),
            power_name: String::from(&data[3]),
        }),
        None => None,
    }
}

pub fn extract_autohit_two(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = AUTOHIT_TWO_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::AutohitPower {
            data_position: DataPosition::new(line_number, &data[1]),
            source: String::from("player"),
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
