package _Self.buildTypes

import jetbrains.buildServer.configs.kotlin.*

project {
    buildType(Check)
    buildType(Build)
    buildType(Test)

    sequential  {
        buildType(Check)
        buildType(Build)
        buildType(Test)
    }
}

object Check : BuildType({
    name = "Check"

    vcs {
        root(HttpsGithubComRustMultiplatformBaseProjectTemplateRefsHeadsMain1)
    }

    steps {
        step {
            name = "Check"
            type = "cargo"
            param("cargo-build-package", "platform_linux")  # Our Rust package
            param("cargo-verbosity", "--verbose")           # Verbose output
            param("cargo-command", "check")                 # Cargo command
        }
    }

    triggers {
        vcs {
        }
    }
})

object Build : BuildType({
    name = "Build"

    vcs {
        root(HttpsGithubComRustMultiplatformBaseProjectTemplateRefsHeadsMain1)
    }

    steps {
        step {
            name = "Build"
            type = "cargo"
            param("cargo-build-package", "platform_linux")  # Our Rust package
            param("cargo-verbosity", "--verbose")           # Verbose output
            param("cargo-command", "build")                 # Cargo command
        }
    }

    triggers {
        vcs {
        }
    }
})

object Test : BuildType({
    name = "Test"

    vcs {
        root(HttpsGithubComRustMultiplatformBaseProjectTemplateRefsHeadsMain1)
    }

    steps {
        step {
            name = "Test"
            type = "cargo"
            param("cargo-build-package", "platform_linux")  # Our Rust package
            param("cargo-verbosity", "--verbose")           # Verbose output
            param("cargo-command", "test")                  # Cargo command
        }
    }

    triggers {
        vcs {
        }
    }
})
