use serde_json::{json, Value};
use socketioxide::{BroadcastError, SocketIo};
use std::borrow::Cow;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::info;

use crate::simulation::ThreadEvent;
use crate::utils::tick_system::TickSystem;

fn calculate_delta(global_timer: &mut Instant) -> Duration {
    let tmp = global_timer.elapsed();
    *global_timer = Instant::now();

    tmp
}

fn emit_async(
    socket_io: SocketIo,
    last_message: &mut Instant,
    event: impl Into<Cow<'static, str>> + std::marker::Send + 'static,
    data: Value,
) -> JoinHandle<Result<(), BroadcastError>> {
    // if last_message.elapsed().as_secs() <= 1 {
    //     return tokio::spawn(async { Ok(()) });
    // }

    // *last_message = Instant::now();
    tokio::spawn(async move { socket_io.of("/simulation").unwrap().emit(event, data) })
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
struct SimulationState {
    pub days_passed: u64,
    pub hours_passed: u64,
    pub ticks_until_nex_hour: u64,
}

impl SimulationState {
    pub const TICKS_PER_HOUR: u64 = 2;
    pub const HOURS_PER_DAY: u64 = 24;

    pub const DAYS_PER_MOTH: u64 = 30;

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

pub async fn new(socket_io: SocketIo, mut receiver: mpsc::UnboundedReceiver<ThreadEvent>) {
    let mut tick_system = TickSystem::new(100);
    let mut global_timer = Instant::now();
    let pause = false;

    let mut tps_tracker = 0.0;
    let mut last_socket = Instant::now();

    let mut old_state = SimulationState::default();
    let mut state = SimulationState::default();

    loop {
        match receiver.try_recv() {
            Ok(ThreadEvent::Shutdown) => {
                info!("Shutting down gracefully");
                break;
            }
            Ok(ThreadEvent::ChangeTargetTPS(new_target)) => tick_system.target_tps = new_target,
            Err(TryRecvError::Empty) => (),
            _ => panic!(),
        }

        if !pause {
            let delta = calculate_delta(&mut global_timer);
            tick_system.accumulate(delta);

            // Run the schedule until we run out of accumulated time
            while tick_system.expend() {
                if state.handle_hours() {
                    if state.hours_passed == 6 {
                        _ = emit_async(
                            socket_io.clone(),
                            &mut last_socket,
                            "announcer",
                            json!("Day started"),
                        )
                        .await
                        .unwrap();
                    }

                    if state.hours_passed == 18 {
                        _ = emit_async(
                            socket_io.clone(),
                            &mut last_socket,
                            "announcer",
                            json!("Day ending"),
                        )
                        .await
                        .unwrap();
                    }
                }

                if state.handle_days() && state.days_passed % SimulationState::DAYS_PER_MOTH == 0 {
                    _ = emit_async(
                        socket_io.clone(),
                        &mut last_socket,
                        "announcer",
                        json!("Month ending"),
                    )
                    .await
                    .unwrap();
                }
            }

            let current_tps = if tick_system.delta != Duration::ZERO {
                1.0 / tick_system.delta.as_secs_f64()
            } else {
                0.0
            };

            if current_tps != tps_tracker {
                _ = emit_async(
                    socket_io.clone(),
                    &mut last_socket,
                    "tick_debug",
                    json!({
                        "current_tps": current_tps,
                        "target_tps": tick_system.target_tps,
                    }),
                )
                .await
                .unwrap();

                tps_tracker = current_tps;
            }

            if old_state.days_passed.abs_diff(state.days_passed) >= 10 {
                _ = emit_async(
                    socket_io.clone(),
                    &mut last_socket,
                    "tick_debug",
                    json!({
                        "days_passed": state.days_passed,
                        "hours_passed": state.hours_passed,
                    }),
                )
                .await
                .unwrap();

                old_state = state.clone();
            }
        }
    }
}
