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
CREATE TABLE IF NOT EXISTS hit_or_miss (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, hit INTEGER NOT NULL CHECK ((hit IN (0, 1))), chance_to_hit INTEGER NOT NULL CHECK ((chance_to_hit <= 100)), source_type TEXT CHECK (source_type IN ('Player', 'PlayerPet', 'Mob', 'MobPet')) NOT NULL, source_name TEXT NOT NULL, target_name TEXT NOT NULL, power_name TEXT NOT NULL, streakbreaker INTEGER CHECK ((streakbreaker IN (0, 1))) NOT NULL DEFAULT (0),sim_hit INTEGER CHECK ((sim_hit IN (0, 1))) NOT NULL DEFAULT (0), PRIMARY KEY (summary_key, line_number, log_date, power_name), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: player_activation
DROP TABLE IF EXISTS player_activation;
CREATE TABLE IF NOT EXISTS player_activation (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, power_name TEXT NOT NULL, proc_fire INTEGER CHECK ((proc_fire IN (0, 1))) NOT NULL DEFAULT (0), PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

-- Table: player_power_recharged
DROP TABLE IF EXISTS player_power_recharged;
CREATE TABLE IF NOT EXISTS player_power_recharged (summary_key INTEGER NOT NULL, line_number INTEGER NOT NULL, log_date TEXT NOT NULL, power_name TEXT NOT NULL, PRIMARY KEY (summary_key, line_number, log_date), FOREIGN KEY (summary_key) REFERENCES summary (summary_key) ON DELETE CASCADE) STRICT;

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
summary_key,
substring(log_date, 0, 11) as log_date,
player_name,
(last_line_number-first_line_number) as data_points,
log_file_name as file
from summary;

-- View: damage_report_by_power
DROP VIEW IF EXISTS damage_report_by_power;
CREATE VIEW IF NOT EXISTS damage_report_by_power AS 
   SELECT summary_key,
           power_name,
           activations,
           proc_fires,
           sum(hits) AS hits,
           sum(streak_breakers) AS streak_breakers,
           sum(misses) misses,
           ROUND(1.0 * sum(hits) / (sum(hits) + sum(misses) ) * 100) AS hit_percentage,
           sum(power_total_damage) AS power_total_damage,
           (sum(power_total_damage) / activations) AS dpa,
           (CASE WHEN sum(hits) IS NOT NULL AND 
                      sum(hits) > 0 THEN sum(power_total_damage) / sum(hits) ELSE (CASE WHEN sum(proc_fires) IS NOT NULL AND 
                                                                                             sum(proc_fires) > 0 THEN sum(power_total_damage) / sum(proc_fires) ELSE NULL END) END) AS dph,
           (ROUND(1.0 * sum(hits + misses) / activations) ) AS ate,
           sum(direct_damage) AS direct_damage,
           sum(dot_damage) AS dot_damage,
           sum(critical_damage) AS critical_damage,
           sum(critical_hits) AS critical_hits,
           (ROUND(1.0 * sum(critical_hits) / (sum(hits) ) * 100) ) AS percent_hits_critical,
           (ROUND(1.0 * sum(critical_damage) / (sum(power_total_damage) ) * 100) ) AS percent_damage_critical,
           average_recharge
      FROM (
               SELECT pa.summary_key,
                      pa.power_name,
                      (CASE WHEN pa.proc_fire == 0 THEN count(pa.power_name) ELSE 0 END) AS activations,
                      (CASE WHEN pa.proc_fire == 1 THEN count(pa.power_name) ELSE 0 END) AS proc_fires,
                      0 AS hits,
                      0 AS streak_breakers,
                      0 AS misses,
                      0 AS power_total_damage,
                      0 AS direct_damage,
                      0 AS dot_damage,
                      0 AS critical_damage,
                      0 AS critical_hits,
                      (
                          SELECT average_recharge
                            FROM average_power_recharge ar
                           WHERE pa.power_name = ar.power_name
                      )
                      AS average_recharge
                 FROM player_activation pa
                GROUP BY summary_key,
                         power_name
               UNION ALL
               SELECT hm.summary_key,
                      hm.power_name,
                      0 AS activations,
                      0 AS proc_fires,
                      sum(hm.hit) AS hits,
                      sum(hm.streakbreaker) AS streak_breakers,
                      sum(CASE WHEN hit = 0 THEN 1 ELSE 0 END) AS misses,
                      0 AS power_total_damage,
                      0 AS direct_damage,
                      0 AS dot_damage,
                      0 AS critical_damage,
                      0 AS critical_hits,
                      0 AS average_recharge
                 FROM hit_or_miss hm
                WHERE hm.source_type IN ('Player', 'PlayerPet') 
                GROUP BY summary_key,
                         power_name
               UNION ALL
               SELECT da.summary_key,
                      da.power_name,
                      0 AS activations,
                      0 AS proc_fires,
                      0 AS hits,
                      0 AS streak_breakers,
                      0 AS misses,
                      sum(da.damage) AS power_total_damage,
                      SUM(CASE WHEN da.damage_mode = 'Direct' AND 
                                    da.source_type IN ('Player', 'PlayerPet') THEN da.damage ELSE 0 END) AS direct_damage,
                      SUM(CASE WHEN da.damage_mode = 'DoT' AND 
                                    da.source_type IN ('Player', 'PlayerPet') THEN da.damage ELSE 0 END) AS dot_damage,
                      SUM(CASE WHEN da.damage_mode = 'Critical' AND 
                                    da.source_type IN ('Player', 'PlayerPet') THEN da.damage ELSE 0 END) AS critical_damage,
                      SUM(CASE WHEN da.damage_mode = 'Critical' AND 
                                    da.source_type IN ('Player', 'PlayerPet') THEN 1 ELSE 0 END) AS critical_hits,
                      0 AS average_recharge
                 FROM damage_action da
                WHERE da.source_type IN ('Player', 'PlayerPet') 
                GROUP BY summary_key,
                         power_name
           )
     GROUP BY summary_key,
              power_name
     ORDER BY power_total_damage DESC;


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
    SELECT summary_key,
           target_name,
           power_name,
           proc_fires,
           hits,
           misses,
           chance_to_hit,
           (CASE WHEN misses = 0 THEN 100 ELSE ROUND(1.0 * hits / (hits + misses) * 100) END) AS hit_percent,
           total_damage,
           (CASE WHEN total_damage = 0 OR 
                      hits = 0 THEN 0 ELSE ROUND(1.0 * total_damage / hits) END) AS damage_per_hit,
           0 AS overkill
      FROM (
               SELECT da1.summary_key,
                      da1.target_name,
                      da1.power_name,
                      (CASE WHEN (
                              SELECT proc_fire
                                FROM player_activation pa
                               WHERE da1.summary_key = pa.summary_key AND 
                                     da1.power_name = pa.power_name AND 
                                     pa.proc_fire = 1
                          )
                          IS NULL THEN NULL ELSE sum( (
                                                          SELECT proc_fire
                                                            FROM player_activation pa
                                                           WHERE da1.summary_key = pa.summary_key AND 
                                                                 da1.power_name = pa.power_name AND 
                                                                 pa.proc_fire = 1
                                                      )
                          ) END) AS proc_fires,
                      (CASE WHEN (
                              SELECT sum(hit) 
                                FROM hit_or_miss hm1
                               WHERE hm1.hit = 1 AND 
                                     da1.summary_key = hm1.summary_key AND 
                                     hm1.source_type IN ('Player', 'PlayerPet') AND 
                                     da1.source_type = hm1.source_type AND 
                                     da1.target_name = hm1.target_name AND 
                                     da1.power_name = hm1.power_name
                          )
                          IS NULL THEN 0 ELSE (
                              SELECT count(hit) 
                                FROM hit_or_miss hm1
                               WHERE hm1.hit = 1 AND 
                                     da1.summary_key = hm1.summary_key AND 
                                     hm1.source_type IN ('Player', 'PlayerPet') AND 
                                     da1.source_type = hm1.source_type AND 
                                     da1.target_name = hm1.target_name AND 
                                     da1.power_name = hm1.power_name
                          )
                      END) AS hits,
                      (CASE WHEN (
                              SELECT count(hit) 
                                FROM hit_or_miss hm1
                               WHERE hm1.hit = 0 AND 
                                     da1.summary_key = hm1.summary_key AND 
                                     hm1.source_type IN ('Player', 'PlayerPet') AND 
                                     da1.source_type = hm1.source_type AND 
                                     da1.target_name = hm1.target_name AND 
                                     da1.power_name = hm1.power_name
                          )
                          IS NULL THEN 0 ELSE (
                              SELECT count(hit) 
                                FROM hit_or_miss hm1
                               WHERE hm1.hit = 0 AND 
                                     da1.summary_key = hm1.summary_key AND 
                                     hm1.source_type IN ('Player', 'PlayerPet') AND 
                                     da1.source_type = hm1.source_type AND 
                                     da1.target_name = hm1.target_name AND 
                                     da1.power_name = hm1.power_name
                          )
                      END) AS misses,
                      ROUND( (CASE WHEN (
                                     SELECT avg(chance_to_hit) 
                                       FROM hit_or_miss hm1
                                      WHERE hm1.hit = 1 AND 
                                            da1.summary_key = hm1.summary_key AND 
                                            hm1.source_type IN ('Player', 'PlayerPet') AND 
                                            da1.source_type = hm1.source_type AND 
                                            da1.target_name = hm1.target_name AND 
                                            da1.power_name = hm1.power_name
                                 )
                                 IS NULL THEN 0 ELSE (
                                     SELECT avg(chance_to_hit) 
                                       FROM hit_or_miss hm1
                                      WHERE hm1.hit = 1 AND 
                                            da1.summary_key = hm1.summary_key AND 
                                            hm1.source_type IN ('Player', 'PlayerPet') AND 
                                            da1.source_type = hm1.source_type AND 
                                            da1.target_name = hm1.target_name AND 
                                            da1.power_name = hm1.power_name
                                 )
                             END) ) AS chance_to_hit,
                      sum(da1.damage) AS total_damage
                 FROM damage_action da1
                WHERE da1.source_type IN ('Player', 'PlayerPet') 
                GROUP BY da1.summary_key,
                         da1.target_name,
                         da1.power_name
                ORDER BY da1.power_name
           );


DROP VIEW IF EXISTS average_power_recharge;
CREATE VIEW IF NOT EXISTS average_power_recharge AS
SELECT power_name,
           ROUND(sum(recharge_time) / count(power_name) ) AS average_recharge
      FROM (
               SELECT summary_key,
                      power_name,
                      ROUND( (JULIANDAY(recharge_log_date) - JULIANDAY(activation_log_date) ) * 86400) AS recharge_time
                 FROM (
                          SELECT summary_key,
                                 pa.log_date AS activation_log_date,
                                 power_name,
                                 (
                                     SELECT log_date
                                       FROM player_power_recharged ppr
                                      WHERE pa.summary_key = ppr.summary_key AND 
                                            pa.line_number < ppr.line_number AND 
                                            pa.power_name = ppr.power_name
                                      ORDER BY ppr.line_number
                                      LIMIT 1
                                 )
                                 AS recharge_log_date
                            FROM player_activation pa
                           WHERE pa.proc_fire = 0
                      )
                WHERE activation_log_date IS NOT NULL AND 
                      recharge_log_date IS NOT NULL
           )
     GROUP BY summary_key,
              power_name;

DROP VIEW IF EXISTS last_interesting_date;
CREATE VIEW IF NOT EXISTS last_interesting_date AS
    SELECT log_date
      FROM (
               SELECT log_date
                 FROM damage_action
               UNION
               SELECT log_date
                 FROM debuff_action
               UNION
               SELECT log_date
                 FROM defeated_targets
               UNION
               SELECT log_date
                 FROM hit_or_miss
               UNION
               SELECT log_date
                 FROM player_activation
               UNION
               SELECT log_date
                 FROM player_power_recharged
               UNION
               SELECT log_date
                 FROM reward
               UNION
               SELECT log_date
                 FROM summary
           )
     ORDER BY log_date DESC
     LIMIT 1;

DROP VIEW IF EXISTS session_stats;
CREATE VIEW IF NOT EXISTS session_stats AS
    SELECT summary_key,
           (CASE WHEN total_dps IS NULL THEN 0 ELSE total_dps END) AS total_dps,
           (CASE WHEN dps_5 IS NULL THEN 0 ELSE dps_5 END) AS dps_5,
           (CASE WHEN total_exp IS NULL THEN 0 ELSE total_exp END) AS total_exp,
           (CASE WHEN exp_5 IS NULL THEN 0 ELSE exp_5 END) AS exp_5,
           (CASE WHEN total_inf IS NULL THEN 0 ELSE total_inf END) AS total_inf,
           (CASE WHEN inf_5 IS NULL THEN 0 ELSE inf_5 END) AS inf_5
      FROM (
               SELECT summary_key,
                      (
                          SELECT CASE WHEN ROUND(total_damage / delta) < 0 THEN 0 ELSE ROUND(total_damage / delta) END
                            FROM (
                                     SELECT sum(damage) AS total_damage,
                                            (unixepoch( (
                                                            SELECT log_date
                                                              FROM damage_action
                                                             WHERE s.summary_key = summary_key AND 
                                                                   (source_type = 'Player' OR 
                                                                    source_type = 'PlayerPet') 
                                                             ORDER BY line_number DESC
                                                             LIMIT 1
                                                        )
                                             ) - (unixepoch( (
                                                                 SELECT log_date
                                                                   FROM damage_action
                                                                  WHERE s.summary_key = summary_key AND 
                                                                        (source_type = 'Player' OR 
                                                                         source_type = 'PlayerPet') 
                                                                  ORDER BY line_number ASC
                                                                  LIMIT 1
                                                             )
                                                 ) ) ) AS delta
                                       FROM damage_action
                                 )
                      )
                      AS total_dps,
                      (
                          SELECT CASE WHEN ROUND(total_damage / 300) < 0 THEN 0 ELSE ROUND(total_damage / 300) END
                            FROM (
                                     SELECT (
                                                SELECT sum(damage) 
                                                  FROM damage_action
                                                 WHERE s.summary_key = summary_key AND 
                                                       source_type IN ('Player', 'PlayerPet') AND 
                                                       unixepoch(log_date) >= ( (
                                                                                    SELECT unixepoch(log_date) 
                                                                                      FROM damage_action
                                                                                     WHERE s.summary_key = summary_key AND 
                                                                                           source_type IN ('Player', 'PlayerPet') 
                                                                                     ORDER BY line_number DESC
                                                                                     LIMIT 1
                                                                                )
-                                                                             300) 
                                            )
                                            AS total_damage
                                 )
                      )
                      AS dps_5,
                      (
                          SELECT sum(experience) 
                            FROM reward
                           WHERE s.summary_key = summary_key AND 
                                 experience > 0
                      )
                      AS total_exp,
                      (
                          SELECT sum(experience) 
                            FROM reward
                           WHERE s.summary_key = summary_key AND 
                                 experience > 0 AND 
                                 unixepoch(log_date) >= ( (
                                                              SELECT unixepoch(log_date) 
                                                                FROM reward
                                                               WHERE s.summary_key = summary_key AND 
                                                                     experience > 0
                                                               ORDER BY line_number DESC
                                                               LIMIT 1
                                                          )
-                                                       300) 
                      )
                      AS exp_5,
                      (
                          SELECT sum(influence) 
                            FROM reward
                           WHERE s.summary_key = summary_key AND 
                                 influence > 0
                      )
                      AS total_inf,
                      (
                          SELECT sum(influence) 
                            FROM reward
                           WHERE s.summary_key = summary_key AND 
                                 influence > 0 AND 
                                 unixepoch(log_date) >= ( (
                                                              SELECT unixepoch(log_date) 
                                                                FROM reward
                                                               WHERE s.summary_key = summary_key AND 
                                                                     influence > 0
                                                               ORDER BY line_number DESC
                                                               LIMIT 1
                                                          )
-                                                       300) 
                      )
                      AS inf_5
                 FROM summary s
           );
