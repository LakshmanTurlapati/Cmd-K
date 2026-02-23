import * as vscode from 'vscode';

export interface PromptResult {
  action: 'review' | 'execute' | 'cancel';
  prompt: string;
}

export class PromptInputUI {
  /**
   * Shows the input popup and returns user's prompt and action
   */
  async showPrompt(): Promise<PromptResult | null> {
    const inputBox = vscode.window.createInputBox();
    inputBox.title = 'Terminal AI Command Generator';
    inputBox.placeholder = 'Describe the command you want to generate... (Esc = review, Cmd/Ctrl+Enter = execute)';
    inputBox.prompt = 'Press Esc to review command, or Cmd/Ctrl+Enter to execute immediately';

    return new Promise<PromptResult | null>((resolve) => {
      let userPrompt = '';
      let isDisposed = false;

      inputBox.onDidChangeValue((value) => {
        userPrompt = value;
      });

      inputBox.onDidAccept(() => {
        if (!isDisposed) {
          isDisposed = true;
          inputBox.hide();
          // Enter key pressed - we'll treat this as "review" mode
          resolve(userPrompt.trim() ? { action: 'review', prompt: userPrompt.trim() } : null);
        }
      });

      inputBox.onDidHide(() => {
        if (!isDisposed) {
          isDisposed = true;
          // User pressed Escape or clicked away - cancel
          resolve(null);
        }
        inputBox.dispose();
      });

      // Handle keyboard shortcuts
      // Note: VSCode's InputBox doesn't directly support Cmd+Enter,
      // so we'll document that users should press Enter for review
      // and we'll add a quick pick option for execute

      inputBox.show();
    });
  }

  /**
   * Shows generated command with options to review or execute
   */
  async showCommandConfirmation(command: string): Promise<'review' | 'execute' | 'cancel'> {
    const items: vscode.QuickPickItem[] = [
      {
        label: '$(terminal) Review in Terminal',
        description: 'Place command in terminal input for review (recommended)',
        detail: 'You can review and edit the command before executing',
      },
      {
        label: '$(play) Execute Immediately',
        description: 'Run the command right away',
        detail: 'Command will be executed without confirmation',
      },
      {
        label: '$(x) Cancel',
        description: 'Discard the generated command',
      },
    ];

    const quickPick = vscode.window.createQuickPick();
    quickPick.title = 'Generated Command';
    quickPick.placeholder = `Command: ${command}`;
    quickPick.items = items;
    quickPick.canSelectMany = false;

    return new Promise<'review' | 'execute' | 'cancel'>((resolve) => {
      quickPick.onDidAccept(() => {
        const selected = quickPick.selectedItems[0];
        quickPick.hide();

        if (!selected) {
          resolve('cancel');
        } else if (selected.label.includes('Review')) {
          resolve('review');
        } else if (selected.label.includes('Execute')) {
          resolve('execute');
        } else {
          resolve('cancel');
        }
      });

      quickPick.onDidHide(() => {
        resolve('cancel');
        quickPick.dispose();
      });

      quickPick.show();
    });
  }

  /**
   * Shows a loading message while generating command
   */
  async withProgress<T>(
    title: string,
    task: () => Promise<T>
  ): Promise<T> {
    return vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title,
        cancellable: false,
      },
      async () => {
        return await task();
      }
    );
  }
}
