pub const fn kilobytes(bytes: usize) -> usize {
    bytes * 1024
}

pub const fn megabytes(bytes: usize) -> usize {
    kilobytes(bytes) * 1024
}

pub const fn gigabytes(bytes: usize) -> usize {
    megabytes(bytes) * 1024
}
