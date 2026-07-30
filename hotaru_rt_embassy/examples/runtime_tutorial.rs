//! A first tour of `hotaru_rt_embassy`.
//!
//! Run this desktop version with:
//!
//! ```text
//! cargo run -p hotaru_rt_embassy --example runtime_tutorial --no-default-features --features std
//! ```
//!
//! A board project uses the same runtime macro and `RuntimeSpec` methods, but
//! supplies its chip-specific Embassy entry point and peripherals.

use core::time::Duration;

use hotaru_core::app::runtime::RuntimeSpec;

// The macro creates one runtime type, one runtime storage value, and a pool of
// Embassy worker tasks. Queue capacity and worker count do not have to match.
hotaru_rt_embassy::define_runtime_worker_pool!(
    pub TutorialRuntime,
    worker_count = 2,
    job_queue_capacity = 8,
);

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    // Initialize before calling any Hotaru API that spawns a task. Calling
    // `init` again is harmless: workers are started only on the first call.
    TutorialRuntime::init(spawner);
    assert!(TutorialRuntime::is_initialized());

    // A detached task runs in the background and has no return handle.
    TutorialRuntime::spawn_detached(async {
        println!("detached task: hello from a Hotaru Embassy worker");
    });

    // `spawn` returns an awaitable handle when the caller needs the result.
    let calculation = TutorialRuntime::spawn(async {
        TutorialRuntime::sleep(Duration::from_millis(20)).await;
        6 * 7
    });

    let answer = calculation.await.expect("calculation task failed");
    println!("joined task result: {answer}");

    // Runtime helpers keep application code independent of Embassy's concrete
    // timer types.
    let timed = TutorialRuntime::timeout(Duration::from_millis(10), async {
        TutorialRuntime::sleep(Duration::from_millis(50)).await;
        "finished"
    })
    .await;
    assert!(timed.is_err());
    println!("timeout example: the slower future was cancelled");

    // Embassy executors are designed to keep running. A real firmware entry
    // point would normally await the application's main loop here.
    println!("tutorial complete; press Ctrl+C to stop the executor");
    core::future::pending::<()>().await;
}
