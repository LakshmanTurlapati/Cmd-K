import * as vscode from 'vscode';
import { CommandGenerationContext } from './aiProviders/base';
import { TerminalManager } from './terminalManager';

export class ContextBuilder {
  constructor(private terminalManager: TerminalManager) {}

  /**
   * Builds complete context for AI command generation
   */
  buildContext(
    terminal: vscode.Terminal,
    userPrompt: string,
    workspacePath?: string
  ): CommandGenerationContext {
    const os = this.getOSName();
    const shell = this.terminalManager.detectShell(terminal);
    const workingDirectory = workspacePath || this.getWorkingDirectory();
    const terminalHistory = this.terminalManager.getHistory(terminal);

    return {
      os,
      shell,
      workingDirectory,
      terminalHistory,
      userPrompt,
    };
  }

  private getOSName(): string {
    const platform = process.platform;
    switch (platform) {
      case 'darwin':
        return 'macOS';
      case 'win32':
        return 'Windows';
      case 'linux':
        return 'Linux';
      default:
        return platform;
    }
  }

  private getWorkingDirectory(): string {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders && workspaceFolders.length > 0) {
      return workspaceFolders[0].uri.fsPath;
    }
    return '~';
  }
}
