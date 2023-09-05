#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdateFrequency {
    OnCycle,
    OnSecond,
}

impl UpdateFrequency {
    #[allow(non_upper_case_globals)]
    pub const OnFrame: UpdateFrequency = UpdateFrequency::OnCycle;
    #[allow(non_upper_case_globals)]
    pub const Slow: UpdateFrequency = UpdateFrequency::OnSecond;
    #[allow(non_upper_case_globals)]
    pub const Fast: UpdateFrequency = UpdateFrequency::OnCycle;
}

pub trait Updateable {
    fn update(&mut self, delta_time: f64);

    fn update_frequency(&self) -> UpdateFrequency;
}
