use serde_json::json;
use socketioxide::SocketIo;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::info;

use crate::simulation::implementation::SimulationState;
use crate::simulation::ThreadEvent;
use crate::utils::tick_system::TickSystem;

use super::implementation::{batch_ticking, real_time_ticking, update};
use super::ws_emit;

fn calculate_delta(global_timer: &mut Instant) -> JoinHandle<Duration> {
    let tmp = global_timer.elapsed();
    *global_timer = Instant::now();

    tokio::spawn(async move { tmp })
}

pub async fn new(socket_io: SocketIo, mut receiver: mpsc::UnboundedReceiver<ThreadEvent>) {
    let mut tick_system = TickSystem::new(100);
    let mut global_timer = Instant::now();
    let pause = false;

    let mut tps_tracker = 0.0;

    let mut previous_state = SimulationState::default();
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
            let delta = calculate_delta(&mut global_timer).await.unwrap();
            tick_system.accumulate(delta);

            let mut tick_delta = 0;
            // Run the schedule until we run out of accumulated time
            while tick_system.expend() {
                tick_delta += 1;

                real_time_ticking(&mut previous_state, &mut state, socket_io.clone());
            }

            batch_ticking(
                &mut previous_state,
                &mut state,
                tick_delta,
                socket_io.clone(),
            );

            // #region Debug TPS
            let current_tps = if tick_system.delta != Duration::ZERO {
                1.0 / tick_system.delta.as_secs_f64()
            } else {
                0.0
            };

            if current_tps != tps_tracker {
                ws_emit(
                    socket_io.clone(),
                    "tick_debug",
                    json!({
                                    "current_tps": current_tps,
                                    "target_tps": tick_system.target_tps,
                    }),
                )
                .unwrap();

                tps_tracker = current_tps;
            }

            update(&mut previous_state, &mut state, socket_io.clone());
        }
    }
}
