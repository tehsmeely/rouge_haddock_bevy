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
pub enum PowerEvent {
    PowerFired,
}

#[derive(Debug)]
pub enum GameEvent {
    PhaseComplete(GamePhase),
    PlayerDied,
    PlayerHooked,
    EndOfLevel,
}

#[derive(Debug)]
pub enum InfoEvent {
    // Events specifically for info and not necessarily drive systems
    EnemyKilled,
    PlayerHurt,
    PlayerMoved,
}
