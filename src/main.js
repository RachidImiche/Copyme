const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__.invoke;
const listen = window.__TAURI__?.event?.listen || window.__TAURI__.listen;

const searchInput = document.getElementById("searchInput");
const clipboardList = document.getElementById("clipboardList");
let currentSearch = "";
let selectedIndex = -1;
let historyItems = [];

const previewPanel = document.getElementById("previewPanel");
const previewText = document.getElementById("previewText");
const previewImage = document.getElementById("previewImage");
const closePreviewBtn = document.getElementById("closePreviewBtn");
const previewPasteBtn = document.getElementById("previewPasteBtn");
const mainView = document.getElementById("mainView");
let currentPreviewItemId = null;
let previewRequestToken = 0;
let isPasting = false;

function setPasteUiState(active) {
  previewPasteBtn.disabled = active;
  document.body.classList.toggle("is-pasting", active);
}

function getItemLabel(item) {
  if (item.content_type === "image") {
    return "Image";
  }
  return item.content.length > 150
    ? `${item.content.substring(0, 150)}...`
    : item.content;
}

async function openPreview(item) {
  currentPreviewItemId = item.id;
  const requestToken = ++previewRequestToken;

  previewText.hidden = true;
  previewImage.hidden = true;
  previewText.textContent = "";
  previewImage.removeAttribute("src");

  if (item.content_type === "image") {
    previewText.hidden = false;
    previewText.textContent = "Loading image preview...";
  } else {
    previewText.hidden = false;
    previewText.textContent = item.content;
  }

  previewPanel.classList.add("open");
  mainView.classList.add("slide-out");

  if (item.content_type === "image") {
    try {
      const imageDataUrl = await invoke("get_image_data", { id: item.id });
      if (
        requestToken !== previewRequestToken ||
        currentPreviewItemId !== item.id
      ) {
        return;
      }
      previewImage.src = imageDataUrl;
      previewImage.hidden = false;
      previewText.hidden = true;
    } catch (error) {
      if (
        requestToken !== previewRequestToken ||
        currentPreviewItemId !== item.id
      ) {
        return;
      }
      previewImage.hidden = true;
      previewText.hidden = false;
      previewText.textContent = "Failed to load image preview.";
      console.error("Failed to get image preview:", error);
    }
  }
}

function closePreview() {
  previewRequestToken += 1;
  previewPanel.classList.remove("open");
  mainView.classList.remove("slide-out");
  previewImage.removeAttribute("src");
  previewText.textContent = "";
  currentPreviewItemId = null;
}

closePreviewBtn.onclick = closePreview;

previewPasteBtn.onclick = () => {
  if (currentPreviewItemId !== null) {
    pasteItem(currentPreviewItemId);
  }
};

async function fetchHistory() {
  try {
    const searchParam = currentSearch.trim() === "" ? null : currentSearch;
    historyItems = await invoke("get_history", { search: searchParam });

    if (historyItems.length === 0) {
      selectedIndex = -1;
    } else if (selectedIndex >= historyItems.length) {
      selectedIndex = historyItems.length - 1;
    }

    renderHistory();
  } catch (error) {
    console.error("Failed to fetch history:", error);
  }
}

function renderHistory() {
  clipboardList.innerHTML = "";

  if (historyItems.length === 0) {
    clipboardList.innerHTML =
      '<li class="empty-state">No clipboard history found</li>';
    return;
  }

  historyItems.forEach((item, index) => {
    const li = document.createElement("li");
    li.className = `clipboard-item ${index === selectedIndex ? "focused" : ""}`;

    const textContent = document.createElement("div");
    textContent.className = `item-content ${item.content_type === "image" ? "image-item-content" : ""}`;

    if (item.content_type === "image") {
      textContent.innerHTML =
        '<span class="image-badge" aria-hidden="true"><svg class="image-item-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><path d="m21 15-5-5L5 21"></path></svg></span><span>Image</span>';
    } else {
      textContent.textContent = getItemLabel(item);
    }

    const actions = document.createElement("div");
    actions.className = "item-actions";

    const pinBtn = document.createElement("button");
    pinBtn.className = `action-btn pin-btn ${item.pinned ? "active" : ""}`;
    pinBtn.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path></svg>';
    pinBtn.onclick = (e) => {
      e.stopPropagation();
      togglePin(item.id);
    };

    const delBtn = document.createElement("button");
    delBtn.className = "action-btn delete-btn";
    delBtn.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';
    delBtn.onclick = (e) => {
      e.stopPropagation();
      deleteItem(item.id);
    };

    const previewBtn = document.createElement("button");
    previewBtn.className = "action-btn preview-btn";
    previewBtn.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>';
    previewBtn.onclick = (e) => {
      e.stopPropagation();
      openPreview(item);
    };
    actions.appendChild(previewBtn);

    actions.appendChild(pinBtn);
    actions.appendChild(delBtn);

    li.appendChild(textContent);
    li.appendChild(actions);

    li.addEventListener("mouseenter", () => {
      selectedIndex = index;
      document.body.classList.remove("is-keyboard-navigating");
      updateSelection();
    });

    li.onclick = () => pasteItem(item.id);

    clipboardList.appendChild(li);
  });
}

async function togglePin(id) {
  try {
    await invoke("toggle_pin", { id });
    fetchHistory();
  } catch (err) {
    console.error(err);
  }
}

async function deleteItem(id) {
  try {
    await invoke("delete_item", { id });
    fetchHistory();
  } catch (err) {
    console.error(err);
  }
}

async function pasteItem(id) {
  if (isPasting || id == null) {
    return;
  }

  isPasting = true;
  setPasteUiState(true);

  try {
    await invoke("paste_item", { id });
  } catch (error) {
    console.error("Failed to paste item:", error);
  } finally {
    setTimeout(() => {
      isPasting = false;
      setPasteUiState(false);
    }, 80);
  }
}

let searchTimeout;
searchInput.addEventListener("input", (e) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    currentSearch = e.target.value;
    selectedIndex = -1;
    fetchHistory();
  }, 100);
});

function updateSelection() {
  const items = clipboardList.children;
  for (let i = 0; i < items.length; i += 1) {
    const item = items[i];
    const shouldFocus = i === selectedIndex;
    item.classList.toggle("focused", shouldFocus);
    item.classList.toggle(
      "mouse-active",
      shouldFocus &&
        !document.body.classList.contains("is-keyboard-navigating"),
    );
  }
}

clipboardList.addEventListener("mouseleave", () => {
  document.body.classList.add("is-keyboard-navigating");
  updateSelection();
});

window.addEventListener("keydown", (e) => {
  if (["ArrowUp", "ArrowDown"].includes(e.key)) {
    document.body.classList.add("is-keyboard-navigating");
  }

  if (e.key === "Escape") {
    if (previewPanel.classList.contains("open")) {
      closePreview();
    } else {
      invoke("hide_window");
    }
  } else if (e.key === "ArrowDown") {
    e.preventDefault();
    if (selectedIndex < historyItems.length - 1) {
      selectedIndex += 1;
      updateSelection();
      scrollToSelected();
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (selectedIndex > 0) {
      selectedIndex -= 1;
      updateSelection();
      scrollToSelected();
    }
  } else if (e.key === "Enter") {
    e.preventDefault();
    if (selectedIndex >= 0 && selectedIndex < historyItems.length) {
      pasteItem(historyItems[selectedIndex].id);
    }
  }
});

function scrollToSelected() {
  const selectedEl = clipboardList.children[selectedIndex];
  if (selectedEl) {
    selectedEl.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
}

window.addEventListener("blur", () => {
  invoke("hide_window");
});

window.addEventListener("focus", () => {
  closePreview();
  searchInput.focus();
  searchInput.select();
  selectedIndex = -1;
  currentSearch = "";
  searchInput.value = "";
  fetchHistory();
});

fetchHistory();

if (listen) {
  listen("clipboard-update", () => {
    fetchHistory();
  });
} else {
  console.warn("Tauri event listen API not found on window.__TAURI__");
}
