#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/logo/Orbital.svg"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/logo/Orbital_Logo.png"
)]
#![doc(html_playground_url = "https://play.rust-lang.org")]

//! ![Orbital Logo](https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/logo/Orbital_Logo.png)
//! 
//! The Akimo-Project is a many faced project.  
//! It is mainly intended to be used to make [Games].  
//! However, you can also use it to make [Apps] of any kind.
//!
//! # Layer choice
//!
//! Depending on your needs, you can choose to use the project
//! as a framework (i.e. do more things yourself) or solely focus
//! on your project and use it as an engine (i.e. most things
//! are taken care for you!).
//!
//! Roughly speaking, there are four layers of usage:  
//! ![Diagram: Managed vs. Unmanaged](https://raw.githubusercontent.com/SakulFlee/Akimo-Project/main/.github/images/diagram_managed_vs_unmanaged.png)
//! Let's look at this from bottom to top.
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
//! # Getting Started
//!
//! # [App]/[Game] Implementation
//!
//! TODO
//!
//! # [Elements] & [World]
//!
//! TODO
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
pub mod app;
pub mod cache;
pub mod cgmath;
pub mod entity;
pub mod error;
pub mod event;
pub mod game;
pub mod log;
pub mod renderer;
pub mod resources;
pub mod timer;
pub mod util;
pub mod wgpu;
pub mod winit;
