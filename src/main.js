const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const appWindow = getCurrentWindow();

// DOM elements
const inputText = document.getElementById('input-text');
const resultText = document.getElementById('result-text');
const fixBtn = document.getElementById('fix-btn');
const clearBtn = document.getElementById('clear-btn');
const copyBtn = document.getElementById('copy-btn');
const closeBtn = document.getElementById('close-btn');
const settingsBtn = document.getElementById('settings-btn');
const settingsPanel = document.getElementById('settings-panel');
const apiKeyInput = document.getElementById('api-key-input');
const saveKeyBtn = document.getElementById('save-key-btn');
const keyStatus = document.getElementById('key-status');
const statusEl = document.getElementById('status');
const autostartToggle = document.getElementById('autostart-toggle');

// ── Close → hide to tray ──
closeBtn.addEventListener('click', async () => {
  await appWindow.hide();
});

// ── Settings toggle ──
settingsBtn.addEventListener('click', async () => {
  settingsPanel.classList.toggle('hidden');
  if (!settingsPanel.classList.contains('hidden')) {
    try {
      const key = await invoke('get_api_key');
      apiKeyInput.value = key || '';
    } catch (e) {
      console.error(e);
    }
    try {
      const enabled = await invoke('get_autostart');
      autostartToggle.setAttribute('aria-checked', String(enabled));
    } catch (e) {
      console.error(e);
    }
    apiKeyInput.focus();
  }
});

// ── Autostart toggle ──
autostartToggle.addEventListener('click', async () => {
  const current = autostartToggle.getAttribute('aria-checked') === 'true';
  const next = !current;
  try {
    await invoke('set_autostart', { enabled: next });
    autostartToggle.setAttribute('aria-checked', String(next));
  } catch (e) {
    console.error('Failed to toggle autostart:', e);
  }
});

// ── Save API key ──
saveKeyBtn.addEventListener('click', async () => {
  const key = apiKeyInput.value.trim();
  if (!key) {
    showKeyStatus('Please enter an API key', 'error');
    return;
  }
  try {
    await invoke('save_api_key', { key });
    showKeyStatus('API key saved', 'success');
    setTimeout(() => {
      settingsPanel.classList.add('hidden');
      keyStatus.textContent = '';
    }, 1200);
  } catch (e) {
    showKeyStatus('Failed to save: ' + e, 'error');
  }
});

// ── Fix Grammar ──
fixBtn.addEventListener('click', fixGrammar);

async function fixGrammar() {
  const text = inputText.value.trim();
  if (!text) {
    showStatus('Please enter some text', 'error');
    return;
  }

  fixBtn.classList.add('loading');
  fixBtn.disabled = true;
  showStatus('Fixing...', 'loading');
  resultText.value = '';

  try {
    const result = await invoke('fix_grammar', { text });
    resultText.value = result;
    showStatus('Done — copied to clipboard', 'success');
    await navigator.clipboard.writeText(result);
  } catch (e) {
    resultText.value = '';
    showStatus(String(e), 'error');
  } finally {
    fixBtn.classList.remove('loading');
    fixBtn.disabled = false;
  }
}

// ── Copy button ──
copyBtn.addEventListener('click', async () => {
  const text = resultText.value;
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    showStatus('Copied to clipboard', 'success');
  } catch (e) {
    resultText.select();
    document.execCommand('copy');
    showStatus('Copied', 'success');
  }
});

// ── Clear ──
clearBtn.addEventListener('click', () => {
  inputText.value = '';
  resultText.value = '';
  showStatus('Ready', '');
  inputText.focus();
});

// ── Ctrl+Enter shortcut ──
document.addEventListener('keydown', (e) => {
  if (e.ctrlKey && e.key === 'Enter') {
    e.preventDefault();
    fixGrammar();
  }
});

// ── Status helpers ──
function showStatus(msg, type) {
  statusEl.textContent = msg;
  statusEl.className = type || '';
}

function showKeyStatus(msg, type) {
  keyStatus.textContent = msg;
  keyStatus.className = type || '';
}
