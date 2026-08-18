use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;

pub fn init() -> Result<()> {
    let project_root = config::find_project_root()?;
    let android_dir = project_root.join("Android");

    if android_dir.exists() {
        println!("Android/ directory already exists. Skipping generation.");
        return Ok(());
    }

    let android_config = config::load_android_config()?;

    println!("Generating Android project...");
    println!("  Package: {}", android_config.package_name());
    println!("  Min SDK: {}", android_config.min_sdk());
    println!("  Target SDK: {}", android_config.target_sdk());

    generate_project(&android_dir, &android_config)?;

    println!(
        "\nAndroid project generated successfully at {}",
        android_dir.display()
    );
    println!("\nNext steps:");
    println!("  Run: orbital build android");

    Ok(())
}

fn generate_project(android_dir: &Path, config: &config::AndroidConfig) -> Result<()> {
    let replacements = create_replacements(config);

    // Create directory structure
    fs::create_dir_all(
        android_dir
            .join("app")
            .join("src")
            .join("main")
            .join("res")
            .join("values"),
    )
    .context("Failed to create Android directory structure")?;
    fs::create_dir_all(android_dir.join("gradle").join("wrapper"))
        .context("Failed to create gradle wrapper directory")?;

    // Write template files
    write_template_file(
        android_dir.join("build.gradle"),
        BUILD_GRADLE,
        &replacements,
    )?;
    write_template_file(
        android_dir.join("settings.gradle"),
        SETTINGS_GRADLE,
        &replacements,
    )?;
    write_template_file(
        android_dir.join("gradle.properties"),
        GRADLE_PROPERTIES,
        &replacements,
    )?;
    write_template_file(
        android_dir.join("app").join("build.gradle"),
        APP_BUILD_GRADLE,
        &replacements,
    )?;
    write_template_file(
        android_dir
            .join("app")
            .join("src")
            .join("main")
            .join("AndroidManifest.xml"),
        ANDROID_MANIFEST,
        &replacements,
    )?;
    write_template_file(
        android_dir
            .join("app")
            .join("src")
            .join("main")
            .join("res")
            .join("values")
            .join("strings.xml"),
        STRINGS_XML,
        &replacements,
    )?;

    // Write gradle wrapper files
    fs::write(
        android_dir
            .join("gradle")
            .join("wrapper")
            .join("gradle-wrapper.properties"),
        GRADLE_WRAPPER_PROPERTIES,
    )
    .context("Failed to write gradle-wrapper.properties")?;

    // Copy gradle-wrapper.jar from the template directory
    let template_jar = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("template")
        .join("gradle")
        .join("wrapper")
        .join("gradle-wrapper.jar");
    fs::copy(
        &template_jar,
        android_dir
            .join("gradle")
            .join("wrapper")
            .join("gradle-wrapper.jar"),
    )
    .context("Failed to copy gradle-wrapper.jar")?;

    // Copy gradlew scripts
    let template_gradlew = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("template")
        .join("gradlew");

    // On Unix, strip CRLF line endings from gradlew so the shebang works.
    // The template may have CRLF from being checked out on Windows.
    #[cfg(unix)]
    {
        let content =
            fs::read_to_string(&template_gradlew).context("Failed to read gradlew template")?;
        let content = content.replace("\r\n", "\n");
        fs::write(android_dir.join("gradlew"), content).context("Failed to write gradlew")?;
    }
    #[cfg(not(unix))]
    {
        fs::copy(&template_gradlew, android_dir.join("gradlew"))
            .context("Failed to copy gradlew")?;
    }

    let template_gradlew_bat = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("template")
        .join("gradlew.bat");
    fs::copy(&template_gradlew_bat, android_dir.join("gradlew.bat"))
        .context("Failed to copy gradlew.bat")?;

    // Make gradlew executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let gradlew_path = android_dir.join("gradlew");
        let mut perms = fs::metadata(&gradlew_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&gradlew_path, perms)?;
    }

    Ok(())
}

fn create_replacements(config: &config::AndroidConfig) -> Vec<(String, String)> {
    vec![
        (
            "@@@PACKAGE_NAME@@@".to_string(),
            config.package_name().to_string(),
        ),
        ("@@@MIN_SDK@@@".to_string(), config.min_sdk().to_string()),
        (
            "@@@TARGET_SDK@@@".to_string(),
            config.target_sdk().to_string(),
        ),
        ("@@@AGP_VERSION@@@".to_string(), "9.3.1".to_string()),
        (
            "@@@NDK_VERSION@@@".to_string(),
            config.ndk_version().to_string(),
        ),
        // @@@LIBRARY_NAME@@@ and @@@APP_NAME@@@ are intentionally left
        // unreplaced here; they're finalized during build with the actual
        // crate/lib names (see update_android_project).
    ]
}

fn write_template_file(
    path: PathBuf,
    content: &str,
    replacements: &[(String, String)],
) -> Result<()> {
    let replaced_content = replace_placeholders(content, replacements);
    fs::write(&path, replaced_content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(())
}

fn replace_placeholders(content: &str, replacements: &[(String, String)]) -> String {
    let mut result = content.to_string();
    for (placeholder, value) in replacements {
        result = result.replace(placeholder, value);
    }
    result
}

// Template files as static strings
const BUILD_GRADLE: &str = r#"plugins {
    id 'com.android.application' version '@@@AGP_VERSION@@@' apply false
}

task clean(type: Delete) {
    delete rootProject.buildDir
}
"#;

const SETTINGS_GRADLE: &str = r#"pluginManagement {
    repositories {
        gradlePluginPortal()
        google()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

include ':app'
"#;

const GRADLE_PROPERTIES: &str = r#"org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
android.useAndroidX=true
android.nonTransitiveRClass=true
"#;

const APP_BUILD_GRADLE: &str = r#"plugins {
    id 'com.android.application'
}

android {
    namespace '@@@PACKAGE_NAME@@@'
    compileSdk @@@TARGET_SDK@@@

    defaultConfig {
        applicationId "@@@PACKAGE_NAME@@@"
        minSdk @@@MIN_SDK@@@
        targetSdk @@@TARGET_SDK@@@
        ndkVersion '@@@NDK_VERSION@@@'
    }

    buildTypes {
        release {
            minifyEnabled false
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }
    }
}

dependencies {
}
"#;

const ANDROID_MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    android:versionCode="1"
    android:versionName="1.0">
    <application
        android:hasCode="false"
        android:label="@string/app_name">
        <activity
            android:name="android.app.NativeActivity"
            android:configChanges="orientation|keyboardHidden|screenSize"
            android:screenOrientation="@@@SCREEN_ORIENTATION@@@"
            android:exported="true"
            android:theme="@android:style/Theme.NoTitleBar.Fullscreen">
            <meta-data
                android:name="android.app.lib_name"
                android:value="@@@LIBRARY_NAME@@@" />
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#;

const STRINGS_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">@@@APP_NAME@@@</string>
</resources>
"#;

const GRADLE_WRAPPER_PROPERTIES: &str = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-9.6.1-bin.zip
networkTimeout=10000
validateDistributionUrl=true
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
"#;
