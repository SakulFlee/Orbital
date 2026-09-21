use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;

pub fn init() -> Result<()> {
    let project_root = config::find_project_root()?;
    let ios_dir = project_root.join("iOS");

    if ios_dir.exists() {
        println!("iOS/ directory already exists. Skipping generation.");
        return Ok(());
    }

    let ios_config = config::load_ios_config()?;

    println!("Generating iOS project...");
    println!("  Bundle ID: {}", ios_config.bundle_id());
    println!("  Deployment Target: {}", ios_config.deployment_target());
    println!("  App Name: {}", ios_config.app_name());

    generate_project(&ios_dir, &ios_config)?;

    println!(
        "\niOS project generated successfully at {}",
        ios_dir.display()
    );
    println!("\nNext steps:");
    println!("  Run: orbital build ios");

    Ok(())
}

fn generate_project(ios_dir: &Path, config: &config::IosConfig) -> Result<()> {
    let replacements = create_replacements(config);

    // Create directory structure
    fs::create_dir_all(ios_dir.join("Orbital.xcodeproj"))
        .context("Failed to create xcodeproj directory")?;
    fs::create_dir_all(ios_dir.join("Orbital").join("Assets.xcassets"))
        .context("Failed to create Assets.xcassets directory")?;
    fs::create_dir_all(
        ios_dir.join("Orbital").join("Assets.xcassets").join("AppIcon.appiconset"),
    )
    .context("Failed to create AppIcon.appiconset directory")?;

    // Write template files
    write_template_file(
        ios_dir.join("Orbital.xcodeproj").join("project.pbxproj"),
        PROJECT_PBXPROJ,
        &replacements,
    )?;
    write_template_file(
        ios_dir.join("Orbital").join("Info.plist"),
        INFO_PLIST,
        &replacements,
    )?;
    write_template_file(
        ios_dir.join("Orbital").join("AppDelegate.swift"),
        APP_DELEGATE_SWIFT,
        &replacements,
    )?;
    write_template_file(
        ios_dir.join("Orbital").join("LaunchScreen.storyboard"),
        LAUNCH_SCREEN_STORYBOARD,
        &replacements,
    )?;
    write_template_file(
        ios_dir
            .join("Orbital")
            .join("Assets.xcassets")
            .join("Contents.json"),
        ASSETS_CONTENTS_JSON,
        &replacements,
    )?;
    write_template_file(
        ios_dir
            .join("Orbital")
            .join("Assets.xcassets")
            .join("AppIcon.appiconset")
            .join("Contents.json"),
        APPICON_CONTENTS_JSON,
        &replacements,
    )?;

    Ok(())
}

fn create_replacements(config: &config::IosConfig) -> Vec<(String, String)> {
    vec![
        ("@@@BUNDLE_ID@@@".to_string(), config.bundle_id().to_string()),
        (
            "@@@DEPLOYMENT_TARGET@@@".to_string(),
            config.deployment_target().to_string(),
        ),
        ("@@@APP_NAME@@@".to_string(), config.app_name().to_string()),
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

// Template files — pbxproj loaded from file to avoid Rust string escaping issues
const PROJECT_PBXPROJ: &str = include_str!("../template/ios/project.pbxproj");

const INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>$(DEVELOPMENT_LANGUAGE)</string>
	<key>CFBundleExecutable</key>
	<string>$(EXECUTABLE_NAME)</string>
	<key>CFBundleIdentifier</key>
	<string>@@@BUNDLE_ID@@@</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>@@@APP_NAME@@@</string>
	<key>CFBundlePackageType</key>
	<string>$(PRODUCT_BUNDLE_PACKAGE_TYPE)</string>
	<key>CFBundleShortVersionString</key>
	<string>$(MARKETING_VERSION)</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>LSRequiresIPhoneOS</key>
	<true/>
	<key>UIApplicationSceneManifest</key>
	<dict>
		<key>UIApplicationSupportsMultipleScenes</key>
		<false/>
	</dict>
	<key>UILaunchStoryboardName</key>
	<string>LaunchScreen</string>
	<key>UIRequiredDeviceCapabilities</key>
	<array>
		<string>arm64</string>
		<string>metal</string>
	</array>
	<key>UIRequiresFullScreen</key>
	<true/>
	<key>UISupportedInterfaceOrientations</key>
	<array>
		<string>UIInterfaceOrientationLandscapeLeft</string>
	</array>
	<key>UIStatusBarHidden</key>
	<false/>
</dict>
</plist>
"#;

const APP_DELEGATE_SWIFT: &str = r#"import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        window = UIWindow(frame: UIScreen.main.bounds)
        window?.rootViewController = UIViewController()
        window?.makeKeyAndVisible()
        return true
    }
}
"#;

const LAUNCH_SCREEN_STORYBOARD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<document type="com.apple.InterfaceBuilder3.CocoaTouch.Storyboard.XIB" version="3.0" toolsVersion="21701" targetRuntime="AppleSDK" propertyAccessControl="none" useAutolayout="YES" launchScreen="YES" useTraitCollections="YES" useSafeAreas="YES" colorMatched="YES" initialViewController="01J-lp-oVM">
    <scenes>
        <scene sceneID="EHf-IW-A2E">
            <objects>
                <viewController id="01J-lp-oVM" sceneMemberID="viewController">
                    <view key="view" contentMode="scaleToFill" id="Ze5-6b-2t3">
                        <rect key="frame" x="0.0" y="0.0" width="393" height="852"/>
                        <autoresizingMask key="autoresizingMask" widthSizable="YES" heightSizable="YES"/>
                        <subviews>
                            <label opaque="NO" userInteractionEnabled="NO" contentMode="left" horizontalHuggingPriority="251" verticalHuggingPriority="251" text="@@@APP_NAME@@@" textAlignment="center" lineBreakMode="tailTruncation" baselineAdjustment="alignBaselines" adjustsFontSizeToFit="NO" translatesAutoresizingMaskIntoConstraints="NO" id="lbl-01">
                                <fontDescription key="fontDescription" type="system" pointSize="32"/>
                                <color key="textColor" white="1" alpha="1" colorSpace="custom" customColorSpace="genericGamma22GrayColorSpace"/>
                            </label>
                        </subviews>
                        <viewLayoutGuide key="safeArea" id="6Tk-OE-BBY"/>
                        <color key="backgroundColor" red="0.1" green="0.1" blue="0.12" alpha="1" colorSpace="custom" customColorSpace="sRGB"/>
                        <constraints>
                            <constraint firstItem="lbl-01" firstAttribute="centerX" secondItem="Ze5-6b-2t3" secondAttribute="centerX" id="c1"/>
                            <constraint firstItem="lbl-01" firstAttribute="centerY" secondItem="Ze5-6b-2t3" secondAttribute="centerY" id="c2"/>
                        </constraints>
                    </view>
                </viewController>
                <placeholder placeholderIdentifier="IBFirstResponder" id="iYj-Kq-Ea1" userLabel="First Responder" sceneMemberID="firstResponder"/>
            </objects>
            <point key="canvasLocation" x="53" y="375"/>
        </scene>
    </scenes>
</document>
"#;

const ASSETS_CONTENTS_JSON: &str = r#"{
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
"#;

const APPICON_CONTENTS_JSON: &str = r#"{
  "images" : [
    {
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
"#;
