#![no_std]

hotaru_rt_embassy::define_runtime_worker_pool!(2);

pub const WORKER_COUNT: usize = EmbassyRuntime::WORKER_COUNT;
