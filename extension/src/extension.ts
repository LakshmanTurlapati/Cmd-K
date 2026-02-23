import * as vscode from 'vscode';
import { TerminalManager } from './terminalManager';
import { SettingsPanel } from './webview/settingsPanel';
import { TerminalOverlay } from './webview/terminalOverlay';

let terminalManager: TerminalManager;
let terminalOverlay: TerminalOverlay;

export function activate(context: vscode.ExtensionContext) {
  console.log('CMD+K extension is now active');

  // Initialize managers
  const config = vscode.workspace.getConfiguration('terminalAI');
  const maxHistoryLines = config.get<number>('maxHistoryLines', 20);
  terminalManager = new TerminalManager(maxHistoryLines);
  terminalOverlay = new TerminalOverlay(context.extensionUri, terminalManager);

  // Listen for configuration changes to refresh overlay
  const configChangeDisposable = vscode.workspace.onDidChangeConfiguration((e) => {
    if (e.affectsConfiguration('terminalAI')) {
      terminalOverlay.refreshProviderInfo();
    }
  });

  // Register generate command
  const generateCommand = vscode.commands.registerCommand(
    'terminalAI.generateCommand',
    async () => {
      await handleGenerateCommand();
    }
  );

  // Register settings command
  const settingsCommand = vscode.commands.registerCommand(
    'terminalAI.openSettings',
    () => {
      SettingsPanel.createOrShow(context.extensionUri);
    }
  );

  // Listen for terminal close events to cleanup history
  const terminalCloseDisposable = vscode.window.onDidCloseTerminal((terminal) => {
    terminalManager.cleanupTerminal(terminal);
  });

  context.subscriptions.push(
    generateCommand,
    settingsCommand,
    terminalCloseDisposable,
    configChangeDisposable
  );
}

async function handleGenerateCommand() {
  try {
    // Check if terminal is active
    const activeTerminal = vscode.window.activeTerminal;
    if (!activeTerminal) {
      vscode.window.showErrorMessage('No active terminal. Please open a terminal first.');
      return;
    }

    // Show overlay
    await terminalOverlay.show(activeTerminal);
  } catch (error: any) {
    vscode.window.showErrorMessage(`CMD+K Error: ${error.message || 'Unknown error'}`);
    console.error('CMD+K Error:', error);
  }
}

export function deactivate() {
  // Cleanup if needed
}
