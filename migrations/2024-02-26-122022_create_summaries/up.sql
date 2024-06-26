-- Your SQL goes here
--
DROP TABLE IF EXISTS damage_action;
CREATE TABLE IF NOT EXISTS damage_action (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, target_name TEXT NOT NULL, power_name TEXT NOT NULL, damage INTEGER NOT NULL, damage_type TEXT NOT NULL, damage_mode TEXT CHECK (damage_mode IN ('Direct', 'DoT', 'Critical')) NOT NULL, source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, source_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

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
CREATE TABLE IF NOT EXISTS reward (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, experience INTEGER, influence INTEGER, item_drop TEXT, reward_type TEXT NOT NULL CHECK (reward_type IN ('ExpAndInf', 'Threads', 'Item')), PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: summary
DROP TABLE IF EXISTS summary;
CREATE TABLE IF NOT EXISTS summary (summary_key INTEGER PRIMARY KEY UNIQUE NOT NULL, first_line_number INTEGER NOT NULL CHECK ((first_line_number > 0)), last_line_number INTEGER NOT NULL, log_date TEXT NOT NULL, player_name TEXT NOT NULL, log_file_name TEXT NOT NULL CHECK ((last_line_number > first_line_number))) STRICT;

-- View: activations_per_power
DROP VIEW IF EXISTS activations_per_power;
CREATE VIEW IF NOT EXISTS activations_per_power AS select
summary_key,
power_name,
count(power_name) as activations
from player_activation
group by summary_key, power_name;

-- View: damage_intervals
DROP VIEW IF EXISTS damage_intervals;
CREATE VIEW IF NOT EXISTS damage_intervals AS select 
da1.summary_key, 
da1.line_number,
da1.log_date,
da1.damage as damage,
(CASE
WHEN 
  ROUND(((julianday((select da2.log_date from damage_action da2 where da1.summary_key = da2.summary_key AND da2.line_number > da1.line_number limit 1)) - julianday(da1.log_date)) * 86400)) IS NULL
THEN 1000000
ELSE
  ROUND(((julianday((select da2.log_date from damage_action da2 where da1.summary_key = da2.summary_key AND da2.line_number > da1.line_number limit 1)) - julianday(da1.log_date)) * 86400))
END)
as delta
from damage_action da1
where da1.source_type IN ('Player', 'PlayerPet')
order by da1.summary_key;

-- View: index_details
DROP VIEW IF EXISTS index_details;
CREATE VIEW IF NOT EXISTS index_details AS select
substring(log_date, 0, 11) as log_date,
player_name,
(last_line_number-first_line_number) as data_points,
log_file_name as file
from summary;

-- View: damage_report_by_power
DROP VIEW IF EXISTS damage_report_by_power;
CREATE VIEW IF NOT EXISTS damage_report_by_power AS select
summary_key,
power_name,
activations,
sum(hits) as hits,
sum(streak_breakers) as streak_breakers,
sum(misses) misses,
ROUND(1.0 * sum(hits) / (sum(hits) + sum(misses)) * 100) as hit_percentage,
sum(power_total_damage) as power_total_damage,
(sum(power_total_damage)/activations) as dpa,
(ROUND(1.0 * sum(hits + misses) / activations)) as ate,
sum(direct_damage) as direct_damage,
sum(dot_damage) as dot_damage,
sum(critical_damage) as critical_damage,
sum(critical_hits) as critical_hits,
(ROUND(1.0 * sum(critical_hits) / (sum(hits)) * 100)) as percent_hits_critical,
(ROUND(1.0 * sum(critical_damage) / (sum(power_total_damage)) * 100)) as percent_damage_critical
from (
select 
pa.summary_key, 
pa.power_name,
count(pa.power_name) as activations,
0 as hits,
0 as streak_breakers,
0 as misses,
0 as power_total_damage,
0 as direct_damage,
0 as dot_damage,
0 as critical_damage,
0 as critical_hits
from
player_activation pa
group by summary_key, power_name
UNION ALL
select 
hm.summary_key,
hm.power_name,
0 as activations,
sum(hm.hit) AS hits,
sum(hm.streakbreaker) as streak_breakers,
sum(CASE WHEN hit = 0 THEN 1 ELSE 0 END) AS misses,
0 as power_total_damage,
0 as direct_damage,
0 as dot_damage,
0 as critical_damage,
0 as critical_hits
from
hit_or_miss hm
where hm.source_type IN ('Player', 'PlayerPet')
group by summary_key, power_name
UNION ALL
select
da.summary_key,
da.power_name,
0 as activations,
0 as hits,
0 as streak_breakers,
0 as misses,
sum(da.damage) as power_total_damage,
SUM(CASE WHEN da.damage_mode = 'Direct' AND da.source_type IN ('Player', 'PlayerPet') THEN da.damage ELSE 0 END) AS direct_damage,
SUM(CASE WHEN da.damage_mode = 'DoT' AND da.source_type IN ('Player', 'PlayerPet') THEN da.damage ELSE 0 END) AS dot_damage,
SUM(CASE WHEN da.damage_mode = 'Critical' AND da.source_type IN ('Player', 'PlayerPet') THEN da.damage ELSE 0 END) AS critical_damage,
SUM(CASE WHEN da.damage_mode = 'Critical' AND da.source_type IN ('Player', 'PlayerPet') THEN 1 ELSE 0 END) AS critical_hits
from
damage_action da
where da.source_type IN ('Player', 'PlayerPet')
group by summary_key, power_name)
group by summary_key, power_name
order by power_total_damage desc;

-- View: total_damage_report
DROP VIEW IF EXISTS total_damage_report;
CREATE VIEW IF NOT EXISTS total_damage_report AS select s.summary_key, 
(select count(*) from player_activation pa where s.summary_key = pa.summary_key) as activations,
(select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 1 AND source_type IN ('Player', 'PlayerPet')) AS hits,
(select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.streakbreaker = 1 AND source_type IN ('Player', 'PlayerPet')) AS streak_breakers,
(select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 0 AND source_type IN ('Player', 'PlayerPet')) AS misses,
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND source_type IN ('Player', 'PlayerPet')) AS total_damage,
(CASE WHEN(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Direct' AND source_type IN ('Player', 'PlayerPet')) IS NULL
THEN 0
ELSE
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Direct' AND source_type IN ('Player', 'PlayerPet'))
END) AS direct_damage,

(CASE WHEN(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'DoT' AND source_type IN ('Player', 'PlayerPet')) IS NULL
THEN 0
ELSE
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'DoT' AND source_type IN ('Player', 'PlayerPet'))
END) AS dot_damage,

(CASE WHEN(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) IS NULL
THEN 0
ELSE
(select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet'))
END) AS critical_damage,

(select count(da.damage) from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) AS critical_hits,

(CASE
WHEN (select ROUND(1.0 * count(da.damage) / (select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 1) *100 )
 from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) IS NULL
THEN 0
ELSE
(select ROUND(1.0 * count(da.damage) / (select count(*) from hit_or_miss hm where s.summary_key = hm.summary_key AND hm.hit = 1) *100 )
 from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet'))
END) AS critical_hit_percentage,
 
(CASE 
WHEN (select ROUND((1.0 * sum(da.damage) / (select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND source_type IN ('Player', 'PlayerPet')) * 100))
 from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) IS NULL
THEN 0
ELSE
(select ROUND((1.0 * sum(da.damage) / (select sum(da.damage) from damage_action da where s.summary_key = da.summary_key AND source_type IN ('Player', 'PlayerPet')) * 100))
 from damage_action da where s.summary_key = da.summary_key AND damage_mode = 'Critical' AND source_type IN ('Player', 'PlayerPet')) 
END) AS critical_damage_percentage
 
from summary s
order by total_damage desc;


-- View: damage_taken
DROP VIEW IF EXISTS damage_taken;
CREATE VIEW IF NOT EXISTS damage_taken AS
select
summary_key,
hits,
misses,
(CASE WHEN
hits = 0 or misses = 0
THEN
0
ELSE
ROUND( 1.0 * hits / (hits + misses) * 100)
END) as hit_percentage,
total_damage_taken,
(CASE WHEN
total_damage_taken = 0 OR hits = 0
THEN
0
ELSE
ROUND( 1.0 * total_damage_taken / hits)
END) as damage_per_hit
from (
select
summary_key,
(select count(hit) 
from 
hit_or_miss hm1
where
da1.summary_key = hm1.summary_key
AND
hm1.hit = 1
AND
hm1.target_name = 'Player') as hits,
(select count(hit) 
from 
hit_or_miss hm1
where
da1.summary_key = hm1.summary_key
AND
hm1.hit = 0
AND
hm1.target_name = 'Player') as misses,
sum(damage) as total_damage_taken
from
damage_action da1
where
da1.source_type IN ('Mob', 'MobPet')
AND
da1.target_name = 'Player'
group by da1.summary_key)
group by summary_key;


-- View: damage_dealt_by_type
DROP VIEW IF EXISTS damage_dealt_by_type;
CREATE VIEW IF NOT EXISTS damage_dealt_by_type AS
select
s.summary_key,
da.damage_type,
sum(da.damage) as total_damage,
(CASE
WHEN (select sum(da2.damage) from damage_action da2 where s.summary_key = da2.summary_key AND da2.source_type IN ('Player', 'PlayerPet')) IS NULL
THEN
0
ELSE
ROUND(1.0 * sum(damage) / (select sum(da2.damage) from damage_action da2 where s.summary_key = da2.summary_key AND da2.source_type IN ('Player', 'PlayerPet')) * 100)
END) as damage_percent
from
summary s,
damage_action da
where
s.summary_key = da.summary_key
AND
da.source_type IN ('Player', 'PlayerPet')
group by s.summary_key, da.damage_type
order by s.summary_key, total_damage desc;

-- View: damage_taken_by_type
DROP VIEW IF EXISTS damage_taken_by_type;
CREATE VIEW IF NOT EXISTS damage_taken_by_type AS
select
s.summary_key,
da.damage_type,
sum(da.damage) as total_damage,
(CASE
WHEN (select sum(da2.damage) from damage_action da2 where s.summary_key = da2.summary_key AND da2.source_type IN ('Mob', 'MobPet') AND target_name = 'Player') IS NULL
THEN
0
ELSE
ROUND(1.0 * sum(damage) / (select sum(da2.damage) from damage_action da2 where s.summary_key = da2.summary_key AND da2.source_type IN ('Mob', 'MobPet')) * 100)
END) as damage_percent
from
summary s,
damage_action da
where
s.summary_key = da.summary_key
AND
da.source_type IN ('Mob', 'MobPet')
AND
da.target_name = 'Player'
group by s.summary_key, da.damage_type
order by s.summary_key, total_damage desc;

-- View: damage_taken_by_mob
DROP VIEW IF EXISTS damage_taken_by_mob;
CREATE VIEW IF NOT EXISTS damage_taken_by_mob AS
select
summary_key,
source_name,
hits,
CASE WHEN
avg_hit_chance IS NULL
THEN
0
ELSE
avg_hit_chance
END as avg_hit_chance,
total_damage,
(CASE 
WHEN hits = 0
THEN
0
ELSE
ROUND(1.0 * total_damage / hits) 
END) as damage_per_hit
from 
(select
summary_key,
source_name,
(select count(hit) 
from
hit_or_miss hm1 
where 
da1.summary_key = hm1.summary_key
AND
hm1.hit = 1
AND
da1.source_name = hm1.source_name
AND
hm1.target_name = 'Player') as hits,
(select ROUND(avg(hm1.chance_to_hit)) 
from 
hit_or_miss hm1
where
da1.summary_key = hm1.summary_key
AND
da1.source_name = hm1.source_name
AND
da1.power_name = hm1.power_name
AND
hm1.target_name = 'Player') as avg_hit_chance,
sum(damage) as total_damage,
ROUND( 1.0 * sum(damage) / count(power_name)) as damage_per_hit
from damage_action da1
where
source_type IN ('Mob', 'MobPet')
AND
target_name = 'Player'
group by summary_key, source_name
order by summary_key, total_damage desc);

-- View: damage_taken_by_mob_power
DROP VIEW IF EXISTS damage_taken_by_mob_power;
CREATE VIEW IF NOT EXISTS damage_taken_by_mob_power AS
select 
summary_key,
source_name,
power_name,
damage_type,
hits,
CASE WHEN
avg_hit_chance IS NULL
THEN
0
ELSE
avg_hit_chance
END as avg_hit_chance,
total_damage,
(CASE 
WHEN hits = 0
THEN
0
ELSE
ROUND(1.0 * total_damage / hits) 
END) as damage_per_hit
from (
select
da1.summary_key,
da1.source_name,
da1.source_type,
da1.power_name,
da1.damage_type,
(select count(hit) 
from 
hit_or_miss hm1
where
da1.summary_key = hm1.summary_key
AND
hm1.hit = 1
AND
da1.source_name = hm1.source_name
AND
da1.power_name = hm1.power_name
AND
hm1.target_name = 'Player') as hits,
(select ROUND(avg(hm1.chance_to_hit)) 
from 
hit_or_miss hm1
where
da1.summary_key = hm1.summary_key
AND
da1.source_name = hm1.source_name
AND
da1.power_name = hm1.power_name
AND
hm1.target_name = 'Player') as avg_hit_chance,
sum(damage) as total_damage
from damage_action da1
where
da1.source_type IN ('Mob', 'MobPet')
AND
da1.target_name = 'Player'
group by da1.summary_key, da1.source_name, da1.source_type, da1.power_name, da1.damage_type
order by da1.summary_key, total_damage desc, da1.power_name, da1.damage_type);

DROP VIEW IF EXISTS damage_dealt_to_mob_by_power;
CREATE VIEW IF NOT EXISTS damage_dealt_to_mob_by_power AS
select
summary_key,
target_name,
power_name,
hits,
misses,
chance_to_hit,
(CASE WHEN
misses = 0
THEN
100
ELSE
ROUND(1.0 * hits / (hits + misses)* 100)
END) as hit_percent,
total_damage,
(CASE WHEN
total_damage = 0 OR hits = 0
THEN
0
ELSE
ROUND(1.0 * total_damage / hits)
END) as damage_per_hit
from
(select
da1.summary_key,
da1.target_name,
da1.power_name,
(CASE
WHEN
(select 
sum(hit)
from
hit_or_miss hm1
where
hm1.hit = 1
AND
da1.summary_key = hm1.summary_key
AND
hm1.source_type IN ('Player', 'PlayerPet')
AND
da1.source_type = hm1.source_type
AND
da1.target_name = hm1.target_name
AND
da1.power_name = hm1.power_name
) IS NULL
THEN
0
ELSE
(select 
count(hit)
from
hit_or_miss hm1
where
hm1.hit = 1
AND
da1.summary_key = hm1.summary_key
AND
hm1.source_type IN ('Player', 'PlayerPet')
AND
da1.source_type = hm1.source_type
AND
da1.target_name = hm1.target_name
AND
da1.power_name = hm1.power_name
)
END) as hits,
(CASE
WHEN
(select 
count(hit)
from
hit_or_miss hm1
where
hm1.hit = 0
AND
da1.summary_key = hm1.summary_key
AND
hm1.source_type IN ('Player', 'PlayerPet')
AND
da1.source_type = hm1.source_type
AND
da1.target_name = hm1.target_name
AND
da1.power_name = hm1.power_name
) IS NULL
THEN
0
ELSE
(select 
count(hit)
from
hit_or_miss hm1
where
hm1.hit = 0
AND
da1.summary_key = hm1.summary_key
AND
hm1.source_type IN ('Player', 'PlayerPet')
AND
da1.source_type = hm1.source_type
AND
da1.target_name = hm1.target_name
AND
da1.power_name = hm1.power_name
)
END) as misses,
ROUND((CASE
WHEN
(select 
avg(chance_to_hit)
from
hit_or_miss hm1
where
hm1.hit = 1
AND
da1.summary_key = hm1.summary_key
AND
hm1.source_type IN ('Player', 'PlayerPet')
AND
da1.source_type = hm1.source_type
AND
da1.target_name = hm1.target_name
AND
da1.power_name = hm1.power_name
) IS NULL
THEN
0
ELSE
(select 
avg(chance_to_hit)
from
hit_or_miss hm1
where
hm1.hit = 1
AND
da1.summary_key = hm1.summary_key
AND
hm1.source_type IN ('Player', 'PlayerPet')
AND
da1.source_type = hm1.source_type
AND
da1.target_name = hm1.target_name
AND
da1.power_name = hm1.power_name
)
END)) as chance_to_hit,
sum(da1.damage) as total_damage
from
damage_action da1
where
da1.source_type IN ('Player', 'PlayerPet')
group by da1.summary_key, da1.target_name, da1.power_name
order by da1.power_name);