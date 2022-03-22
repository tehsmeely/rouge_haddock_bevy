use crate::game::components::{Facing, Health, MapDirection, MovementAnimate, Player};
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
    Turn(MapDirection),
}

impl MoveDecision {
    pub fn to_move_position(&self) -> Option<TilePos> {
        match self {
            Self::Nothing | Self::Turn(_) | Self::AttackAndDontMove(_) => None,
            Self::Move((tilepos, _)) | Self::AttackAndMaybeMove((tilepos, _, _)) => {
                Some(*tilepos)
            }
        }
    }
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
            can_attack_enemy: false,
            can_attack_player: true,
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
            move_direction,
            target_entity,
        )),
        false => MoveDecision::AttackAndDontMove((target_entity, move_direction)),
    }
}

pub fn decide_move(
    current_pos: &TilePos,
    move_direction: &MapDirection,
    max_move_distance: usize,
    attack_criteria: &AttackCriteria,
    move_query: Query<(Entity, &TilePos, Option<&Player>, Option<&Enemy>)>,
    map_query: &mut MapQuery,
    tile_type_query: &Query<&HasTileType>,
    additional_ignore_tilepos: &Vec<TilePos>,
) -> MoveDecision {
    let destination_tilepos_list = {
        let mut v = Vec::new();
        let mut prev = current_pos;
        for _ in 0..max_move_distance {
            let new_pos = prev.add(move_direction.to_pos_move());
            v.push(new_pos);
            prev = v.last().unwrap()
        }
        v
    };

    let mut decision = MoveDecision::Turn(move_direction.clone());
    let mut stopped_early = false;

    for destination_tilepos in destination_tilepos_list.iter() {
        if stopped_early {
            break;
        }

        let new_tile_entity = map_query
            .get_tile_entity(*destination_tilepos, 0, 0)
            .unwrap();
        let can_move = match tile_type_query.get(new_tile_entity) {
            Ok(HasTileType(tt)) => tt.can_enter(),
            Err(_) => false,
        };

        let can_move = can_move && !additional_ignore_tilepos.contains(destination_tilepos);

        if !can_move {
            break;
        } else {
            decision = MoveDecision::Move((*destination_tilepos, move_direction.clone()))
        }

        for (target_entity, tilepos, maybe_player, maybe_enemy) in move_query.iter() {
            if tilepos.eq(destination_tilepos) {
                if maybe_player.is_some() {
                    if attack_criteria.can_attack_player {
                        decision = attack_decision(
                            attack_criteria,
                            *destination_tilepos,
                            move_direction.clone(),
                            target_entity,
                        );
                    } else {
                        decision = MoveDecision::Turn(move_direction.clone());
                    }
                    stopped_early = true;
                    break;
                } else if maybe_enemy.is_some() {
                    if attack_criteria.can_attack_enemy {
                        decision = attack_decision(
                            attack_criteria,
                            *destination_tilepos,
                            move_direction.clone(),
                            target_entity,
                        );
                    } else {
                        decision = MoveDecision::Turn(move_direction.clone());
                    }
                    stopped_early = true;
                    break;
                }
            }
        }
    }
    decision
}

pub fn apply_move_single(
    entity: Entity,
    move_decision: &MoveDecision,
    move_query: &mut Query<(&mut TilePos, &mut MovementAnimate, &Transform, &mut Facing)>,
    health_query: &mut Query<&mut Health>,
) {
    let (maybe_tilepos, maybe_facing) = match move_decision {
        MoveDecision::Nothing => (None, None),
        MoveDecision::Turn(facing) => (None, Some(facing)),
        MoveDecision::Move((tilepos, facing)) => (Some(tilepos), Some(facing)),
        MoveDecision::AttackAndDontMove((target, facing)) => {
            let target_health = health_query.get_mut(*target);
            match target_health {
                Ok(mut health) => {
                    health.decr_by(1);
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
                *tilepos = *new_tilepos
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
) {
    //Apply decisions:
    for (entity, decision) in move_decisions.iter() {
        apply_move_single(*entity, decision, &mut move_query, &mut health_query)
    }
}
