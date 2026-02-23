import * as vscode from 'vscode';

export interface TerminalInfo {
  shell: string;
  history: string[];
}

export class TerminalManager {
  private terminalHistory: Map<string, string[]> = new Map();
  private readonly maxHistorySize: number;

  constructor(maxHistorySize: number = 20) {
    this.maxHistorySize = maxHistorySize;
  }

  /**
   * Captures command when it's sent to terminal
   */
  captureCommand(terminal: vscode.Terminal, command: string): void {
    const terminalId = this.getTerminalId(terminal);

    if (!this.terminalHistory.has(terminalId)) {
      this.terminalHistory.set(terminalId, []);
    }

    const history = this.terminalHistory.get(terminalId)!;
    history.push(command.trim());

    // Keep only recent commands
    if (history.length > this.maxHistorySize) {
      history.shift();
    }
  }

  /**
   * Gets terminal history for context
   */
  getHistory(terminal: vscode.Terminal): string[] {
    const terminalId = this.getTerminalId(terminal);
    return this.terminalHistory.get(terminalId) || [];
  }

  /**
   * Detects the shell type being used
   */
  detectShell(terminal: vscode.Terminal): string {
    // Try to get shell from terminal creation options
    const shellPath = (terminal.creationOptions as any).shellPath;

    if (shellPath) {
      const shellName = this.extractShellName(shellPath);
      if (shellName) {
        return shellName;
      }
    }

    // Fallback: detect based on platform
    const platform = process.platform;
    if (platform === 'win32') {
      return 'powershell';
    } else if (platform === 'darwin') {
      return 'zsh'; // macOS default since Catalina
    } else {
      return 'bash'; // Linux default
    }
  }

  /**
   * Streams a text chunk to terminal without executing
   */
  streamTextChunk(terminal: vscode.Terminal, chunk: string): void {
    terminal.show();
    terminal.sendText(chunk, false);
  }

  /**
   * Sends command to terminal for review (user can press Enter to execute)
   */
  sendCommandForReview(terminal: vscode.Terminal, command: string): void {
    terminal.show();
    terminal.sendText(command, false); // false = don't auto-execute
  }

  /**
   * Sends command to terminal and executes it immediately
   */
  executeCommand(terminal: vscode.Terminal, command: string): void {
    terminal.show();
    this.captureCommand(terminal, command);
    terminal.sendText(command, true); // true = auto-execute
  }

  /**
   * Cleans up history for closed terminals
   */
  cleanupTerminal(terminal: vscode.Terminal): void {
    const terminalId = this.getTerminalId(terminal);
    this.terminalHistory.delete(terminalId);
  }

  private getTerminalId(terminal: vscode.Terminal): string {
    // Use terminal name + process ID as unique identifier
    // This is a simple approach; in production, you might track terminals more robustly
    return `${terminal.name}_${terminal.processId || Date.now()}`;
  }

  private extractShellName(shellPath: string): string | null {
    const lowerPath = shellPath.toLowerCase();

    if (lowerPath.includes('bash')) {
      return 'bash';
    }
    if (lowerPath.includes('zsh')) {
      return 'zsh';
    }
    if (lowerPath.includes('fish')) {
      return 'fish';
    }
    if (lowerPath.includes('powershell') || lowerPath.includes('pwsh')) {
      return 'powershell';
    }
    if (lowerPath.includes('cmd')) {
      return 'cmd';
    }

    return null;
  }
}
