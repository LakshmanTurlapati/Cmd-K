const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods to the renderer process
contextBridge.exposeInMainWorld('electronAPI', {
  // Generate command from prompt
  generate: (prompt) => ipcRenderer.invoke('generate', prompt),

  // Get current config
  getConfig: () => ipcRenderer.invoke('get-config'),

  // Save config
  saveConfig: (config) => ipcRenderer.invoke('save-config', config),

  // Get available models per provider
  getModels: () => ipcRenderer.invoke('get-models'),

  // Copy text to clipboard
  copyToClipboard: (text) => ipcRenderer.invoke('copy-to-clipboard', text),

  // Inject command into active terminal
  injectToTerminal: (command) => ipcRenderer.invoke('inject-to-terminal', command),

  // Close the overlay window
  closeWindow: () => ipcRenderer.invoke('close-window'),

  // Resize window based on view
  resizeWindow: (viewName) => ipcRenderer.invoke('resize-window', viewName),

  // Open external URL in default browser
  openExternal: (url) => ipcRenderer.invoke('open-external', url),

  // Listen for reset-view event from main process
  onResetView: (callback) => ipcRenderer.on('reset-view', callback),

  // Listen for focus-input event from main process
  onFocusInput: (callback) => ipcRenderer.on('focus-input', callback),
});
