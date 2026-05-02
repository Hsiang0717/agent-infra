import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('geminicli.start', async () => {
        // Aggressive strategy: dispose any existing Gemini CLI terminal to ensure a fresh state
        let existingTerminal = vscode.window.terminals.find(t => t.name === 'Gemini CLI');
        if (existingTerminal) {
            existingTerminal.dispose();
        }

        // Create a new terminal in the Editor area
        const terminal = vscode.window.createTerminal({
            name: 'Gemini CLI',
            location: vscode.TerminalLocation.Editor,
        });
        
        terminal.sendText('gemini');
        terminal.show(false); 

        // Move to next group with enough delay (200ms)
        await new Promise(resolve => setTimeout(resolve, 200));
        await vscode.commands.executeCommand('workbench.action.moveEditorToNextGroup');
    });

    context.subscriptions.push(disposable);
}

export function deactivate() {}
