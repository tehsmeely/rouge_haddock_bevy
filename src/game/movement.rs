use crate::game::components::{Facing, Health, MapDirection, MovementAnimate, Player};
use crate::game::enemy::Enemy;
use crate::game::tilemap::TilePosExt;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilePos;
use std::collections::HashMap;

enum MoveDecision {
    Move((TilePos, MapDirection)),
    Nothing,
    AttackAndMaybeMove((Tilepos, MapDirection, Entity)),
    AttackAndDontMove((Entity, MapDirection)),
}

type MoveDecisions = HashMap<Entity, MoveDecision>;

pub struct AttackCriteria {
    damage: usize,
    can_attack_enemy: bool,
    can_attack_player: bool,
}
impl AttackCriteria {
    pub fn for_player() -> Self {
        Self {
            damage: 1,
            can_attack_enemy: true,
            can_attack_player: false,
        }
    }
    pub fn for_enemy() -> Self {
        Self {
            damage: 1,
            can_attack_enemy: true,
            can_attack_player: false,
        }
    }
}

pub fn _decide_move(
    move_query: Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
) -> MoveDecisions {
    // Make decisions
    let mut move_decisions: HashMap<Entity, MoveDecision> = HashMap::new();
    for (entity, tilepos, maybe_player, maybe_enemy) in move_query.q0().iter() {
        // Make decision
    }
    move_decisions
}

pub fn decide_move(
    current_pos: &TilePos,
    move_direction: &MapDirection,
    attack_criteria: &AttackCriteria,
    moving_entity: Entity,
    move_query: Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
) -> MoveDecisions {
    // Make decisions
    let mut move_decisions: HashMap<Entity, MoveDecision> = HashMap::new();

    let destination_tilepos = current_pos.add(move_direction.to_pos_move());

    let mut decision = MoveDecision::Nothing;
    for (entity, tilepos, maybe_player, maybe_enemy) in move_query.iter() {
        // Make decision
        if tilepos.eq(destination_tilepos) {
            if maybe_player.is_some() && attack_criteria.can_attack_player {
                decision =
                    MoveDecision::AttackAndMaybeMove((destination_tilepos, move_direction, entity));
                break;
            } else if maybe_enemy.is_some() && attack_criteria.can_attack_enemy {
            }
        }
    }
    move_decisions.insert(moving_entity, decision);
    move_decisions
}

pub fn apply_move(
    move_decisions: MoveDecisions,
    mut move_query: Query<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    mut health_query: Query<&mut Health>,
) {
    //Apply decisions:
    for (entity, decision) in move_decisions.iter() {
        let (maybe_tilepos, maybe_facing) = match decision {
            MoveDecision::Nothing => (None, None),
            MoveDecision::Move((tilepos, facing)) => (Some(tilepos), Some(facing)),
            MoveDecision::AttackAndDontMove((target, facing)) => {
                let target_health = health_query.get_mut(*target);
                match target_health {
                    Ok(&mut health) => health.decr_by(1),
                    Err(e) => warn!("Error getting health to attack: {:?}", e),
                }
                (None, Some(facing))
            }
            MoveDecision::AttackAndMaybeMove((move_to, facing, target)) => {
                let target_health = health_query.get_mut(*target);
                let result_tilepos = match target_health {
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
                };
                (result_tilepos, Some(facing))
            }
        };
        if tilepos.is_some() || facing.is_some() {
            if let Ok((mut tilepos, mut move_animation, transform, mut facing)) =
                move_query.get_mut(entity)
            {
                if let Some(new_tilepos) = maybe_tilepos {
                    move_animation.set(new_tilepos.to_world_pos(transform.translation.z));
                    *tilepos = new_tilepos
                }
                if let Some(new_facing) = maybe_facing {
                    facing.0 = new_facing;
                }
            }
        }
    }
}
