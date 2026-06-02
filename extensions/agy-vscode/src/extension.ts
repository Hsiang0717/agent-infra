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

        // Build environment variables (keep VIRTUAL_ENV to prevent MS-Python from double-activating)
        const env: { [key: string]: string } = {
            VIRTUAL_ENV: venvPath || 'prevent-activation',
            CONDA_PREFIX: 'prevent-activation'
        };

        // Create clean PowerShell terminal
        const terminal = vscode.window.createTerminal({
            name: 'AGY CLI',
            location: vscode.TerminalLocation.Editor,
            shellPath: 'pwsh',
            shellArgs: ['-NoProfile'],
            env: env
        });

        // Show terminal
        terminal.show(false);

        // Allow shell process to initialize (prevent PSReadLine from swallowing the newline)
        await delay(1000);

        // Build startup command: activate venv first (if present), then run agy
        let startCommand = 'agy';
        if (venvPath) {
            if (process.platform === 'win32') {
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