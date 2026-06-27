import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {

    let disposable = vscode.commands.registerCommand('agy.start', async () => {

        // Dispose existing terminal
        const existingTerminal = vscode.window.terminals.find(
            t => t.name === 'AGY CLI'
        );

        if (existingTerminal) {
            existingTerminal.dispose();
        }

        // Detect workspace root and check for .venv
        const workspaceFolders = vscode.workspace.workspaceFolders;
        let venvPath: string | undefined;

        if (workspaceFolders && workspaceFolders.length > 0) {
            const workspaceRoot = workspaceFolders[0].uri.fsPath;
            const possibleVenvPath = path.join(workspaceRoot, '.venv');
            if (fs.existsSync(possibleVenvPath)) {
                venvPath = possibleVenvPath;
            }
        }

        // Determine shell based on platform
        const isWin = process.platform === 'win32';
        let shellPath = isWin ? 'powershell.exe' : '/bin/bash';

        // Check if pwsh is available on Windows
        if (isWin) {
            const envPaths = (process.env.PATH || '').split(path.delimiter);
            const hasPwsh = envPaths.some(p => {
                try {
                    return fs.existsSync(path.join(p, 'pwsh.exe')) || fs.existsSync(path.join(p, 'pwsh'));
                } catch {
                    return false;
                }
            });
            if (hasPwsh) {
                shellPath = 'pwsh';
            }
        }

        // Build environment variables (prevent double-activation conflicts)
        const env: { [key: string]: string } = {
            VIRTUAL_ENV: venvPath || 'prevent-activation',
            CONDA_PREFIX: 'prevent-activation'
        };

        // Create clean terminal
        const terminal = vscode.window.createTerminal({
            name: 'AGY CLI',
            location: vscode.TerminalLocation.Editor,
            shellPath: shellPath,
            shellArgs: isWin ? ['-NoProfile'] : [],
            env: env
        });

        // Show terminal
        terminal.show(false);

        // Allow shell process to initialize
        await delay(800);

        // Build startup command: activate venv first (if present), then run agy
        let startCommand = 'agy';
        if (venvPath) {
            if (isWin) {
                const activateScript = path.join(venvPath, 'Scripts', 'Activate.ps1');
                startCommand = `& "${activateScript}"; agy`;
            } else {
                const activateScript = path.join(venvPath, 'bin', 'activate');
                startCommand = `. "${activateScript}" && agy`;
            }
        }

        // Send command and execute automatically
        terminal.sendText(startCommand, true);

        // Move terminal editor group
        await delay(100);

        await vscode.commands.executeCommand(
            'workbench.action.moveEditorToNextGroup'
        );
    });

    context.subscriptions.push(disposable);
}

function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export function deactivate() {}