use tera::Context;

use crate::{db, game_data::{self, get_mob_hp}, models::DamageDealtToMobByPower};

use super::web_structs_enums::{PowersMobsData, SortDirection};

pub fn process(tera_context: &mut Context, query: &PowersMobsData) {
    match &query.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => tera_context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => tera_context.insert("sort_dir", &SortDirection::ASC),
        },
        None => tera_context.insert("sort_dir", &SortDirection::DESC),
    }

    tera_context.insert(
        "damaging_powers",
        &db::queries::get_damaging_powers(&query),
    );
    tera_context.insert("mobs_damaged", &db::queries::get_mobs_damaged(&query));
    tera_context.insert("mob_levels", &game_data::MINION_HP_TABLE.as_slice());
    tera_context.insert("headers", &headers());
    if query.mob_level.is_some() {
        tera_context.insert("mob_level", &i32::from_str_radix(&query.mob_level.as_ref().unwrap(), 10).unwrap());
    } else {
        tera_context.insert("mob_level", &54);
    } 

    match db::queries::get_damage_dealt_by_power_or_mob(&query) {
        Some(mut data) => {
            let mob_level = query.mob_level.as_ref().unwrap();
            data.iter_mut()
                .for_each(|r| r.overkill = 
                           calc_overkill(r.damage_per_hit,
                           get_mob_hp(&mob_level).into()));

            if query.sort_field.is_some() {
                sort(
                    query.sort_field.clone().unwrap(),
                    query.sort_dir.clone().unwrap(),
                    &mut data,
                );
            }

            if query.power_name.is_some() && !query.power_name.as_ref().unwrap().is_empty() {
                tera_context.insert("power_name", &query.power_name);
            } else if query.mob_name.is_some() && !query.mob_name.as_ref().unwrap().is_empty() {
                tera_context.insert("mob_name", &query.mob_name);
            }
            let rows = flatten(data);
            tera_context.insert("table_rows", &rows);
        }
        None => (),
    }

}

pub fn headers() -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::<(&'static str, &'static str)>::new();
    headers.push(("target_name", "Target Name"));
    headers.push(("power_name", "Power Name"));
    headers.push(("proc_fires", "Procs"));
    headers.push(("hits", "Hits"));
    headers.push(("misses", "Misses"));
    headers.push(("chance_to_hit", "Chance To Hit"));
    headers.push(("hit_percent", "Hit Percent"));
    headers.push(("total_damage", "Total Damage"));
    headers.push(("damage_per_hit", "Damage Per Hit"));
    headers.push(("overkill", "Overkill"));

    headers
}

pub fn flatten(data: Vec<DamageDealtToMobByPower>) -> Vec<Vec<String>> {
    let mut result = Vec::<Vec<String>>::new();

    for d in data {
        let mut row = Vec::<String>::new();
        row.push(d.target_name);
        row.push(d.power_name);
        match d.proc_fires {
            Some(count) => row.push(count.to_string()),
            None => row.push("".to_string())
        }
        row.push(d.hits.to_string());
        row.push(d.misses.to_string());
        row.push(d.chance_to_hit.to_string());
        row.push(d.hit_percent.to_string());
        row.push(d.total_damage.to_string());
        row.push(d.damage_per_hit.to_string());
        row.push(d.overkill.to_string());
        result.push(row);
    }
    result
}

pub fn sort(sort_field: String, sort_dir: SortDirection, data: &mut Vec<DamageDealtToMobByPower>) {
    match sort_field.as_str() {
        "target_name" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.target_name.cmp(&a.target_name)),
            SortDirection::ASC => data.sort_by(|a, b| a.target_name.cmp(&b.target_name)),
        },
        "power_name" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.power_name.cmp(&a.power_name)),
            SortDirection::ASC => data.sort_by(|a, b| a.power_name.cmp(&b.power_name)),
        },
        "proc_fires" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.proc_fires.cmp(&a.proc_fires)),
            SortDirection::ASC => data.sort_by(|a, b| a.proc_fires.cmp(&b.proc_fires)),
        },
        "hits" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.hits.cmp(&a.hits)),
            SortDirection::ASC => data.sort_by(|a, b| a.hits.cmp(&b.hits)),
        },
        "misses" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.misses.cmp(&a.misses)),
            SortDirection::ASC => data.sort_by(|a, b| a.misses.cmp(&b.misses)),
        },
        "chance_to_hit" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.chance_to_hit.cmp(&a.chance_to_hit)),
            SortDirection::ASC => data.sort_by(|a, b| a.chance_to_hit.cmp(&b.chance_to_hit)),
        },
        "hit_percent" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.hit_percent.cmp(&a.hit_percent)),
            SortDirection::ASC => data.sort_by(|a, b| a.hit_percent.cmp(&b.hit_percent)),
        },
        "total_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_damage.cmp(&a.total_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_damage.cmp(&b.total_damage)),
        },
        "damage_per_hit" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.damage_per_hit.cmp(&a.damage_per_hit)),
            SortDirection::ASC => data.sort_by(|a, b| a.damage_per_hit.cmp(&b.damage_per_hit)),
        },
        "overkill" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.overkill.cmp(&a.overkill)),
            SortDirection::ASC => data.sort_by(|a, b| a.overkill.cmp(&b.overkill)),
        },
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}

pub fn calc_overkill(dph: i32, mob_hp: i32) -> i32 {
    if dph > 0 {
        ((dph as f32 / mob_hp as f32) * 100.0).round() as i32
    } else {
        0
    }
}