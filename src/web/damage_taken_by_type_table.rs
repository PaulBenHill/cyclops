use tera::Context;

use crate::{
    db_actions, models::DamageTakenByType, web::{SortDirection, TableQuery}
};

pub fn process(context: &mut Context, query: &TableQuery) {
    match &query.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => context.insert("sort_dir", &SortDirection::ASC),
        },
        None => context.insert("sort_dir", &SortDirection::DESC),
    };
    match db_actions::get_damage_taken_by_type_query(&query) {
        Some(mut data) => {
            context.insert("table_title", "Damage Taken By Type");
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
    headers.push(("damage_type", "Type"));
    headers.push(("total_damage", "Total Damage"));
    headers.push(("damage_percent", "Damage Percent"));

    headers
}

pub fn flatten(data: Vec<DamageTakenByType>) -> Vec<Vec<String>> {
    let mut result = Vec::<Vec<String>>::new();

    for d in data {
        let mut row = Vec::<String>::new();
        row.push(d.damage_type);
        row.push(d.total_damage.to_string());
        row.push(d.damage_percent.to_string());
        result.push(row);
    }
    result
}

pub fn sort(sort_field: String, sort_dir: SortDirection, data: &mut Vec<DamageTakenByType>) {
    match sort_field.as_str() {
        "damage_type" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.damage_type.cmp(&a.damage_type)),
            SortDirection::ASC => data.sort_by(|a, b| a.damage_type.cmp(&b.damage_type)),
        },
        "total_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_damage.cmp(&a.total_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_damage.cmp(&b.total_damage)),
        },
        "damage_percent" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.damage_percent.cmp(&a.damage_percent)),
            SortDirection::ASC => data.sort_by(|a, b| a.damage_percent.cmp(&b.damage_percent)),
        },
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}
