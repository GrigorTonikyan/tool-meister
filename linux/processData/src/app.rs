use std::time::Instant;
use sysinfo::System;

use crate::fuzzy;
use crate::memory::{self, MemoryRegion, StringMatch};
use crate::monitor::{NewProcessEvent, ProcessMonitor};
use crate::process::{self, ProcessDetail, ProcessInfo};

// ── Modes ────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppMode {
    ProcessList,
    ProcessDetail,
    StringView,
    MonitorMode,
    MemoryInspect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Mem,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputTarget {
    None,
    Filter,
    Search,
    MinLen,
    Address,
}

// ── Default column widths ────────────────────────────────────────────────────

pub const NUM_COLS: usize = 7;
pub const COL_NAMES: [&str; NUM_COLS] = [
    "PID", "Name", "CPU%", "Memory", "Status", "Threads", "Command",
];
const DEFAULT_COL_WIDTHS: [u16; NUM_COLS] = [8, 20, 8, 12, 12, 8, 40];
const MIN_COL_WIDTH: u16 = 4;
const MAX_COL_WIDTH: u16 = 80;

// ── Application State ────────────────────────────────────────────────────────

pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,

    // System handle
    pub sys: System,

    // Process list
    pub processes: Vec<ProcessInfo>,
    pub process_scroll: usize,
    pub sort_col: SortColumn,
    pub sort_asc: bool,
    pub filter_text: String,

    // Horizontal scroll & column widths
    pub h_scroll: usize,
    pub col_widths: [u16; NUM_COLS],
    pub active_col: usize,
    pub col_resize_mode: bool,

    // Detail view
    pub selected_pid: Option<u32>,
    pub detail: Option<ProcessDetail>,
    pub detail_maps: Vec<MemoryRegion>,
    pub detail_scroll: usize,

    // String extraction
    pub strings: Vec<StringMatch>,
    pub strings_scroll: usize,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
    pub search_match_idx: usize,
    pub min_string_len: usize,
    pub extracting: bool,

    // Monitor
    pub monitor: ProcessMonitor,
    pub monitor_events: Vec<NewProcessEvent>,
    pub monitor_scroll: usize,
    pub monitored_pids: Vec<u32>,
    pub monitor_started: Instant,

    // Memory Inspector
    pub mem_pid: u32,
    pub mem_address: u64,
    pub mem_data: Vec<u8>,

    // Input mode
    pub input_target: InputTarget,
    pub input_buf: String,

    // Status message
    pub status_msg: String,
}

impl App {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let processes = process::list_user_processes(&sys);
        let monitor = ProcessMonitor::new();

        Self {
            mode: AppMode::ProcessList,
            should_quit: false,

            sys,

            processes,
            process_scroll: 0,
            sort_col: SortColumn::Pid,
            sort_asc: true,
            filter_text: String::new(),

            h_scroll: 0,
            col_widths: DEFAULT_COL_WIDTHS,
            active_col: 0,
            col_resize_mode: false,

            selected_pid: None,
            detail: None,
            detail_maps: Vec::new(),
            detail_scroll: 0,

            strings: Vec::new(),
            strings_scroll: 0,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            search_match_idx: 0,
            min_string_len: 4,
            extracting: false,

            monitor,
            monitor_events: Vec::new(),
            monitor_scroll: 0,
            monitored_pids: Vec::new(),
            monitor_started: Instant::now(),

            mem_pid: 0,
            mem_address: 0,
            mem_data: Vec::new(),

            input_target: InputTarget::None,
            input_buf: String::new(),

            status_msg: String::new(),
        }
    }

    // ── Tick (called every ~200ms) ───────────────────────────────────────────

    pub fn tick(&mut self) {
        if self.mode == AppMode::MonitorMode {
            let events = self.monitor.poll();
            for ev in events {
                self.monitor_events.push(ev);
            }
        }
    }

    // ── Refresh process list ─────────────────────────────────────────────────

    pub fn refresh_processes(&mut self) {
        self.sys.refresh_all();
        self.processes = process::list_user_processes(&self.sys);
        self.apply_sort();
        self.status_msg = format!("Refreshed — {} processes", self.processes.len());
    }

    // ── Get filtered process list ────────────────────────────────────────────

    pub fn filtered_processes(&self) -> Vec<&ProcessInfo> {
        if self.filter_text.is_empty() {
            self.processes.iter().collect()
        } else {
            let q = &self.filter_text;
            let mut scored: Vec<(i64, &ProcessInfo)> = self
                .processes
                .iter()
                .filter_map(|p| {
                    let pid_str = p.pid.to_string();
                    let fields = [p.name.as_str(), p.cmd.as_str(), pid_str.as_str()];
                    fuzzy::fuzzy_match_multi(q, &fields).map(|(score, _)| (score, p))
                })
                .collect();
            // Sort by score descending (best match first)
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            scored.into_iter().map(|(_, p)| p).collect()
        }
    }

    // ── Sorting ──────────────────────────────────────────────────────────────

    pub fn apply_sort(&mut self) {
        let asc = self.sort_asc;
        match self.sort_col {
            SortColumn::Pid => self.processes.sort_by(|a, b| {
                if asc {
                    a.pid.cmp(&b.pid)
                } else {
                    b.pid.cmp(&a.pid)
                }
            }),
            SortColumn::Name => self.processes.sort_by(|a, b| {
                let ord = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if asc {
                    ord
                } else {
                    ord.reverse()
                }
            }),
            SortColumn::Cpu => self.processes.sort_by(|a, b| {
                let ord = a
                    .cpu_pct
                    .partial_cmp(&b.cpu_pct)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if asc {
                    ord
                } else {
                    ord.reverse()
                }
            }),
            SortColumn::Mem => self.processes.sort_by(|a, b| {
                let ord = a.mem_bytes.cmp(&b.mem_bytes);
                if asc {
                    ord
                } else {
                    ord.reverse()
                }
            }),
            SortColumn::Status => self.processes.sort_by(|a, b| {
                let ord = a.status.cmp(&b.status);
                if asc {
                    ord
                } else {
                    ord.reverse()
                }
            }),
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_col = match self.sort_col {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Mem,
            SortColumn::Mem => SortColumn::Status,
            SortColumn::Status => SortColumn::Pid,
        };
        self.apply_sort();
        self.status_msg = format!(
            "Sorted by {:?} {}",
            self.sort_col,
            if self.sort_asc { "↑" } else { "↓" }
        );
    }

    pub fn toggle_sort_dir(&mut self) {
        self.sort_asc = !self.sort_asc;
        self.apply_sort();
    }

    // ── Process selection ────────────────────────────────────────────────────

    pub fn select_process(&mut self) {
        let filtered = self.filtered_processes();
        if let Some(p) = filtered.get(self.process_scroll) {
            let pid = p.pid;
            self.selected_pid = Some(pid);
            self.sys.refresh_all();
            self.detail = process::get_process_detail(&self.sys, pid);
            self.detail_maps = memory::parse_maps(pid);
            self.detail_scroll = 0;
            self.mode = AppMode::ProcessDetail;
            self.status_msg = format!("Viewing PID {}", pid);
        }
    }

    // ── String extraction ────────────────────────────────────────────────────

    pub fn extract_strings(&mut self) {
        if let Some(pid) = self.selected_pid {
            self.extracting = true;
            self.status_msg = format!(
                "Extracting strings from PID {} (min len {})…",
                pid, self.min_string_len
            );
            self.strings = memory::extract_strings(pid, self.min_string_len);
            self.strings_scroll = 0;
            self.search_query.clear();
            self.filtered_indices = (0..self.strings.len()).collect();
            self.search_match_idx = 0;
            self.extracting = false;
            self.mode = AppMode::StringView;
            self.status_msg = format!("{} strings extracted from PID {}", self.strings.len(), pid);
        }
    }

    // ── String search ────────────────────────────────────────────────────────

    pub fn apply_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.strings.len()).collect();
        } else {
            let q = &self.search_query;
            let mut scored: Vec<(i64, usize)> = self
                .strings
                .iter()
                .enumerate()
                .filter_map(|(i, s)| fuzzy::fuzzy_match(q, &s.value).map(|(score, _)| (score, i)))
                .collect();
            // Sort by score descending (best match first)
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered_indices = scored.into_iter().map(|(_, i)| i).collect();
        }
        self.search_match_idx = 0;
        self.strings_scroll = 0;
        self.status_msg = format!(
            "{} fuzzy matches for \"{}\"",
            self.filtered_indices.len(),
            self.search_query
        );
    }

    pub fn next_match(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.search_match_idx = (self.search_match_idx + 1) % self.filtered_indices.len();
            self.strings_scroll = self.search_match_idx;
        }
    }

    pub fn prev_match(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.search_match_idx == 0 {
                self.search_match_idx = self.filtered_indices.len() - 1;
            } else {
                self.search_match_idx -= 1;
            }
            self.strings_scroll = self.search_match_idx;
        }
    }

    // ── Monitor mode ─────────────────────────────────────────────────────────

    pub fn enter_monitor(&mut self) {
        self.monitor = ProcessMonitor::new();
        self.monitor_events.clear();
        self.monitor_scroll = 0;
        self.monitor_started = Instant::now();
        self.mode = AppMode::MonitorMode;
        self.status_msg = "Monitor mode — launch apps to detect new processes".into();
    }

    pub fn monitor_select(&mut self) {
        if let Some(ev) = self.monitor_events.get(self.monitor_scroll) {
            let pid = ev.info.pid;
            self.selected_pid = Some(pid);
            self.sys.refresh_all();
            self.detail = process::get_process_detail(&self.sys, pid);
            self.detail_maps = memory::parse_maps(pid);
            self.detail_scroll = 0;
            self.mode = AppMode::ProcessDetail;
            self.status_msg = format!("Viewing monitored PID {}", pid);
        }
    }

    // ── Input handling ───────────────────────────────────────────────────────

    pub fn start_input(&mut self, target: InputTarget) {
        self.input_target = target;
        self.input_buf.clear();
    }

    pub fn finish_input(&mut self) {
        match self.input_target {
            InputTarget::Filter => {
                self.filter_text = self.input_buf.clone();
                self.process_scroll = 0;
                let count = self.filtered_processes().len();
                self.status_msg = format!("Filter: \"{}\" — {} matches", self.filter_text, count);
            }
            InputTarget::Search => {
                self.search_query = self.input_buf.clone();
                self.apply_search();
            }
            InputTarget::MinLen => {
                if let Ok(n) = self.input_buf.parse::<usize>() {
                    if n > 0 {
                        self.min_string_len = n;
                        self.status_msg = format!("Min string length set to {}", n);
                    }
                }
            }
            InputTarget::Address => {
                self.goto_address(&self.input_buf.clone());
            }
            InputTarget::None => {}
        }
        self.input_target = InputTarget::None;
        self.input_buf.clear();
    }

    pub fn cancel_input(&mut self) {
        self.input_target = InputTarget::None;
        self.input_buf.clear();
    }

    // ── Navigation helpers ───────────────────────────────────────────────────

    pub fn scroll_up(&mut self) {
        match self.mode {
            AppMode::ProcessList => {
                self.process_scroll = self.process_scroll.saturating_sub(1);
            }
            AppMode::ProcessDetail => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            AppMode::StringView => {
                self.strings_scroll = self.strings_scroll.saturating_sub(1);
            }
            AppMode::MonitorMode => {
                self.monitor_scroll = self.monitor_scroll.saturating_sub(1);
            }
            AppMode::MemoryInspect => {
                self.memory_scroll_up();
            }
        }
    }

    pub fn scroll_down(&mut self) {
        match self.mode {
            AppMode::ProcessList => {
                let max = self.filtered_processes().len().saturating_sub(1);
                if self.process_scroll < max {
                    self.process_scroll += 1;
                }
            }
            AppMode::ProcessDetail => {
                let max = self.detail_maps.len().saturating_sub(1);
                if self.detail_scroll < max {
                    self.detail_scroll += 1;
                }
            }
            AppMode::StringView => {
                let max = self.filtered_indices.len().saturating_sub(1);
                if self.strings_scroll < max {
                    self.strings_scroll += 1;
                }
            }
            AppMode::MonitorMode => {
                let max = self.monitor_events.len().saturating_sub(1);
                if self.monitor_scroll < max {
                    self.monitor_scroll += 1;
                }
            }
            AppMode::MemoryInspect => {
                self.memory_scroll_down();
            }
        }
    }

    pub fn page_up(&mut self) {
        for _ in 0..20 {
            self.scroll_up();
        }
    }

    pub fn page_down(&mut self) {
        for _ in 0..20 {
            self.scroll_down();
        }
    }

    pub fn go_back(&mut self) {
        match self.mode {
            AppMode::StringView => {
                self.mode = AppMode::ProcessDetail;
            }
            AppMode::ProcessDetail => {
                self.mode = AppMode::ProcessList;
            }
            AppMode::MonitorMode => {
                self.mode = AppMode::ProcessList;
                self.refresh_processes();
            }
            AppMode::ProcessList => {
                if self.col_resize_mode {
                    self.col_resize_mode = false;
                    self.status_msg = "Column resize mode off".into();
                }
            }
            AppMode::MemoryInspect => {
                // Return to where we came from?
                // Usually detail view or string view.
                // For simplicity, back to detail if pid matches selected, else list?
                if let Some(pid) = self.selected_pid {
                    if pid == self.mem_pid {
                        self.mode = AppMode::ProcessDetail;
                    } else {
                        self.mode = AppMode::ProcessList;
                    }
                } else {
                    self.mode = AppMode::ProcessList;
                }
                self.h_scroll = 0;
            }
        }
    }

    // ── Memory Inspector ─────────────────────────────────────────────────────

    pub fn enter_memory_inspect(&mut self, pid: u32, address: u64) {
        self.mem_pid = pid;
        self.mem_address = address;
        self.refresh_memory_view();
        self.mode = AppMode::MemoryInspect;
        self.h_scroll = 0;
        self.status_msg = format!("Inspecting memory of PID {}", pid);
    }

    pub fn refresh_memory_view(&mut self) {
        // Read enough bytes to fill a typical screen (e.g. 30 rows * 16 bytes = 480)
        let size = 1024;
        self.mem_data = memory::read_memory(self.mem_pid, self.mem_address, size);
    }

    pub fn memory_scroll_up(&mut self) {
        self.mem_address = self.mem_address.saturating_sub(16);
        self.refresh_memory_view();
    }

    pub fn memory_scroll_down(&mut self) {
        self.mem_address = self.mem_address.saturating_add(16);
        self.refresh_memory_view();
    }

    pub fn memory_page_up(&mut self) {
        self.mem_address = self.mem_address.saturating_sub(16 * 20);
        self.refresh_memory_view();
    }

    pub fn memory_page_down(&mut self) {
        self.mem_address = self.mem_address.saturating_add(16 * 20);
        self.refresh_memory_view();
    }

    pub fn goto_address(&mut self, addr_str: &str) {
        // Parse hex or decimal
        let clean = addr_str.trim_start_matches("0x");
        if let Ok(addr) = u64::from_str_radix(clean, 16) {
            self.mem_address = addr;
            self.refresh_memory_view();
            self.status_msg = format!("Jumped to 0x{:x}", addr);
        } else if let Ok(addr) = addr_str.parse::<u64>() {
            self.mem_address = addr;
            self.refresh_memory_view();
            self.status_msg = format!("Jumped to {}", addr);
        } else {
            self.status_msg = "Invalid address format".into();
        }
    }

    // ── Horizontal scroll ────────────────────────────────────────────────────

    pub fn scroll_left(&mut self) {
        self.h_scroll = self.h_scroll.saturating_sub(4);
    }

    pub fn scroll_right(&mut self) {
        self.h_scroll = self.h_scroll.saturating_add(4);
    }

    // ── Column resize ────────────────────────────────────────────────────────

    pub fn toggle_col_resize(&mut self) {
        self.col_resize_mode = !self.col_resize_mode;
        if self.col_resize_mode {
            self.status_msg = format!(
                "Column resize: Tab=cycle  +/-=resize  active={}",
                COL_NAMES[self.active_col]
            );
        } else {
            self.status_msg = "Column resize mode off".into();
        }
    }

    pub fn cycle_active_col(&mut self) {
        self.active_col = (self.active_col + 1) % NUM_COLS;
        self.status_msg = format!("Active column: {}", COL_NAMES[self.active_col]);
    }

    pub fn widen_col(&mut self) {
        let w = &mut self.col_widths[self.active_col];
        *w = (*w + 2).min(MAX_COL_WIDTH);
    }

    pub fn narrow_col(&mut self) {
        let w = &mut self.col_widths[self.active_col];
        *w = w.saturating_sub(2).max(MIN_COL_WIDTH);
    }
}
