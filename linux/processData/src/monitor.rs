use crate::process::{list_user_processes, ProcessInfo};
use std::collections::HashSet;
use std::time::Instant;
use sysinfo::System;

/// Tracks processes and detects new ones appearing.
pub struct ProcessMonitor {
    known_pids: HashSet<u32>,
    sys: System,
    last_poll: Instant,
}

/// A newly detected process with a timestamp.
#[derive(Clone, Debug)]
pub struct NewProcessEvent {
    pub info: ProcessInfo,
    pub detected_at: Instant,
}

impl ProcessMonitor {
    /// Create a new monitor, seeding with the current process list.
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let known_pids: HashSet<u32> = list_user_processes(&sys).iter().map(|p| p.pid).collect();

        Self {
            known_pids,
            sys,
            last_poll: Instant::now(),
        }
    }

    /// Poll for newly appeared processes since the last call.
    /// Returns info about each new process and updates internal state.
    pub fn poll(&mut self) -> Vec<NewProcessEvent> {
        self.sys.refresh_all();
        let current = list_user_processes(&self.sys);
        let now = Instant::now();

        let mut new_events = Vec::new();

        for proc_info in &current {
            if !self.known_pids.contains(&proc_info.pid) {
                new_events.push(NewProcessEvent {
                    info: proc_info.clone(),
                    detected_at: now,
                });
                self.known_pids.insert(proc_info.pid);
            }
        }

        // Remove PIDs that no longer exist
        let current_pids: HashSet<u32> = current.iter().map(|p| p.pid).collect();
        self.known_pids.retain(|pid| current_pids.contains(pid));

        self.last_poll = now;
        new_events
    }

    /// Get a reference to the internal System for reuse.
    pub fn system(&self) -> &System {
        &self.sys
    }

    /// Get elapsed time since monitor started (approximated from last poll reset).
    pub fn uptime(&self) -> std::time::Duration {
        self.last_poll.elapsed()
    }
}
