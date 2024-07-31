use std::time::Duration;

use chrono::DateTime;
use tera::Context;

use crate::db;
use crate::web::SortDirection;
use crate::web::TableQuery;
use crate::AppContext;
struct Interval {
    start_line: i32,
    end_line: i32,
    total_lines: i32,
    elapsed_seconds: i64,
    pretty_elapsed: String,
    total_damage: i32,
    dps: i32,
}

pub fn process(app_context: &AppContext, tera_context: &mut Context, query: &TableQuery) {
    match &query.sort_dir {
        Some(dir) => match dir {
            SortDirection::ASC => tera_context.insert("sort_dir", &SortDirection::DESC),
            SortDirection::DESC => tera_context.insert("sort_dir", &SortDirection::ASC),
        },
        None => tera_context.insert("sort_dir", &SortDirection::DESC),
    };
    let mut dps_interval = generate_dps_report(&app_context, &query);
    tera_context.insert(
        "table_title",
        &format!("DPS Using Interval Of {}", app_context.dps_interval),
    );
    tera_context.insert("table_name", &query.table_name);
    tera_context.insert("headers", &headers());
    if query.sort_field.is_some() {
        sort(
            query.sort_field.clone().unwrap(),
            query.sort_dir.clone().unwrap(),
            &mut dps_interval,
        );
    }
    tera_context.insert("table_rows", &flatten(dps_interval));
}

fn generate_dps_report(context: &AppContext, query: &TableQuery) -> Vec<Interval> {
    let mut conn1 = db::get_file_conn(query.db_path.clone().into());
    let binding = db::queries::get_summary(&mut conn1, query.key);
    let summary = binding.get(0).unwrap();
    let mut conn2 = db::get_file_conn(query.db_path.clone().into());
    let damage_intervals =
        db::queries::get_damage_intervals_query(&mut conn2, query.key, context.dps_interval as i32);
    let line_count = summary.last_line_number - summary.first_line_number;

    let mut result = Vec::<Interval>::new();

    for intervals in damage_intervals {
        let first_interval = intervals.first().unwrap();
        let last_interval = intervals.last().unwrap();

        let mut end_line: i32 = 0;
        if end_line < line_count {
            end_line = last_interval.line_number;
        } else {
            end_line = line_count;
        }

        let total_damage: i32 = intervals.iter().map(|i| i.damage).sum();

        let elapsed_seconds = DateTime::parse_from_rfc3339(last_interval.log_date.as_str())
            .unwrap()
            .timestamp()
            - DateTime::parse_from_rfc3339(first_interval.log_date.as_str())
                .unwrap()
                .timestamp();
        if elapsed_seconds > 0 {
            let elapsed_duration = Duration::from_secs(elapsed_seconds as u64).as_secs();
            let pretty_elapsed = format!(
                "{} min(s) {} second(s)",
                elapsed_duration / 60,
                elapsed_duration % 60
            );

            let mut dps = total_damage as i64;
            if elapsed_seconds > 0 {
                dps = dps / elapsed_seconds;
            }

            result.push(Interval {
                start_line: first_interval.line_number,
                end_line: end_line,
                total_lines: intervals.len() as i32,
                elapsed_seconds: elapsed_seconds,
                pretty_elapsed: pretty_elapsed,
                total_damage: total_damage,
                dps: dps as i32,
            });
        }
    }

    result
}

fn headers() -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::<(&'static str, &'static str)>::new();
    headers.push(("starting_line", "Starting Line"));
    headers.push(("ending_line", "Ending Line"));
    headers.push(("total_lines", "Total Lines"));
    headers.push(("elapsed_seconds", "Elapsed Seconds"));
    headers.push(("pretty_elapsed", "Elapsed Min(s)/Seconds(s)"));
    headers.push(("total_damage", "Total Damage"));
    headers.push(("dps", "DPS"));

    headers
}

fn flatten(data: Vec<Interval>) -> Vec<Vec<String>> {
    let mut result = Vec::<Vec<String>>::new();

    for d in data {
        let mut row = Vec::<String>::new();
        row.push(d.start_line.to_string());
        row.push(d.end_line.to_string());
        row.push(d.total_lines.to_string());
        row.push(d.elapsed_seconds.to_string());
        row.push(d.pretty_elapsed.to_string());
        row.push(d.total_damage.to_string());
        row.push(d.dps.to_string());
        result.push(row);
    }
    result
}

fn sort(sort_field: String, sort_dir: SortDirection, data: &mut Vec<Interval>) {
    match sort_field.as_str() {
        "starting_line" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.start_line.cmp(&a.start_line)),
            SortDirection::ASC => data.sort_by(|a, b| a.start_line.cmp(&b.start_line)),
        },
        "ending_line" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.end_line.cmp(&a.end_line)),
            SortDirection::ASC => data.sort_by(|a, b| a.end_line.cmp(&b.end_line)),
        },
        "total_lines" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_lines.cmp(&a.total_lines)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_lines.cmp(&b.total_lines)),
        },
        "elapsed_seconds" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.elapsed_seconds.cmp(&a.elapsed_seconds)),
            SortDirection::ASC => data.sort_by(|a, b| a.elapsed_seconds.cmp(&b.elapsed_seconds)),
        },
        // Using elapsed seconds on purpose
        "pretty_elapsed" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.elapsed_seconds.cmp(&a.elapsed_seconds)),
            SortDirection::ASC => data.sort_by(|a, b| a.elapsed_seconds.cmp(&b.elapsed_seconds)),
        },
        "total_damage" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.total_damage.cmp(&a.total_damage)),
            SortDirection::ASC => data.sort_by(|a, b| a.total_damage.cmp(&b.total_damage)),
        },
        "dps" => match sort_dir {
            SortDirection::DESC => data.sort_by(|a, b| b.dps.cmp(&a.dps)),
            SortDirection::ASC => data.sort_by(|a, b| a.dps.cmp(&b.dps)),
        },
        _ => println!("Unknown sort field provided: {}", sort_field),
    }
}
