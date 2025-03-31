#!/usr/bin/env bash

# Settings
ANDROID_TEMPLATE_DIR="Android/"
TARGET_DIR="target/"
ANDROID_PROJECT_DESTINATION="$TARGET_DIR/AndroidProject"

./android-build.sh $@
if [ $? -ne 0 ]; then
    exit -1
fi

# Build
cd "$ANDROID_PROJECT_DESTINATION"
./gradlew installDebug
