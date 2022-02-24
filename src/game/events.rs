use crate::game::components::MapDirection;
use crate::game::turn::GamePhase;

#[derive(Debug)]
pub enum InputEvent {
    MoveDirection(MapDirection),
    TurnDirection(MapDirection),
    Wait,
    Power,
}

#[derive(Debug)]
pub enum GameEvent {
    PhaseComplete(GamePhase),
}