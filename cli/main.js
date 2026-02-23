const { app, BrowserWindow, globalShortcut, ipcMain, clipboard, shell } = require('electron');
const { execSync, exec } = require('child_process');
const path = require('path');
const Store = require('electron-store');
const { generate, MODELS } = require('./ai');

// Initialize config store
const store = new Store({
  defaults: {
    provider: 'openai',
    model: 'gpt-4o',
    apiKeys: {}
  }
});

let mainWindow = null;

function createWindow() {
  // Create the browser window as frameless overlay
  mainWindow = new BrowserWindow({
    width: 600,
    height: 180,
    frame: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    show: false,
    resizable: false,
    movable: true,
    hasShadow: true,
    acceptFirstMouse: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      backgroundThrottling: false,
    },
  });

  // Critical: Make window visible on all workspaces including fullscreen
  mainWindow.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });

  // Critical: Set window level to screen-saver (highest)
  mainWindow.setAlwaysOnTop(true, 'screen-saver', 1);

  // Load the renderer
  mainWindow.loadFile(path.join(__dirname, 'renderer', 'index.html'));

  // Center the window on screen
  mainWindow.center();

  // Handle window close - just hide, don't destroy
  mainWindow.on('close', (event) => {
    event.preventDefault();
    mainWindow.hide();
  });
}

function showWindow() {
  if (!mainWindow) {
    createWindow();
  }

  // Reset to main view
  mainWindow.webContents.send('reset-view');

  mainWindow.center();
  mainWindow.show();
  mainWindow.focus();

  // Add delay to ensure window is fully shown before focusing
  setTimeout(() => {
    mainWindow.webContents.focus();
    // Send message to renderer to focus input
    mainWindow.webContents.send('focus-input');
  }, 50);
}

function hideWindow() {
  if (mainWindow) {
    mainWindow.hide();
  }
}

// Prevent multiple instances - show window if second instance launched
const gotTheLock = app.requestSingleInstanceLock();
if (!gotTheLock) {
  app.quit();
} else {
  app.on('second-instance', () => {
    showWindow();
  });
}

app.whenReady().then(() => {
  // Hide from dock - run as background daemon
  if (app.dock) {
    app.dock.hide();
  }

  createWindow();

  // Register CMD+K to show overlay
  globalShortcut.register('CommandOrControl+K', () => {
    if (mainWindow && mainWindow.isVisible()) {
      hideWindow();
    } else {
      showWindow();
    }
  });
});

// Don't quit when all windows closed - stay running as daemon
app.on('window-all-closed', (event) => {
  // Do nothing - keep running
});

// Cleanup on actual quit
app.on('will-quit', () => {
  globalShortcut.unregisterAll();
});

// ============ IPC Handlers ============

// Generate command from prompt
ipcMain.handle('generate', async (event, prompt) => {
  try {
    const provider = store.get('provider');
    const model = store.get('model');
    const apiKeys = store.get('apiKeys');
    const apiKey = apiKeys[provider];

    if (!apiKey) {
      throw new Error(`No API key configured for ${provider}. Use /settings to add one.`);
    }

    const command = await generate(prompt, { provider, apiKey, model });
    return { success: true, command };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Get current config
ipcMain.handle('get-config', () => {
  return {
    provider: store.get('provider'),
    model: store.get('model'),
    hasApiKey: {
      openai: !!store.get('apiKeys.openai'),
      anthropic: !!store.get('apiKeys.anthropic'),
      xai: !!store.get('apiKeys.xai')
    }
  };
});

// Save config
ipcMain.handle('save-config', (event, config) => {
  if (config.provider) {
    store.set('provider', config.provider);
  }
  if (config.model) {
    store.set('model', config.model);
  }
  if (config.apiKey && config.provider) {
    store.set(`apiKeys.${config.provider}`, config.apiKey);
  }
  return { success: true };
});

// Get available models
ipcMain.handle('get-models', () => {
  return MODELS;
});

// Copy to clipboard
ipcMain.handle('copy-to-clipboard', (event, text) => {
  clipboard.writeText(text);
  return { success: true };
});

// Inject command into active terminal
ipcMain.handle('inject-to-terminal', (event, command) => {
  try {
    // Copy to clipboard first
    clipboard.writeText(command);

    // Hide window
    if (mainWindow) {
      mainWindow.hide();
    }

    // Activate terminal and paste
    const script = `
      -- Try to find and activate a terminal app
      tell application "System Events"
        set terminalApps to {"Terminal", "iTerm2", "iTerm", "Hyper", "Alacritty", "kitty", "WezTerm"}
        repeat with termApp in terminalApps
          if exists (application process termApp) then
            tell application termApp to activate
            delay 0.3
            keystroke "v" using command down
            return
          end if
        end repeat
        -- Fallback: just paste to whatever has focus
        keystroke "v" using command down
      end tell
    `;

    exec(`osascript -e '${script}'`, (error) => {
      if (error) {
        console.error('Paste failed:', error);
      }
    });

    // Don't quit - stay running as daemon
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Close/hide window
ipcMain.handle('close-window', () => {
  hideWindow();
});

// Resize window based on view
ipcMain.handle('resize-window', (event, viewName) => {
  if (!mainWindow) return;

  const heights = {
    main: 180,
    result: 220,
    settings: 380,
    models: 280,
    suggestions: 320
  };

  const height = heights[viewName] || 180;
  mainWindow.setSize(600, height);

  // Don't center when toggling suggestions dropdown
  if (viewName !== 'suggestions' && viewName !== 'main') {
    mainWindow.center();
  }
});

// Open external URL in default browser
ipcMain.handle('open-external', (event, url) => {
  shell.openExternal(url);
});
