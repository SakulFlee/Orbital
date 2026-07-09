import jetbrains.buildServer.configs.kotlin.*
import jetbrains.buildServer.configs.kotlin.buildSteps.script
import jetbrains.buildServer.configs.kotlin.projectFeatures.githubAppConnection
import jetbrains.buildServer.configs.kotlin.projectFeatures.githubConnection
import jetbrains.buildServer.configs.kotlin.projectFeatures.githubIssues
import jetbrains.buildServer.configs.kotlin.triggers.vcs

version = "2026.1"

project {
    description = "Orbital (formerly: Akimo) is a cross-platform real-time graphical rendering engine & framework written in Rust. The goal of this project is to bridge well established industry-standard techniques with upcoming (next-gen) and experimental techniques to form a robust new foundation for next-gen visual computing."

    params {
        checkbox("teamcity.internal.pipelines.creation.enabled", "true",
                  checked = "true")
    }

    features {
        githubAppConnection {
            id = "PROJECT_EXT_348"
            displayName = "TeamCity Open-Source"
            appId = "906997"
            clientId = "Iv23liJditQYZhrCa8nl"
            clientSecret = "credentialsJSON:da83f6e2-3c0f-4a06-8990-887250e181b6"
            privateKey = "credentialsJSON:dbc0f3ae-1273-422c-b0a6-97b93308039a"
            webhookSecret = "credentialsJSON:7c3a8738-c1fa-4656-ae48-69d4c2a90240"
            ownerUrl = "https://github.com/SakulFlee"
        }
        githubConnection {
            id = "PROJECT_EXT_357"
            displayName = "GitHub.com"
            clientId = "Ov23liEW53FmrvN7kv4X"
            clientSecret = "credentialsJSON:35c836a9-6f3b-4007-bd8d-47c635072d43"
        }
        githubIssues {
            id = "PROJECT_EXT_591"
            displayName = "rust-multiplatform/Base-Project-Template"
            repositoryURL = "https://github.com/rust-multiplatform/Base-Project-Template"
        }
    }

    buildType(Lint)
    buildType(Test)
    buildType(Build_x86_64)
    buildType(Build_aarch64)
    buildType(Build_windows)
    buildType(Build_windows_arm64)
}

object Lint : BuildType({
    name = "Lint (fmt + clippy)"

    vcs {
        root(DslContext.settingsRoot)
    }

    params {
        param("env.SKIP_GLTF_EXPORT", "true")
    }

    steps {
        script {
            name = "Format check"
            scriptContent = """
                docker run --rm -e SKIP_GLTF_EXPORT -v "${'$'}PWD:/work" -w /work rust:latest bash -c '
                  apt-get update && apt-get install -y pkg-config libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev mesa-vulkan-drivers &&
                  rustup component add rustfmt &&
                  cargo fmt --all --check
                '
            """.trimIndent()

        }
        script {
            name = "Clippy"
            scriptContent = """
                docker run --rm -e SKIP_GLTF_EXPORT -v "${'$'}PWD:/work" -w /work rust:latest bash -c '
                  apt-get update && apt-get install -y pkg-config libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev mesa-vulkan-drivers &&
                  rustup component add clippy &&
                  cargo clippy
                '
            """.trimIndent()

        }
    }

    triggers {
        vcs {
            branchFilter = """
                +:refs/heads/*
            """.trimIndent()
        }
    }

    requirements {
        contains("teamcity.agent.jvm.os.name", "Linux")
    }
})

object Test : BuildType({
    name = "Test"

    vcs {
        root(DslContext.settingsRoot)
    }

    params {
        param("env.SKIP_GLTF_EXPORT", "true")
    }

    steps {
        script {
            name = "Cargo test"
            scriptContent = """
                docker run --rm -e SKIP_GLTF_EXPORT -v "${'$'}PWD:/work" -w /work rust:latest bash -c '
                  apt-get update && apt-get install -y pkg-config libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev mesa-vulkan-drivers &&
                  cargo test --no-fail-fast
                '
            """.trimIndent()

        }
    }

    triggers {
        vcs {
            branchFilter = """
                +:refs/heads/*
            """.trimIndent()
        }
    }

    requirements {
        contains("teamcity.agent.jvm.os.name", "Linux")
    }

    dependencies {
        snapshot(Lint) { }
    }
})

object Build_x86_64 : BuildType({
    name = "Build x86_64-unknown-linux-gnu"

    vcs {
        root(DslContext.settingsRoot)
    }

    artifactRules = "x86_64-unknown-linux-gnu.zip"

    params {
        param("env.SKIP_GLTF_EXPORT", "true")
    }

    steps {
        script {
            name = "Build and package"
            scriptContent = """
                docker run --rm -e SKIP_GLTF_EXPORT -v "${'$'}PWD:/work" -w /work rust:latest bash -c '
                  set -e
                  apt-get update
                  apt-get install -y pkg-config libudev-dev libwayland-dev libxkbcommon-dev libvulkan-dev zip
                  cargo build --release --target x86_64-unknown-linux-gnu
                  mkdir upload
                  find target/x86_64-unknown-linux-gnu/release -mindepth 1 -maxdepth 1 -type f \
                    -not -name "*.d" -not -name "*.rlib" -not -name ".cargo-lock" -not -name "CACHEDIR.TAG" \
                    -exec mv {} upload/ \;
                  mv Assets/ upload/
                  cd upload && zip -r ../x86_64-unknown-linux-gnu.zip .
                '
            """.trimIndent()

        }
    }

    triggers {
        vcs {
            branchFilter = """
                +:refs/tags/v*
            """.trimIndent()
        }
    }

    requirements {
        contains("teamcity.agent.jvm.os.name", "Linux")
    }

    dependencies {
        snapshot(Test) {
            reuseBuilds = ReuseBuilds.SUCCESSFUL
        }
    }
})

object Build_aarch64 : BuildType({
    name = "Build aarch64-unknown-linux-gnu"

    vcs {
        root(DslContext.settingsRoot)
    }

    artifactRules = "aarch64-unknown-linux-gnu.zip"

    params {
        param("env.SKIP_GLTF_EXPORT", "true")
    }

    steps {
        script {
            name = "Build and package"
            scriptContent = """
                docker run --rm -e SKIP_GLTF_EXPORT -v "${'$'}PWD:/work" -w /work rust:latest bash -c '
                  set -e
                  dpkg --add-architecture arm64
                  apt-get update
                  apt-get install -y gcc-aarch64-linux-gnu pkg-config
                  apt-get install -y libudev-dev:arm64 libwayland-dev:arm64 libxkbcommon-dev:arm64 libvulkan-dev:arm64 zip
                  rustup target add aarch64-unknown-linux-gnu
                  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
                  CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
                  PKG_CONFIG_ALLOW_CROSS=1 \
                  PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig \
                  cargo build --release --target aarch64-unknown-linux-gnu
                  mkdir upload
                  find target/aarch64-unknown-linux-gnu/release -mindepth 1 -maxdepth 1 -type f \
                    -not -name "*.d" -not -name "*.rlib" -not -name ".cargo-lock" -not -name "CACHEDIR.TAG" \
                    -exec mv {} upload/ \;
                  mv Assets/ upload/
                  cd upload && zip -r ../aarch64-unknown-linux-gnu.zip .
                '
            """.trimIndent()

        }
    }

    triggers {
        vcs {
            branchFilter = """
                +:refs/tags/v*
            """.trimIndent()
        }
    }

    requirements {
        contains("teamcity.agent.jvm.os.name", "Linux")
    }

    dependencies {
        snapshot(Test) {
            reuseBuilds = ReuseBuilds.SUCCESSFUL
        }
    }
})

object Build_windows : BuildType({
    name = "Build x86_64-pc-windows-msvc"

    vcs {
        root(DslContext.settingsRoot)
    }

    artifactRules = "target/x86_64-pc-windows-msvc/release/*.exe"

    steps {
        script {
            name = "Build release"
            scriptContent = """
                where rustup >nul 2>nul || powershell -NoProfile -ExecutionPolicy Bypass -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'"
                where rustup >nul 2>nul || "%TEMP%\rustup-init.exe" -y --default-toolchain stable
                set PATH=%PATH%;%USERPROFILE%\.cargo\bin
                rustup default stable
                rustup target add x86_64-pc-windows-msvc
                cargo build --release --target x86_64-pc-windows-msvc
            """.trimIndent()
        }
    }

    triggers {
        vcs {
            branchFilter = """
                +:refs/tags/v*
            """.trimIndent()
        }
    }

    requirements {
        contains("teamcity.agent.jvm.os.name", "Windows")
    }

    dependencies {
        snapshot(Test) {
            reuseBuilds = ReuseBuilds.SUCCESSFUL
        }
    }
})

object Build_windows_arm64 : BuildType({
    name = "Build aarch64-pc-windows-msvc"

    vcs {
        root(DslContext.settingsRoot)
    }

    artifactRules = "target/aarch64-pc-windows-msvc/release/*.exe"

    steps {
        script {
            name = "Build release"
            scriptContent = """
                where rustup >nul 2>nul || powershell -NoProfile -ExecutionPolicy Bypass -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'"
                where rustup >nul 2>nul || "%TEMP%\rustup-init.exe" -y --default-toolchain stable
                set PATH=%PATH%;%USERPROFILE%\.cargo\bin
                rustup default stable
                rustup target add aarch64-pc-windows-msvc
                cargo build --release --target aarch64-pc-windows-msvc
            """.trimIndent()
        }
    }

    triggers {
        vcs {
            branchFilter = """
                +:refs/tags/v*
            """.trimIndent()
        }
    }

    requirements {
        contains("teamcity.agent.jvm.os.name", "Windows")
    }

    dependencies {
        snapshot(Test) {
            reuseBuilds = ReuseBuilds.SUCCESSFUL
        }
    }
})


