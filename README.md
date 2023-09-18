# Akimo Engine

[![Multiplatform Build](https://github.com/Sakul6499/Akimo-Engine/actions/workflows/multiplatform-build.yml/badge.svg?branch=main)](https://github.com/Sakul6499/Akimo-Engine/actions/workflows/multiplatform-build.yml)

The _Akimo Engine_ is a multi-purpose rendering engine created by myself to make indie-games.  
We currently support 2D, as well as, 3D games, but this library can also be used for computational tasks.
The main goal of this project is to create an easy way of rendering, where a given developer only has to care about their _game world_ and nothing else.
However, the engine is also modularly build and highly customizable & extensible.

This engine is written fully in [Rust].
However, bindings for other languages are [planned](#planned-features).

_Akimo_ is a very old project I've been working on for years.  
If you are interested in finding out more checkout the [history](#history) section.

This project is supported by the following platforms:

- ✅ Platform: Windows
- ✅ Platform: Linux
- ❓ Platform: macOS
- ❓ Platform: Android
- ❓ Platform: iOS
- ❓ Platform: WebAssembly

To use this engine include the following into your `Cargo.toml`:

```toml
akimo_engine = {git = "https://github.com/Sakul6499/Akimo-Engine/fork", branch = "main"}
```

Alternatively, you can [fork] this repository and directly add your own sub-crates for your game!

## License

This project is dual licensed in Rust's fashion:

- [MIT License](https://spdx.org/licenses/MIT.html)
- [Apache License 2.0](https://spdx.org/licenses/Apache-2.0.html)

For your own project you can chose whichever fits you better.
For templates/examples we recommend to also dual-licensing.

> We highly encourage everyone to share their sub-crates with the community so that others can benefit from it too!

## Project layout

| Folder                                       | Description                                                                                                                                                                                                   |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ./                                           | Workspace root; `Cargo.toml` contains all project folders (internal crates)                                                                                                                                   |  |
| [crates/](crates/)                           | All sub crates life here.                                                                                                                                                                                     |
| [crates/akimo_engine/](crates/akimo_engine/) | The engine with all mandatory features included.                                                                                                                                                              |
| [crates/*/](crates/*/)                       | Any sub-crate that extends the Engine's functionality or offer structures to make things easier. **Some of these may depend on each other and may be included (and re-exported!) in the _Akimo Engine_ crate. |

To break this down:
You are most likely interested in the [crates/akimo_engine/](crates/akimo_engine/) folder.  
It contains the base engine which you want to use in your own projects.

Further additions and helpers may be included under [crates/*/](crates/*/).

## Planned features

> Ticked items are implemented.  
> Unticked items are not yet implemented.

- [ ] Basic rendering capabilities including full handling of:
  - [ ] Window
  - [ ] Render Backend
  - [ ] App World and Objects
- [ ] Config extension support
- [ ] Input handling
  - [ ] Keyboard
  - [ ] Mouse
  - [ ] Controller
- [ ] Platform support
  - [ ] Windows
  - [ ] Linux
  - [ ] macOS
  - [ ] Android
  - [ ] iOS
  - [ ] WebAssembly / WASM (Web)
- [ ] Language bindings
  - [ ] C#
  - [ ] C++
  - [ ] Java
  - [ ] JavaScript

## Targets & Architectures

This project is aiming to work across all platforms **and targets**.
All **Tier 1** targets are tested in CI's of this repository.
Additionally, _some_ **Tier 2** targets are tested.

However, this should work on all targets. If you find an issue please report it.

[Rust's Tier Policies](https://doc.rust-lang.org/rustc/target-tier-policy.html)
[Rust's Platform Support & Targets](https://doc.rust-lang.org/rustc/platform-support.html)

## Building & Running

**Building & Running all projects at once only works if your host platform has all required packages installed.**
**Unfortunately, do to Apple's restrictions, macOS and iOS platforms can _only_ be build on macOS.**
**This also means that macOS is the only host platform that can build _all_ platforms at once.**

However, we can build and run individual parts (`packages`) matching our host platform and we can use a combination of cross-compilation, Docker and/or Virtual Machines (VM) to build everything on one host platform.

Host (top) vs. Target (left) compatibility matrix:

|                         | Host: Windows                                                                                                                                                                                                                                                    | Host: Linux                                                                                                                                                                     | Host: macOS                                                                                                                                    |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **Target: Windows**     | ✅: [Visual Studio](https://visualstudio.com/)                                                                                                                                                                                                                    | 🔀: [MinGW](https://www.mingw-w64.org/)                                                                                                                                          | 🔀: [MinGW](https://www.mingw-w64.org/)                                                                                                         |
| **Target: Linux**       | ⚠️: [WSL](https://docs.microsoft.com/en-us/windows/wsl/) or VM or Docker                                                                                                                                                                                          | ✅: [GCC](https://gcc.gnu.org/) or [Clang](https://clang.llvm.org/)                                                                                                              | 🔀: Docker or VM                                                                                                                                |
| **Target: macOS**       | ⚠️: [Docker-OSX (inside WSL with Docker)](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM (inside WSL or VM)](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox (inside/with WSL and/or MSYS2/MinGW)](https://github.com/myspaghetti/macos-virtualbox) | ⚠️: [Docker-OSX](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox](https://github.com/myspaghetti/macos-virtualbox) | ✅: [XCode](https://developer.apple.com/xcode/)                                                                                                 |
| **Target: Android**     | 🔀: [Android Studio](https://developer.android.com/studio/) or [Android CommandLine-Tools](https://developer.android.com/studio/#command-tools)                                                                                                                   | 🔀: [Android Studio](https://developer.android.com/studio/) or [Android CommandLine-Tools](https://developer.android.com/studio/#command-tools)                                  | 🔀: [Android Studio](https://developer.android.com/studio/) or [Android CommandLine-Tools](https://developer.android.com/studio/#command-tools) |
| **Target: iOS**         | ⚠️: [Docker-OSX (inside WSL with Docker)](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM (inside WSL or VM)](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox (inside/with WSL and/or MSYS2/MinGW)](https://github.com/myspaghetti/macos-virtualbox) | ⚠️: [Docker-OSX](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox](https://github.com/myspaghetti/macos-virtualbox) | ✅: [XCode](https://developer.apple.com/xcode/)                                                                                                 |
| **Target: WebAssembly** | ✅: [Wasm-Pack](https://rustwasm.github.io/wasm-pack/installer/)                                                                                                                                                                                                  | ✅: [Wasm-Pack](https://rustwasm.github.io/wasm-pack/installer/)                                                                                                                 | ✅: [Wasm-Pack](https://rustwasm.github.io/wasm-pack/installer/)                                                                                |

✅ = Natively supported.
🔀 = Cross-Compilation & Toolchain needed.
⚠️ = Possible, but takes some more effort and/or special setups or VM to work.

Building can be done via:

```shell
cargo build --package <package>
```

Or run it directly (running will build the project beforehand!):

```shell
cargo run --package <package>
```

If there are tests present for the project, we can test them:

```shell
cargo test --package <package>
```

Or check if the project configuration is valid & build-able:

```shell
cargo check --package <package>
```

> Note: Adding `--release` to either of the commands will build a release version, instead of a debug version.

**Do note that some platforms (like iOS and Android) require special tools and `cargo` extensions to properly build.**
While we could do that step manually, it is much more convenient and easier to use this way.
Check the `README.md` of a platform to learn more about requirements and tools.

Since we can't build for all target platforms on a single host platform (without major modification; see above), the `--package <package>` part is very important.
Simply replace `<package>` with the package name inside the `Cargo.toml` to build it.
Names commonly will be `platform_<platform>` for platform-specific packages (e.g. `platform_windows` or `platform_ios`) or `shared` for the shared code.
In case multiple shared projects are present, check their `Cargo.toml` for their name (commonly: folder name).

However, since we share most of our code on all target platforms, we only really need to validate the code working on **one platform** (ideally your host platform for best performance or main target platform).
Only rarely should we need platform-specific code which, if it exists, needs to be tested.
Though, a continuous integration pipeline (CI) can take care of that for you mostly!
Check [Continuous Integration](#Continuous-Integration) for more.

## Continuous Integration

[![Multiplatform Build](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/multiplatform-build.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/multiplatform-build.yml)

This project utilizes the GitHub Actions CI (= Continuous Integration) to showcase how to build for all platforms.
For most platforms we just need a runner on the target platform (Windows, Linux or macOS) and install Rust.
This can be simply done by following [rustup.rs](https://rustup.rs/) (check the [other install options](https://rust-lang.github.io/rustup/installation/other.html) for automatically installing inside an CI).
Something like:

```shell
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable --profile full -y
```

should work for most platforms.

Note that we _may_ need more tools installed depending on the CI provider and platform.
Check the host <-> target matrix at [Building & Running](#Building-&-Running)

Additionally, often we have to `source` the profile changes. Something like:

```shell
source $HOME/.cargo/env
```

## Contributing & Getting Help

We welcome any help we get and try to answer questions as good as possible!
Generally speaking, please open an [issue here on GitHub](issues/new) or contact me directly.
No matter the problem or question.

In case you've got an idea/request for an example/template, please do open an [issue here on GitHub](issues/new).

Want to add your own example/template project to the organization and use our CI's?
Please open an [issue here on GitHub](issues/new).

## History

_Akimo_ is a very old project made by myself.  

_Akimo_ originally was written in [Java] and was intended to be a full game.  
However, back then I was still very much at the beginning of learning rendering, and thus v1.0 ended up being a CPU-renderer.

The concept was incredibly simple:  
The Engine had only one job: to make the singular Game I had in mind run.
Said game was basically similar to [The Binding of Issac].
A simple 2D "shooter" where you have limited hearts, lots of enemies ("monsters") attacking you with dodge-able bullets and lastly boss fights.
Items, of course, where also a thing which enhanced your abilities, gave you new abilities, movements or even could help you by healing you, giving you more hearts, or many other things.

After getting the basics working I noticed that my renderer is quite slow and could use some power-ups ... so I began optimizing my code heavily to the point where I would even outperform a later [OpenGL] based renderer.  
The engine managed to, similar like to how a GPU does it, find what objects are in visible and upfront and only would colour in pixels for these, while ignoring everything off-camera, out of view and hidden behind something.
Multiple layers where used to split the game rendering into multiple steps of which many could also be buffered and only needed to be updated (i.e. "re-rendered") in rare scenarios.

With all of this I had a very basic game ready:  
You had a character that was controlled by the `WASD` keys.
Your mouse cursor would draw in a circle around your character and on `LMB` (left-mouse-button) press you would shoot.  
Items could be used with `RMB` (right-mouse-button) and picked up by walking over them.
Each level would be a random connection of multiple rooms which I had predefined (i.e. no full procedural generation per-room, but the whole level would be procedurally generated).
Each room also had different enemies spawning, which all attacked and moved differently.

Which all sounds a bit complex, but compared to [The Binding of Issac] I had 4x different room types, 3x enemy types, 3x items and not even bosses.
Finding the exit of a level would just re-generate a new set of rooms and spawn you in that level.

At this point I noticed that I am very much interested in finding out how to _"actually"_ render something.
Not just CPU-Rendered but utilizing the GPU!
At this point I would switch to [LWJGL], which at this point only supported OpenGL as a backend.
After lots of studying I eventually got the same version of the game going in [OpenGL], but surprisingly it was running slower (i.e. less FPS) than my CPU-Renderer.

Unfortunately, at this point in my life school has became much more stressful and I never really progressed from this point on.
Furthermore, the source code of this Engine+Game project got lost over time.
However, the ideas and principles behind it still remained!

Much later [Vulkan] was released and I wanted to step up from [OpenGL] to [Vulkan].
At the same time [LWJGL] also added support for [Vulkan] so I started learning again and ported this "Game" over to [Vulkan].
After a lot of time I had it all working finally and running better than my CPU-Renderer this time!

However, at this time I was also very interested in native development with [C++]. I also heard that [C++] is so much faster than [Java] and that [LWJGL] is rather slow.
_Which turned out now to be true in the end, but that's what I heard!_
So ... I began rewriting the whole thing, mostly the engine part, in [C++].
Took a lot of time again but eventually I succeeded.
However, this time it felt like my engine has gotten well enough to go beyond 2D: So I tried some 3D stuff.
Nothing every really came out of it and the [Vulkan] based engine once again got abandoned for a while.

In between here I was also toying around a lot with existing Engines like [Unity], [UnrealEngine] and eventually [Godot].
I tried to make multiple games, all called "Akimo", but neither reached publishing/ready status.  
On this node, what does "Akimo" even stand for?  
Nothing! That's the simple answer.  
When starting a new project I pick a random letter that comes to mind, 'A' in this case.
I then start adding letters to it until I find a name that I like.
Eventually I arrived at 'Akimo'.
In some sense this means something like "Game" or "Game Engine" for me and I may even call a finished game "Akimo" at some point.  
Thus, "Akimo" and "Akimo Engine" are two separate things!  
One is a game, never released, one is an Engine released with this repo :)

Eventually, [Rust] was published and I loved it so much.
Native, that is much easier and more powerful than [C++], while also having a lot more security and being cross-platform checks so many boxes for me.
With that, I decided to _once again_ rewrite _Akimo_ but in [Rust] this time.
Unfortunately, the source code of the old engine ([Java] and [C++] version!) has been lost at this point so I was going off memory.
Eventually my engine was back, but I had no idea what to do with it as my original "[The Binding of Issac] clone" wasn't cutting it for me anymore.

At this moment (relatively recently as of writing this actually!) I learned about [WGPU].
[WGPU] gives you a HAL (Hardware Abstract Layer) that works across basically any modern graphics API backend.
[Vulkan] (universal), [DirectX] (Windows), [Metal] (macOS/Apple) and even older backends like [OpenGL] are supported.
Furthermore, [WGPU] is supposed to replace [WebGL] (a [OpenGL] fork for the web), so browser native rendering is also supported!

Soooo ... once again I started rewriting my engine, from scratch, while preserving the initial concepts of it.
Though, arguably, some concepts I was using in [Java] aren't applicable in [Rust] so they got changed, but the essence is still the same!  
This time I am also, once again, purposefully building this engine for a game I have in mind.

This is now. Basically.  
I plan on extending the engine a lot, however keeping in mind that I am making a game this time.

## Coverage

A combination of [grcov](https://github.com/mozilla/grcov) and [codecov.io](https://codecov.io) is used to provide code-to-test coverage.  
**Please note that it is impossible to reach 100% coverage on some platforms as e.g. bindgen-code (i.e. dynamically generated code / macros) is NOT covered by `grcov` and certain platform specific tools (like `cargo-apk`) generate additional code that also is NOT included in the coverage.**

Test-to-Code coverage status: [![codecov](https://codecov.io/gh/rust-multiplatform/Base-Project-Template/branch/main/graph/badge.svg?token=XpGvuQVirP)](https://codecov.io/gh/rust-multiplatform/Base-Project-Template)

Below are several charts showing/highlighting the distribution of **all platforms**.

### Sunburst

![Sunburst](https://codecov.io/gh/rust-multiplatform/Base-Project-Template/branch/main/graphs/sunburst.svg?token=XpGvuQVirP)

### Grid

![Grid](https://codecov.io/gh/rust-multiplatform/Base-Project-Template/branch/main/graphs/tree.svg?token=XpGvuQVirP)

### Icicle

![Icicle](https://codecov.io/gh/rust-multiplatform/Base-Project-Template/branch/main/graphs/icicle.svg?token=XpGvuQVirP)

[Rust]: https://www.rust-lang.org/
[fork]: https://github.com/Sakul6499/Akimo-Engine/fork
[java]: https://www.java.com/en/
[The Binding of Issac]: https://store.steampowered.com/app/113200/The_Binding_of_Isaac/
[OpenGL]: https://www.opengl.org/
[LWJGL]: https://www.lwjgl.org/
[Vulkan]: https://vulkan.org/
[C++]: https://cplusplus.com/
[WGPU]: https://wgpu.rs/
[DirectX]: https://support.microsoft.com/en-us/topic/how-to-install-the-latest-version-of-directx-d1f5ffa5-dae2-246c-91b1-ee1e973ed8c2
[Metal]: <https://developer.apple.com/metal/>
[WebGL]: https://www.khronos.org/webgl/
