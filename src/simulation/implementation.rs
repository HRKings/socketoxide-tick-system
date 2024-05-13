use serde::Serialize;
use serde_json::json;
use socketioxide::SocketIo;

use super::ws_emit;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize)]
pub struct SimulationState {
    pub days_passed: u64,
    pub hours_passed: u64,
    pub months_passed: u64,
    pub ticks_until_nex_hour: u64,
}

impl SimulationState {
    pub const TICKS_PER_HOUR: u64 = 2;
    pub const HOURS_PER_DAY: u64 = 24;

    pub const DAY_START: u64 = 6;
    pub const DAY_END: u64 = 18;

    pub const DAYS_PER_MONTH: u64 = 30;

    pub fn handle_hours(&mut self) -> bool {
        self.ticks_until_nex_hour += 1;

        if self.ticks_until_nex_hour >= Self::TICKS_PER_HOUR {
            self.ticks_until_nex_hour -= Self::TICKS_PER_HOUR;
            self.hours_passed += 1;

            return true;
        }

        false
    }

    pub fn handle_days(&mut self) -> bool {
        if self.hours_passed >= Self::HOURS_PER_DAY {
            self.hours_passed -= Self::HOURS_PER_DAY;
            self.days_passed += 1;

            return true;
        }

        false
    }
}

pub fn update(
    previous_state: &mut SimulationState,
    state: &mut SimulationState,
    socket_io: SocketIo,
) {
    if previous_state.days_passed.abs_diff(state.days_passed) >= 10 {
        ws_emit(socket_io.clone(), "tick_debug", json!(state)).unwrap();

        *previous_state = state.clone();
    }
}

pub fn real_time_ticking(
    previous_state: &mut SimulationState,
    state: &mut SimulationState,
    socket_io: SocketIo,
) {
    let _ = previous_state;

    if state.handle_hours() {
        if state.hours_passed == SimulationState::DAY_START {
            ws_emit(socket_io.clone(), "announcer", json!("Day started")).unwrap();
        }

        if state.hours_passed == SimulationState::DAY_END {
            ws_emit(socket_io.clone(), "announcer", json!("Day ending")).unwrap();
        }
    }

    if state.handle_days() && state.days_passed % SimulationState::DAYS_PER_MONTH == 0 {
        state.months_passed += 1;
        ws_emit(socket_io.clone(), "announcer", json!("Month ending")).unwrap();
    }
}

pub fn batch_ticking(
    previous_state: &mut SimulationState,
    state: &mut SimulationState,
    tick_delta: usize,
    socket_io: SocketIo,
) {
    let _ = socket_io;
    let _ = tick_delta;
    let _ = state;
    let _ = previous_state;
}
