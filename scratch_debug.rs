fn main() {
    let track_str = "[behind 13]";
    if let Some(b) = track_str.split("behind ").nth(1).and_then(|s| s.split(|c: char| !c.is_numeric()).next()) {
        println!("behind: {}", b.parse::<usize>().unwrap_or(0));
    }
}
