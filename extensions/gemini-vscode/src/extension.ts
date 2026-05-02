import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('geminicli.start', async () => {
        // Find existing Gemini CLI terminal or create a new one
        let terminal = vscode.window.terminals.find(t => t.name === 'Gemini CLI');
        let isNew = false;
        
        if (!terminal) {
            isNew = true;
            // Create terminal directly in the Editor area
            terminal = vscode.window.createTerminal({
                name: 'Gemini CLI',
                location: vscode.TerminalLocation.Editor,
            });
            
            terminal.sendText('gemini');
        }
        
        // Show terminal and take focus so that subsequent workbench commands act on it
        terminal.show(false); 

        // If it's a new terminal, move it to the right (Next Group)
        if (isNew) {
            await vscode.commands.executeCommand('workbench.action.moveEditorToNextGroup');
        }
    });

    context.subscriptions.push(disposable);
}

export function deactivate() {}
