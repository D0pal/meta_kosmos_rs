use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_current_ts() -> Duration {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH).expect("Time went backwards")
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_current_ts_in_second() {
        let ts = get_current_ts();
        assert_eq!(ts.as_secs().to_string().len(), 10);
        assert_eq!(ts.as_millis().to_string().len(), 13);
    }
}
