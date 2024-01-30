use chrono::{self, DateTime, Local, NaiveDateTime};
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
struct HitOrMiss {
    target: String,
    power_name: String,
    chance_to_hit: f32,
}

#[derive(Debug)]
struct DataPosition {
    line_number: u32,
    date: DateTime<Local>,
}

#[derive(Debug)]
pub enum FileDataPoint {
    EndParse,
    ExpGain,
    InfuenceGain,
    LootMisc,
    LootReceipe,
    MobHit,
    MobMiss,
    ProcDamage,
    ProcDebuff,
    ProcHeal,
    ProcEnd,
    PlayerDamage,
    PlayerDamageCritical,
    PlayerDamageDoT,
    PlayerHeal,
    PlayerHealDoT,
    PlayerHit {
        data_position: DataPosition,
        action_result: HitOrMiss,
    },
    PlayerMiss,
    PlayerPowerActivation {
        data_position: DataPosition,
        power_name: String,
    },
    PlayerPowerRecharge,
    PsuedoPetHit {
        data_position: DataPosition,
        pet_name: String,
        action_result: HitOrMiss,
    },
    PsuedoPetMiss {
        data_position: DataPosition,
        pet_name: String,
        action_result: HitOrMiss,
    },
    PsuedoPetDamage,
    PsuedoPetDamageCritical,
    PsuedoPetDamageDoT,
    PsuedoPetKnockdown,
    StartParse,
    WelcomeMessageHero,
    WelcomeMessageVillian,
}

lazy_static! {

    static ref DATE_MATCHER: Regex =
        Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) .+").unwrap();
    static ref ACTIVATION_MATCHER: Regex =
        Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You activated the (.+) power.$")
            .unwrap();
    static ref PLAYER_HIT_MATCHER: Regex = Regex::new("^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref GLOBAL_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[Local\] (.+): STARTPARSE (.+)").unwrap();
    static ref END_PARSE_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) \[Local\] (.+): ENDPARSE (.+)").unwrap();
    static ref PLAYER_KNOCKBACK_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You knock (.+) off their feet with your (.+)!").unwrap();
    static ref PET_KNOCKBACK_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.*):\s+You knock (.+) off their feet with your (.+)!").unwrap();
    static ref WELCOME_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Welcome to City of Heroes, (.+)!").unwrap();
    static ref MOB_MISS_MATCHER: Regex 	= Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) MISSES! (.+) power had a (.+)% chance to hit, but rolled a (.+)[.]").unwrap();
    static ref PSEUDO_PET_HIT_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  HIT (.+)! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();
    static ref PSEUDO_PET_MISS_MATCHER: Regex = Regex::new(r"^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).").unwrap();


    /*
    public static final String PATTERN_WELCOME_VILLIAN	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Now entering the Rogue Isles, (.+)!";

    public static final String PATTERN_RECHARGED	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) is recharged.$";
    public static final String PATTERN_HIT_AUTO	 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) HIT (.+)! Your (.+) power is autohit.";
    public static final String PATTERN_MISS 	 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) MISSED (.+)!! Your (.+) power had a (.+)% chance to hit, you rolled a (.+).";

    public static final String PATTERN_HIT_YOU 		= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) HITS you! (.+) power had a (.+)% chance to hit and rolled a (.+)[.]";
    public static final String PATTERN_MISS_YOU 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) MISSES! (.+) power had a (.+)% chance to hit, but rolled a (.+)[.]";


    public static final String PATTERN_PSEUDOKNOCK 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You knock (.+) off their feet with your (.+)!";

    public static final String PATTERN_HOLD 	 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You (Stun|Hold|Immobilize|Confuse|Taunt|Terrify?) (.+?) with your ([a-zA-Z ]+?)[.]";
    public static final String PATTERN_PSEUDOHOLD 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You (Stun|Hold|Immobilize|Confuse|Taunt|Terrify?) (.+) with your ([a-zA-Z ]+)[.]";

    public static final String PATTERN_DAMAGE 	 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for ([0-9.]+) points of (.+) damage(.*)[.]";
    public static final String PATTERN_DAMAGE_OVER_TIME
                                                    = "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for (.+) points of (.+) damage over time.";
    public static final String PATTERN_DAM_PROC		= "(.+: Chance for .+)|(.+/Chance for .+)";
    public static final String PATTERN_DAM_CRIT		= ".+\\[CRITICAL\\].+";

    public static final String PATTERN_PSEUDODAM	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You hit (.+) with your (.+) for (.+) points of (.+) damage.$";

    public static final String PATTERN_DAMAGE_YOU 	 	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) for ([0-9.]+) points of (.+) damage.*[.]";
    public static final String PATTERN_DAMAGE_OVER_TIME_YOU
                                                    = "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) hits you with their (.+) for ([0-9.]+) points of (.+) damage over time[.]";


    public static final String PATTERN_PSEUDODAMAGE_OVER_TIME
                                                    = "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+?):  You hit (.+) with your (.+) for (.+) points of (.+) damage over time.";

    public static final String PATTERN_HEAL			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You heal (.+) with (.+) for (.+) health points(.*)[.]$";
    public static final String PATTERN_HEAL_OVER_TIME
                                                    = "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You are healed by your (.+) for (.+) health points.*[.]$";
    public static final String PATTERN_PSEUDOHEAL	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You heal (.+) with (.+) for (.+) health points.";

    public static final String PATTERN_END			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) granting them (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_END2			= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) Your (.+) grants you (.+) points of endurance(.*)[.]$";
    public static final String PATTERN_PSEUDOEND	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You hit (.+) with your (.+) granting them (.+) points of endurance(.*)[.]$";

    public static final String PATTERN_END_DAM		= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You hit (.+) with your (.+) for (.+) points of their endurance(.*)[.]$";
    public static final String PATTERN_PSEUDOEND_DAM= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+):  You hit (.+) with your (.+) for (.+) points of their endurance(.*)[.]$";

    public static final String PATTERN_DEFEATS_SELF = "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You have defeated (.+)";
    public static final String PATTERN_DEFEATS_OTHER= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) (.+) has defeated (.+)";

    public static final String PATTERN_XP_INF		= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You gain ([0-9,]+) experience and ([0-9,]+) inf.+";
    public static final String PATTERN_XP_INCARN	= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You have defeated (.+)";

    public static final String PATTERN_DROPS		= "^([0-9]+-[0-9]+-[0-9]+ [0-9]+:[0-9]+:[0-9]+) You received (.+)[.]";
    public static final String PATTERN_DROP_COMMON	= "^Invention:.+";
    public static final String PATTERN_DROP_SET		= ".+\\(Recipe\\).*";

    public static final String COMBAT 			= "combat_";
    public static final String GLOBAL_PREFIX 	= "global_";
    public static final String SUMMARY 			= "summary_";
    */
}

pub fn initialize_matcher() -> Vec<fn(u32, &String) -> Option<FileDataPoint>> {
    vec![
        extract_activation,
        extract_player_hit,
        extract_pseudo_pet_hit,
        extract_pseudo_pet_miss,
        //extract_player_knockback,
        //extract_pet_knockback,
    ]
}

pub fn extract_date(date_str: &str) -> DateTime<Local> {
    NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
}

pub fn extract_activation(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = ACTIVATION_MATCHER.captures(line);
    match caps {
        Some(data) => Some(FileDataPoint::PlayerPowerActivation {
            data_position: DataPosition {
                line_number: line_number,
                date: extract_date(&data[1]),
            },
            power_name: String::from(&data[2]),
        }),
        None => None,
    }
}

pub fn extract_player_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PLAYER_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PlayerHit {
            data_position: DataPosition {
                line_number: line_number,
                date: extract_date(&data[1]),
            },
            action_result: HitOrMiss {
                target: String::from(&data[2]),
                power_name: String::from(&data[3]),
                chance_to_hit: data[4].parse().unwrap(),
            },
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_hit(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_HIT_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetHit {
            data_position: DataPosition {
                line_number: line_number,
                date: extract_date(&data[1]),
            },
            pet_name: String::from(&data[2]),
            action_result: HitOrMiss {
                target: String::from(&data[3]),
                power_name: String::from(&data[4]),
                chance_to_hit: data[5].parse().unwrap(),
            },
        }),
        None => None,
    }
}

pub fn extract_pseudo_pet_miss(line_number: u32, line: &String) -> Option<FileDataPoint> {
    let caps = PSEUDO_PET_MISS_MATCHER.captures(line);

    match caps {
        Some(data) => Some(FileDataPoint::PsuedoPetMiss {
            data_position: DataPosition {
                line_number: line_number,
                date: extract_date(&data[1]),
            },
            pet_name: String::from(&data[2]),
            action_result: HitOrMiss {
                target: String::from(&data[3]),
                power_name: String::from(&data[4]),
                chance_to_hit: data[5].parse().unwrap(),
            },
        }),
        None => None,
    }
}

pub fn extract_player_knockback(line: &String) -> Option<Vec<String>> {
    let caps = PLAYER_KNOCKBACK_MATCHER.captures(line);

    match caps {
        Some(data) => Some(vec![
            String::from(&data[1]),
            String::from(&data[2]),
            String::from(&data[3]),
        ]),
        None => None,
    }
}
