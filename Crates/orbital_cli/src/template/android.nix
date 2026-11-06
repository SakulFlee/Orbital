{ pkgs, pkgsAndroid, fenix, system }:
let
  androidSystemImageAbi =
    if pkgs.stdenv.hostPlatform.isDarwin && pkgs.stdenv.hostPlatform.isAarch64 then
      "arm64-v8a"
    else
      "x86_64";

  composeAndroidSdk =
    { includeEmulator ? false, platformVersions ? [ "34" "35" "36" ] }:
    (pkgsAndroid.androidenv.composeAndroidPackages {
      cmdLineToolsVersion = "22.0";
      inherit platformVersions;
      buildToolsVersions = [ "36.0.0" ];
      includeNDK = true;
      ndkVersions = [ "26.2.11394342" ];
      inherit includeEmulator;
      includeSystemImages = includeEmulator;
      systemImageTypes = [ "default" ];
      abiVersions = [ androidSystemImageAbi ];
    }).androidsdk;

  androidSdk = composeAndroidSdk { };
  androidEmulatorSdk = composeAndroidSdk {
    includeEmulator = true;
    platformVersions = [ "34" ];
  };

  androidSdkRoot = "${androidSdk}/libexec/android-sdk";
  androidEmulatorSdkRoot = "${androidEmulatorSdk}/libexec/android-sdk";

  fenixPkgs = fenix.packages.${system};
  rustToolchainAndroid = fenixPkgs.combine [
    fenixPkgs.stable.rustc
    fenixPkgs.stable.cargo
    fenixPkgs.stable.rust-std
    fenixPkgs.targets.aarch64-linux-android.stable.rust-std
    fenixPkgs.targets.armv7-linux-androideabi.stable.rust-std
    fenixPkgs.targets.x86_64-linux-android.stable.rust-std
    fenixPkgs.targets.i686-linux-android.stable.rust-std
  ];

  commonTools = with pkgs; [
    pkgs.gcc
    pkgs.binutils
    pkgs.gnumake
    pkgs.git
    pkgs.pkg-config
  ];

  commonLinuxBuildInputs = with pkgs; [
    pkgs.glibc.dev
    pkgs.systemd
    wayland
    libxkbcommon
    vulkan-loader
    pkgs.mesa
  ];

  commonLinuxLdLibraryPath = pkgs.lib.makeLibraryPath [
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.vulkan-loader
    pkgs.mesa
  ];

  mkAndroidShell = { sdk, sdkRoot, shellName }: pkgs.mkShell {
    nativeBuildInputs = [
      rustToolchainAndroid
      pkgsAndroid.cargo-ndk
      pkgsAndroid.jdk25
      sdk
    ] ++ commonTools;

    buildInputs = commonLinuxBuildInputs;

    ORBITAL_ANDROID_SHELL = shellName;
    ORBITAL_SKIP_CLI_INSTALL = "1";
    SKIP_GLTF_EXPORT = "1";
    ANDROID_HOME = sdkRoot;
    ANDROID_SDK_ROOT = sdkRoot;
    ANDROID_NDK_HOME = "${sdkRoot}/ndk/26.2.11394342";
    ANDROID_NDK_ROOT = "${sdkRoot}/ndk/26.2.11394342";
    JAVA_HOME = "${pkgsAndroid.jdk25}";
    GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${sdkRoot}/build-tools/36.0.0/aapt2";

    shellHook = ''
      export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${commonLinuxLdLibraryPath}"
      mkdir -p .cargo/bin
      ln -sf "$(which gcc)" .cargo/bin/x86_64-linux-gnu-gcc
      export PATH="$PWD/.cargo/bin:$PATH"
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(which gcc)"
    '';
  };
in
{
  android = mkAndroidShell {
    sdk = androidSdk;
    sdkRoot = androidSdkRoot;
    shellName = "android";
  };
  android-emulator = mkAndroidShell {
    sdk = androidEmulatorSdk;
    sdkRoot = androidEmulatorSdkRoot;
    shellName = "android-emulator";
  };
}
