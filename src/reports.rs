use crate::parser_model::{DamageDealt, DataPosition, FileDataPoint, HitOrMiss};
use chrono::{self, DateTime, Local};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::to_string;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryReport {
    pub player_name: String,
    pub start_date: String,
    pub line_number: u32,
    pub total_activations: u32,
    pub total_hits: u32,
    pub total_misses: u32,
    pub total_damage: u32,
    pub total_direct_damage: u32,
    pub total_dot_damage: u32,
    pub total_critical_hits: u32,
    pub total_critical_damage: u32,
    pub has_pets: bool,
    pub damage_powers: HashMap<String, AttackPower>,
    #[serde(skip)]
    pub damage_points: Vec<DamagePoint>,
}

impl SummaryReport {
    fn new() -> SummaryReport {
        SummaryReport {
            player_name: "NO PLAYER NAME".to_string(),
            start_date: "".to_string(),
            line_number: 1.to_owned(),
            total_activations: 0.to_owned(),
            total_hits: 0.to_owned(),
            total_misses: 0.to_owned(),
            total_damage: 0.to_owned(),
            total_direct_damage: 0.to_owned(),
            total_dot_damage: 0.to_owned(),
            total_critical_hits: 0.to_owned(),
            total_critical_damage: 0.to_owned(),
            has_pets: false,
            damage_powers: HashMap::new(),
            damage_points: Vec::new(),
        }
    }

    fn has_data(&self) -> bool {
        if self.total_hits > 0 || self.total_misses > 0 {
            return true;
        }

        false
    }

    fn format_meta_name(source_name: String, power_name: String) -> String {
        format!("{}: {}", source_name, power_name)
    }

    fn update_master_damage_totals(&mut self, effect: &DamageDealt) {
        self.total_direct_damage += effect.damage.round() as u32;
        self.total_damage += effect.damage.round() as u32;
    }

    fn update_master_dot_damage_totals(&mut self, effect: &DamageDealt) {
        self.total_dot_damage += effect.damage.round() as u32;
        self.total_damage += effect.damage.round() as u32;
    }

    fn update_master_critical_damage_totals(&mut self, effect: &DamageDealt) {
        self.total_critical_hits += 1;
        self.total_critical_damage += effect.damage.round() as u32;
        self.total_damage += effect.damage.round() as u32;
    }

    fn update_master_hits(&mut self) {
        self.total_hits += 1;
    }

    fn update_master_misses(&mut self) {
        self.total_misses += 1;
    }

    fn update_hits_entry(&mut self, source_name: String) {
        let power_entry = self.get_or_create_damage_power(&source_name);

        power_entry.hits += 1;
    }
    fn update_misses_entry(&mut self, source_name: String) {
        let power_entry = self.get_or_create_damage_power(&source_name);

        power_entry.misses += 1;
    }

    fn update_direct_damage_power_entry(&mut self, power_key: String, effect: &DamageDealt) {
        let power_entry = self.get_or_create_damage_power(&power_key);

        power_entry.direct_damage += effect.damage.round() as u32;
        power_entry.total_damage += effect.damage.round() as u32;
        if power_key.contains(": Chance for") || power_key.contains("Superior Spider's Bite") {
            power_entry.hits += 1;
        }
    }

    fn update_dot_damage_power_entry(&mut self, power_key: String, effect: &DamageDealt) {
        let power_entry = self.get_or_create_damage_power(&power_key);

        power_entry.dot_damage += effect.damage.round() as u32;
        power_entry.total_damage += effect.damage.round() as u32;
    }

    fn update_critical_damage_power_entry(&mut self, power_key: String, effect: &DamageDealt) {
        let power_entry = self.get_or_create_damage_power(&power_key);

        power_entry.critical_hits += 1;
        power_entry.critical_damage += effect.damage.round() as u32;
        power_entry.total_damage += effect.damage.round() as u32;
    }

    fn update_direct_damage(&mut self, source_name: String, effect: &DamageDealt) {
        self.update_master_damage_totals(effect);
        self.update_direct_damage_power_entry(effect.power_name.clone(), effect);
        self.update_direct_damage_power_entry(
            Self::format_meta_name(source_name.clone(), effect.power_name.clone()),
            effect,
        );
        self.update_direct_damage_power_entry(source_name.clone(), effect);
    }

    fn update_dot_damage(&mut self, source_name: String, effect: &DamageDealt) {
        self.update_master_dot_damage_totals(effect);
        self.update_dot_damage_power_entry(effect.power_name.clone(), effect);
        self.update_dot_damage_power_entry(
            Self::format_meta_name(source_name.clone(), effect.power_name.clone()),
            effect,
        );
        self.update_dot_damage_power_entry(source_name.clone(), effect);
    }

    fn update_critical_damage(&mut self, source_name: String, effect: &DamageDealt) {
        self.update_master_critical_damage_totals(effect);
        self.update_critical_damage_power_entry(effect.power_name.clone(), effect);
        self.update_critical_damage_power_entry(
            Self::format_meta_name(source_name.clone(), effect.power_name.clone()),
            effect,
        );
        self.update_critical_damage_power_entry(source_name.clone(), effect);
    }

    fn update_pseudo_pet_direct_damage(&mut self, source_name: String, effect: &DamageDealt) {
        self.update_master_damage_totals(effect);

        self.update_direct_damage_power_entry(
            format!("{}: {}", source_name, effect.power_name),
            effect,
        );
        self.update_direct_damage_power_entry(source_name, effect);
    }

    fn update_pseudo_pet_dot_damage(&mut self, source_name: String, effect: &DamageDealt) {
        self.update_master_dot_damage_totals(effect);

        self.update_dot_damage_power_entry(
            Self::format_meta_name(source_name.clone(), effect.power_name.clone()),
            effect,
        );
        self.update_dot_damage_power_entry(source_name.clone(), effect);
    }

    fn update_pseudo_pet_critical_damage(&mut self, source_name: String, effect: &DamageDealt) {
        self.update_master_critical_damage_totals(effect);

        self.update_critical_damage_power_entry(
            Self::format_meta_name(source_name.clone(), effect.power_name.clone()),
            effect,
        );
        self.update_critical_damage_power_entry(source_name.clone(), effect);
    }

    fn update_player_activation(&mut self, source_name: String, power_name: &String) {
        self.total_activations += 1;

        self.update_activations_entry(power_name.clone());
        self.update_activations_entry(Self::format_meta_name(
            source_name.clone(),
            power_name.clone(),
        ));
        self.update_activations_entry(source_name.clone());
    }

    fn update_activations_entry(&mut self, power_key: String) {
        let power_entry = self.get_or_create_damage_power(&power_key);
        power_entry.activations += 1;
    }

    fn update_player_hits(&mut self, source_name: String, hit_result: &HitOrMiss) {
        self.update_master_hits();
        self.update_hits_entry(hit_result.power_name.clone());
        self.update_hits_entry(Self::format_meta_name(
            source_name.clone(),
            hit_result.power_name.clone(),
        ));
        self.update_hits_entry(source_name.clone());
    }

    fn update_player_misses(&mut self, source_name: String, miss_result: &HitOrMiss) {
        self.update_master_misses();
        self.update_misses_entry(miss_result.power_name.clone());
        self.update_misses_entry(Self::format_meta_name(
            source_name.clone(),
            miss_result.power_name.clone(),
        ));
        self.update_misses_entry(source_name.clone());
    }

    fn update_pseudo_pet_hits(&mut self, source_name: String, hit_result: &HitOrMiss) {
        self.update_master_hits();
        self.update_hits_entry(Self::format_meta_name(
            source_name.clone(),
            hit_result.power_name.clone(),
        ));
        self.update_hits_entry(source_name.clone());
        self.has_pets = true;
    }

    fn update_pseudo_pet_misses(&mut self, source_name: String, miss_result: &HitOrMiss) {
        self.update_master_misses();
        self.update_misses_entry(format!("{}: {}", source_name, miss_result.power_name));
        self.update_misses_entry(source_name);
    }

    fn get_or_create_damage_power(&mut self, power_name: &String) -> &mut AttackPower {
        let entry = self.damage_powers.entry(power_name.to_owned()).or_default();
        entry.name = power_name.to_owned();
        entry
    }

    fn create_damage_point(&mut self, pos: &DataPosition, damage: f32) {
        self.damage_points.push(DamagePoint::new(pos, damage))
    }

    pub fn sort_powers_by_total_damage(&self) -> Vec<AttackPower> {
        let values = self.damage_powers.values();
        let mut powers: Vec<AttackPower> = Vec::new();

        // For summaries without pets, remove the player prefixed powers to make it cleaner
        for v in values {
            if self.has_pets {
                powers.push(v.clone());
            } else if !v.name.starts_with(&self.player_name) {
                powers.push(v.clone());
            }
        }

        powers.sort_by(|a, b| b.total_damage.partial_cmp(&a.total_damage).unwrap());

        powers
    }

    pub fn get_damage_points_by_interval(&self, interval: i64) -> Vec<Vec<DamagePoint>> {
        let mut result: Vec<Vec<DamagePoint>> = Vec::new();
        let mut last_timestamp: i64 = self.damage_points.get(0).unwrap().date.timestamp();

        let mut current_interval: Vec<DamagePoint> = Vec::new();

        for dp in self.damage_points.clone() {
            if (dp.date.timestamp() - last_timestamp) >= interval {
                result.push(current_interval);
                current_interval = Vec::new();
            }
            current_interval.push(dp);
            last_timestamp = dp.date.timestamp();
        }
        result.push(current_interval);

        result
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttackPower {
    pub name: String,
    pub activations: u32,
    pub hits: u32,
    pub misses: u32,
    pub total_damage: u32,
    pub direct_damage: u32,
    pub dot_damage: u32,
    pub critical_hits: u32,
    pub critical_damage: u32,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct DamagePoint {
    pub line_number: u32,
    #[serde(serialize_with = "date_to_string")]
    date: DateTime<Local>,
    pub damage_dealt: u32,
}

impl DamagePoint {
    fn new(pos: &DataPosition, damage: f32) -> DamagePoint {
        DamagePoint {
            line_number: pos.line_number,
            date: pos.date,
            damage_dealt: damage.round() as u32,
        }
    }

    pub fn get_delta_in_seconds(points: &Vec<DamagePoint>) -> u64 {
        let last_point = points.last().unwrap();
        let first_point = points.first().unwrap();

        (last_point.date.timestamp() - first_point.date.timestamp()) as u64
    }

    pub fn get_total_damage(points: &Vec<DamagePoint>) -> u64 {
        let total: u32 = points.iter().map(|p| p.damage_dealt).sum();
        total as u64
    }
}

fn date_to_string<S>(date: &DateTime<Local>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{}", date.format("%H:%M:%S")))
}

pub fn total_player_attacks(data_points: &Vec<FileDataPoint>) -> Vec<SummaryReport> {
    let mut summaries: Vec<SummaryReport> = Vec::new();
    let mut report = SummaryReport::new();

    for point in data_points {
        match point {
            FileDataPoint::SessionMarker {
                data_position,
                player_name,
            } => {
                if report.has_data() {
                    //println!("New session marker found, previous report {:?}", report);
                    //println!("New session marker {:?}", point);
                    summaries.push(report);
                } else {
                    //println!("Tossed session due to no data: {:?}", report);
                }
                report = SummaryReport::new();
                report.player_name = player_name.to_owned();
                report.start_date = data_position.date.to_rfc2822();
                report.line_number = data_position.line_number;
            }
            FileDataPoint::PlayerPowerActivation {
                data_position: _,
                power_name,
            } => report.update_player_activation(report.player_name.clone(), power_name),
            FileDataPoint::PlayerDirectDamage {
                data_position,
                damage_dealt,
            } => {
                report.update_direct_damage(report.player_name.clone(), damage_dealt);
                report.create_damage_point(data_position, damage_dealt.damage);
            }
            FileDataPoint::PlayerDamageDoT {
                data_position: _,
                damage_dealt,
            } => report.update_dot_damage(report.player_name.clone(), damage_dealt),
            FileDataPoint::PlayerCriticalDamage {
                data_position: _,
                damage_dealt,
                critical_type: _,
            } => report.update_critical_damage(report.player_name.clone(), damage_dealt),
            FileDataPoint::PsuedoPetDirectDamage {
                data_position,
                pet_name,
                damage_dealt,
            } => {
                report.update_pseudo_pet_direct_damage(pet_name.clone(), damage_dealt);
                report.create_damage_point(data_position, damage_dealt.damage);
            }
            FileDataPoint::PsuedoPetDamageDoT {
                data_position: _,
                pet_name,
                damage_dealt,
            } => report.update_pseudo_pet_dot_damage(pet_name.clone(), damage_dealt),
            FileDataPoint::PsuedoPetCriticalDamage {
                data_position: _,
                pet_name,
                damage_dealt,
                critical_type: _,
            } => report.update_pseudo_pet_critical_damage(pet_name.clone(), damage_dealt),
            FileDataPoint::PlayerHit {
                data_position: _,
                action_result,
            } => report.update_player_hits(report.player_name.clone(), action_result),
            FileDataPoint::PlayerMiss {
                data_position: _,
                action_result,
            } => report.update_player_misses(report.player_name.clone(), action_result),
            FileDataPoint::PsuedoPetHit {
                data_position: _,
                name,
                action_result,
            } => report.update_pseudo_pet_hits(name.clone(), action_result),
            FileDataPoint::PsuedoPetMiss {
                data_position: _,
                name,
                action_result,
            } => report.update_pseudo_pet_misses(name.clone(), action_result),
            _ => (),
        };
    }

    // We don't always have clean starts at the beginning of the file
    // If the report has data add it to the summaries
    if report.has_data() {
        summaries.push(report);
    } else {
        println!(
            "End of file reached. Tossed session due to no data: {:?}",
            report
        );
    }
    summaries
}
