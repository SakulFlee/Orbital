use serde::{Deserialize, Serialize};
use winit::{event_loop::EventLoop, window::Fullscreen};

use super::config_monitor::ConfigMonitor;

#[derive(Serialize, Deserialize)]
pub enum WrapperFullscreen {
    Borderless = 0,
    Exclusive = 1,
}

impl WrapperFullscreen {
    pub fn to_winit_fullscreen(
        &self,
        event_loop: &EventLoop<()>,
        monitor_config: &ConfigMonitor,
    ) -> Fullscreen {
        // Try finding the last monitor with that position
        let monitor_position = monitor_config.to_physical_position();
        let monitor_handle = event_loop
            .available_monitors()
            .into_iter()
            .find(|x| x.position() == monitor_position)
            .unwrap_or_else(|| {
                // If the last monitor can't be found, return the primary one
                event_loop
                    .primary_monitor()
                    .expect("no primary monitor found!")
            });

        // Try finding a matching video mode (size & refresh rate)
        let monitor_size = monitor_config.to_physical_size();
        let matching_video_mode = monitor_handle
            .video_modes()
            .find(|x| {
                x.refresh_rate_millihertz() == monitor_config.refresh_rate
                    && x.size() == monitor_size
            })
            .unwrap_or(
                monitor_handle
                    .video_modes()
                    .next()
                    .expect("no video modes found"),
            );

        match &monitor_config.fullscreen {
            WrapperFullscreen::Borderless => Fullscreen::Borderless(Some(monitor_handle)),
            WrapperFullscreen::Exclusive => Fullscreen::Exclusive(matching_video_mode),
        }
    }
}
