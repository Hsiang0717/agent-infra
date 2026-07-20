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
            shellArgs: isWin ? ['-NoProfile'] : [],
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
        await vscode.commands.executeCommand('workbench.action.moveEditorToNextGroup');
    });

    context.subscriptions.push(startDisposable);

    // ─── Intent Conflict: Wrap selection as merge conflict skeleton ───────────

    const wrapConflictDisposable = vscode.commands.registerCommand('agy.wrapAsConflict', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) { return; }

        const selection = editor.selection;
        const selectedText = editor.document.getText(selection);
        const fileName = path.basename(editor.document.fileName);
        const startLine = selection.start.line + 1; // 1-indexed for display

        const conflictBlock =
            `<<<<<<< Current (Line ${startLine}, ${fileName})\n` +
            selectedText +
            `\n=======\n` +
            `>>>>>>> Incoming`;

        await editor.edit(editBuilder => {
            editBuilder.replace(selection, conflictBlock);
        });

        // Move cursor to just after ======= (the blank intent line)
        const insertedLines = conflictBlock.split('\n');
        const intentLine = selection.start.line + insertedLines.length - 2; // line after =======
        const intentPos = new vscode.Position(intentLine, 0);
        editor.selection = new vscode.Selection(intentPos, intentPos);
        editor.revealRange(new vscode.Range(intentPos, intentPos));
    });

    context.subscriptions.push(wrapConflictDisposable);

    // ─── Intent Diff: Copy conflict blocks (with line numbers) as prompt ────────

    const copyDiffDisposable = vscode.commands.registerCommand('agy.copyAsIntentDiff', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showWarningMessage('No active editor to copy from.');
            return;
        }

        const doc = editor.document;
        const relPath = vscode.workspace.asRelativePath(doc.fileName);
        const lines = doc.getText().split('\n');

        // Extract each conflict block with its 1-indexed start line
        const blocks: string[] = [];
        let inBlock = false;
        let blockStart = 0;
        let blockLines: string[] = [];

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            if (line.startsWith('<<<<<<<')) {
                inBlock = true;
                blockStart = i + 1; // 1-indexed
                blockLines = [line];
            } else if (inBlock) {
                blockLines.push(line);
                if (line.startsWith('>>>>>>>')) {
                    blocks.push(`# Lines ${blockStart}–${i + 1}\n` + blockLines.join('\n'));
                    inBlock = false;
                    blockLines = [];
                }
            }
        }

        if (blocks.length === 0) {
            vscode.window.showWarningMessage('No conflict blocks found. Use Alt+- to wrap a selection first.');
            return;
        }

        const config = vscode.workspace.getConfiguration('agy.intentDiff');
        const prefixTemplate = config.get<string>('prefixTemplate', '').trim();

        const body = `File: ${relPath}\n\n` + blocks.join('\n\n');
        const finalOutput = prefixTemplate ? `${prefixTemplate}\n\n${body}` : body;

        await vscode.env.clipboard.writeText(finalOutput);
        vscode.window.showInformationMessage(`Copied ${blocks.length} conflict block(s) from ${relPath}`);
    });

    context.subscriptions.push(copyDiffDisposable);
}

function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export function deactivate() {}