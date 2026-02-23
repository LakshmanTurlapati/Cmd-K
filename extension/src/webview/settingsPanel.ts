import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { ConfigManager } from '../config';
import { APIValidator } from '../validators/apiValidator';

export class SettingsPanel {
  public static currentPanel: SettingsPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;
  private _disposables: vscode.Disposable[] = [];

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;

    // Set HTML content
    this._panel.webview.html = this._getHtmlContent(extensionUri);

    // Handle messages from webview
    this._panel.webview.onDidReceiveMessage(
      async (message) => {
        switch (message.command) {
          case 'loadSettings':
            await this._loadSettings();
            break;
          case 'applySettings':
            await this._applySettings(message.settings);
            break;
          case 'openExternal':
            vscode.env.openExternal(vscode.Uri.parse(message.url));
            break;
        }
      },
      null,
      this._disposables
    );

    // Handle panel disposal
    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
  }

  public static createOrShow(extensionUri: vscode.Uri) {
    const column = vscode.ViewColumn.One;

    // If we already have a panel, show it
    if (SettingsPanel.currentPanel) {
      SettingsPanel.currentPanel._panel.reveal(column);
      return;
    }

    // Otherwise, create a new panel
    const panel = vscode.window.createWebviewPanel(
      'cmdkSettings',
      'CMD+K Settings',
      column,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(extensionUri, 'src', 'webview', 'html')]
      }
    );

    SettingsPanel.currentPanel = new SettingsPanel(panel, extensionUri);
  }

  public dispose() {
    SettingsPanel.currentPanel = undefined;

    this._panel.dispose();

    while (this._disposables.length) {
      const disposable = this._disposables.pop();
      if (disposable) {
        disposable.dispose();
      }
    }
  }

  private async _loadSettings() {
    const config = ConfigManager.getConfig();
    this._panel.webview.postMessage({
      command: 'settingsLoaded',
      settings: config
    });
  }

  private async _applySettings(settings: any) {
    try {
      // Save settings to VS Code configuration
      const config = vscode.workspace.getConfiguration('terminalAI');

      await config.update('provider', settings.provider, vscode.ConfigurationTarget.Global);
      await config.update('openai.apiKey', settings.openai.apiKey, vscode.ConfigurationTarget.Global);
      await config.update('openai.model', settings.openai.model, vscode.ConfigurationTarget.Global);
      await config.update('anthropic.apiKey', settings.anthropic.apiKey, vscode.ConfigurationTarget.Global);
      await config.update('anthropic.model', settings.anthropic.model, vscode.ConfigurationTarget.Global);
      await config.update('xai.apiKey', settings.xai.apiKey, vscode.ConfigurationTarget.Global);
      await config.update('xai.model', settings.xai.model, vscode.ConfigurationTarget.Global);

      // Validate API keys
      const statuses = await this._validateProviders(settings);

      // Send success response
      this._panel.webview.postMessage({
        command: 'validationResult',
        success: true,
        message: 'Settings saved successfully!',
        statuses
      });

      vscode.window.showInformationMessage('CMD+K settings saved successfully!');
    } catch (error: any) {
      this._panel.webview.postMessage({
        command: 'validationResult',
        success: false,
        message: `Failed to save settings: ${error.message}`
      });

      vscode.window.showErrorMessage(`Failed to save settings: ${error.message}`);
    }
  }

  private async _validateProviders(settings: any): Promise<any[]> {
    const statuses = [];

    // Validate OpenAI
    if (settings.openai.apiKey) {
      try {
        const isValid = await APIValidator.validateOpenAI(
          settings.openai.apiKey,
          settings.openai.model
        );
        statuses.push({ provider: 'openai', valid: isValid });
      } catch (error) {
        statuses.push({ provider: 'openai', valid: false });
      }
    } else {
      statuses.push({ provider: 'openai', valid: false });
    }

    // Validate Anthropic
    if (settings.anthropic.apiKey) {
      try {
        const isValid = await APIValidator.validateAnthropic(
          settings.anthropic.apiKey,
          settings.anthropic.model
        );
        statuses.push({ provider: 'anthropic', valid: isValid });
      } catch (error) {
        statuses.push({ provider: 'anthropic', valid: false });
      }
    } else {
      statuses.push({ provider: 'anthropic', valid: false });
    }

    // Validate xAI
    if (settings.xai.apiKey) {
      try {
        const isValid = await APIValidator.validateXAI(
          settings.xai.apiKey,
          settings.xai.model
        );
        statuses.push({ provider: 'xai', valid: isValid });
      } catch (error) {
        statuses.push({ provider: 'xai', valid: false });
      }
    } else {
      statuses.push({ provider: 'xai', valid: false });
    }

    return statuses;
  }

  private _getHtmlContent(extensionUri: vscode.Uri): string {
    const htmlPath = path.join(
      extensionUri.fsPath,
      'src',
      'webview',
      'html',
      'settings.html'
    );

    let html = fs.readFileSync(htmlPath, 'utf8');

    // Replace resource URIs if needed
    // (Currently all resources are inline)

    return html;
  }
}
