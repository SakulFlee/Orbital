#[derive(Debug)]
pub enum ChangeType {
    Model { label: Option<String> },
    Light { label: Option<String> },
    Camera { label: Option<String> },
    WorldEnvironment { label: Option<String> },
}
