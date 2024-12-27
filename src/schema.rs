// @generated automatically by Diesel CLI.

diesel::table! {
    damage_action (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        target_name -> Text,
        power_name -> Text,
        damage -> Integer,
        damage_type -> Text,
        damage_mode -> Text,
        source_type -> Text,
        source_name -> Text,
    }
}

diesel::table! {
    debuff_action (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        source_type -> Nullable<Text>,
        source_name -> Text,
        power_name -> Nullable<Text>,
        target_name -> Nullable<Text>,
        debuff_type -> Nullable<Text>,
    }
}

diesel::table! {
    defeated_targets (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        source_name -> Text,
        target_name -> Text,
    }
}

diesel::table! {
    hit_or_miss (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        hit -> Integer,
        chance_to_hit -> Integer,
        source_type -> Text,
        source_name -> Text,
        target_name -> Text,
        power_name -> Text,
        streakbreaker -> Integer,
        sim_hit -> Integer,
    }
}

diesel::table! {
    player_activation (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        power_name -> Text,
        proc_fire -> Integer,
    }
}

diesel::table! {
    player_power_recharged (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        power_name -> Text,
    }
}

diesel::table! {
    reward (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        experience -> Nullable<Integer>,
        influence -> Nullable<Integer>,
        item_drop -> Nullable<Text>,
        reward_type -> Text,
    }
}

diesel::table! {
    summary (summary_key) {
        summary_key -> Integer,
        first_line_number -> Integer,
        last_line_number -> Integer,
        log_date -> Text,
        player_name -> Text,
        log_file_name -> Text,
    }
}

diesel::table! {
    index_details (log_date) {
        summary_key -> Integer,
        log_date-> Text,
        player_name -> Text,
        data_points -> Text,
        file -> Text,
    }
}

diesel::table! {
    total_damage_report (summary_key) {
        summary_key -> Integer,
        activations -> Integer,
        hits -> Integer,
        streak_breakers -> Integer,
        misses -> Integer,
        total_damage -> Integer,
        direct_damage -> Integer,
        dot_damage -> Integer,
        critical_damage -> Integer,
        critical_hits -> Integer,
        critical_hit_percentage -> Integer,
        critical_damage_percentage -> Integer,
    }
}

diesel::table! {
    damage_intervals (summary_key) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        damage -> Integer,
        delta -> Integer,
    }
}

diesel::table! {
    activations_per_power (summary_key) {
        summary_key -> Integer,
        power_name -> Text,
        activations -> Integer,
    }
}

diesel::table! {
    damage_report_by_power (summary_key) {
        summary_key -> Integer,
        power_name -> Text,
        activations -> Integer,
        proc_fires -> Integer,
        hits -> Integer,
        streak_breakers -> Integer,
        misses -> Integer,
        hit_percentage -> Nullable<Integer>,
        power_total_damage -> Integer,
        dpa -> Nullable<Integer>,
        dph -> Nullable<Integer>,
        ate -> Nullable<Integer>,
        direct_damage -> Integer,
        dot_damage -> Integer,
        critical_damage -> Integer,
        critical_hits -> Integer,
        percent_hits_critical -> Nullable<Integer>,
        percent_damage_critical -> Nullable<Integer>,
        average_recharge -> Nullable<Integer>,
    }
}

diesel::table! {
    rewards_defeats (summary_key) {
        summary_key -> Integer,
        experience -> Integer,
        influence -> Integer,
        mobs_defeated -> Integer,
    }
}

diesel::table! {
    damage_taken (summary_key) {
        summary_key -> Integer,
        hits -> Integer,
        misses -> Integer,
        hit_percentage -> Integer,
        total_damage_taken -> Integer,
        damage_per_hit -> Integer,
    }
}

diesel::table! {
    damage_dealt_by_type (summary_key) {
        summary_key -> Integer,
        damage_type -> Text,
        total_damage -> Integer,
        damage_percent -> Integer,
    }
}

diesel::table! {
    damage_taken_by_type (summary_key) {
        summary_key -> Integer,
        damage_type -> Text,
        total_damage -> Integer,
        damage_percent -> Integer,
    }
}

diesel::table! {
    damage_taken_by_mob (summary_key) {
        summary_key -> Integer,
        source_name -> Text,
        hits -> Integer,
        avg_hit_chance -> Integer,
        total_damage -> Integer,
        damage_per_hit -> Integer,
    }
}

diesel::table! {
    damage_taken_by_mob_power (summary_key) {
        summary_key -> Integer,
        source_name -> Text,
        power_name -> Text,
        damage_type -> Text,
        hits -> Integer,
        avg_hit_chance -> Integer,
        total_damage -> Integer,
        damage_per_hit -> Integer,
    }
}

diesel::table! {
    damage_dealt_to_mob_by_power (summary_key) {
        summary_key -> Integer,
        target_name -> Text,
        power_name -> Text,
        proc_fires -> Nullable<Integer>,
        hits -> Integer,
        misses -> Integer,
        chance_to_hit -> Integer,
        hit_percent -> Integer,
        total_damage -> Integer,
        damage_per_hit -> Integer,
        overkill -> Integer,
    }
}

diesel::table! {
    last_interesting_date (log_date) {
        log_date -> Text
    }
}

diesel::table!(
    session_stats (summary_key) {
        summary_key -> Integer,
        total_dps -> Integer,
        dps_5 -> Integer,
        total_exp -> Integer,
        exp_5 -> Integer,
        total_inf -> Integer,
        inf_5 -> Integer,
    }
);



diesel::joinable!(damage_action -> summary (summary_key));
diesel::joinable!(debuff_action -> summary (summary_key));
diesel::joinable!(defeated_targets -> summary (summary_key));
diesel::joinable!(hit_or_miss -> summary (summary_key));
diesel::joinable!(player_activation -> summary (summary_key));
diesel::joinable!(player_power_recharged -> summary (summary_key));
diesel::joinable!(reward -> summary (summary_key));
diesel::joinable!(total_damage_report -> summary (summary_key));
diesel::joinable!(damage_taken -> summary (summary_key));
diesel::joinable!(damage_dealt_by_type -> summary (summary_key));
diesel::joinable!(damage_taken_by_type -> summary (summary_key));
diesel::joinable!(damage_taken_by_mob -> summary (summary_key));
diesel::joinable!(damage_taken_by_mob_power -> summary (summary_key));
diesel::joinable!(damage_dealt_to_mob_by_power -> summary (summary_key));

diesel::allow_tables_to_appear_in_same_query!(
    damage_action,
    debuff_action,
    defeated_targets,
    hit_or_miss,
    player_activation,
    player_power_recharged,
    reward,
    summary,
    total_damage_report,
    damage_report_by_power,
    index_details,
    damage_taken,
    damage_dealt_by_type,
    damage_taken_by_type,
    damage_taken_by_mob,
    damage_taken_by_mob_power,
    damage_dealt_to_mob_by_power,
    session_stats,
);
