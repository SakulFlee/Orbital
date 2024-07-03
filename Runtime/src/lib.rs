#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/logo/Orbital.svg"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/logo/Orbital.png"
)]
#![doc(html_playground_url = "https://play.rust-lang.org")]

//! ![Orbital Logo](https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/logo/Orbital_Logo.png)
//!
//! The Orbital Runtime & Engine is a multi-layer project.  
//! It is mainly intended to be used to make [Games].  
//! However, you can also use it to make [Apps] of any kind.
//!
//! The main goal of a [Game] implementation is to provide as much simplicity
//! as physically possible.
//! This way, the end user (i.e. _YOU_!) can fully focus on making a [Game],
//! rather than learning hundreds of different tools and how to do X in Engine
//! Y efficiently.
//! All optimizations are enabled by default and will be run without the user
//! even knowing.
//!
//! However, some don't need this much optimization or want to do
//! it themselves.  
//! For those, we also provide a more rudimentary [App] trait which
//! allows you to make exactly what you need to without too much
//! background automation & optimization.
//!
//! Individual parts of this project are designed in a way where they should be
//! fully interchangeable with another part, or even your own implementation of
//! said part.
//! This should give _you_ the most flexibility when using this project.
//!
//! # Getting Started
//!
//! To get started, you will need to choose a layer.  
//! Like explained above, we have a [Game] and [App] layer,
//! but there is more:
//!
//! ![Diagram: Managed vs. Unmanaged](https://raw.githubusercontent.com/SakulFlee/Akimo-Project/ec00739b23e5aaca8a00b53c6bf099f2e423d057/.github/images/diagram_managed_vs_unmanaged.drawio.png)
//!
//! At the bottom, we have the _Framework_.
//! This is the most **DIY** (_do it yourself_) layer and approach.
//! Most components of the project are interchangeable and useable outside
//! of the runtime(s).
//! You will have to do many things yourself, but are also fully in control
//! over everything.
//! We call this _Framework_ as it gives you the building blocks to build
//! your own stuff!
//!
//! Next, there is the [App Runtime].
//! This is intended to be used by [Apps].
//! This layer gives you a much easier to use entrypoint
//! and will expose certain things like a GPU [Device] & [Queue]
//! to you for usage.
//! But, similar to the _Framework_, you still have to take
//! care of many things yourself.
//! > ⚠️ Especially if you are planning on making a [Game]!
//!
//! Now that we have mentioned [Games], the [Game Runtime]
//! is intended for this.
//! It builds **on-top** of the [App Runtime] and handles
//! many things for you like:
//! - Providing a [World]
//! - Automated rendering of [Elements]
//! - Messages (sharing information)
//! - Input Management
//! - and many more!
//!
//! This should be the simplest approach to making a [Game].
//! > Do note, that this _can also_ be used to make [Apps]
//! > of course.
//! > Nothing is telling you to do either or the other.
//! > Point being, that the [App Runtime] is much simpler
//! > compared to the [Game Runtime].
//!
//! Lastly, there is the [Game] _Modules_ layer.
//! This approach is probably closest to what you would expect
//! from a _Game Engine_.
//! More or less everything is handled for you.
//! All you do is define [Elements] for your [World].
//!
//! Once a layer is chosen, you should continue reading at the corresponding
//! pages.
//! In most cases: [App] & [AppRuntime], or, [Game] & [GameRuntime]!  
//! However, you should check out the following section about platform support,
//! as some platforms require a special setup.
//!  
//! # Platforms
//!
//! > This section is only relevant if you are using a **Runtime**!  
//! > I.e. [AppRuntime] or [GameRuntime].
//!
//! The Orbital Runtime & Engine aims to be as multi-platform as possible.  
//! The [App]/[Game] traits are virtually already multi-platform friendly
//! wrappers.
//! You can write your code once and run it technically anywhere!
//!
//! However, there is one obstacle!
//! [Winit], the windowing library being used, requires full control over the
//! window being used.
//! This is mainly to provide all those nice events we want to get from it.
//! It achieves this by requiring you to make an [EventLoop].  
//! Depending on the target platform choice, this is done differently.  
//! Additionally, some platforms require a more advanced setup to get going.
//!
//! > You can also use the Example project found inside the GitHub repository
//! > as a starting point. It is meant to work equally on **all** platforms
//! > and thus also makes use of any platform specific difference.
//!
//! ## Desktop Platforms (Linux, Windows, macOS)
//!
//! Desktop platforms are the easiest!  
//! Simply make a new binary Rust project and add Orbital as a dependency.
//!
//! In your main function you will need to acquire an [EventLoop].  
//! This can be done like so:
//!
//! ```rust
//! let event_loop = EventLoop::builder().build().unwrap();
//! ```
//!
//! Now, just call the **Runtime** with the acquired [EventLoop]!
//!
//! ## Android
//! For Android you will need to use some kind of wrapper that initializes the
//! Android App, calls our Rust code and passes through events.
//!
//! **Normally**, I would recommend using [Cargo-APK] for this task.
//! It automates basically everything, builds your code for the correct
//! architectures and packs everything into an APK ready to be used on Android.  
//! **⚠️ However, this tool has been deprecated and isn't receiving updates
//! anymore.**
//! Supposedly, it is being replaced by [xBuild], but that tool seems to be
//! not receiving updates **either**.
//! Nor, are issues and PRs being handled.  
//! Unfortunately, for those reasons **I cannot recommend either tool**.
//!
//! However! It isn't that complicated to make this yourself.  
//! The background tools both [Cargo-APK] and [xBuild] are using **are working
//! just fine and are receiving updates still**.  
//! The main tool in question is [Cargo-NDK] which compiles our [App] into the
//! given architectures, ready to be bundled.
//!
//! The only missing part is the Android App part itself.  
//! But that's also relatively easy to make!  
//! Android has support for a so called `NativeActivity` which gives you a
//! single `Activity`, which is fully controlled natively.
//! This is originally intended to be used with C++, but works with Rust too!
//! There is an example on their official NDK-Samples GitHub: [Android/NDK-Samples/Native-Activity](https://github.com/android/ndk-samples/tree/main/native-activity).
//!
//! All we need to change from there is removing the CMake plugin and C++ code,
//! and adding our own stuff like so:  
//! First of all, in `app/build.gradle` remove both `externalNativeBuild`
//! blocks.  
//! > You may also change the `namespace` and `applicationId` here.  
//! Next, under `app/src/main/AndroidManifest.xml` search for the
//! following line:
//! ```xml
//! <meta-data android:name="android.app.lib_name"
//!            android:value="native-activity" />
//! ```
//! The `value` (`native-activity`) here is the **name of the library build by
//! [Cargo-NDK] __without__ the `.so` part**!
//!
//! That's all the changes we have to do.  
//! However, you may also wanna change your icons (`app/src/main/res/*`)
//! and name (`app/src/main/res/values/strings.xml`).
//!
//! Android does expect the build `*.so` files to be in a specific location
//! (usually: `src/main/res/jniLibs/*`, but can change!) and you will have to
//! run two build commands every time: First the [Cargo-NDK] command, then
//! e.g. `gradle assembleDebug` to make the Android App.  
//! To make this process easier there is one last step we can
//! **optionally** do:  
//! There is a Gradle plugin that automatically calls [Cargo-NDK] for us, copies
//! the libraries in the correct location and then builds the Android App
//! for us.
//! This plugin is called [Cargo-NDK-Android-Gradle].  
//! All that needs to be changes is in your `build.gradle` you want to add the
//! plugin into the plugins block:
//!
//! ```gradle
//! plugins {
//!     id 'com.github.willir.rust.cargo-ndk-android'
//! }
//! ```
//!
//! and at the bottom of the `build.gradle` add the following:
//!
//! ```gradle
//! cargoNdk {
//!     targets = ["arm64", "arm", "x86", "x86_64"]
//! }
//! ```
//!
//! > ⚠️ There are many more parameters you can use to customize the build
//! > process further if needed. Check the GitHub documentation of
//! > [Cargo-NDK-Android-Gradle] for more!
//!
//! Simply calling e.g. `gradle assembleDebug` now will automate
//! everything for us!  
//! But, sometimes a Rust build can fail and, for whatever reason, the
//! App still seems to build.
//! To prevent this and your App not getting updated I would recommend also
//! adding the following line into your `clean` block (add it if it
//! doesn't exist yet):
//!
//! ```gradle
//! clean {
//!     delete android.sourceSets.main.getJniLibsDirectories()
//! }
//! ```
//!
//! This will clear out **only** the designated `jniLibs` folder which is where
//! Android expects us to put our compiled `.so` libraries into.  
//! This does NOT clean Rust's `target/` folder or anything else.
//!
//! Now, with [Cargo-NDK] you will have an Android entrypoint like so:
//!
//! ```rust
//! #[no_mangle]
//! fn android_main(app: AndroidApp) {
//!     let event_loop = EventLoop::builder()
//!         .with_android_app(app)
//!         .build()
//!         .unwrap();
//! }
//! ```
//!
//! > [Winit] basically takes over the `AndroidApp` and uses that instead
//! of making a new window.
//!
//! Now, just call the **Runtime** with the acquired [EventLoop]!
//!
//! ## ⚠️🚧 iOS
//!
//! iOS is supported on a technical level, but no specifics instructions
//! exist as of now.
//! This means, the Orbital Engine & Runtime should work, without issues, on iOS
//! like it does on any other platform.
//! However, like with Android, a special setup will be required to get the app
//! running.
//!
//! > Unfortunately, I don't have access to a mac and i-Device at the moment.
//! > Thus, I cannot test and provide further instructions here.  
//! > iOS should have a similar concept of a `NativeActivity` and should
//! > roughly work about the same as Android.
//! >
//! > If anyone wants to invest some time in validating that this actually
//! > works and can either give me a write-up on how to setup such a project
//! > for iOS, or, is able to share their macOS device temporarily, send me a
//! > message or open an issue on GitHub! ❤️
//!
//! ## ⚠️🚧 Web
//!
//! Web(site) support is limited at this moment.  
//! In theory, most components should work like on any other platform.  
//! However, there are two issues as of now:
//!
//! 1. There is no support for requesting files from a webserver at the moment.
//! Meaning, anywhere where you are trying to load a file from disk will fail.
//! If you use assets directly (e.g. by embedding them) this will work!
//! (e.g. instead of loading a model from a glTF file, you embed the vertices
//! and indices into the binary and construct the Mesh this way manually.)
//!
//! 2. Multi-Threading isn't possible on websites.
//! There are workarounds like using Async/Await and workers, but as
//! of now, nothing has been implemented in that regard.
//! As of now, the engine also doesn't multi-thread yet.  
//! However, dependencies used _may_ and we haven't fully validated that
//! everything is indeed working!
//!
//! Full web support is planned in the future and full instructions will be
//! updated here once this is done.
//! In the meantime, if you want to try anyways, here are the steps necessary
//! to get this working:
//!
//! 1. You will need a website. The simpler, the better for now.
//! A simple HTML website with minimal CSS would work best.
//!
//! 2. Either, create a `canvas` inside your HTML and tag it,
//! or, make use of crates like
//! [web-sys](https://docs.rs/web-sys/latest/web_sys/)
//! to spawn in a canvas.
//!
//! 3. **Optionally** use CSS to make the `canvas` as big as the viewport.
//!
//! 4. Create an [EventLoop] like under Android, but with the `canvas` as a
//! reference. Read more [here](https://docs.rs/winit/latest/winit/platform/web/index.html).
//!
//! 5. Make something awesome! ❤️
//!
//! [AppRuntime]: crate::app::AppRuntime
//! [App]: crate::app::App
//! [Apps]: crate::app::App
//! [GameRuntime]: crate::game::GameRuntime
//! [Game]: crate::game::Game
//! [Games]: crate::game::Game
//! [Device]: crate::wgpu::Device
//! [Queue]: crate::wgpu::Queue
//! [Elements]: crate::game::world::element::Element
//! [World]: crate::game::world::World
//! [Winit]: crate::winit
//! [EventLoop]: crate::winit::event_loop::EventLoop

// Modules
pub mod app;
pub mod cache;
pub mod error;
pub mod game;
pub mod logging;
pub mod renderer;
pub mod resources;
pub mod timer;
pub mod util;
pub mod variant;

// Re-exports
macro_rules! reexport {
    ($name: ident) => {
        pub mod $name {
            #[doc(hidden)]
            pub use $name::*;
        }
    };
}

reexport!(cgmath);
reexport!(hashbrown);
reexport!(log);
reexport!(ulid);
reexport!(wgpu);
reexport!(winit);
