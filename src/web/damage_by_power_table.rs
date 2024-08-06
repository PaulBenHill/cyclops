use std::collections::HashMap;
use std::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;
use tera::Context;

use crate::db;
use crate::game_data;
use crate::game_data::get_mob_hp;

use lazy_static::lazy_static;

use super::web_structs_enums::DamageByPowerQuery;
use super::web_structs_enums::PowerTableActions;
use super::web_structs_enums::SortDirection;

lazy_static! {
    static ref ROW_STATE: Mutex<HashMap<i32, Vec<PowerRow>>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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
    dph: Option<i32>,
    overkill: Option<i32>,
    ate: Option<i32>,
    direct_damage: i32,
    dot_damage: i32,
    critical_damage: i32,
    critical_hits: i32,
    percent_hits_critical: Option<i32>,
    percent_damage_critical: Option<i32>,
}

impl PowerRow {
    fn new() -> PowerRow {
        PowerRow {
            power_name: "".to_string(),
            activations: 0,
            hits: 0,
            streak_breakers: 0,
            misses: 0,
            hit_percentage: None,
            total_damage: 0,
            total_damage_percent: 0,
            dpa: None,
            dph: None,
            overkill: None,
            ate: None,
            direct_damage: 0,
            dot_damage: 0,
            critical_damage: 0,
            critical_hits: 0,
            percent_hits_critical: None,
            percent_damage_critical: None,
        }
    }
}

pub fn process(tera_context: &mut Context, query: &DamageByPowerQuery) {
    match &query.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => tera_context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => tera_context.insert("sort_dir", &SortDirection::ASC),
        },
        None => tera_context.insert("sort_dir", &SortDirection::DESC),
    };

    tera_context.insert("table_title", &"Attack Summary By Power");
    tera_context.insert("headers", &headers());
    if query.mob_level.is_some() {
        tera_context.insert("mob_level", &i32::from_str_radix(&query.mob_level.as_ref().unwrap(), 10).unwrap());
    } else {
        tera_context.insert("mob_level", &54);
    } 
    tera_context.insert("mob_levels", &game_data::MINION_HP_TABLE.as_slice());

    let mut rows = retrieve_copy(query);

    match &query.mob_level {
        Some(level) => {
            rows.iter_mut()
                .for_each(|r| r.overkill = calc_overkill(r.dph, get_mob_hp(&level).into()));
        }
        None => {
            rows.iter_mut().for_each(|r| {
                r.overkill = calc_overkill(r.dph, get_mob_hp(&"54".to_string()).into())
            });
        }
    }

    if query.action.is_some() {
        rows = handle_action(query, rows);
    }

    if query.sort_field.is_some() {
        sort(
            query.sort_field.clone().unwrap(),
            query.sort_dir.clone().unwrap(),
            &mut rows,
        );
    }

    update_cache(query.key, rows.clone());

    tera_context.insert("table_rows", &rows);
}

fn retrieve_copy(query: &DamageByPowerQuery) -> Vec<PowerRow> {
    match ROW_STATE.lock() {
        Ok(mut row_map) => match row_map.get_mut(&query.key) {
            Some(rows) => rows.clone(),
            None => {
                let powers = generate_power_rows(query);
                row_map.insert(query.key, powers.clone());
                powers
            }
        },
        Err(_) => {
            println!("Unable to lock row cache. Very bad! Return empty list.");
            return Vec::<PowerRow>::new();
        }
    }
}

fn update_cache(key: i32, rows: Vec<PowerRow>) {
    match ROW_STATE.lock() {
        Ok(mut row_map) => {
            row_map.insert(key, rows);
        }
        Err(_) => {
            println!("Unable to lock row cache. Very bad! Return empty list.");
        }
    }
}

fn handle_action(query: &DamageByPowerQuery, rows: Vec<PowerRow>) -> Vec<PowerRow> {
    match query.action.as_ref().unwrap() {
        PowerTableActions::Revert => generate_power_rows(query),
        PowerTableActions::RemoveNonDamaging => {
            rows.into_iter().filter(|r| r.total_damage > 0).collect()
        }
        PowerTableActions::Merge => match &query.power_row {
            Some(indexes) => {
                let mut final_list = rows.clone();
                let merge_rows: Vec<PowerRow> = indexes
                    .into_iter()
                    .map(|i| final_list.get(*i as usize).unwrap().clone())
                    .collect();

                let mut retain_list: Vec<bool> = vec![true; rows.len()];
                for i in indexes {
                    let _ = std::mem::replace(&mut retain_list[*i as usize], false);
                }
                let mut index_iter = retain_list.iter();
                final_list.retain(|_| *index_iter.next().unwrap());

                let mut new_row = PowerRow::new();
                for r in merge_rows {
                    if new_row.power_name == "" {
                        new_row.power_name = r.power_name;
                    } else {
                        new_row.power_name = format!("{},{}", new_row.power_name, r.power_name);
                    }
                    // Careful here, make sure the new row data is updated before using it
                    // in a later calculation
                    new_row.activations += r.activations;
                    new_row.hits += r.hits;
                    new_row.streak_breakers += r.streak_breakers;
                    new_row.misses += r.misses;
                    new_row.hit_percentage = calc_hit_percent(new_row.hits, new_row.misses);
                    new_row.total_damage += r.total_damage;
                    new_row.total_damage_percent += r.total_damage_percent;
                    new_row.dpa = calc_dpa(new_row.activations, new_row.total_damage);
                    new_row.dph = calc_dph(new_row.hits, new_row.total_damage);
                    new_row.overkill = calc_overkill(new_row.dph, get_mob_hp(&query.mob_level.as_ref().unwrap()));
                    new_row.ate = avg_ate(new_row.ate, r.ate);
                    new_row.direct_damage += r.direct_damage;
                    new_row.dot_damage += r.dot_damage;
                    new_row.critical_damage += r.critical_damage;
                    new_row.critical_hits += r.critical_hits;
                    new_row.percent_hits_critical =
                        calc_hit_critical_percent(new_row.hits, new_row.critical_hits);
                    new_row.percent_damage_critical =
                        calc_damage_critical_percent(new_row.total_damage, new_row.critical_damage)
                }
                final_list.insert(0, new_row);
                final_list
            }
            None => rows.clone(),
        },
        PowerTableActions::Delete => match &query.power_row {
            Some(indexes) => {
                let mut final_list = rows.clone();
                let mut retain_list: Vec<bool> = vec![true; rows.len()];
                for i in indexes {
                    let _ = std::mem::replace(&mut retain_list[*i as usize], false);
                }
                let mut index_iter = retain_list.iter();
                final_list.retain(|_| *index_iter.next().unwrap());
                final_list
            }
            None => rows.clone(),
        },
    }
}

fn calc_hit_percent(hits: i32, misses: i32) -> Option<i32> {
    if hits > 0 {
        let v = ((hits as f32 / ((hits + misses) as f32)) * 100.0).round() as i32;
        Some(v)
    } else {
        None
    }
}

fn calc_dpa(activations: i32, total_damage: i32) -> Option<i32> {
    if activations > 0 && total_damage > 0 {
        let v = total_damage / activations;
        Some(v)
    } else {
        None
    }
}

fn calc_dph(hits: i32, total_damage: i32) -> Option<i32> {
    if hits > 0 && total_damage > 0 {
        let v = (total_damage as f32 / hits as f32).round() as i32;
        Some(v)
    } else {
        None
    }
}

fn calc_overkill(dph: Option<i32>, mob_hp: i32) -> Option<i32> {
    if dph.is_some() && dph.unwrap() > 0 {
        Some(((dph.unwrap() as f32 / mob_hp as f32) * 100.0).round() as i32)
    } else {
        None
    }
}

fn avg_ate(first: Option<i32>, second: Option<i32>) -> Option<i32> {
    match first {
        Some(v1) => match second {
            Some(v2) => Some(((v1 as f32 + v2 as f32) / 2.0).round() as i32),
            None => Some(v1),
        },
        None => match second {
            Some(v2) => Some(v2),
            None => None,
        },
    }
}

fn calc_hit_critical_percent(hits: i32, critical_hits: i32) -> Option<i32> {
    if hits > 0 && critical_hits > 0 {
        return Some(((critical_hits as f32 / hits as f32) * 100.0).round() as i32);
    }
    None
}

fn calc_damage_critical_percent(total_damage: i32, critical_damage: i32) -> Option<i32> {
    if total_damage > 0 && critical_damage > 0 {
        return Some(((critical_damage as f32 / total_damage as f32) * 100.0).round() as i32);
    }
    None
}

fn generate_power_rows(query: &DamageByPowerQuery) -> Vec<PowerRow> {
    let powers = db::queries::get_damage_by_power_report(query);
    let total_damage = db::queries::get_total_damage(query);
    let mut rows = Vec::<PowerRow>::new();

    for p in powers {
        let total_damage_percent =
            (((p.power_total_damage as f32 / total_damage as f32) * 100.0).round()) as i32;
        rows.push(PowerRow {
            power_name: p.power_name,
            activations: p.activations,
            hits: p.hits,
            streak_breakers: p.streak_breakers,
            misses: p.misses,
            hit_percentage: p.hit_percentage,
            total_damage: p.power_total_damage,
            total_damage_percent: total_damage_percent,
            dpa: p.dpa,
            dph: p.dph,
            overkill: None,
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
    headers.push(("dph", "Damage Per Hit"));
    headers.push(("overkill", "Overkill Percent"));
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
            SortDirection::DESC => {
                data.sort_by(|a, b| b.total_damage_percent.cmp(&a.total_damage_percent))
            }
            SortDirection::ASC => {
                data.sort_by(|a, b| a.total_damage_percent.cmp(&b.total_damage_percent))
            }
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
            SortDirection::DESC => {
                data.sort_by(|a, b| b.percent_hits_critical.cmp(&a.percent_hits_critical))
            }
            SortDirection::ASC => {
                data.sort_by(|a, b| a.percent_hits_critical.cmp(&b.percent_hits_critical))
            }
        },
        "percent_damage_critical" => match sort_dir {
            SortDirection::DESC => {
                data.sort_by(|a, b| b.percent_damage_critical.cmp(&a.percent_damage_critical))
            }
            SortDirection::ASC => {
                data.sort_by(|a, b| a.percent_damage_critical.cmp(&b.percent_damage_critical))
            }
        },
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}
