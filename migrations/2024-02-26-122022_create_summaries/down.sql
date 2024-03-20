-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS damage_action;
DROP TABLE IF EXISTS debuff_action;
DROP TABLE IF EXISTS defeated_targets;
DROP TABLE IF EXISTS hit_or_miss;
DROP TABLE IF EXISTS player_activation;
DROP TABLE IF EXISTS reward;
DROP TABLE IF EXISTS summary;
DROP VIEW IF EXISTS activations_per_power;
DROP VIEW IF EXISTS damage_intervals;
DROP VIEW IF EXISTS damage_report_by_power;
DROP VIEW IF EXISTS total_damage_report;
