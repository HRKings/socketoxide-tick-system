use serde::Serialize;
use serde_json::json;
use socketioxide::SocketIo;

use super::ws_emit;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Serialize)]
pub struct SimulationState {
    pub current_day: u64,
    pub current_hour: u64,
    pub current_month: u64,
    pub current_year: u64,
    pub ticks_until_next_hour: u64,
}

impl SimulationState {
    pub const TICKS_PER_HOUR: u64 = 2;
    pub const HOURS_PER_DAY: u64 = 24;

    pub const OPENING_HOUR: u64 = 6;
    pub const ENDING_HOUR: u64 = 18;

    pub const DAYS_PER_MONTH: u64 = 30;
    pub const MONTHS_PER_YEAR: u64 = 12;

    pub fn handle_hours(&mut self) -> bool {
        self.ticks_until_next_hour += 1;

        if self.ticks_until_next_hour >= Self::TICKS_PER_HOUR {
            self.ticks_until_next_hour -= Self::TICKS_PER_HOUR;
            self.current_hour += 1;

            return true;
        }

        false
    }

    pub fn handle_days(&mut self) -> bool {
        if self.current_hour >= Self::HOURS_PER_DAY {
            self.current_hour -= Self::HOURS_PER_DAY;
            self.current_day += 1;

            return true;
        }

        false
    }

    pub fn handle_months(&mut self) -> bool {
        if self.current_day >= Self::DAYS_PER_MONTH {
            self.current_day -= Self::DAYS_PER_MONTH;
            self.current_month += 1;

            return true;
        }

        false
    }

    pub fn handle_years(&mut self) -> bool {
        if self.current_month >= Self::MONTHS_PER_YEAR {
            self.current_month -= Self::MONTHS_PER_YEAR;
            self.current_year += 1;

            return true;
        }

        false
    }

    pub fn get_total_hours(&self) -> u128 {
        let years_to_hours = self.current_year as u128
            * Self::MONTHS_PER_YEAR as u128
            * Self::DAYS_PER_MONTH as u128
            * Self::HOURS_PER_DAY as u128;

        let months_to_hours =
            self.current_month as u128 * Self::DAYS_PER_MONTH as u128 * Self::HOURS_PER_DAY as u128;

        let days_to_hours = self.current_day as u128 * Self::HOURS_PER_DAY as u128;

        years_to_hours + months_to_hours + days_to_hours + self.current_hour as u128
    }
}

pub fn update(
    previous_state: &mut SimulationState,
    state: &mut SimulationState,
    socket_io: SocketIo,
) {
    if previous_state.get_total_hours() - state.get_total_hours() > 24 * 10 {
        ws_emit(
            socket_io.clone(),
            "state_debug",
            json!({ "state": state, "total_elapsed_hours": state.get_total_hours()}),
        )
        .unwrap();

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
        if state.current_hour == SimulationState::OPENING_HOUR {
            ws_emit(socket_io.clone(), "announcer", json!("Opening hours")).unwrap();
        }

        if state.current_hour == SimulationState::ENDING_HOUR {
            ws_emit(socket_io.clone(), "announcer", json!("Closing hours")).unwrap();
        }
    }

    if state.handle_days() {
        ws_emit(socket_io.clone(), "announcer", json!("Day starting")).unwrap();
    }

    if state.handle_months() {
        ws_emit(socket_io.clone(), "announcer", json!("Month starting")).unwrap();
    }

    if state.handle_years() {
        ws_emit(socket_io.clone(), "announcer", json!("Year starting")).unwrap();
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
