
use log::warn;

#[derive(Debug)]
pub struct TurnCounter(usize);

impl TurnCounter {
    pub fn incr(&mut self) {
        self.0 += 1;
    }
}
impl Default for TurnCounter {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug)]
pub struct GlobalTurnCounter {
    pub turn_count: usize,
    pub current_phase: GamePhase,
}

impl Default for GlobalTurnCounter {
    fn default() -> Self {
        Self {
            turn_count: 1,
            current_phase: GamePhase::PlayerMovement,
        }
    }
}

impl GlobalTurnCounter {
    pub fn step(&mut self, from_phase: &GamePhase) {
        if *from_phase == self.current_phase {
            if self.current_phase.last() {
                self.turn_count += 1;
            }
            self.current_phase = self.current_phase.next();
        } else {
            warn!("Attempted to step phase from non-current phase. Current_phase:{:?}, step_from:{:?}", self.current_phase, from_phase);
        }
    }

    pub fn can_take_turn(&self, local_count: &TurnCounter, phase: GamePhase) -> bool {
        local_count.0 < self.turn_count && phase == self.current_phase
    }
}

#[derive(Debug, PartialEq)]
pub enum GamePhase {
    PlayerMovement,
    PlayerPowerEffect,
    EnemyMovement,
}

impl GamePhase {
    fn next(&self) -> Self {
        match self {
            GamePhase::PlayerMovement => GamePhase::PlayerPowerEffect,
            GamePhase::PlayerPowerEffect => GamePhase::EnemyMovement,
            GamePhase::EnemyMovement => GamePhase::PlayerMovement,
        }
    }

    fn last(&self) -> bool {
        match self {
            GamePhase::EnemyMovement => true,
            _ => false,
        }
    }
}
