use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};

/// One readable memory region from /proc/<pid>/maps.
#[derive(Clone, Debug)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub perms: String,
    pub offset: String,
    pub dev: String,
    pub inode: String,
    pub path: String,
}

/// A string found in process memory.
#[derive(Clone, Debug)]
pub struct StringMatch {
    pub value: String,
    pub region_idx: usize,
    pub offset_in_region: u64,
}

/// Maximum region size we'll read (256 MB).
const MAX_REGION_SIZE: u64 = 256 * 1024 * 1024;

/// Parse /proc/<pid>/maps into a list of memory regions.
pub fn parse_maps(pid: u32) -> Vec<MemoryRegion> {
    let maps_path = format!("/proc/{}/maps", pid);
    let content = match fs::read_to_string(&maps_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let addr_range = parts.next()?;
            let perms = parts.next()?.to_string();
            let offset = parts.next().unwrap_or("").to_string();
            let dev = parts.next().unwrap_or("").to_string();
            let inode = parts.next().unwrap_or("").to_string();
            let path = parts.collect::<Vec<_>>().join(" ");

            let (start_hex, end_hex) = addr_range.split_once('-')?;
            let start = u64::from_str_radix(start_hex, 16).ok()?;
            let end = u64::from_str_radix(end_hex, 16).ok()?;

            Some(MemoryRegion {
                start,
                end,
                perms,
                offset,
                dev,
                inode,
                path,
            })
        })
        .collect()
}

/// Extract printable ASCII strings from all readable regions of a process.
///
/// This reads `/proc/<pid>/mem` directly — a passive operation that does
/// NOT use ptrace, send signals, or influence the process in any way.
pub fn extract_strings(pid: u32, min_len: usize) -> Vec<StringMatch> {
    let regions = parse_maps(pid);
    let mem_path = format!("/proc/{}/mem", pid);

    let mut mem_file = match File::open(&mem_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();

    for (idx, region) in regions.iter().enumerate() {
        // Only read regions marked as readable
        if !region.perms.starts_with('r') {
            continue;
        }

        let size = region.end - region.start;
        if size == 0 || size > MAX_REGION_SIZE {
            continue;
        }

        // Seek to the start of this region
        if mem_file.seek(SeekFrom::Start(region.start)).is_err() {
            continue;
        }

        // Read the region — may partially fail on guard pages, that's fine
        let mut buf = vec![0u8; size as usize];
        let bytes_read = match mem_file.read(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };

        // Extract printable ASCII sequences
        extract_strings_from_buf(&buf[..bytes_read], min_len, idx, &mut results);
    }

    results
}

/// Pull printable ASCII runs from a buffer.
fn extract_strings_from_buf(
    buf: &[u8],
    min_len: usize,
    region_idx: usize,
    out: &mut Vec<StringMatch>,
) {
    let mut current = Vec::new();
    let mut start_offset: u64 = 0;

    for (i, &byte) in buf.iter().enumerate() {
        if byte >= 0x20 && byte < 0x7F {
            if current.is_empty() {
                start_offset = i as u64;
            }
            current.push(byte);
        } else {
            if current.len() >= min_len {
                if let Ok(s) = String::from_utf8(current.clone()) {
                    out.push(StringMatch {
                        value: s,
                        region_idx,
                        offset_in_region: start_offset,
                    });
                }
            }
            current.clear();
        }
    }
    // Flush remainder
    if current.len() >= min_len {
        if let Ok(s) = String::from_utf8(current) {
            out.push(StringMatch {
                value: s,
                region_idx,
                offset_in_region: start_offset,
            });
        }
    }
}

/// Read raw memory from a process at the given address.
pub fn read_memory(pid: u32, start: u64, size: usize) -> Vec<u8> {
    let mem_path = format!("/proc/{}/mem", pid);
    let mut f = match File::open(&mem_path) {
        Ok(f) => f,
        Err(_) => return vec![0u8; size], // Return zeroed buffer on error
    };

    if f.seek(SeekFrom::Start(start)).is_err() {
        return vec![0u8; size];
    }

    let mut buf = vec![0u8; size];
    match f.read(&mut buf) {
        Ok(n) => {
            // If we hit EOF or error, the rest of the buffer remains 0
            // We could resize, but keeping the requested size simplifies the UI
            buf.truncate(n);
            // Pad back to requested size with 0s if needed?
            // Better to return what we read + 0s for unreadable parts or just let UI handle it.
            // Let's pad with 0s to keep UI consistent.
            buf.resize(size, 0);
            buf
        }
        Err(_) => vec![0u8; size],
    }
}
