// Slash commands
const COMMANDS = [
  { cmd: '/settings', desc: 'Configure API keys' },
  { cmd: '/models', desc: 'Select default model' },
  { cmd: '/about', desc: 'View on GitHub' },
];

// DOM elements
const views = {
  main: document.getElementById('main-view'),
  result: document.getElementById('result-view'),
  settings: document.getElementById('settings-view'),
  models: document.getElementById('models-view'),
};

const elements = {
  commandInput: document.getElementById('command-input'),
  suggestions: document.getElementById('suggestions'),
  status: document.getElementById('status'),
  resultCommand: document.getElementById('result-command'),
  pasteBtn: document.getElementById('paste-btn'),
  cancelBtn: document.getElementById('cancel-btn'),
  providerSelect: document.getElementById('provider-select'),
  modelSelect: document.getElementById('model-select'),
  apiKeyInput: document.getElementById('api-key-input'),
  saveSettingsBtn: document.getElementById('save-settings-btn'),
  backSettingsBtn: document.getElementById('back-settings-btn'),
  settingsStatus: document.getElementById('settings-status'),
  defaultModelSelect: document.getElementById('default-model-select'),
  setDefaultBtn: document.getElementById('set-default-btn'),
  backModelsBtn: document.getElementById('back-models-btn'),
  modelsStatus: document.getElementById('models-status'),
};

// State
let currentCommand = '';
let availableModels = {};
let selectedSuggestionIndex = 0;
let filteredCommands = [];

// Initialize
async function init() {
  // Load available models
  availableModels = await window.electronAPI.getModels();

  // Focus input
  elements.commandInput.focus();

  // Setup event listeners
  setupEventListeners();

  // Listen for reset-view from main process (when CMD+K pressed again)
  window.electronAPI.onResetView(() => {
    showView('main');
    elements.commandInput.value = '';
    elements.commandInput.focus();
  });

  // Listen for focus-input from main process
  window.electronAPI.onFocusInput(() => {
    elements.commandInput.focus();
  });
}

function setupEventListeners() {
  // Main input - keydown for special keys
  elements.commandInput.addEventListener('keydown', handleKeyDown);

  // Main input - input for text changes
  elements.commandInput.addEventListener('input', handleInputChange);

  // Result view buttons
  elements.pasteBtn.addEventListener('click', handlePaste);
  elements.cancelBtn.addEventListener('click', () => {
    window.electronAPI.closeWindow();
  });

  // Settings view
  elements.providerSelect.addEventListener('change', updateModelOptions);
  elements.saveSettingsBtn.addEventListener('click', handleSaveSettings);
  elements.backSettingsBtn.addEventListener('click', () => showView('main'));

  // Models view
  elements.setDefaultBtn.addEventListener('click', handleSetDefault);
  elements.backModelsBtn.addEventListener('click', () => showView('main'));
}

function handleKeyDown(e) {
  const suggestionsVisible = elements.suggestions.classList.contains('active');

  if (e.key === 'Tab' && suggestionsVisible && filteredCommands.length > 0) {
    e.preventDefault();
    // Autocomplete with selected suggestion
    elements.commandInput.value = filteredCommands[selectedSuggestionIndex].cmd;
    hideSuggestions();
    return;
  }

  if (e.key === 'ArrowDown' && suggestionsVisible) {
    e.preventDefault();
    selectedSuggestionIndex = Math.min(selectedSuggestionIndex + 1, filteredCommands.length - 1);
    updateSuggestionSelection();
    return;
  }

  if (e.key === 'ArrowUp' && suggestionsVisible) {
    e.preventDefault();
    selectedSuggestionIndex = Math.max(selectedSuggestionIndex - 1, 0);
    updateSuggestionSelection();
    return;
  }

  if (e.key === 'Enter') {
    if (suggestionsVisible && filteredCommands.length > 0) {
      e.preventDefault();
      elements.commandInput.value = filteredCommands[selectedSuggestionIndex].cmd;
      hideSuggestions();
      handleSubmit();
    } else {
      handleSubmit();
    }
    return;
  }

  if (e.key === 'Escape') {
    if (suggestionsVisible) {
      e.preventDefault();
      e.stopPropagation();
      hideSuggestions();
    } else {
      e.preventDefault();
      window.electronAPI.closeWindow();
    }
  }
}

function handleInputChange() {
  const text = elements.commandInput.value;

  if (text.startsWith('/')) {
    // Filter commands based on input
    const query = text.toLowerCase();
    filteredCommands = COMMANDS.filter(c => c.cmd.toLowerCase().startsWith(query));

    if (filteredCommands.length > 0) {
      showSuggestions();
    } else {
      hideSuggestions();
    }
  } else {
    hideSuggestions();
  }
}

function showSuggestions() {
  selectedSuggestionIndex = 0;

  elements.suggestions.innerHTML = filteredCommands.map((cmd, index) => `
    <div class="suggestion-item ${index === 0 ? 'selected' : ''}" data-index="${index}">
      <div>
        <span class="suggestion-cmd">${cmd.cmd}</span>
        <span class="suggestion-desc"> - ${cmd.desc}</span>
      </div>
      ${index === 0 ? '<span class="suggestion-hint">Tab</span>' : ''}
    </div>
  `).join('');

  elements.suggestions.classList.add('active');

  // Expand window to fit dropdown
  window.electronAPI.resizeWindow('suggestions');

  // Add click handlers to suggestions
  elements.suggestions.querySelectorAll('.suggestion-item').forEach(item => {
    item.addEventListener('click', () => {
      const index = parseInt(item.dataset.index);
      elements.commandInput.value = filteredCommands[index].cmd;
      hideSuggestions();
      elements.commandInput.focus();
      handleSubmit();
    });
  });
}

function hideSuggestions() {
  elements.suggestions.classList.remove('active');
  filteredCommands = [];
  // Restore normal window size
  window.electronAPI.resizeWindow('main');
}

function updateSuggestionSelection() {
  elements.suggestions.querySelectorAll('.suggestion-item').forEach((item, index) => {
    if (index === selectedSuggestionIndex) {
      item.classList.add('selected');
      // Update Tab hint
      const existingHint = elements.suggestions.querySelector('.suggestion-hint');
      if (existingHint) existingHint.remove();
      const hint = document.createElement('span');
      hint.className = 'suggestion-hint';
      hint.textContent = 'Tab';
      item.appendChild(hint);
    } else {
      item.classList.remove('selected');
    }
  });
}

function showView(viewName) {
  Object.values(views).forEach(v => v.classList.remove('active'));
  views[viewName].classList.add('active');

  // Resize window to fit content
  window.electronAPI.resizeWindow(viewName);

  if (viewName === 'main') {
    elements.commandInput.focus();
    elements.commandInput.value = '';
    elements.status.textContent = '';
    hideSuggestions();
  }
}

async function handleSubmit() {
  const text = elements.commandInput.value.trim();
  hideSuggestions();

  if (!text) return;

  if (text === '/settings') {
    await loadSettings();
    showView('settings');
    return;
  }

  if (text === '/models') {
    await loadModels();
    showView('models');
    return;
  }

  if (text === '/about') {
    await window.electronAPI.openExternal('https://github.com/LakshmanTurlapati/Cmd-K');
    window.electronAPI.closeWindow();
    return;
  }

  // Generate command
  await generateCommand(text);
}

async function generateCommand(prompt) {
  elements.status.innerHTML = '<span class="loading"></span>Generating...';
  elements.status.className = 'status';
  elements.commandInput.disabled = true;

  try {
    const result = await window.electronAPI.generate(prompt);

    if (!result.success) {
      throw new Error(result.error || 'Failed to generate command');
    }

    // Inject directly to terminal (handler hides window and quits)
    await window.electronAPI.injectToTerminal(result.command);
  } catch (error) {
    elements.status.textContent = error.message;
    elements.status.className = 'status error';
    elements.commandInput.disabled = false;
  }
}

async function handlePaste() {
  // Inject command into active terminal
  await window.electronAPI.injectToTerminal(currentCommand);
  window.electronAPI.closeWindow();
}

async function loadSettings() {
  try {
    const config = await window.electronAPI.getConfig();

    // Set provider
    if (config.provider) {
      elements.providerSelect.value = config.provider;
    }

    // Update model options
    updateModelOptions();

    // Show which providers have API keys configured
    const statusParts = [];
    if (config.hasApiKey.openai) statusParts.push('OpenAI');
    if (config.hasApiKey.anthropic) statusParts.push('Anthropic');
    if (config.hasApiKey.xai) statusParts.push('xAI');

    if (statusParts.length > 0) {
      elements.settingsStatus.textContent = `Configured: ${statusParts.join(', ')}`;
      elements.settingsStatus.className = 'status success';
    } else {
      elements.settingsStatus.textContent = 'No API keys configured yet';
      elements.settingsStatus.className = 'status';
    }

    // Clear API key input
    elements.apiKeyInput.value = '';
  } catch (error) {
    elements.settingsStatus.textContent = 'Failed to load settings';
    elements.settingsStatus.className = 'status error';
  }
}

function updateModelOptions() {
  const provider = elements.providerSelect.value;
  const models = availableModels[provider] || [];

  elements.modelSelect.innerHTML = '';
  models.forEach(model => {
    const option = document.createElement('option');
    option.value = model;
    option.textContent = model;
    elements.modelSelect.appendChild(option);
  });
}

async function handleSaveSettings() {
  const provider = elements.providerSelect.value;
  const model = elements.modelSelect.value;
  const apiKey = elements.apiKeyInput.value.trim();

  if (!apiKey) {
    elements.settingsStatus.textContent = 'API key is required';
    elements.settingsStatus.className = 'status error';
    return;
  }

  try {
    await window.electronAPI.saveConfig({ provider, model, apiKey });

    elements.settingsStatus.textContent = 'Settings saved!';
    elements.settingsStatus.className = 'status success';

    setTimeout(() => showView('main'), 1000);
  } catch (error) {
    elements.settingsStatus.textContent = error.message;
    elements.settingsStatus.className = 'status error';
  }
}

async function loadModels() {
  try {
    const config = await window.electronAPI.getConfig();

    elements.defaultModelSelect.innerHTML = '';

    // Add models from all configured providers
    let hasModels = false;

    for (const [provider, models] of Object.entries(availableModels)) {
      if (config.hasApiKey[provider]) {
        models.forEach(model => {
          const option = document.createElement('option');
          option.value = `${provider}:${model}`;
          option.textContent = `${provider}: ${model}`;
          if (config.provider === provider && config.model === model) {
            option.selected = true;
          }
          elements.defaultModelSelect.appendChild(option);
          hasModels = true;
        });
      }
    }

    if (!hasModels) {
      const option = document.createElement('option');
      option.textContent = 'No API keys configured';
      option.disabled = true;
      elements.defaultModelSelect.appendChild(option);
    }

    elements.modelsStatus.textContent = '';
  } catch (error) {
    elements.modelsStatus.textContent = 'Failed to load models';
    elements.modelsStatus.className = 'status error';
  }
}

async function handleSetDefault() {
  const selected = elements.defaultModelSelect.value;
  if (!selected) return;

  const [provider, model] = selected.split(':');

  try {
    await window.electronAPI.saveConfig({ provider, model });

    elements.modelsStatus.textContent = 'Default model updated!';
    elements.modelsStatus.className = 'status success';

    setTimeout(() => showView('main'), 1000);
  } catch (error) {
    elements.modelsStatus.textContent = error.message;
    elements.modelsStatus.className = 'status error';
  }
}

// Initialize on load
init();
