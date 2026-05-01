import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('geminicli.start', async () => {
        // Find existing Gemini CLI terminal or create a new one
        let terminal = vscode.window.terminals.find(t => t.name === 'Gemini CLI');
        
        if (!terminal) {
            // Create terminal directly in the Editor area, split to the right
            terminal = vscode.window.createTerminal({
                name: 'Gemini CLI',
                location: vscode.TerminalLocation.Editor,
            });
            
            terminal.sendText('gemini');
        }
        
        // Show terminal in the 'Beside' column (right side)
        terminal.show(true); 
        await vscode.commands.executeCommand('workbench.action.terminal.moveToEditor');
        await vscode.commands.executeCommand('workbench.action.moveEditorToNextGroup');
    });

    context.subscriptions.push(disposable);
}

export function deactivate() {}
