#!/usr/bin/env bash

# Settings
ANDROID_TEMPLATE_DIR="Android/"
TARGET_DIR="target/"
ANDROID_PROJECT_DESTINATION="$TARGET_DIR/AndroidProject"

PATH_STRINGS="App/src/main/res/values/strings.xml"
PATH_MANIFEST="App/src/main/AndroidManifest.xml"
PATH_GRADLE="App/build.gradle"

TAG_CRATE="@@@CRATE_TAG@@@"
TAG_APP_NAME="@@@APP_NAME_TAG@@@"
TAG_EXAMPLE_FOLDER="@@@EXAMPLE_FOLDER@@@"

# Argument check
if [ $# -lt 3 ]; then
    echo "Not enough arguments!"
    echo "$0 <Example Crate Name> <Example Folder Name> \"<Android App name>\""
    echo "For example:"
    echo "$0 example_clear_screen ClearScreen \"ClearScreen Example (Akimo-Project)\""
    exit -1
fi

# Internal settings
CRATE_NAME="$1"
EXAMPLE_FOLDER="$2"
APP_NAME="$3"

echo "Crate name: $CRATE_NAME"
echo "Example folder: $EXAMPLE_FOLDER"
echo "App name: $APP_NAME"

# Ensure android project destination is empty
if [ -d "$ANDROID_PROJECT_DESTINATION" ]; then
    echo "Android project found at '$ANDROID_PROJECT_DESTINATION'"
    echo "Removing old project ..."

    rm -rf "$ANDROID_PROJECT_DESTINATION"
fi

# Copy template
cp -R "$ANDROID_TEMPLATE_DIR/." "$ANDROID_PROJECT_DESTINATION/"

function replace_tags_in_file {
    sed -i -e "s/$TAG_CRATE/$CRATE_NAME/g" "$1"
    sed -i -e "s/$TAG_APP_NAME/$APP_NAME/g" "$1"
    sed -i -e "s/$TAG_EXAMPLE_FOLDER/$EXAMPLE_FOLDER/g" "$1"
}

replace_tags_in_file "$ANDROID_PROJECT_DESTINATION/$PATH_STRINGS"
replace_tags_in_file "$ANDROID_PROJECT_DESTINATION/$PATH_MANIFEST"
replace_tags_in_file "$ANDROID_PROJECT_DESTINATION/$PATH_GRADLE"

# Build
cd "$ANDROID_PROJECT_DESTINATION"
./gradlew assembleDebug
