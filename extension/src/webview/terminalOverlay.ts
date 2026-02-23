import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { ConfigManager, AIProviderType } from '../config';
import { TerminalManager } from '../terminalManager';
import { StreamHandler } from '../streaming/streamHandler';
import { SettingsPanel } from './settingsPanel';

export class TerminalOverlay {
  private panel: vscode.WebviewPanel | undefined;
  private disposables: vscode.Disposable[] = [];
  private streamHandler: StreamHandler | undefined;
  private currentTerminal: vscode.Terminal | undefined;
  private selectedProvider: AIProviderType | undefined;
  private selectedModel: string | undefined;

  constructor(
    private extensionUri: vscode.Uri,
    private terminalManager: TerminalManager
  ) {}

  public async show(terminal: vscode.Terminal) {
    this.currentTerminal = terminal;

    if (this.panel) {
      this.panel.reveal();
      await this._sendProviderInfo();
      return;
    }

    // Create webview panel - positioned in active column
    this.panel = vscode.window.createWebviewPanel(
      'cmdkOverlay',
      'CMD+K',
      { viewColumn: vscode.ViewColumn.Active, preserveFocus: true },
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(this.extensionUri, 'src', 'webview', 'html')]
      }
    );

    // Set HTML content
    this.panel.webview.html = this._getHtmlContent();

    // Send current provider info
    await this._sendProviderInfo();

    // Handle messages from webview
    this.panel.webview.onDidReceiveMessage(
      async (message) => {
        await this._handleMessage(message);
      },
      null,
      this.disposables
    );

    // Handle panel disposal
    this.panel.onDidDispose(
      () => {
        this.panel = undefined;
        this.streamHandler = undefined;
        while (this.disposables.length) {
          const disposable = this.disposables.pop();
          if (disposable) {
            disposable.dispose();
          }
        }
      },
      null,
      this.disposables
    );
  }

  public hide() {
    if (this.panel) {
      this.panel.dispose();
    }
  }

  public refreshProviderInfo() {
    if (this.panel) {
      this._sendProviderInfo();
    }
  }

  private async _handleMessage(message: any) {
    switch (message.command) {
      case 'ready':
        await this._sendProviderInfo();
        break;

      case 'generate':
        // Use selected provider/model if specified, otherwise use config defaults
        const provider = message.provider || this.selectedProvider;
        const model = message.model || this.selectedModel;
        await this._generateCommand(message.prompt, provider, model);
        break;

      case 'providerChanged':
        this.selectedProvider = message.provider;
        await this._updateModelList(message.provider);
        break;

      case 'modelChanged':
        this.selectedModel = message.model;
        break;

      case 'openSettings':
        SettingsPanel.createOrShow(this.extensionUri);
        break;

      case 'execute':
        this._executeCommand(message.commandText);
        break;

      case 'cancel':
        this.hide();
        break;
    }
  }

  private async _sendProviderInfo() {
    if (!this.panel) return;

    const config = ConfigManager.getConfig();
    const availableProviderIds = ConfigManager.getAvailableProviders();

    // Build providers list with models
    const providers = availableProviderIds.map(providerId => {
      const models = ConfigManager.getModelsForProvider(providerId).map(modelId => ({
        id: modelId,
        name: ConfigManager.getModelDisplayName(modelId)
      }));

      let currentModel = '';
      switch (providerId) {
        case 'openai':
          currentModel = config.openai.model;
          break;
        case 'anthropic':
          currentModel = config.anthropic.model;
          break;
        case 'xai':
          currentModel = config.xai.model;
          break;
      }

      return {
        id: providerId,
        name: ConfigManager.getProviderDisplayName(providerId),
        models,
        currentModel
      };
    });

    // Set default selections if not already set
    if (!this.selectedProvider && providers.length > 0) {
      const defaultProvider = providers.find(p => p.id === config.provider) || providers[0];
      this.selectedProvider = defaultProvider.id;
      this.selectedModel = defaultProvider.currentModel;
    }

    this.panel.webview.postMessage({
      command: 'setAvailableProviders',
      providers,
      currentProvider: this.selectedProvider || config.provider,
      currentModel: this.selectedModel || config.openai.model
    });
  }

  private async _updateModelList(provider: AIProviderType) {
    if (!this.panel) return;

    const config = ConfigManager.getConfig();
    const availableProviderIds = ConfigManager.getAvailableProviders();

    const providers = availableProviderIds.map(providerId => {
      const models = ConfigManager.getModelsForProvider(providerId).map(modelId => ({
        id: modelId,
        name: ConfigManager.getModelDisplayName(modelId)
      }));

      let currentModel = '';
      switch (providerId) {
        case 'openai':
          currentModel = config.openai.model;
          break;
        case 'anthropic':
          currentModel = config.anthropic.model;
          break;
        case 'xai':
          currentModel = config.xai.model;
          break;
      }

      return {
        id: providerId,
        name: ConfigManager.getProviderDisplayName(providerId),
        models,
        currentModel
      };
    });

    // Find the selected provider and update model
    const selectedProviderObj = providers.find(p => p.id === provider);
    if (selectedProviderObj) {
      this.selectedModel = selectedProviderObj.currentModel;

      this.panel.webview.postMessage({
        command: 'setAvailableProviders',
        providers,
        currentProvider: provider,
        currentModel: this.selectedModel
      });
    }
  }

  private async _generateCommand(prompt: string, provider?: AIProviderType, model?: string) {
    if (!this.panel || !this.currentTerminal) return;

    try {
      // Notify start of stream
      this.panel.webview.postMessage({ command: 'streamStart' });

      // Create stream handler
      this.streamHandler = new StreamHandler(
        this.extensionUri,
        this.terminalManager,
        this.currentTerminal,
        prompt,
        provider,
        model
      );

      // Set up chunk callback to stream directly to terminal
      this.streamHandler.onChunk = (chunk: string) => {
        if (this.currentTerminal) {
          this.terminalManager.streamTextChunk(this.currentTerminal, chunk);
        }
      };

      // Generate command with streaming
      const command = await this.streamHandler.generate();

      // Notify completion
      if (this.panel) {
        this.panel.webview.postMessage({
          command: 'streamComplete',
          commandText: command
        });
      }
    } catch (error: any) {
      if (this.panel) {
        this.panel.webview.postMessage({
          command: 'error',
          error: error.message || 'Failed to generate command'
        });
      }
    }
  }

  private _executeCommand(command: string) {
    if (this.currentTerminal) {
      this.terminalManager.executeCommand(this.currentTerminal, command);
    }
  }

  private _getHtmlContent(): string {
    const htmlPath = path.join(
      this.extensionUri.fsPath,
      'src',
      'webview',
      'html',
      'overlay.html'
    );

    let html = fs.readFileSync(htmlPath, 'utf8');

    // Replace resource URIs if needed
    // (Currently all resources are inline)

    return html;
  }
}
