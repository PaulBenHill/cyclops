--
-- File generated with SQLiteStudio v3.4.4 on Sat Mar 2 12:47:51 2024
--
-- Text encoding used: System
--
PRAGMA foreign_keys = off;
BEGIN TRANSACTION;

-- Table: damage_action
DROP TABLE IF EXISTS damage_action;
CREATE TABLE IF NOT EXISTS damage_action (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, target TEXT NOT NULL, power_name TEXT NOT NULL, damage INTEGER NOT NULL, damage_type TEXT NOT NULL, damage_mode TEXT CHECK (damage_mode IN ('Direct', 'DoT', 'Critical')) NOT NULL, source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, source_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: debuff_action
DROP TABLE IF EXISTS debuff_action;
CREATE TABLE IF NOT EXISTS debuff_action (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')), source_name TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, power_name TEXT, target_name TEXT, debuff_type TEXT, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: defeated_targets
DROP TABLE IF EXISTS defeated_targets;
CREATE TABLE IF NOT EXISTS defeated_targets (summary_key INTEGER REFERENCES summary (summary_key) NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, source_name TEXT NOT NULL, target_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: hit_or_miss
DROP TABLE IF EXISTS hit_or_miss;
CREATE TABLE IF NOT EXISTS hit_or_miss (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, hit INTEGER NOT NULL CHECK ((hit IN (0, 1))), chance_to_hit INTEGER NOT NULL CHECK ((chance_to_hit <= 100)), source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, source_name TEXT NOT NULL, target_name TEXT NOT NULL, power_name TEXT NOT NULL, streakbreaker INTEGER CHECK ((streakbreaker IN (0, 1))) NOT NULL DEFAULT (0), PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: player_activation
DROP TABLE IF EXISTS player_activation;
CREATE TABLE IF NOT EXISTS player_activation (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, power_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: reward
DROP TABLE IF EXISTS reward;
CREATE TABLE IF NOT EXISTS reward (session_key INTEGER REFERENCES summary (summary_key) NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, experience INTEGER CHECK ((experience > 0)), influence INTEGER CHECK ((influence > 0)), item_drop TEXT, reward_type TEXT, PRIMARY KEY (session_key, line_number, log_date)) STRICT;

-- Table: summary
DROP TABLE IF EXISTS summary;
CREATE TABLE IF NOT EXISTS summary (summary_key INTEGER PRIMARY KEY UNIQUE NOT NULL, first_line_number INTEGER NOT NULL CHECK ((first_line_number > 0)), last_line_number INTEGER NOT NULL, log_date TEXT NOT NULL, player_name TEXT NOT NULL, log_file_name TEXT NOT NULL CHECK ((last_line_number > first_line_number))) STRICT;

-- View: total_damage_report
DROP VIEW IF EXISTS total_damage_report;
CREATE VIEW IF NOT EXISTS total_damage_report AS select s.summary_key, 
(select count(*) from player_activation pa where s.summary_key = pa.summary_key) as activations,
(select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 1 ) AS hits,
(select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.streakbreaker = 1 ) AS streakbreaker,
(select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 0 ) AS misses,
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND source_type IN ('Player', 'PlayerPet')) AS total_damage,
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Direct' AND source_type IN ('Player', 'PlayerPet')) AS direct_damage,
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'DoT' AND source_type IN ('Player', 'PlayerPet')) AS dot_damage,
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) AS critical_damage,
(select count(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) AS critical_hits,
(select ROUND(1.0 * count(da.damage) / (select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 1) *100 )
 from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) AS critical_hit_percentage,
(select ROUND((1.0 * sum(da.damage) / (select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND source_type IN ('Player', 'PlayerPet')) * 100))
 from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) AS critical_damage_percentage
from summary s;

COMMIT TRANSACTION;
PRAGMA foreign_keys = on;
