//! Organizing a small application on `hotaru_rt_embassy`.
//!
//! Run this desktop simulation with:
//!
//! ```text
//! cargo run -p hotaru_rt_embassy --example app_tutorial --no-default-features --features std
//! ```
//!
//! The example models a sensor application. One task produces readings while
//! another task reports the latest shared state. On hardware, replace the
//! simulated value and `println!` calls with sensor and display/network drivers.

use core::time::Duration;
use std::sync::Arc;

use hotaru_core::app::runtime::{AsyncMutexCap, RuntimeSpec};
use hotaru_rt_embassy::EmbassyMutex;

hotaru_rt_embassy::define_runtime_worker_pool!(
    pub AppRuntime,
    worker_count = 2,
    job_queue_capacity = 8,
);

#[derive(Default)]
struct AppState {
    sample_count: u32,
    latest_temperature_c: i16,
}

struct SensorApp {
    // Every background task receives an `Arc` clone of the same mutex, so all
    // tasks observe one application state without using global mutable data.
    state: Arc<EmbassyMutex<AppState>>,
}

impl SensorApp {
    fn new() -> Self {
        Self {
            state: Arc::new(EmbassyMutex::new(AppState::default())),
        }
    }

    fn start(&self) {
        let sampler_state = Arc::clone(&self.state);
        AppRuntime::spawn_detached(async move {
            let mut simulated_temperature = 20;

            loop {
                // Keep the guard in a small scope. It is dropped before the
                // timer is awaited, allowing the reporter to lock the state.
                {
                    let mut state = sampler_state.lock().await;
                    state.sample_count += 1;
                    state.latest_temperature_c = simulated_temperature;
                }

                simulated_temperature = 20 + (simulated_temperature - 19) % 6;
                AppRuntime::sleep(Duration::from_millis(500)).await;
            }
        });

        let reporter_state = Arc::clone(&self.state);
        AppRuntime::spawn_detached(async move {
            loop {
                AppRuntime::sleep(Duration::from_secs(1)).await;

                let state = reporter_state.lock().await;
                println!(
                    "sample #{}, temperature: {} C",
                    state.sample_count, state.latest_temperature_c
                );
            }
        });
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    // The runtime owns scheduling; the application owns domain state and
    // decides which long-lived tasks make up its behavior.
    AppRuntime::init(spawner);

    let app = SensorApp::new();
    app.start();

    // Keep the entry task alive while the application's worker tasks run.
    core::future::pending::<()>().await;
}
