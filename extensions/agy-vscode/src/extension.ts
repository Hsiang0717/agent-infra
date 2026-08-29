import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {
    const startDisposable = vscode.commands.registerCommand('agy.start', async () => {
        const config = vscode.workspace.getConfiguration('agy');
        const reuseTerminal = config.get<boolean>('reuseTerminal', false);
        const terminalLocationSetting = config.get<string>('terminalLocation', 'editor');
        const customPath = config.get<string>('customPath', '').trim();
        const customArguments = config.get<string>('customArguments', '').trim();
        const customVenvPath = config.get<string>('venvPath', '').trim();

        const existingTerminal = vscode.window.terminals.find(t => t.name === 'AGY CLI');
        if (existingTerminal) {
            if (reuseTerminal) {
                existingTerminal.show(false);
                return;
            }
            existingTerminal.dispose();
        }

        const workspaceFolders = vscode.workspace.workspaceFolders;
        const workspaceRoot = workspaceFolders && workspaceFolders.length > 0 ? workspaceFolders[0].uri.fsPath : undefined;
        const venvPath = resolveVenvPath(workspaceRoot, customVenvPath);

        const isWin = process.platform === 'win32';
        const shellPath = resolveShellPath(isWin);

        const env: { [key: string]: string } = {
            VIRTUAL_ENV: venvPath || 'prevent-activation',
            CONDA_PREFIX: 'prevent-activation'
        };

        const terminalLocation = terminalLocationSetting === 'panel'
            ? vscode.TerminalLocation.Panel
            : vscode.TerminalLocation.Editor;

        const terminal = vscode.window.createTerminal({
            name: 'AGY CLI',
            location: terminalLocation,
            shellPath,
            shellArgs: isWin ? ['-NoProfile', '-ExecutionPolicy', 'Bypass'] : [],
            env
        });

        terminal.show(false);

        const agyExecutable = customPath || 'agy';
        const fullAgyCmd = customArguments ? `${agyExecutable} ${customArguments}` : agyExecutable;

        let startCommand = fullAgyCmd;
        if (venvPath) {
            if (isWin) {
                const activateScript = path.join(venvPath, 'Scripts', 'Activate.ps1');
                if (fs.existsSync(activateScript)) {
                    startCommand = `& "${activateScript}"; ${fullAgyCmd}`;
                }
            } else {
                const activateScript = path.join(venvPath, 'bin', 'activate');
                if (fs.existsSync(activateScript)) {
                    startCommand = `. "${activateScript}" && ${fullAgyCmd}`;
                }
            }
        }

        terminal.sendText(startCommand, true);

        if (terminalLocation === vscode.TerminalLocation.Editor) {
            try {
                await vscode.commands.executeCommand('workbench.action.moveEditorToNextGroup');
            } catch {
                // Ignore error if moving editor group is not applicable
            }
        }
    });

    context.subscriptions.push(startDisposable);
}

function resolveVenvPath(workspaceRoot?: string, customVenvPath?: string): string | undefined {
    if (customVenvPath) {
        const resolved = workspaceRoot && !path.isAbsolute(customVenvPath)
            ? path.join(workspaceRoot, customVenvPath)
            : customVenvPath;
        if (fs.existsSync(resolved)) {
            return resolved;
        }
    }

    if (workspaceRoot) {
        const candidatePaths = [
            path.join(workspaceRoot, '.venv'),
            path.join(workspaceRoot, 'venv')
        ];
        for (const candidate of candidatePaths) {
            if (fs.existsSync(candidate)) {
                return candidate;
            }
        }
    }

    // Check VS Code Python extension default interpreter path if present
    const pythonConfig = vscode.workspace.getConfiguration('python');
    const defaultInterpreter = pythonConfig.get<string>('defaultInterpreterPath');
    if (defaultInterpreter && fs.existsSync(defaultInterpreter)) {
        // e.g. .venv/Scripts/python.exe -> .venv
        const possibleVenv = path.dirname(path.dirname(defaultInterpreter));
        const hasScriptsOrBin = fs.existsSync(path.join(possibleVenv, 'Scripts')) || fs.existsSync(path.join(possibleVenv, 'bin'));
        if (hasScriptsOrBin) {
            return possibleVenv;
        }
    }

    return undefined;
}

function resolveShellPath(isWin: boolean): string {
    if (!isWin) {
        return process.env.SHELL || '/bin/bash';
    }

    const envPaths = (process.env.PATH || '').split(path.delimiter);
    const hasPwsh = envPaths.some(p => {
        try {
            return fs.existsSync(path.join(p, 'pwsh.exe')) || fs.existsSync(path.join(p, 'pwsh'));
        } catch {
            return false;
        }
    });

    return hasPwsh ? 'pwsh' : 'powershell.exe';
}

export function deactivate() {}