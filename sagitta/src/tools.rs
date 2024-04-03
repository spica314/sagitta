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
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
