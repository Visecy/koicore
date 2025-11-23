#!/bin/bash

# Build script for KoiLang VSCode extension
echo "Building KoiLang VSCode extension..."

# Check if vsce is installed
if ! command -v vsce &> /dev/null; then
    echo "vsce is not installed. Installing via npm..."
    npm install -g vsce
fi

# Package the extension
echo "Packaging extension..."
vsce package

echo "Extension built successfully!"
echo "You can now install the .vsix file in VSCode."