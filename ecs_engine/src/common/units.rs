pub const fn bytes(bytes: usize) -> usize {
    bytes
}

pub const fn kilobytes(bytes: usize) -> usize {
    bytes * 1024
}

pub const fn megabytes(bytes: usize) -> usize {
    kilobytes(bytes) * 1024
}

pub const fn gigabytes(bytes: usize) -> usize {
    megabytes(bytes) * 1024
}

#[cfg(debug_assertions)]
pub fn format_bytes_pretty(bytes: usize) -> String {
    if bytes < kilobytes(1) {
        format!("{} B", bytes)
    } else if bytes < megabytes(1) {
        format!("{:.2} KB", bytes as f64 / 1024.)
    } else if bytes < gigabytes(1) {
        format!("{:.2} MB", bytes as f64 / (1024 * 1024) as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / (1024 * 1024 * 1024) as f64)
    }
}
