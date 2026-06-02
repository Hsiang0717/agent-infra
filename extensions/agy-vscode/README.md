# AGY CLI VS Code Extension

This extension provides a quick way to launch the AGY CLI directly within VS Code.

## Features

- **Quick Launch Icon**: Adds a robot icon to the editor title bar (top right).
- **Secondary Sidebar Support**: Automatically attempts to move the AGY CLI terminal to the secondary sidebar for a non-intrusive experience.
- **Native Terminal**: Uses VS Code's high-performance native terminal.

## How to Use

1. Click the robot icon in the top right corner of any editor.
2. Alternatively, open the Command Palette (`Ctrl+Shift+P`) and type `Open AGY CLI`.
3. The terminal will open and automatically start the `agy` command.

## Development

- Press `F5` to open a new window with the extension loaded.
- The source code is located in `src/extension.ts`.