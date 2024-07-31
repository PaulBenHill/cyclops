use tera::Context;

use crate::db;
use crate::models::DamageTakenByMobPower;
use crate::web::TableQuery;
use crate::web::SortDirection;

pub fn process(context: &mut Context, query: &TableQuery) {
    match &query.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => context.insert("sort_dir", &SortDirection::ASC),
        },
        None => context.insert("sort_dir", &SortDirection::DESC),
    };
    match db::queries::get_damage_taken_by_mob_power_query(&query) {
        Some(mut data) => {
            context.insert("table_title", "Damage Taken By Mob");
            context.insert("table_name", &query.table_name);
            context.insert("headers", &headers());
            if query.sort_field.is_some() {
                sort(
                    query.sort_field.clone().unwrap(),
                    query.sort_dir.clone().unwrap(),
                    &mut data,
                );
            }
            context.insert("table_rows", &flatten(data));
        }
        None => println!("Damage taken by type returned no data"),
    };
}

pub fn headers() -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::<(&'static str, &'static str)>::new();
    headers.push(("source_name", "Mob"));
    headers.push(("power_name", "Power Name"));
    headers.push(("damage_type", "Damage Type"));
    headers.push(("hits", "Hits"));
    headers.push(("avg_hit_chance", "Average Hit Chance"));
    headers.push(("total_damage", "Total Damage"));
    headers.push(("damage_per_hit", "Damage Per Hit"));

    headers
}

pub fn flatten(data: Vec<DamageTakenByMobPower>) -> Vec<Vec<String>> {
    let mut result = Vec::<Vec<String>>::new();

    for d in data {
        let mut row = Vec::<String>::new();
        row.push(d.source_name);
        row.push(d.power_name);
        row.push(d.damage_type);
        row.push(d.hits.to_string());
        row.push(d.avg_hit_chance.to_string());
        row.push(d.total_damage.to_string());
        row.push(d.damage_per_hit.to_string());
        result.push(row);
    }
    result
}

pub fn sort(sort_field: String, sort_dir: SortDirection, data: &mut Vec<DamageTakenByMobPower>) {
    match sort_field.as_str() {
        "source_name" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.source_name.cmp(&a.source_name)),
            SortDirection::ASC => data.sort_by(|a, b| a.source_name.cmp(&b.source_name)),
        },
        "power_name" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.power_name.cmp(&a.power_name)),
            SortDirection::ASC => data.sort_by(|a, b| a.power_name.cmp(&b.power_name)),
        },
        "damage_type" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.damage_type.cmp(&a.damage_type)),
            SortDirection::ASC => data.sort_by(|a, b| a.damage_type.cmp(&b.damage_type)),
        },
        "hits" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.hits.cmp(&a.hits)),
            SortDirection::ASC => data.sort_by(|a, b| a.hits.cmp(&b.hits)),
        },
        "avg_hit_chance" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.avg_hit_chance.cmp(&a.avg_hit_chance)),
            SortDirection::ASC => data.sort_by(|a, b| a.avg_hit_chance.cmp(&b.avg_hit_chance)),
        },
        "total_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_damage.cmp(&a.total_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_damage.cmp(&b.total_damage)),
        },
        "damage_per_hit" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.damage_per_hit.cmp(&a.damage_per_hit)),
            SortDirection::ASC => data.sort_by(|a, b| a.damage_per_hit.cmp(&b.damage_per_hit)),
        },
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}
