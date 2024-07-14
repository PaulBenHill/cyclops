use crate::{
    models::DamageDealtToMobByPower,
    web::{PowersMobsData, SortDirection},
};

pub fn headers() -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::<(&'static str, &'static str)>::new();
    headers.push(("Target Name", "target_name"));
    headers.push(("Power Name", "power_name"));
    headers.push(("Hits", "hits"));
    headers.push(("Misses", "misses"));
    headers.push(("Chance To Hit", "chance_to_hit"));
    headers.push(("Hit Percent", "hit_perecent"));
    headers.push(("Total Damage", "total_damage"));
    headers.push(("Damage Per Hit", "damage_per_hit"));

    headers
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
        _ => panic!("Unknown sort field provided"),
    }
}
