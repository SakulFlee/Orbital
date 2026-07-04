import jetbrains.buildServer.configs.kotlin.*
import jetbrains.buildServer.configs.kotlin.projectFeatures.githubAppConnection
import jetbrains.buildServer.configs.kotlin.projectFeatures.githubConnection
import jetbrains.buildServer.configs.kotlin.projectFeatures.githubIssues

/*
The settings script is an entry point for defining a TeamCity
project hierarchy. The script should contain a single call to the
project() function with a Project instance or an init function as
an argument.

VcsRoots, BuildTypes, Templates, and subprojects can be
registered inside the project using the vcsRoot(), buildType(),
template(), and subProject() methods respectively.

To debug settings scripts in command-line, run the

    mvnDebug org.jetbrains.teamcity:teamcity-configs-maven-plugin:generate

command and attach your debugger to the port 8000.

To debug in IntelliJ Idea, open the 'Maven Projects' tool window (View
-> Tool Windows -> Maven Projects), find the generate task node
(Plugins -> teamcity-configs -> teamcity-configs:generate), the
'Debug' option is available in the context menu for the task.
*/

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
}
