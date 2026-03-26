use std::fs;
use sysinfo::{Pid, System};

/// Compact info for the process list view.
#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub cmd: String,
    pub user: String,
    pub thread_count: usize,
}

/// Extended info shown in the detail view.
#[derive(Clone, Debug)]
pub struct ProcessDetail {
    pub info: ProcessInfo,
    pub exe: String,
    pub cwd: String,
    pub environ_count: usize,
    pub threads: usize,
    pub fd_count: usize,
    pub maps_count: usize,
}

/// Read the Tgid (thread group ID) from /proc/<pid>/status.
/// Returns None if the file can't be read.
fn read_tgid(pid: u32) -> Option<u32> {
    let path = format!("/proc/{}/status", pid);
    let content = fs::read_to_string(&path).ok()?;
    for line in content.lines() {
        if line.starts_with("Tgid:") {
            return line.split_whitespace().nth(1)?.parse::<u32>().ok();
        }
    }
    None
}

/// Read the accurate process state from /proc/<pid>/stat.
/// The state is a single character: R, S, D, Z, T, t, W, X, etc.
fn read_proc_state(pid: u32) -> Option<char> {
    let path = format!("/proc/{}/stat", pid);
    let content = fs::read_to_string(&path).ok()?;
    // Format: pid (comm) state ...
    // Find the closing ')' then the state char follows after a space
    let close_paren = content.rfind(')')?;
    let after = &content[close_paren + 1..];
    after.trim().chars().next()
}

/// Map /proc/pid/stat state character to a human-readable status string.
fn state_char_to_status(c: char) -> String {
    match c {
        'R' => "Running".into(),
        'S' => "Sleeping".into(),
        'D' => "Disk Sleep".into(),
        'Z' => "Zombie".into(),
        'T' => "Stopped".into(),
        't' => "Tracing".into(),
        'W' => "Paging".into(),
        'X' | 'x' => "Dead".into(),
        'K' => "Wakekill".into(),
        'P' => "Parked".into(),
        'I' => "Idle".into(),
        other => format!("{}", other),
    }
}

/// Return all processes owned by the current user.
/// Only includes main processes (pid == tgid), filtering out individual threads.
pub fn list_user_processes(sys: &System) -> Vec<ProcessInfo> {
    let uid = get_current_uid();
    let mut out: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .filter_map(|p| {
            let proc_uid = p.user_id().map(|u| **u as u32);
            if proc_uid != Some(uid) {
                return None;
            }

            let pid = p.pid().as_u32();

            // Only keep main thread (pid == tgid) to avoid listing every thread
            if let Some(tgid) = read_tgid(pid) {
                if pid != tgid {
                    return None;
                }
            }

            // Read accurate status from /proc/pid/stat
            let status = read_proc_state(pid)
                .map(state_char_to_status)
                .unwrap_or_else(|| "Unknown".into());

            // Count threads from /proc/<pid>/task
            let thread_count = fs::read_dir(format!("/proc/{}/task", pid))
                .map(|rd| rd.count())
                .unwrap_or(1);

            Some(ProcessInfo {
                pid,
                name: p.name().to_string_lossy().to_string(),
                cpu_pct: p.cpu_usage(),
                mem_bytes: p.memory(),
                status,
                cmd: p
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                user: uid.to_string(),
                thread_count,
            })
        })
        .collect();
    out.sort_by_key(|p| p.pid);
    out
}

/// Get extended detail for a single process.
pub fn get_process_detail(sys: &System, pid: u32) -> Option<ProcessDetail> {
    let sysinfo_pid = Pid::from_u32(pid);
    let p = sys.process(sysinfo_pid)?;
    let proc_path = format!("/proc/{}", pid);

    let exe = p
        .exe()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "N/A".into());

    let cwd_path = format!("{}/cwd", proc_path);
    let cwd = fs::read_link(&cwd_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "N/A".into());

    let environ_count = p.environ().len();
    let threads = count_dir_entries(&format!("{}/task", proc_path));
    let fd_count = count_dir_entries(&format!("{}/fd", proc_path));
    let maps_count = count_maps(pid);

    let status = read_proc_state(pid)
        .map(state_char_to_status)
        .unwrap_or_else(|| "Unknown".into());

    let info = ProcessInfo {
        pid,
        name: p.name().to_string_lossy().to_string(),
        cpu_pct: p.cpu_usage(),
        mem_bytes: p.memory(),
        status,
        cmd: p
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" "),
        user: "".into(),
        thread_count: threads,
    };

    Some(ProcessDetail {
        info,
        exe,
        cwd,
        environ_count,
        threads,
        fd_count,
        maps_count,
    })
}

fn count_dir_entries(path: &str) -> usize {
    fs::read_dir(path).map(|rd| rd.count()).unwrap_or(0)
}

fn count_maps(pid: u32) -> usize {
    let path = format!("/proc/{}/maps", pid);
    fs::read_to_string(&path)
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

fn get_current_uid() -> u32 {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Uid:") {
                if let Some(uid_str) = line.split_whitespace().nth(1) {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        return uid;
                    }
                }
            }
        }
    }
    u32::MAX
}

/// Format bytes into human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
