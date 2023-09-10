# WGPU-Engine

A Engine & Framework, possibly game at some point, written in [WGPU] to learn more about it and comparing it to [Vulkan] :)

## Structure

This project is split into three main crates and will, potentially at some later point, be ported into it's own repositories.

### Engine

The engine crate is basically the heart and bones of this project.  
It includes everything needed to work with [WGPU] and provides a way of interacting with the GPU.

### Framework

The framework crate is the brain of this project.  
It includes optional additions _on top_ of the Engine to make working with the Engine easier.

Such additions include a way of easier rendering and organizing of objects in a given scene.

> Other frameworks commonly call this "World".

### bin_*

All `bin_*` targets are basically "Games".  
Each binary defines what objects should be added to a scene, how stuff is handled and most importantly: run the application on the given platform.

For some platforms there is a universal binary (such as desktop platforms), but especially for more specific platforms like mobile devices or consoles, there is a special binary target.

[WGPU]: https://wgpu.rs/
[Vulkan]: https://www.vulkan.org/
