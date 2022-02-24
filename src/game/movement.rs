use crate::game::components::{Health, MovementAnimate, Player};
use crate::game::enemy::Enemy;
use crate::game::tilemap::TilePosExt;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;
use std::collections::HashMap;

enum MoveDecision {
    Move(TilePos),
    Nothing,
    Attack_and_maybe_move((Tilepos, Entity)),
    Attack_and_dont_move(Entity),
}

type MoveDecisions = HashMap<Entity, MoveDecision>;

pub fn decide_move(
    move_query: Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
) -> MoveDecisions {
    // Make decisions
    let mut move_decisions: HashMap<Entity, MoveDecision> = HashMap::new();
    for (entity, tilepos, maybe_player, maybe_enemy) in move_query.q0().iter() {
        // Make decision
    }
    move_decisions
}

pub fn apply_move(
    move_decisions: MoveDecisions,
    move_query: Query<(&mut TilePos, &mut MovementAnimate, &Transform)>,
    mut health_query: Query<&mut Health>,
) {
    //Apply decisions:
    for (entity, decision) in move_decisions.iter() {
        let tilepos = match decision {
            MoveDecision::Nothing => None,
            MoveDecision::Move(tilepos) => Some(tilepos),
            MoveDecision::Attack_and_dont_move(target) => {
                let target_health = health_query.get_mut(*target);
                match target_health {
                    Ok(&mut health) => health.decr_by(1),
                    Err(e) => warn!("Error getting health to attack: {:?}", e),
                }
                None
            }
            MoveDecision::Attack_and_maybe_move((move_to, target)) => {
                let target_health = health_query.get_mut(*target);
                match target_health {
                    Ok(&mut health) => {
                        health.decr_by(1);
                        if health.hp == 0 {
                            Some(move_to)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        warn!("Error getting health to attack: {:?}", e);
                        None
                    }
                }
            }
        };
        if let Some(new_tilepos) = tilepos {
            if let Ok((mut tilepos, mut move_animation, transform)) =
                move_query.q1().get_mut(entity)
            {
                move_animation.set(new_tilepos.to_world_pos(transform.translation.z));
                *tilepos = new_tilepos
            }
        }
    }
}
