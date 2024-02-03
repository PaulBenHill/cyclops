use crate::parser_model::FileDataPoint;

#[derive(Debug)]
pub struct DamageReport {
    pub total_damage: f32,
    pub normal_damage: f32,
    pub critical_damage: f32,
}

impl DamageReport {
    fn new(normal_damage: &f32, critical_damage: &f32) -> DamageReport {
        DamageReport {
            total_damage: (normal_damage + critical_damage),
            normal_damage: normal_damage.to_owned(),
            critical_damage: critical_damage.to_owned(),
        }
    }
}

pub fn total_player_damage(data_points: &Vec<FileDataPoint>) -> DamageReport {
    /*
    data_points
        .into_iter()
        .map(|p| find_player_pet_damage(p))
        .sum()
        */

    let mut normal_damage: f32 = 0.0;
    let mut critical_damage: f32 = 0.0;
    for point in data_points {
        match point {
            FileDataPoint::PlayerDamage {
                data_position: _,
                damage_dealt,
            } => normal_damage += damage_dealt.damage,
            FileDataPoint::PlayerDamageDoT {
                data_position: _,
                damage_dealt,
            } => normal_damage += damage_dealt.damage,
            FileDataPoint::PlayerCriticalDamage {
                data_position: _,
                damage_dealt,
                critical_type: _,
            } => {
                critical_damage += damage_dealt.damage;
            }
            FileDataPoint::PsuedoPetDamage {
                data_position: _,
                pet_name: _,
                damage_dealt,
            } => normal_damage += damage_dealt.damage,
            FileDataPoint::PsuedoPetDamageDoT {
                data_position: _,
                pet_name: _,
                damage_dealt,
            } => normal_damage += damage_dealt.damage,
            FileDataPoint::PsuedoPetCriticalDamage {
                data_position: _,
                pet_name: _,
                damage_dealt,
                critical_type: _,
            } => {
                critical_damage += damage_dealt.damage;
            }
            _ => (),
        };
    }
    DamageReport::new(&normal_damage, &critical_damage)
}

fn find_player_and_pet_damage(data_point: FileDataPoint) -> f32 {
    let damage = match data_point {
        FileDataPoint::PlayerDamage {
            data_position: _,
            damage_dealt,
        } => damage_dealt.damage,
        FileDataPoint::PlayerDamageDoT {
            data_position: _,
            damage_dealt,
        } => damage_dealt.damage,
        FileDataPoint::PlayerCriticalDamage {
            data_position: _,
            damage_dealt,
            critical_type: _,
        } => damage_dealt.damage,
        FileDataPoint::PsuedoPetDamage {
            data_position: _,
            pet_name: _,
            damage_dealt,
        } => damage_dealt.damage,
        FileDataPoint::PsuedoPetDamageDoT {
            data_position: _,
            pet_name: _,
            damage_dealt,
        } => damage_dealt.damage,
        FileDataPoint::PsuedoPetCriticalDamage {
            data_position: _,
            pet_name: _,
            damage_dealt,
            critical_type: _,
        } => damage_dealt.damage,
        _ => 0.0,
    };
    //println!("Damage {}", damage);
    damage
}
