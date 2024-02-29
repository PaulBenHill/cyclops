// @generated automatically by Diesel CLI.

diesel::table! {
    damage_action (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        target -> Text,
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
    }
}

diesel::table! {
    player_activation (summary_key, line_number, log_date) {
        summary_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        power_name -> Text,
    }
}

diesel::table! {
    reward (session_key, line_number, log_date) {
        session_key -> Integer,
        line_number -> Integer,
        log_date -> Text,
        experience -> Nullable<Integer>,
        influence -> Nullable<Integer>,
        item_drop -> Nullable<Text>,
        reward_type -> Nullable<Text>,
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

diesel::joinable!(damage_action -> summary (summary_key));
diesel::joinable!(debuff_action -> summary (summary_key));
diesel::joinable!(defeated_targets -> summary (summary_key));
diesel::joinable!(hit_or_miss -> summary (summary_key));
diesel::joinable!(player_activation -> summary (summary_key));
diesel::joinable!(reward -> summary (session_key));

diesel::allow_tables_to_appear_in_same_query!(
    damage_action,
    debuff_action,
    defeated_targets,
    hit_or_miss,
    player_activation,
    reward,
    summary,
);
