use std::collections::HashMap;
use std::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;
use tera::Context;

use crate::db_actions;
use crate::web::DamageByPowerQuery;
use crate::web::SortDirection;

use lazy_static::lazy_static;

lazy_static! {
    static ref ROW_STATE: Mutex<HashMap<i32, Vec<PowerRow>>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Debug)]
struct PowerRow {
    power_name: String,
    activations: i32,
    hits: i32,
    streak_breakers: i32,
    misses: i32,
    hit_percentage: Option<i32>,
    total_damage: i32,
    total_damage_percent: i32,
    dpa: Option<i32>,
    ate: Option<i32>,
    direct_damage: i32,
    dot_damage: i32,
    critical_damage: i32,
    critical_hits: i32,
    percent_hits_critical: Option<i32>,
    percent_damage_critical: Option<i32>,
}



pub fn process(tera_context: &mut Context, query: &DamageByPowerQuery) {
    match &query.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => tera_context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => tera_context.insert("sort_dir", &SortDirection::ASC),
        },
        None => tera_context.insert("sort_dir", &SortDirection::DESC),
    };

    let mut powers = generate_power_rows(query);
    // match ROW_STATE.get_mut() {
    //     Ok(row_map) => {
    //         row_map.insert(query.key, powers);
    //     }
    //     Err(_) => println!("Unable to get row state. Very bad!"),
    // }

    tera_context.insert( "table_title", &"Attack Summary By Power");
    tera_context.insert("headers", &headers());
    if query.sort_field.is_some() {
        sort(
            query.sort_field.clone().unwrap(),
            query.sort_dir.clone().unwrap(),
            &mut powers,
        );
    }
    tera_context.insert("table_rows", &powers);
}

fn generate_power_rows(query: &DamageByPowerQuery) -> Vec<PowerRow> {
    let powers = db_actions::get_damage_by_power_report(query);
    let total_damage = db_actions::get_total_damage(query);
    let mut rows = Vec::<PowerRow>::new();

    for p in powers {
        let total_damage_percent = (((p.power_total_damage as f32 / total_damage as f32) * 100.0).round()) as i32;
        rows.push(PowerRow{
            power_name: p.power_name,
            activations: p.activations,
            hits: p.hits,
            streak_breakers: p.streak_breakers,
            misses: p.misses,
            hit_percentage: p.hit_percentage,
            total_damage: p.power_total_damage,
            total_damage_percent: total_damage_percent,
            dpa: p.dpa,
            ate: p.ate,
            direct_damage: p.direct_damage,
            dot_damage: p.dot_damage,
            critical_damage: p.critical_damage,
            critical_hits: p.critical_hits,
            percent_hits_critical: p.percent_hits_critical,
            percent_damage_critical: p.percent_damage_critical,
        });
    }

    rows
}

fn headers() -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::<(&'static str, &'static str)>::new();
    headers.push(("checked", "Select"));
    headers.push(("power_name", "Power"));
    headers.push(("activations", "Activations"));
    headers.push(("hits", "Hits (Streakbreakers)"));
    headers.push(("misses", "Misses"));
    headers.push(("hit_percentage", "Hit Percentage"));
    headers.push(("total_damage", "Total Damage"));
    headers.push(("total_damage_percent", "Total Damage Percent"));
    headers.push(("dpa", "Damage Per Activation"));
    headers.push(("ate", "Average Targets Affected"));
    headers.push(("direct_damage", "Direct Damage"));
    headers.push(("dot_damage", "DoT Damage"));
    headers.push(("critical_damage", "Critical Damage"));
    headers.push(("critical_hits", "Critical Hits"));
    headers.push(("percent_hits_critical", "Percent Hits Critical"));
    headers.push(("percent_damage_critical", "Percent Damage Critical"));

    headers
}

fn sort(sort_field: String, sort_dir: SortDirection, data: &mut Vec<PowerRow>) {
    match sort_field.as_str() {
        "power_name" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.power_name.cmp(&a.power_name)),
            SortDirection::ASC => data.sort_by(|a, b| a.power_name.cmp(&b.power_name)),
        },
        "activations" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.activations.cmp(&a.activations)),
            SortDirection::ASC => data.sort_by(|a, b| a.activations.cmp(&b.activations)),
        },
        "hits" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.hits.cmp(&a.hits)),
            SortDirection::ASC => data.sort_by(|a, b| a.hits.cmp(&b.hits)),
        },
        "misses" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.misses.cmp(&a.misses)),
            SortDirection::ASC => data.sort_by(|a, b| a.misses.cmp(&b.misses)),
        },
        "hit_percentage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.hit_percentage.cmp(&a.hit_percentage)),
            SortDirection::ASC => data.sort_by(|a, b| a.hit_percentage.cmp(&b.hit_percentage)),
        },
        "total_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_damage.cmp(&a.total_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_damage.cmp(&b.total_damage)),
        },
        "total_damage_percent" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_damage_percent.cmp(&a.total_damage_percent)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_damage_percent.cmp(&b.total_damage_percent)),
        },
        "dpa" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.dpa.cmp(&a.dpa)),
            SortDirection::ASC => data.sort_by(|a, b| a.dpa.cmp(&b.dpa)),
        },
        "ate" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.ate.cmp(&a.ate)),
            SortDirection::ASC => data.sort_by(|a, b| a.ate.cmp(&b.ate)),
        },
        "direct_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.direct_damage.cmp(&a.direct_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.direct_damage.cmp(&b.direct_damage)),
        },
        "dot_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.dot_damage.cmp(&a.dot_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.dot_damage.cmp(&b.dot_damage)),
        },
        "critical_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.critical_damage.cmp(&a.critical_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.critical_damage.cmp(&b.critical_damage)),
        },
        "critical_hits" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.critical_hits.cmp(&a.critical_hits)),
            SortDirection::ASC => data.sort_by(|a, b| a.critical_hits.cmp(&b.critical_hits)),
        },
        "percent_hits_critical" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.percent_hits_critical.cmp(&a.percent_hits_critical)),
            SortDirection::ASC => data.sort_by(|a, b| a.percent_hits_critical.cmp(&b.percent_hits_critical)),
        },
        "percent_damage_critical" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.percent_damage_critical.cmp(&a.percent_damage_critical)),
            SortDirection::ASC => data.sort_by(|a, b| a.percent_damage_critical.cmp(&b.percent_damage_critical)),
        },
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}
