# Rust Multi-Platform Base Project Template

> ⚠️ THIS IS A TEMPLATE. ⚠️

- [ ] Platform: Windows  
- [ ] Platform: Linux  
- [ ] Platform: macOS  
- [ ] Platform: Android  
- [ ] Platform: iOS  

## Project layout

| Folder                                 | Description                                                                                                                         | GitHub Actions                                                                                                                                                                                                              | Circle CI                                                                                                                                                                           |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ./                                     | Workspace root; `Cargo.toml` contains all project folders (internal crates)                                                         |                                                                                                                                                                                                                             |
| [platform/](platform/)                 | Platform projects root. Contains every platform this demonstration is supported on incl. instructions on how to build and use them. |                                                                                                                                                                                                                             |                                                                                                                                                                                     |
| [platform/android/](platform/android/) | Contains the Android platform project and instructions on how to build this project for Android and run it.                         | [![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_android.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_android.yml) |                                                                                                                                                                                     |
| [platform/ios/](platform/ios/)         | Contains the iOS platform project and instructions on how to build this project for iOS and run it.                                 | [![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_ios.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_ios.yml)         |                                                                                                                                                                                     |
| [platform/linux/](platform/linux/)     | Contains the Linux platform project and instructions on how to build this project for Linux and run it.                             | [![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_linux.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_linux.yml)     | [![CircleCI](https://circleci.com/gh/rust-multiplatform/Base-Project-Template/tree/main.svg?style=svg)](https://circleci.com/gh/rust-multiplatform/Base-Project-Template/tree/main) |
| [platform/macos/](platform/macos/)     | Contains the macOS platform project and instructions on how to build this project for macOS and run it.                             | [![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_macos.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_macos.yml)     |                                                                                                                                                                                     |
| [platform/windows/](platform/windows/) | Contains the Windows platform project and instructions on how to build this project for Windows and run it.                         | [![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_windows.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/platform_windows.yml) |                                                                                                                                                                                     |
| [shared/](shared/)                     | Contains the **shared** code between **all** projects.                                                                              | [![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/shared.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/shared.yml)                     |                                                                                                                                                                                     |

To break this down:  
The [shared/](shared/) folder contains our **cross/multi-platform code**.  
99.9% of what we do in this project will happen there.

Each of the projects inside [platform/](platform/) are representing a **platform specific project**.  
In most cases, like for Windows, Linux and macOS, there is nothing else to do but call our shared code and compile a **binary**.  
However, on certain platforms, like Android and iOS, we have to use some special commands and tools to get an e.g. .APK (Android) or .APP (iOS) file.
Furthermore, on those systems resources/assets may need to be specially loaded and their `Cargo.toml` will be much more extensive.
Simply said: For those special platforms we will use some cargo extensions which will automatically generate a native project in the background. Values from `Cargo.toml` will be used to generate those.

**Please check the `README.md` inside each platform to see how each platform is build, run and what you will need to do so.**

## Targets & Architectures

Below are the individual targets and architectures.  

They are grouped by tiers.
Rust tier policies can be found [here](https://doc.rust-lang.org/rustc/target-tier-policy.html).

### Windows Targets

| Target                  | Tier Support |
| ----------------------- | ------------ |
| i686-pc-windows-msvc    | Tier 1       |
| x86_64-pc-windows-msvc  | Tier 1       |
| aarch64-pc-windows-msvc | Tier 2       |
| i586-pc-windows-msvc    | Tier 2       |

> The `-gnu` versions (instead of the `-msvc` versions) are the same architecture.  
> The difference is that `-msvc` uses `MSVC`, while `-gnu` uses `GCC`.  
> `MSVC` is Microsoft Visual Studio's build tool.  
> `GCC` is GNU's C-Compiler-Collection.

### Linux Targets

| Target                          | Tier Support |
| ------------------------------- | ------------ |
| aarch64-unknown-linux-gnu       | Tier 1       |
| i686-unknown-linux-gnu          | Tier 1       |
| x86_64-unknown-linux-gnu        | Tier 1       |
| aarch64-unknown-linux-musl      | Tier 2       |
| arm-linux-androideabi           | Tier 2       |
| arm-unknown-linux-gnueabi       | Tier 2       |
| arm-unknown-linux-gnueabihf     | Tier 2       |
| arm-unknown-linux-musleabi      | Tier 2       |
| arm-unknown-linux-musleabihf    | Tier 2       |
| armv5te-unknown-linux-gnueabi   | Tier 2       |
| armv7-linux-androideabi         | Tier 2       |
| armv7-unknown-linux-gnueabihf   | Tier 2       |
| armv7-unknown-linux-musleabihf  | Tier 2       |
| i586-unknown-linux-gnu          | Tier 2       |
| i586-unknown-linux-musl         | Tier 2       |
| i686-unknown-linux-musl         | Tier 2       |
| mips-unknown-linux-gnu          | Tier 2       |
| mips-unknown-linux-musl         | Tier 2       |
| mips64-unknown-linux-gnuabi64   | Tier 2       |
| mips64el-unknown-linux-gnuabi64 | Tier 2       |
| mipsel-unknown-linux-gnu        | Tier 2       |
| mipsel-unknown-linux-musl       | Tier 2       |
| powerpc-unknown-linux-gnu       | Tier 2       |
| powerpc64-unknown-linux-gnu     | Tier 2       |
| powerpc64le-unknown-linux-gnu   | Tier 2       |
| s390x-unknown-linux-gnu         | Tier 2       |
| sparc64-unknown-linux-gnu       | Tier 2       |
| x86_64-unknown-linux-gnux32     | Tier 2       |
| x86_64-unknown-linux-musl       | Tier 2       |

> Targets ending with `-gnu` use GNU's `libc`, while `-musl` use MUSL's `libc` bindings.  
> Both are a common standard used by many linux operating systems.

### macOS Targets

| Target               | Tier Support |
| -------------------- | ------------ |
| x86_64-apple-darwin  | Tier 1       |
| aarch64-apple-darwin | Tier 2       |

> The new M1 & M2 chips from Apple are based on AARCH64.  
> Thus, x86_64 = "legacy" / old macs; aarch64 = new M1 macs.

### iOS Targets

| x86_64-apple-ios | Tier 2 |
| aarch64-apple-ios | Tier 2 |
| aarch64-apple-ios-sim | Tier 2 |

> The new M1 & M2 chips from Apple are based on AARCH64.  
> Thus, x86_64 = "legacy" / old macs; aarch64 = new M1 macs.
>  
> `-sim` is a custom architecture for the iOS Simulator.

### Android Targets

| Target                  | Tier Support |
| ----------------------- | ------------ |
| x86_64-linux-android    | Tier 2       |
| aarch64-linux-android   | Tier 2       |
| i686-linux-android      | Tier 2       |
| armv7-linux-androideabi | Tier 2       |

### WebAssembly

| Target                 | Tier Support |
| ---------------------- | ------------ |
| wasm32-unknown-unknown | Tier 2       |
| wasm32-wasi            | Tier 2       |

### Other architectures

Rust supports a lot of architectures, a list can be found [here](https://doc.rust-lang.org/rustc/platform-support.html).

Other architectures **should** work, but aren't tested in this repository at the moment.  
Feel free to contribute and open an issue for them!

## Building & Running

**Building & Running all projects at once only works if your host platform has all required packages installed.**  
**Unfortunately, do to Apple's restrictions, macOS and iOS platforms can _only_ be build on macOS.**  
**This also means that macOS is the only host platform that can build _all_ platforms at once.**

However, we can build and run individual parts (`packages`) matching our host platform and we can use a combination of cross-compilation, Docker and/or Virtual Machines (VM) to build everything on one host platform.

Host (top) vs. Target (left) compatibility matrix:  

|                     | Host: Windows                                                                                                                                                                                                                                                    | Host: Linux                                                                                                                                                                     | Host: macOS                                                                                                                                    |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **Target: Windows** | ✅: [Visual Studio](https://visualstudio.com/)                                                                                                                                                                                                                    | 🔀: [MinGW](https://www.mingw-w64.org/)                                                                                                                                          | 🔀: [MinGW](https://www.mingw-w64.org/)                                                                                                         |
| **Target: Linux**   | ⚠️: [WSL](https://docs.microsoft.com/en-us/windows/wsl/) or VM or Docker                                                                                                                                                                                          | ✅: [GCC](https://gcc.gnu.org/) or [Clang](https://clang.llvm.org/)                                                                                                              | 🔀: Docker or VM                                                                                                                                |
| **Target: macOS**   | ⚠️: [Docker-OSX (inside WSL with Docker)](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM (inside WSL or VM)](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox (inside/with WSL and/or MSYS2/MinGW)](https://github.com/myspaghetti/macos-virtualbox) | ⚠️: [Docker-OSX](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox](https://github.com/myspaghetti/macos-virtualbox) | ✅: [XCode](https://developer.apple.com/xcode/)                                                                                                 |
| **Target: Android** | 🔀: [Android Studio](https://developer.android.com/studio/) or [Android CommandLine-Tools](https://developer.android.com/studio/#command-tools)                                                                                                                   | 🔀: [Android Studio](https://developer.android.com/studio/) or [Android CommandLine-Tools](https://developer.android.com/studio/#command-tools)                                  | 🔀: [Android Studio](https://developer.android.com/studio/) or [Android CommandLine-Tools](https://developer.android.com/studio/#command-tools) |
| **Target: iOS**     | ⚠️: [Docker-OSX (inside WSL with Docker)](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM (inside WSL or VM)](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox (inside/with WSL and/or MSYS2/MinGW)](https://github.com/myspaghetti/macos-virtualbox) | ⚠️: [Docker-OSX](https://github.com/sickcodes/Docker-OSX) or [OSX-KVM](https://github.com/kholia/OSX-KVM) or [macOS-VirtualBox](https://github.com/myspaghetti/macos-virtualbox) | ✅: [XCode](https://developer.apple.com/xcode/)                                                                                                 |

✅ = Natively supported.  
🔀 = Cross-Compilation & Toolchain needed.  
⚠️ = Possible, but takes some more advanced setup or VM. Check the footnotes.  

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
