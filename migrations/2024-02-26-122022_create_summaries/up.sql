-- Your SQL goes here
--
-- Table: damage_action
DROP TABLE IF EXISTS damage_action;
CREATE TABLE IF NOT EXISTS damage_action (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, target TEXT NOT NULL, power_name TEXT NOT NULL, damage INTEGER NOT NULL, damage_type TEXT NOT NULL, source_type TEXT NOT NULL, source_name TEXT, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key)) STRICT;

-- Table: debuff_action
DROP TABLE IF EXISTS debuff_action;
CREATE TABLE IF NOT EXISTS debuff_action (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')), source_name TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, power_name TEXT, target_name TEXT, debuff_type TEXT, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key)) STRICT;

-- Table: defeated_targets
DROP TABLE IF EXISTS defeated_targets;
CREATE TABLE IF NOT EXISTS defeated_targets (summary_key INTEGER REFERENCES summary (summary_key) NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, source_name TEXT NOT NULL, target_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date)) STRICT;

-- Table: hit_or_miss
DROP TABLE IF EXISTS hit_or_miss;
CREATE TABLE IF NOT EXISTS hit_or_miss (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, hit INTEGER NOT NULL CHECK ((hit IN (0, 1))), chance_to_hit INTEGER NOT NULL CHECK ((chance_to_hit < 100)), hit_roll INTEGER NOT NULL CHECK ((hit_roll < 100)), source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, source_name TEXT NOT NULL, target_name TEXT NOT NULL, power_name TEXT NOT NULL, streakbreaker INTEGER CHECK ((streakbreaker IN (0, 1))) NOT NULL DEFAULT (0), PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key)) STRICT;

-- Table: player_activation
DROP TABLE IF EXISTS player_activation;
CREATE TABLE IF NOT EXISTS player_activation (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, power_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key)) STRICT;

-- Table: reward
DROP TABLE IF EXISTS reward;
CREATE TABLE IF NOT EXISTS reward (session_key INTEGER REFERENCES summary (summary_key) NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, experience INTEGER CHECK ((experience > 0)), influence INTEGER CHECK ((influence > 0)), item_drop TEXT, reward_type TEXT, PRIMARY KEY (session_key, line_number, log_date)) STRICT;

-- Table: summary
DROP TABLE IF EXISTS summary;
CREATE TABLE IF NOT EXISTS summary (summary_key INTEGER PRIMARY KEY UNIQUE NOT NULL, line_number INTEGER NOT NULL CHECK ((line_number > 0)), log_date TEXT NOT NULL, player_name TEXT NOT NULL, log_file_name TEXT NOT NULL) STRICT;


