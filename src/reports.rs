use crate::parser_model::{DamageDealt, FileDataPoint, HitOrMiss};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryReport {
    pub total_activations: u32,
    pub total_hits: u32,
    pub total_misses: u32,
    pub total_damage: u32,
    pub total_direct_damage: u32,
    pub total_dot_damage: u32,
    pub total_critical_hits: u32,
    pub total_critical_damage: u32,
    pub damage_powers: HashMap<String, AttackPower>,
}

impl SummaryReport {
    fn new() -> SummaryReport {
        SummaryReport {
            total_activations: 0.to_owned(),
            total_hits: 0.to_owned(),
            total_misses: 0.to_owned(),
            total_damage: 0.to_owned(),
            total_direct_damage: 0.to_owned(),
            total_dot_damage: 0.to_owned(),
            total_critical_hits: 0.to_owned(),
            total_critical_damage: 0.to_owned(),
            damage_powers: HashMap::new(),
        }
    }

    fn update_direct_damage(&mut self, effect: &DamageDealt) {
        self.total_direct_damage += effect.damage.round() as u32;
        self.total_damage += effect.damage.round() as u32;

        let entry = self.get_or_create_damage_power(&effect.power_name);

        entry.direct_damage += effect.damage.round() as u32;
        entry.total_damage += effect.damage.round() as u32;
    }

    fn update_dot_damage(&mut self, effect: &DamageDealt) {
        self.total_dot_damage += effect.damage.round() as u32;
        self.total_damage += effect.damage.round() as u32;

        let entry = self.get_or_create_damage_power(&effect.power_name);

        entry.dot_damage += effect.damage.round() as u32;
        entry.total_damage += effect.damage.round() as u32;
    }

    fn update_critical_damage(&mut self, effect: &DamageDealt) {
        self.total_critical_damage += effect.damage.round() as u32;

        self.total_critical_hits += 1;
        self.total_damage += effect.damage.round() as u32;

        let entry = self.get_or_create_damage_power(&effect.power_name);

        entry.critical_hits += 1;
        entry.critical_damage += effect.damage.round() as u32;
        entry.total_damage += effect.damage.round() as u32;
    }

    fn update_player_hits(&mut self, hit_result: &HitOrMiss) {
        self.total_activations += 1;
        self.total_hits += 1;
        let entry = self.get_or_create_damage_power(&hit_result.power_name);
        entry.activations += 1;
        entry.hits += 1;
    }

    fn update_player_misses(&mut self, miss_result: &HitOrMiss) {
        self.total_activations += 1;
        self.total_misses += 1;
        let entry = self.get_or_create_damage_power(&miss_result.power_name);
        entry.activations += 1;
        entry.misses += 1;
    }

    fn get_or_create_damage_power(&mut self, power_name: &String) -> &mut AttackPower {
        let entry = self.damage_powers.entry(power_name.to_owned()).or_default();
        entry.name = power_name.to_owned();
        entry
    }

    pub fn sort_powers_by_total_damage(self) -> Vec<AttackPower> {
        let values = self.damage_powers.values();
        let mut powers: Vec<AttackPower> = Vec::new();

        for v in values {
            powers.push(v.clone());
        }

        powers.sort_by(|a, b| b.total_damage.partial_cmp(&a.total_damage).unwrap());

        powers
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttackPower {
    pub name: String,
    pub activations: u32,
    pub hits: u32,
    pub misses: u32,
    pub percentage_hit: u32,
    pub total_damage: u32,
    pub direct_damage: u32,
    pub dot_damage: u32,
    pub critical_hits: u32,
    pub critical_damage: u32,
}

pub fn total_player_attacks(data_points: &Vec<FileDataPoint>) -> SummaryReport {
    let mut report = SummaryReport::new();

    for point in data_points {
        match point {
            FileDataPoint::PlayerDamage {
                data_position: _,
                damage_dealt,
            } => report.update_direct_damage(damage_dealt),
            FileDataPoint::PlayerDamageDoT {
                data_position: _,
                damage_dealt,
            } => report.update_dot_damage(damage_dealt),
            FileDataPoint::PlayerCriticalDamage {
                data_position: _,
                damage_dealt,
                critical_type: _,
            } => report.update_critical_damage(damage_dealt),
            FileDataPoint::PsuedoPetDamage {
                data_position: _,
                pet_name: _,
                damage_dealt,
            } => report.update_direct_damage(damage_dealt),
            FileDataPoint::PsuedoPetDamageDoT {
                data_position: _,
                pet_name: _,
                damage_dealt,
            } => report.update_dot_damage(damage_dealt),
            FileDataPoint::PsuedoPetCriticalDamage {
                data_position: _,
                pet_name: _,
                damage_dealt,
                critical_type: _,
            } => report.update_critical_damage(damage_dealt),
            FileDataPoint::PlayerHit {
                data_position: _,
                action_result,
            } => report.update_player_hits(action_result),
            FileDataPoint::PlayerMiss {
                data_position: _,
                action_result,
            } => report.update_player_misses(action_result),
            _ => (),
        };
    }

    report.total_damage = report.total_direct_damage + report.total_critical_damage;
    report
}
