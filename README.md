<div align="center">

<img src=".github/logo/Orbital.svg" alt="Orbital Engine Logo" width="200"/>

# 🛰️ Orbital Engine & Framework


![](https://img.shields.io/badge/Forgejo-Origin-orange?style=for-the-badge&logo=forgejo&logoSize=auto&link=https%3A%2F%2Fforgejo.sakul-flee.de%2FSakulFlee%2FOrbital%2F)
![](https://img.shields.io/badge/GitHub-Mirror-blue?style=for-the-badge&logo=github&logoSize=auto&link=https%3A%2F%2Fgithub.com%2FSakulFlee%2FOrbital)

_Orbital_ is a cross-platform real-time graphical rendering engine & framework written in Rust.
The goal of this project is to bridge well established industry-standard techniques with upcoming (next-gen) and experimental techniques to form a robust new foundation for next-gen visual computing.

</div>

## Showcase

| glTF demo: Damaged Helmet| |
| - | - |
| ![A](Images/DamagedHelmet.png) | |
| PBR Grid: Spheres | PBR Grid: Cubes |
| ![A](Images/PBRGridSpheres.png) | ![A](Images/PBRGridCubes.png) |

## Core Features 

<div align="center">

| Feature | Description |
|--------|-------------|
| **PBR Rendering** | Physically Based Rendering with realistic material properties and lighting |
| **Image-Based Lighting** | Advanced lighting using HDR environment maps for realistic reflections |
| **GLTF 2.0 Support** | Full import capabilities for complex 3D scenes, materials, and animations |
| **Cross-Platform** | Unified rendering across desktop platforms via WGPU abstraction |
| **Modular Architecture** | Flexible element system with message-based communication |
| **Modern Shaders** | WGSL-based shaders for cross-platform compatibility |

</div>

## Experiments

<div align="center">

| Experimental Concept | Description |
| - | - |
| HJAA (Halton-Jitter Anti-Aliasing) | Advanced spatial smoothing utilizing the [Halton Sequence](https://en.wikipedia.org/wiki/Halton_sequence) |
| IVXGI (Incremental Voxel-based Global Illumination) | A evolution of [Nvidia's VXGI](https://developer.download.nvidia.com/assets/events/GDC15/GEFORCE/VXGI_Dynamic_Global_Illumination_GDC15.pdf) for high performance dynamic lighting. |

</div>

> [!IMPORTANT]
> Documentation for experimental features will be released once they have been fully validated in practical, non-theoretical environments.

## Platform Compatibility

- ✅ **Windows**: Full support
- ✅ **Linux**: Full support
- ✅ **macOS**: Full support
- ⚠️ **Web**: Theoretically possible but not currently supported
- ⚠️ **Android**: Works but requires frequent updates to keep compatible
- ⚠️ **iOS**: Compiles but untested — boilerplate provided, community testing welcome

## Getting Started

The official project templates live in the `Examples/` directory. They are
workspace members and mirror the templates used by `orbital init`:

- **minimal**: A small procedural scene with a camera and a spotlight
- **all-in-one**: Procedural scene, glTF import, lighting, shadows, UI, and overlays
- **2d**: A minimal 2D shape rendering example

The examples are self-contained and can be run from their own directories:

```shell
(cd Examples/minimal && cargo run)
(cd Examples/all-in-one && cargo run)
(cd Examples/2d && cargo run)
```

They can also be run from the repository root:

```shell
cargo run -p template_minimal
cargo run -p template_all_in_one
cargo run -p template_2d
```

`minimal` and `2d` need no asset files. `all-in-one` ships the scene and helmet
model it uses in `Examples/all-in-one/Assets/`.

To create a standalone project from one of these templates, use the Orbital CLI:

```shell
orbital init my-game --template all-in-one
```

To use the engine in your own project, add the following to your `Cargo.toml`:

```toml
orbital = { path = "./Runtime" }
```

## Engine Concepts

### Elements

The primary way to add interactive objects to your world is through **Elements**. Elements are the core game objects that exist in the world and handle their own behavior. Each element can register itself with the world to define what resources it needs (models, cameras, etc.), and can respond to various events during the application lifecycle such as updates and messages from other elements.

### Messaging System

Elements communicate with each other and the world through a message-passing system rather than shared memory. This ensures loose coupling between elements and enables flexible, modular game design. Rather than sharing memory or directly accessing other elements, elements send messages to communicate. This approach is easily scalable - tags can be used multiple times, so messages will be automatically sent to all elements with a matching tag.

### World Architecture

The world manages resource stores (models, cameras, environments, lights), handles the asset import system, and creates the global bind group used by shaders. Resources follow a specific lifecycle involving creation, realization, caching, and cleanup.

## History

The Orbital engine has a rich history spanning multiple iterations and rewrites. The project originally started as "Akimo" and was written in Java as a pure CPU renderer for 2D graphics. Over the years, it evolved through multiple iterations:

- **Akimo (Java)**: Started as a CPU-based 2D renderer, then upgraded to use LWJGL for OpenGL and Vulkan support
- **Akimo (C++)**: A complete rewrite in C++ with Vulkan bindings, still under the "Akimo" name
- **Orbital (Rust/Vulkan)**: A rewrite in Rust but still using Vulkan bindings
- **Orbital (Rust/WGPU/WGSL)**: The current iteration using Rust with WGPU and WGSL, moving away from GLSL to a more modern shader language

Each version learned from the previous ones, with the current implementation built in Rust leveraging the WGPU graphics API abstraction layer for cross-platform support.

## Contributing

[Forgejo](https://forgejo.sakul-flee.de/SakulFlee/Orbital) is the canonical repository; GitHub is a mirror.
Please open pull requests on Forgejo.
If you can only use GitHub, treat a GitHub pull request as a proposal that will be ported to Forgejo before merge.

AI tools are welcome, but the contributor owns the result:

- Inspect and understand every line you submit.
- Verify that it builds and that relevant tests, lints, and examples pass.
- Do not add AI/model attribution to code, commits, documentation, or pull-request metadata. AI tools are not authors, co-authors, owners, or contributors.
- The person opening the pull request is responsible for correctness, licensing, security, and follow-up maintenance.

See [AGENTS.md](AGENTS.md) for the full contribution contract, architecture map, tooling, and pre-PR checklist.

## License

This project is dual licensed in Rust's fashion:

- [MIT License](https://spdx.org/licenses/MIT.html)
- [Apache License 2.0](https://spdx.org/licenses/Apache-2.0.html)

For your own project you can chose whichever fits you better.
For templates/examples we recommend to also dual-licensing.

> We highly encourage everyone to share their sub-crates with the community so that others can benefit from it too!

[Orbital]: https://github.com/SakulFlee/Orbital
[WGPU]: https://wgpu.rs/
