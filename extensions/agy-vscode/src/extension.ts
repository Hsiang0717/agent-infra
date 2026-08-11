import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {

    // ─── Existing: Open AGY CLI ───────────────────────────────────────────────

    const startDisposable = vscode.commands.registerCommand('agy.start', async () => {

        const existingTerminal = vscode.window.terminals.find(t => t.name === 'AGY CLI');
        if (existingTerminal) {
            existingTerminal.dispose();
        }

        const workspaceFolders = vscode.workspace.workspaceFolders;
        let venvPath: string | undefined;

        if (workspaceFolders && workspaceFolders.length > 0) {
            const workspaceRoot = workspaceFolders[0].uri.fsPath;
            const possibleVenvPath = path.join(workspaceRoot, '.venv');
            if (fs.existsSync(possibleVenvPath)) {
                venvPath = possibleVenvPath;
            }
        }

        const isWin = process.platform === 'win32';
        let shellPath = isWin ? 'powershell.exe' : '/bin/bash';

        if (isWin) {
            const envPaths = (process.env.PATH || '').split(path.delimiter);
            const hasPwsh = envPaths.some(p => {
                try {
                    return fs.existsSync(path.join(p, 'pwsh.exe')) || fs.existsSync(path.join(p, 'pwsh'));
                } catch {
                    return false;
                }
            });
            if (hasPwsh) { shellPath = 'pwsh'; }
        }

        const env: { [key: string]: string } = {
            VIRTUAL_ENV: venvPath || 'prevent-activation',
            CONDA_PREFIX: 'prevent-activation'
        };

        const terminal = vscode.window.createTerminal({
            name: 'AGY CLI',
            location: vscode.TerminalLocation.Editor,
            shellPath,
            shellArgs: isWin ? ['-NoProfile', '-ExecutionPolicy', 'Bypass'] : [],
            env
        });

        terminal.show(false);
        await delay(800);

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

        terminal.sendText(startCommand, true);
        await delay(100);
        try {
            await vscode.commands.executeCommand('workbench.action.moveEditorToNextGroup');
        } catch {
            // Ignore error if moving editor group is not applicable
        }
    });

    context.subscriptions.push(startDisposable);

}

function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export function deactivate() {}