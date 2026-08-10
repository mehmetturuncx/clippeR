const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// SVG icon templates
const ICON_DELETE = `<svg viewBox="0 0 24 24"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>`;
const ICON_EXPAND = `<svg viewBox="0 0 24 24"><path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z"/></svg>`;
const ICON_COLLAPSE = `<svg viewBox="0 0 24 24"><path d="M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6-6 6z"/></svg>`;
const ICON_PIN = `<svg viewBox="0 0 24 24"><path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2z"/></svg>`;

// Modifier bitmask: Alt=1, Ctrl=2, Shift=4
function shortcutLabel(modBitmask, keyIndex) {
  const parts = [];
  if (modBitmask & 2) parts.push("Ctrl");
  if (modBitmask & 1) parts.push("Alt");
  if (modBitmask & 4) parts.push("Shift");
  parts.push(String.fromCharCode(65 + keyIndex));
  return parts.join(" + ");
}

let lastHistoryJSON = "";

async function refreshHistory() {
  const history = await invoke("get_history");
  const historyJSON = JSON.stringify(history);
  const searchQuery = document.getElementById("search-bar").value.toLowerCase();

  if (historyJSON === lastHistoryJSON && searchQuery === (refreshHistory._lastQuery || "")) {
    return;
  }
  lastHistoryJSON = historyJSON;
  refreshHistory._lastQuery = searchQuery;

  const container = document.querySelector("#history-list");
  container.innerHTML = "";

  const filteredHistory = history.filter(([id, content, item_type]) =>
    item_type === "image" || content.toLowerCase().includes(searchQuery)
  );

  if (filteredHistory.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = searchQuery ? "No results found" : "Clipboard is empty";
    container.appendChild(empty);
    return;
  }

  for (const [id, content, item_type, pinned] of filteredHistory) {
    const row = document.createElement("div");
    row.className = `item-row${pinned ? " pinned" : ""}`;

    const wrapper = document.createElement("div");
    wrapper.className = "item-content-wrapper";

    const button = document.createElement("button");
    button.className = "item-button";

    if (item_type === "image") {
      const secureLink = window.__TAURI__.core.convertFileSrc(content);
      const img = document.createElement("img");
      img.className = "img-style";
      img.src = secureLink;
      button.appendChild(img);
    } else {
      button.textContent = content;
      button.addEventListener("click", async () => {
        await invoke("copy_item", { content });
      });
    }

    wrapper.appendChild(button);

    // Action bar (expand + pin)
    const actionsBar = document.createElement("div");
    actionsBar.className = "item-actions";

    // Expand/collapse for long text entries
    if (item_type === "text" && (content.length > 120 || (content.match(/\n/g) || []).length >= 2)) {
      const expandBtn = document.createElement("button");
      expandBtn.className = "expand-btn";
      expandBtn.innerHTML = ICON_EXPAND;
      expandBtn.title = "Expand";
      expandBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        button.classList.toggle("expanded");
        if (button.classList.contains("expanded")) {
          expandBtn.innerHTML = ICON_COLLAPSE;
          expandBtn.title = "Collapse";
        } else {
          expandBtn.innerHTML = ICON_EXPAND;
          expandBtn.title = "Expand";
        }
      });
      actionsBar.appendChild(expandBtn);
    }

    // Pin button
    const pinBtn = document.createElement("button");
    pinBtn.className = `pin-btn${pinned ? " active" : ""}`;
    pinBtn.innerHTML = ICON_PIN;
    pinBtn.title = pinned ? "Unpin" : "Pin";
    pinBtn.addEventListener("click", async (e) => {
      e.stopPropagation();
      await invoke("toggle_pin", { id });
      lastHistoryJSON = "";
      await refreshHistory();
    });
    actionsBar.appendChild(pinBtn);

    wrapper.appendChild(actionsBar);

    const deleteBtn = document.createElement("button");
    deleteBtn.className = "delete-button";
    deleteBtn.innerHTML = ICON_DELETE;
    deleteBtn.title = "Delete";
    deleteBtn.addEventListener("click", async () => {
      await invoke("delete_item", { id });
      lastHistoryJSON = "";
      await refreshHistory();
    });

    row.appendChild(wrapper);
    row.appendChild(deleteBtn);
    container.appendChild(row);
  }
}

// Listen for clipboard changes from backend
listen("clipboard-changed", () => {
  lastHistoryJSON = "";
  refreshHistory();
});

document.addEventListener("DOMContentLoaded", async () => {
  const historyList = document.getElementById("history-list");

  // ── Settings Panel ──
  const settingsBtn = document.getElementById("settings-btn");
  const settingsPanel = document.getElementById("settings-panel");
  const settingsClose = document.getElementById("settings-close");
  const historyLimitSlider = document.getElementById("history-limit");
  const limitValueDisplay = document.getElementById("limit-value");
  const shortcutInput = document.getElementById("shortcut-input");
  const autostartToggle = document.getElementById("autostart-toggle");

  // Load saved settings
  const settings = await invoke("get_settings");
  const savedLimit = settings.history_limit || 100;
  const savedMod = settings.shortcut_mod ?? 1;
  const savedKey = settings.shortcut_key ?? 21;
  historyLimitSlider.value = savedLimit;
  limitValueDisplay.textContent = savedLimit;
  shortcutInput.textContent = shortcutLabel(savedMod, savedKey);

  // Load autostart state
  try {
    const isAutostart = await invoke("get_autostart");
    autostartToggle.checked = isAutostart;
  } catch { autostartToggle.checked = false; }

  // Settings toggle (full overlay)
  function openSettings() {
    settingsPanel.classList.remove("hidden");
    historyList.classList.add("hidden");
    settingsBtn.classList.add("active");
  }
  function closeSettings() {
    settingsPanel.classList.add("hidden");
    historyList.classList.remove("hidden");
    settingsBtn.classList.remove("active");
    shortcutInput.classList.remove("listening");
  }

  settingsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    if (!settingsPanel.classList.contains("hidden")) {
      closeSettings();
    } else {
      openSettings();
    }
  });

  settingsClose.addEventListener("click", closeSettings);

  // History limit
  historyLimitSlider.addEventListener("input", () => {
    limitValueDisplay.textContent = historyLimitSlider.value;
  });

  historyLimitSlider.addEventListener("change", async () => {
    const value = parseInt(historyLimitSlider.value);
    await invoke("set_setting", { key: "history_limit", value });
    lastHistoryJSON = "";
    await refreshHistory();
  });

  // ── Shortcut Recorder ──
  shortcutInput.addEventListener("focus", () => {
    shortcutInput.classList.add("listening");
    shortcutInput.textContent = "Press a shortcut...";
  });

  shortcutInput.addEventListener("blur", () => {
    if (shortcutInput.classList.contains("listening")) {
      // Cancelled — restore saved value
      shortcutInput.classList.remove("listening");
      shortcutInput.textContent = shortcutLabel(
        settings.shortcut_mod ?? savedMod,
        settings.shortcut_key ?? savedKey
      );
    }
  });

  shortcutInput.addEventListener("keydown", async (e) => {
    e.preventDefault();
    e.stopPropagation();

    if (!shortcutInput.classList.contains("listening")) return;

    // Build modifier bitmask
    let modBitmask = 0;
    if (e.altKey) modBitmask |= 1;
    if (e.ctrlKey) modBitmask |= 2;
    if (e.shiftKey) modBitmask |= 4;

    // Only accept letter keys (A–Z) with at least one modifier
    const key = e.key.toUpperCase();
    if (key.length !== 1 || key < "A" || key > "Z" || modBitmask === 0) return;

    const keyIndex = key.charCodeAt(0) - 65;

    try {
      await invoke("set_shortcut", { modifier: modBitmask, key: keyIndex });
      // Update stored values for blur fallback
      settings.shortcut_mod = modBitmask;
      settings.shortcut_key = keyIndex;
      shortcutInput.textContent = shortcutLabel(modBitmask, keyIndex);
      shortcutInput.classList.remove("listening");
      shortcutInput.blur();
    } catch (err) {
      shortcutInput.textContent = "Failed — try again";
      setTimeout(() => {
        shortcutInput.textContent = "Press a shortcut...";
      }, 1500);
    }
  });

  // Autostart toggle
  autostartToggle.addEventListener("change", async () => {
    try {
      await invoke("set_autostart", { enabled: autostartToggle.checked });
    } catch (err) {
      console.error("Failed to set autostart:", err);
      autostartToggle.checked = !autostartToggle.checked;
    }
  });

  // ── Clear All ──
  const clearBtn = document.getElementById("clear-all-btn");
  const confirmPopup = document.getElementById("clear-confirm-popup");
  const confirmYes = document.getElementById("confirm-yes");
  const confirmNo = document.getElementById("confirm-no");

  clearBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    confirmPopup.classList.remove("hidden");
  });

  confirmNo.addEventListener("click", () => {
    confirmPopup.classList.add("hidden");
  });

  document.addEventListener("click", (e) => {
    if (!confirmPopup.contains(e.target) && !clearBtn.contains(e.target)) {
      confirmPopup.classList.add("hidden");
    }
  });

  confirmYes.addEventListener("click", async () => {
    confirmPopup.classList.add("hidden");
    await invoke("clear_all");
    lastHistoryJSON = "";
    await refreshHistory();
  });

  document.getElementById("search-bar").addEventListener("input", () => {
    refreshHistory._lastQuery = null;
    refreshHistory();
  });

  refreshHistory();
});