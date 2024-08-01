extern crate karlsen_consensus;
extern crate karlsen_core;
extern crate karlsen_hashes;

use std::sync::Arc;

use karlsen_alloc::init_allocator_with_default_settings;
use karlsen_core::{info, signals::Signals};
use karlsen_utils::fd_budget;
use karlsend_lib::{
    args::parse_args,
    daemon::{create_core, DESIRED_DAEMON_SOFT_FD_LIMIT, MINIMUM_DAEMON_SOFT_FD_LIMIT},
};

#[cfg(feature = "heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() {
    #[cfg(feature = "heap")]
    let _profiler = dhat::Profiler::builder()
        .file_name("karlsend-heap.json")
        .build();

    init_allocator_with_default_settings();

    let args = parse_args();

    match fd_budget::try_set_fd_limit(DESIRED_DAEMON_SOFT_FD_LIMIT) {
        Ok(limit) => {
            if limit < MINIMUM_DAEMON_SOFT_FD_LIMIT {
                println!("Current OS file descriptor limit (soft FD limit) is set to {limit}");
                println!("The karlsend node requires a setting of at least {DESIRED_DAEMON_SOFT_FD_LIMIT} to operate properly.");
                println!("Please increase the limits using the following command:");
                println!("ulimit -n {DESIRED_DAEMON_SOFT_FD_LIMIT}");
            }
        }
        Err(err) => {
            println!("Unable to initialize the necessary OS file descriptor limit (soft FD limit) to: {}", err);
            println!("The karlsend node requires a setting of at least {DESIRED_DAEMON_SOFT_FD_LIMIT} to operate properly.");
        }
    }

    let fd_total_budget = fd_budget::limit()
        - args.rpc_max_clients as i32
        - args.inbound_limit as i32
        - args.outbound_target as i32;
    let (core, _) = create_core(args, fd_total_budget);

    // Bind the keyboard signal to the core
    Arc::new(Signals::new(&core)).init();

    core.run();
    info!("Karlsend has stopped...");
}
