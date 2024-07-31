use crate::{
    models::DamageDealtToMobByPower,
    web::SortDirection,
};

pub fn headers() -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::<(&'static str, &'static str)>::new();
    headers.push(("target_name", "Target Name"));
    headers.push(("power_name", "Power Name"));
    headers.push(("hits", "Hits"));
    headers.push(("misses", "Misses"));
    headers.push(("chance_to_hit", "Chance To Hit"));
    headers.push(("hit_percent", "Hit Percent"));
    headers.push(("total_damage", "Total Damage"));
    headers.push(("damage_per_hit", "Damage Per Hit"));

    headers
}

pub fn flatten(data: Vec<DamageDealtToMobByPower>) -> Vec<Vec<String>> {
    let mut result = Vec::<Vec<String>>::new();

    for d in data {
        let mut row = Vec::<String>::new();
        row.push(d.target_name);
        row.push(d.power_name);
        row.push(d.hits.to_string());
        row.push(d.misses.to_string());
        row.push(d.chance_to_hit.to_string());
        row.push(d.hit_percent.to_string());
        row.push(d.total_damage.to_string());
        row.push(d.damage_per_hit.to_string());
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
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}
