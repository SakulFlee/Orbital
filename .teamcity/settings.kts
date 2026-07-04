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
            scriptContent = "nix develop --command cargo fmt --all --check"

        }
        script {
            name = "Clippy"
            scriptContent = "nix develop --command cargo clippy"

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
        equals("teamcity.agent.os.family", "Linux")
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
            scriptContent = "nix develop --command cargo test --no-fail-fast"

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
        equals("teamcity.agent.os.family", "Linux")
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
            name = "Build"
            scriptContent = """
                nix develop --command cargo build --release --target x86_64-unknown-linux-gnu
            """.trimIndent()

        }
        script {
            name = "Package"
            scriptContent = """
                mkdir upload
                find target/x86_64-unknown-linux-gnu/release -mindepth 1 -maxdepth 1 -type f \
                  -not -name '*.d' -not -name '*.rlib' -not -name '.cargo-lock' -not -name 'CACHEDIR.TAG' \
                  -exec mv {} upload/ \;
                mv Assets/ upload/
                cd upload && zip -r ../x86_64-unknown-linux-gnu.zip .
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
        equals("teamcity.agent.os.family", "Linux")
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
            name = "Build"
            scriptContent = """
                nix develop .#aarch64-cross --command cargo build --release --target aarch64-unknown-linux-gnu
            """.trimIndent()

        }
        script {
            name = "Package"
            scriptContent = """
                mkdir upload
                find target/aarch64-unknown-linux-gnu/release -mindepth 1 -maxdepth 1 -type f \
                  -not -name '*.d' -not -name '*.rlib' -not -name '.cargo-lock' -not -name 'CACHEDIR.TAG' \
                  -exec mv {} upload/ \;
                mv Assets/ upload/
                cd upload && zip -r ../aarch64-unknown-linux-gnu.zip .
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
        equals("teamcity.agent.os.family", "Linux")
    }

    dependencies {
        snapshot(Test) {
            reuseBuilds = ReuseBuilds.SUCCESSFUL
        }
    }
})


