pub fn seconds_to_string(seconds: usize) -> String {
    let mut time_str = String::new();

    let hours = seconds / 3600;
    if hours > 0 {
        time_str += &format!("{}h", hours);
    }
    let minutes = seconds % 3600 / 60;
    if minutes > 0 {
        time_str += &format!("{}m", minutes);
    }
    let seconds = seconds % 60;
    if seconds > 0 {
        time_str += &format!("{}s", seconds);
    }

    time_str
}
