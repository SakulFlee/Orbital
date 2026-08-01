FROM rust:latest

RUN dpkg --add-architecture arm64 \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      pkg-config gcc build-essential \
      libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev \
      mesa-vulkan-drivers libgl1-mesa-dri \
      libudev-dev:arm64 libwayland-dev:arm64 libxkbcommon-dev:arm64 libvulkan-dev:arm64 \
      gcc-aarch64-linux-gnu gcc-mingw-w64-x86-64 \
      zip git curl ca-certificates rsync \
 && rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt clippy \
 && rustup target add aarch64-unknown-linux-gnu x86_64-pc-windows-gnu

ENV PKG_CONFIG_ALLOW_CROSS=1 \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
    CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc \
    AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
