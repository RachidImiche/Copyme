const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__.invoke;
const listen = window.__TAURI__?.event?.listen || window.__TAURI__.listen;

const searchInput = document.getElementById('searchInput');
const clipboardList = document.getElementById('clipboardList');
let currentSearch = '';
let selectedIndex = -1;
let historyItems = [];

const previewPanel = document.getElementById('previewPanel');
const previewText = document.getElementById('previewText');
const closePreviewBtn = document.getElementById('closePreviewBtn');
const previewPasteBtn = document.getElementById('previewPasteBtn');
const mainView = document.getElementById('mainView');
let currentPreviewContent = '';

function openPreview(content) {
  currentPreviewContent = content;
  previewText.textContent = content;
  
  // Use a small timeout instead of rAF to ensure Webview2 doesn't pause the frame
  setTimeout(() => {
    previewPanel.classList.add('open');
    mainView.classList.add('slide-out');
  }, 10);
}

function closePreview() {
  previewPanel.classList.remove('open');
  mainView.classList.remove('slide-out');
}

closePreviewBtn.onclick = closePreview;

previewPasteBtn.onclick = () => {
  closePreview();
  pasteItem(currentPreviewContent);
};

async function fetchHistory() {
  try {
    const searchParam = currentSearch.trim() === '' ? null : currentSearch;
    historyItems = await invoke('get_history', { search: searchParam });
    renderHistory();
  } catch (error) {
    console.error('Failed to fetch history:', error);
  }
}

function renderHistory() {
  clipboardList.innerHTML = '';
  
  if (historyItems.length === 0) {
    clipboardList.innerHTML = '<li class="empty-state">No clipboard history found</li>';
    return;
  }
  
  historyItems.forEach((item, index) => {
    const li = document.createElement('li');
    li.className = `clipboard-item ${index === selectedIndex ? 'focused' : ''}`;
    
    const textContent = document.createElement('div');
    textContent.className = 'item-content';
    textContent.textContent = item.content.length > 150 ? item.content.substring(0, 150) + '...' : item.content;
    
    const actions = document.createElement('div');
    actions.className = 'item-actions';
    
    const pinBtn = document.createElement('button');
    pinBtn.className = `action-btn pin-btn ${item.pinned ? 'active' : ''}`;
    pinBtn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path></svg>';
    pinBtn.onclick = (e) => {
      e.stopPropagation();
      togglePin(item.id);
    };
    
    const delBtn = document.createElement('button');
    delBtn.className = 'action-btn delete-btn';
    delBtn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';
    delBtn.onclick = (e) => {
      e.stopPropagation();
      deleteItem(item.id);
    };

    const previewBtn = document.createElement('button');
    previewBtn.className = 'action-btn preview-btn';
    previewBtn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>';
    previewBtn.onclick = (e) => {
      e.stopPropagation();
      openPreview(item.content);
    };
    
    actions.appendChild(previewBtn);
    actions.appendChild(pinBtn);
    actions.appendChild(delBtn);
    
    li.appendChild(textContent);
    li.appendChild(actions);
    
    li.onclick = () => pasteItem(item.content);
    
    clipboardList.appendChild(li);
  });
}

async function togglePin(id) {
  try {
    await invoke('toggle_pin', { id });
    fetchHistory();
  } catch (err) { console.error(err); }
}

async function deleteItem(id) {
  try {
    await invoke('delete_item', { id });
    fetchHistory();
  } catch (err) { console.error(err); }
}

async function pasteItem(content) {
  try {
    await invoke('paste_item', { content });
  } catch (error) {
    console.error('Failed to paste item:', error);
  }
}

let searchTimeout;
searchInput.addEventListener('input', (e) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    currentSearch = e.target.value;
    selectedIndex = -1;
    fetchHistory();
  }, 100);
});

window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    if (previewPanel.classList.contains('open')) {
      closePreview();
    } else {
      invoke('hide_window');
    }
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    if (selectedIndex < historyItems.length - 1) {
      selectedIndex++;
      renderHistory();
      scrollToSelected();
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    if (selectedIndex > 0) {
      selectedIndex--;
      renderHistory();
      scrollToSelected();
    }
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (selectedIndex >= 0 && selectedIndex < historyItems.length) {
      pasteItem(historyItems[selectedIndex].content);
    }
  }
});

function scrollToSelected() {
  const selectedEl = clipboardList.children[selectedIndex];
  if (selectedEl) {
    selectedEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }
}

// Hide when clicking outside the window or when the window loses focus
window.addEventListener('blur', () => {
  invoke('hide_window');
});

// Auto-focus the search bar when the window appears
window.addEventListener('focus', () => {
  searchInput.focus();
  searchInput.select();
  selectedIndex = -1;
  currentSearch = '';
  searchInput.value = '';
  fetchHistory();
});

// Initialize on load
fetchHistory();

// Listen for updates from backend polling thread
if (listen) {
  listen('clipboard-update', () => {
    fetchHistory();
  });
} else {
  console.warn('Tauri event listen API not found on window.__TAURI__');
}
