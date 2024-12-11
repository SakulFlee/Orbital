use orbital::{
    app::{AppRuntime, AppSettings},
    logging,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

#[cfg(feature = "non_caching_direct_renderer")]
use orbital::renderer::NonCachingDirectRenderer;

#[cfg(feature = "caching_direct_renderer")]
use orbital::renderer::CachingDirectRenderer;

use crate::app::{CacheSettings, MyApp};

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = false;

    #[cfg(all(
        feature = "caching_direct_renderer",
        feature = "non_caching_direct_renderer"
    ))]
    compile_error!("Cannot enable both caching and non-caching renderers at the same time!");
    #[cfg(all(
        feature = "non_caching_direct_renderer",
        not(feature = "caching_direct_renderer")
    ))]
    let app =
        MyApp::<NonCachingDirectRenderer>::new(CacheSettings::default(), CacheSettings::default());
    #[cfg(all(
        feature = "caching_direct_renderer",
        not(feature = "non_caching_direct_renderer")
    ))]
    let app =
        MyApp::<CachingDirectRenderer>::new(CacheSettings::default(), CacheSettings::default());

    AppRuntime::liftoff(event_loop, app_settings, app).expect("Runtime failure");
}
