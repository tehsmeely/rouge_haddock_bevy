use crate::game::components::{Facing, Health, MapDirection, MovementAnimate, Player, TileType};
use crate::game::enemy::Enemy;
use crate::game::tilemap::{HasTileType, TilePosExt};
use bevy::prelude::*;
use bevy_ecs_tilemap::{MapQuery, TilePos};
use std::collections::HashMap;

#[derive(Debug)]
pub enum MoveDecision {
    Move((TilePos, MapDirection)),
    Nothing,
    AttackAndMaybeMove((TilePos, MapDirection, Entity)),
    AttackAndDontMove((Entity, MapDirection)),
}

pub type MoveDecisions = HashMap<Entity, MoveDecision>;

pub struct AttackCriteria {
    damage: usize,
    can_attack_enemy: bool,
    can_attack_player: bool,
    move_on_attack: bool,
}
impl AttackCriteria {
    pub fn for_player() -> Self {
        Self {
            damage: 1,
            can_attack_enemy: true,
            can_attack_player: false,
            move_on_attack: true,
        }
    }
    pub fn for_enemy() -> Self {
        Self {
            damage: 1,
            can_attack_enemy: true,
            can_attack_player: false,
            move_on_attack: false,
        }
    }
}

fn attack_decision(
    attack_criteria: &AttackCriteria,
    destination_tilepos: TilePos,
    move_direction: MapDirection,
    target_entity: Entity,
) -> MoveDecision {
    match attack_criteria.move_on_attack {
        true => MoveDecision::AttackAndMaybeMove((
            destination_tilepos,
            move_direction.clone(),
            target_entity,
        )),
        false => MoveDecision::AttackAndDontMove((target_entity, move_direction.clone())),
    }
}

pub fn decide_move(
    current_pos: &TilePos,
    move_direction: &MapDirection,
    attack_criteria: &AttackCriteria,
    move_query: Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
    mut map_query: &mut MapQuery,
    tile_type_query: &Query<&HasTileType>,
) -> MoveDecision {
    // TODO Rework to allow signle use not hashmap
    // Make decisions
    let destination_tilepos = current_pos.add(move_direction.to_pos_move());

    let new_tile_entity = map_query
        .get_tile_entity(destination_tilepos, 0, 0)
        .unwrap();
    let can_move = match tile_type_query.get(new_tile_entity) {
        Ok(HasTileType(tt)) => tt.can_enter(),
        Err(_) => false,
    };

    let mut decision = match can_move {
        true => MoveDecision::Move((destination_tilepos.clone(), move_direction.clone())),
        false => MoveDecision::Nothing,
    };

    for (target_entity, tilepos, maybe_player, maybe_enemy) in move_query.iter() {
        // Make decision
        if tilepos.eq(&destination_tilepos) {
            if maybe_player.is_some() && attack_criteria.can_attack_player {
                decision = attack_decision(
                    attack_criteria,
                    destination_tilepos,
                    move_direction.clone(),
                    target_entity,
                );
                break;
            } else if maybe_enemy.is_some() && attack_criteria.can_attack_enemy {
                decision = attack_decision(
                    attack_criteria,
                    destination_tilepos,
                    move_direction.clone(),
                    target_entity,
                );
                break;
            }
        }
    }
    decision
}

pub fn apply_move_single(
    entity: Entity,
    move_decision: &MoveDecision,
    mut move_query: &mut Query<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    mut health_query: &mut Query<&mut Health>,
    mut commands: &mut Commands,
) {
    let (maybe_tilepos, maybe_facing) = match move_decision {
        MoveDecision::Nothing => (None, None),
        MoveDecision::Move((tilepos, facing)) => (Some(tilepos), Some(facing)),
        MoveDecision::AttackAndDontMove((target, facing)) => {
            let target_health = health_query.get_mut(*target);
            match target_health {
                Ok(mut health) => {
                    health.decr_by(1);
                    commands.entity(*target).despawn();
                }
                Err(e) => warn!("Error getting health to attack: {:?}", e),
            }
            (None, Some(facing))
        }
        MoveDecision::AttackAndMaybeMove((move_to, facing, target)) => {
            let target_health = health_query.get_mut(*target);
            let result_tilepos = match target_health {
                Ok(mut health) => {
                    health.decr_by(1);
                    if health.hp == 0 {
                        commands.entity(*target).despawn();
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
    if maybe_tilepos.is_some() || maybe_facing.is_some() {
        if let Ok((mut tilepos, mut move_animation, transform, mut facing)) =
            move_query.get_mut(entity)
        {
            if let Some(new_tilepos) = maybe_tilepos {
                move_animation.set(new_tilepos.to_world_pos(transform.translation.z));
                *tilepos = new_tilepos.clone()
            }
            if let Some(new_facing) = maybe_facing {
                facing.0 = new_facing.clone();
            }
        }
    }
}

pub fn apply_move(
    move_decisions: MoveDecisions,
    mut move_query: Query<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    mut health_query: Query<&mut Health>,
    mut commands: &mut Commands,
) {
    //Apply decisions:
    for (entity, decision) in move_decisions.iter() {
        apply_move_single(
            *entity,
            decision,
            &mut move_query,
            &mut health_query,
            commands,
        )
    }
}
