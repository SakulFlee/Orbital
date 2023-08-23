use serde::{Deserialize, Serialize};

use super::wrapper_backend::WrapperBackend;

#[derive(Serialize, Deserialize)]
pub struct ConfigAdapter {
    device_id: u32,
    backend: WrapperBackend
}
