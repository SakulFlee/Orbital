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

// Template files as static strings

const PROJECT_PBXPROJ: &str = r#"// !$*UTF8*$!
{
	archiveVersion = 1;
	classes = {
	};
	objectVersion = 56;
	objects = {

/* Begin PBXBuildFile section */
		AA000001 /* AppDelegate.swift in Sources */ = {isa = PBXBuildFile; fileRef = AA000002 /* AppDelegate.swift */; };
		AA000003 /* Assets.xcassets in Resources */ = {isa = PBXBuildFile; fileRef = AA000004 /* Assets.xcassets */; };
		AA000005 /* LaunchScreen.storyboard in Resources */ = {isa = PBXBuildFile; fileRef = AA000006 /* LaunchScreen.storyboard */; };
		AA000007 /* lib@@@APP_NAME@@@.a in Frameworks */ = {isa = PBXBuildFile; fileRef = AA000008 /* lib@@@APP_NAME@@@.a */; };
/* End PBXBuildFile section */

/* Begin PBXFileReference section */
		AA000002 /* AppDelegate.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = AppDelegate.swift; sourceTree = "<group>"; };
		AA000004 /* Assets.xcassets */ = {isa = PBXFileReference; lastKnownFileType = folder.assetcatalog; path = Assets.xcassets; sourceTree = "<group>"; };
		AA000006 /* LaunchScreen.storyboard */ = {isa = PBXFileReference; lastKnownFileType = file.storyboard; path = LaunchScreen.storyboard; sourceTree = "<group>"; };
		AA000008 /* lib@@@APP_NAME@@@.a */ = {isa = PBXFileReference; lastKnownFileType = archive.ar; name = "lib@@@APP_NAME@@@.a"; path = "target/aarch64-apple-ios/release/lib@@@APP_NAME@@@.a"; sourceTree = SOURCE_ROOT; };
		AA000009 /* Info.plist */ = {isa = PBXFileReference; lastKnownFileType = text.plist.xml; path = Info.plist; sourceTree = "<group>"; };
		AA000010 /* @@@APP_NAME@@@.app */ = {isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = "@@@APP_NAME@@@.app"; sourceTree = BUILT_PRODUCTS_DIR; };
/* End PBXFileReference section */

/* Begin PBXFrameworksBuildPhase section */
		AA000011 /* Frameworks */ = {
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AA000007 /* lib@@@APP_NAME@@@.a in Frameworks */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXFrameworksBuildPhase section */

/* Begin PBXGroup section */
		AA000012 = {
			isa = PBXGroup;
			children = (
				AA000013 /* @@@APP_NAME@@@ */,
				AA000014 /* Products */,
			);
			sourceTree = "<group>";
		};
		AA000013 /* @@@APP_NAME@@@ */ = {
			isa = PBXGroup;
			children = (
				AA000002 /* AppDelegate.swift */,
				AA000004 /* Assets.xcassets */,
				AA000006 /* LaunchScreen.storyboard */,
				AA000008 /* lib@@@APP_NAME@@@.a */,
				AA000009 /* Info.plist */,
			);
			path = @@@APP_NAME@@@;
			sourceTree = "<group>";
		};
		AA000014 /* Products */ = {
			isa = PBXGroup;
			children = (
				AA000010 /* @@@APP_NAME@@@.app */,
			);
			name = Products;
			sourceTree = "<group>";
		};
/* End PBXGroup section */

/* Begin PBXNativeTarget section */
		AA000015 /* @@@APP_NAME@@@ */ = {
			isa = PBXNativeTarget;
			buildConfigurationList = AA000016 /* Build configuration list for PBXNativeTarget "@@@APP_NAME@@@" */;
			buildPhases = (
				AA000017 /* Build Rust library */,
				AA000018 /* Sources */,
				AA000011 /* Frameworks */,
				AA000019 /* Resources */,
			);
			buildRules = (
			);
			dependencies = (
			);
			name = "@@@APP_NAME@@@";
			productName = "@@@APP_NAME@@@";
			productReference = AA000010 /* @@@APP_NAME@@@.app */;
			productType = "com.apple.product-type.application";
		};
/* End PBXNativeTarget section */

/* Begin PBXProject section */
		AA000020 /* Project object */ = {
			isa = PBXProject;
			attributes = {
				BuildIndependentTargetsInParallel = 1;
				LastSwiftUpdateCheck = 1500;
				LastUpgradeCheck = 1500;
				TargetAttributes = {
					AA000015 = {
						CreatedOnToolsVersion = 15.0;
					};
				};
			};
			buildConfigurationList = AA000021 /* Build configuration list for PBXProject "@@@APP_NAME@@@" */;
			compatibilityVersion = "Xcode 14.0";
			developmentRegion = en;
			hasScannedForEncodings = 0;
			knownRegions = (
				en,
				Base,
			);
			mainGroup = AA000012;
			productRefGroup = AA000014 /* Products */;
			projectDirPath = "";
			projectRoot = "";
			targets = (
				AA000015 /* @@@APP_NAME@@@ */,
			);
		};
/* End PBXProject section */

/* Begin PBXResourcesBuildPhase section */
		AA000019 /* Resources */ = {
			isa = PBXResourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AA000003 /* Assets.xcassets in Resources */,
				AA000005 /* LaunchScreen.storyboard in Resources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXResourcesBuildPhase section */

/* Begin PBXShellScriptBuildPhase section */
		AA000017 /* Build Rust library */ = {
			isa = PBXShellScriptBuildPhase;
			buildActionMask = 2147483647;
			files = (
			);
			inputFileListPaths = (
			);
			inputPaths = (
			);
			name = "Build Rust library";
			outputFileListPaths = (
			);
			outputPaths = (
			);
			runOnlyForDeploymentPostprocessing = 0;
			shellPath = /bin/sh;
			shellScript = "# Build the Rust static library for iOS\nif command -v cargo &> /dev/null; then\n    echo \"Building Rust library...\"\n    cargo build --lib --target aarch64-apple-ios --release\n    if [ $? -ne 0 ]; then\n        echo \"error: Rust build failed\"\n        exit 1\n    fi\n    # Copy the static library to the derived data directory\n    cp \"${SRCROOT}/target/aarch64-apple-ios/release/lib${PRODUCT_NAME}.a\" \"${BUILT_PRODUCTS_DIR}/\"\nelse\n    echo \"error: cargo not found. Install Rust from https://rustup.rs\"\n    exit 1\nfi\n";
		};
/* End PBXShellScriptBuildPhase section */

/* Begin PBXSourcesBuildPhase section */
		AA000018 /* Sources */ = {
			isa = PBXSourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AA000001 /* AppDelegate.swift in Sources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		};
/* End PBXSourcesBuildPhase section */

/* Begin XCBuildConfiguration section */
		AA000022 /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
				CLANG_ENABLE_MODULES = YES;
				CODE_SIGN_STYLE = Automatic;
				CURRENT_PROJECT_VERSION = 1;
				GCC_OPTIMIZATION_LEVEL = 0;
				GCC_PREPROCESSOR_DEFINITIONS = (
					"DEBUG=1",
					"$(inherited)",
				);
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = "@@@APP_NAME@@@/Info.plist";
				INFOPLIST_KEY_UIApplicationSupportsIndirectInputEvents = YES;
				INFOPLIST_KEY_UILaunchStoryboardName = LaunchScreen;
				INFOPLIST_KEY_UIRequiredDeviceCapabilities = arm64;
				INFOPLIST_KEY_UIStatusBarHidden = NO;
				INFOPLIST_KEY_UISupportedInterfaceOrientations = UIInterfaceOrientationLandscapeLeft;
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = 1.0;
				MTL_ENABLE_DEBUG_INFO = INCLUDE_SOURCE;
				ONLY_ACTIVE_ARCH = YES;
				SDKROOT = iphoneos;
				SWIFT_ACTIVE_COMPILATION_CONDITIONS = DEBUG;
				SWIFT_OPTIMIZATION_LEVEL = "-Onone";
				TARGETED_DEVICE_FAMILY = "1,2";
			};
			name = Debug;
		};
		AA000023 /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				ALWAYS_SEARCH_USER_PATHS = NO;
				ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;
				CLANG_ENABLE_MODULES = YES;
				CODE_SIGN_STYLE = Automatic;
				CURRENT_PROJECT_VERSION = 1;
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = "@@@APP_NAME@@@/Info.plist";
				INFOPLIST_KEY_UIApplicationSupportsIndirectInputEvents = YES;
				INFOPLIST_KEY_UILaunchStoryboardName = LaunchScreen;
				INFOPLIST_KEY_UIRequiredDeviceCapabilities = arm64;
				INFOPLIST_KEY_UIStatusBarHidden = NO;
				INFOPLIST_KEY_UISupportedInterfaceOrientations = UIInterfaceOrientationLandscapeLeft;
				LD_RUNPATH_SEARCH_PATHS = (
					"$(inherited)",
					"@executable_path/Frameworks",
				);
				MARKETING_VERSION = 1.0;
				MTL_ENABLE_DEBUG_INFO = NO;
				SDKROOT = iphoneos;
				SWIFT_COMPILATION_MODE = wholemodule;
				SWIFT_OPTIMIZATION_LEVEL = "-O";
				TARGETED_DEVICE_FAMILY = "1,2";
				VALIDATE_PRODUCT = YES;
			};
			name = Release;
		};
		AA000016 /* Build configuration list for PBXNativeTarget "@@@APP_NAME@@@" */ = {
			isa = XCConfigurationList;
			buildConfigurations = (
				AA000022 /* Debug */,
				AA000023 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		};
		AA000024 /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				CLANG_ANALYZER_NONNULL = YES;
				CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION = YES_AGGRESSIVE;
				CLANG_CXX_LANGUAGE_STANDARD = "gnu++20";
				CLANG_ENABLE_MODULES = YES;
				CLANG_ENABLE_OBJC_ARC = YES;
				CLANG_ENABLE_OBJC_WEAK = YES;
				CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING = YES;
				CLANG_WARN_BOOL_CONVERSION = YES;
				CLANG_WARN_COMMA = YES;
				CLANG_WARN_CONSTANT_CONVERSION = YES;
				CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS = YES;
				CLANG_WARN_DIRECT_OBJC_ISA_USAGE = YES_ERROR;
				CLANG_WARN_DOCUMENTATION_COMMENTS = YES;
				CLANG_WARN_EMPTY_BODY = YES;
				CLANG_WARN_ENUM_CONVERSION = YES;
				CLANG_WARN_INFINITE_RECURSION = YES;
				CLANG_WARN_INT_CONVERSION = YES;
				CLANG_WARN_NON_LITERAL_NULL_CONVERSION = YES;
				CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES;
				CLANG_WARN_OBJC_LITERAL_CONVERSION = YES;
				CLANG_WARN_OBJC_ROOT_CLASS = YES_ERROR;
				CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER = YES;
				CLANG_WARN_RANGE_LOOP_ANALYSIS = YES;
				CLANG_WARN_STRICT_PROTOTYPES = YES;
				CLANG_WARN_SUSPICIOUS_MOVE = YES;
				CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE;
				CLANG_WARN_UNREACHABLE_CODE = YES;
				CLANG_WARN__DUPLICATE_METHOD_MATCH = YES;
				COPY_PHASE_STRIP = NO;
				DEBUG_INFORMATION_FORMAT = dwarf;
				ENABLE_STRICT_OBJC_MSGSEND = YES;
				ENABLE_TESTABILITY = YES;
				ENABLE_USER_SCRIPT_SANDBOXING = NO;
				GCC_C_LANGUAGE_STANDARD = gnu17;
				GCC_DYNAMIC_NO_PIC = NO;
				GCC_NO_COMMON_BLOCKS = YES;
				GCC_OPTIMIZATION_LEVEL = 0;
				GCC_PREPROCESSOR_DEFINITIONS = (
					"DEBUG=1",
					"$(inherited)",
				);
				GCC_WARN_64_TO_32_BIT_CONVERSION = YES;
				GCC_WARN_ABOUT_RETURN_TYPE = YES_ERROR;
				GCC_WARN_UNDECLARED_SELECTOR = YES;
				GCC_WARN_UNINITIALIZED_AUTOS = YES_AGGRESSIVE;
				GCC_WARN_UNUSED_FUNCTION = YES;
				GCC_WARN_UNUSED_VARIABLE = YES;
				IPHONEOS_DEPLOYMENT_TARGET = @@@DEPLOYMENT_TARGET@@@;
				MTL_ENABLE_DEBUG_INFO = INCLUDE_SOURCE;
				MTL_FAST_MATH = YES;
				ONLY_ACTIVE_ARCH = YES;
				SDKROOT = iphoneos;
				SWIFT_ACTIVE_COMPILATION_CONDITIONS = "DEBUG $(inherited)";
				SWIFT_OPTIMIZATION_LEVEL = "-Onone";
			};
			name = Debug;
		};
		AA000025 /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				CLANG_ANALYZER_NONNULL = YES;
				CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION = YES_AGGRESSIVE;
				CLANG_CXX_LANGUAGE_STANDARD = "gnu++20";
				CLANG_ENABLE_MODULES = YES;
				CLANG_ENABLE_OBJC_ARC = YES;
				CLANG_ENABLE_OBJC_WEAK = YES;
				CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING = YES;
				CLANG_WARN_BOOL_CONVERSION = YES;
				CLANG_WARN_COMMA = YES;
				CLANG_WARN_CONSTANT_CONVERSION = YES;
				CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS = YES;
				CLANG_WARN_DIRECT_OBJC_ISA_USAGE = YES_ERROR;
				CLANG_WARN_DOCUMENTATION_COMMENTS = YES;
				CLANG_WARN_EMPTY_BODY = YES;
				CLANG_WARN_ENUM_CONVERSION = YES;
				CLANG_WARN_INFINITE_RECURSION = YES;
				CLANG_WARN_INT_CONVERSION = YES;
				CLANG_WARN_NON_LITERAL_NULL_CONVERSION = YES;
				CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF = YES;
				CLANG_WARN_OBJC_LITERAL_CONVERSION = YES;
				CLANG_WARN_OBJC_ROOT_CLASS = YES_ERROR;
				CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER = YES;
				CLANG_WARN_RANGE_LOOP_ANALYSIS = YES;
				CLANG_WARN_STRICT_PROTOTYPES = YES;
				CLANG_WARN_SUSPICIOUS_MOVE = YES;
				CLANG_WARN_UNGUARDED_AVAILABILITY = YES_AGGRESSIVE;
				CLANG_WARN_UNREACHABLE_CODE = YES;
				CLANG_WARN__DUPLICATE_METHOD_MATCH = YES;
				COPY_PHASE_STRIP = NO;
				DEBUG_INFORMATION_FORMAT = "dwarf-with-dsym";
				ENABLE_NS_ASSERTIONS = NO;
				ENABLE_STRICT_OBJC_MSGSEND = YES;
				ENABLE_USER_SCRIPT_SANDBOXING = NO;
				GCC_C_LANGUAGE_STANDARD = gnu17;
				GCC_NO_COMMON_BLOCKS = YES;
				GCC_WARN_64_TO_32_BIT_CONVERSION = YES;
				GCC_WARN_ABOUT_RETURN_TYPE = YES_ERROR;
				GCC_WARN_UNDECLARED_SELECTOR = YES;
				GCC_WARN_UNINITIALIZED_AUTOS = YES_AGGRESSIVE;
				GCC_WARN_UNUSED_FUNCTION = YES;
				GCC_WARN_UNUSED_VARIABLE = YES;
				IPHONEOS_DEPLOYMENT_TARGET = @@@DEPLOYMENT_TARGET@@@;
				MTL_ENABLE_DEBUG_INFO = NO;
				MTL_FAST_MATH = YES;
				SDKROOT = iphoneos;
				SWIFT_COMPILATION_MODE = wholemodule;
				VALIDATE_PRODUCT = YES;
			};
			name = Release;
		};
		AA000021 /* Build configuration list for PBXProject "@@@APP_NAME@@@" */ = {
			isa = XCConfigurationList;
			buildConfigurations = (
				AA000024 /* Debug */,
				AA000025 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		};
/* End XCBuildConfiguration section */
	};
	rootObject = AA000020 /* Project object */;
}
"#;

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
