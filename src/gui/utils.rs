
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    } else if bytes < 1024_u64.pow(2) {
        return format!("{:.2} KB", bytes as f64 / 1024.0);
    } else if bytes < 1024_u64.pow(3) {
        return format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0);
    } else if bytes < 1024_u64.pow(4) {
        return format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0);
    } else if bytes < 1024_u64.pow(5) {
        return format!("{:.2} TB", bytes as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0);
    } else {
        return format!("{:.2} PB", bytes as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0 / 1024.0);
    }
}