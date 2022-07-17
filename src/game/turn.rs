use log::warn;

#[derive(Debug)]
pub struct TurnCounter(usize);

impl TurnCounter {
    pub fn incr(&mut self) {
        self.0 += 1;
    }

    pub fn reset(&mut self) {
        self.0 = 0;
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
    pub reset: bool,
}

impl Default for GlobalTurnCounter {
    fn default() -> Self {
        Self {
            turn_count: 1,
            current_phase: GamePhase::PlayerMovement,
            reset: true,
        }
    }
}

impl GlobalTurnCounter {
    pub fn step(&mut self, from_phase: &GamePhase) {
        if *from_phase == self.current_phase {
            if self.current_phase.last() {
                self.turn_count += 1;
                if self.reset {
                    self.reset = false;
                }
            }
            self.current_phase = self.current_phase.next();
        } else {
            warn!("Attempted to step phase from non-current phase. Current_phase:{:?}, step_from:{:?}", self.current_phase, from_phase);
        }
    }

    pub fn can_take_turn(&self, local_count: &mut TurnCounter, phase: GamePhase) -> bool {
        if self.reset && local_count.0 > self.turn_count {
            local_count.0 = self.turn_count - 1;
            println!("Resetting local count: {:?}", self);
        }

        local_count.0 < self.turn_count && phase == self.current_phase
    }

    pub fn reset(&mut self) {
        let default = Self::default();
        self.turn_count = default.turn_count;
        self.current_phase = default.current_phase;
        self.reset = true;
    }
}

#[derive(Debug, PartialEq)]
pub enum GamePhase {
    PlayerMovement,
    PlayerPowerEffect,
    PreEnemyMovement,
    EnemyPowerEffect,
    EnemyMovement,
}

impl GamePhase {
    fn next(&self) -> Self {
        match self {
            GamePhase::PlayerMovement => GamePhase::PlayerPowerEffect,
            GamePhase::PlayerPowerEffect => GamePhase::PreEnemyMovement,
            GamePhase::PreEnemyMovement => GamePhase::EnemyPowerEffect,
            GamePhase::EnemyPowerEffect => GamePhase::EnemyMovement,
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

pub struct GlobalLevelCounter {
    level_count: usize,
}

impl Default for GlobalLevelCounter {
    fn default() -> Self {
        Self { level_count: 1 }
    }
}

impl GlobalLevelCounter {
    pub fn increment(&mut self) {
        self.level_count += 1;
    }

    pub fn level(&self) -> usize {
        self.level_count
    }

    pub fn reset(&mut self) {
        self.level_count = 1;
    }
}
