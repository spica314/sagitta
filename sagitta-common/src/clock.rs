#[derive(Debug, Clone)]
pub struct Clock {
    pub fixed_time: Option<std::time::SystemTime>,
}

impl Clock {
    pub fn new() -> Self {
        Self { fixed_time: None }
    }

    pub fn new_with_fixed_time(fixed_time: std::time::SystemTime) -> Self {
        Self {
            fixed_time: Some(fixed_time),
        }
    }

    pub fn now(&self) -> std::time::SystemTime {
        self.fixed_time.unwrap_or_else(std::time::SystemTime::now)
    }

    pub fn is_fixed(&self) -> bool {
        self.fixed_time.is_some()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock() {
        let clock = Clock::default();
        let now = clock.now();
        assert!(now.elapsed().unwrap().as_secs() < 1);
    }

    #[test]
    fn test_clock_with_fixed_time() {
        let fixed_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(42);
        let clock = Clock::new_with_fixed_time(fixed_time);
        let now = clock.now();
        assert_eq!(now, fixed_time);
    }
}
