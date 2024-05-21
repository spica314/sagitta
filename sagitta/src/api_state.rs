use sagitta_common::clock::Clock;

#[derive(Clone)]
pub struct ApiState {
    pub clock: Clock,
}

impl ApiState {
    pub async fn new(clock: Clock) -> Self {
        Self { clock }
    }
}
