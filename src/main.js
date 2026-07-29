const { invoke } = window.__TAURI__.core;

// SVG icon templates
const ICON_DELETE = `<svg viewBox="0 0 24 24"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>`;
const ICON_EXPAND = `<svg viewBox="0 0 24 24"><path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z"/></svg>`;
const ICON_COLLAPSE = `<svg viewBox="0 0 24 24"><path d="M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6-6 6z"/></svg>`;

let lastHistoryJSON = "";
let lastSearchQuery = "";

async function refreshHistory() {
  const history = await invoke("get_history");
  const historyJSON = JSON.stringify(history);
  const searchQuery = document.getElementById("search-bar").value.toLowerCase();

  if (historyJSON === lastHistoryJSON && searchQuery === lastSearchQuery) {
    return;
  }
  lastHistoryJSON = historyJSON;
  lastSearchQuery = searchQuery;

  const container = document.querySelector("#history-list");
  container.innerHTML = "";

  const filteredHistory = history.filter(([id, content]) =>
    content.toLowerCase().includes(searchQuery)
  );

  if (filteredHistory.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = searchQuery ? "No results found" : "Clipboard is empty";
    container.appendChild(empty);
    return;
  }

  for (const [id, content, item_type] of filteredHistory) {
    const row = document.createElement("div");
    row.className = "item-row";

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
      wrapper.appendChild(expandBtn);
    }

    const deleteBtn = document.createElement("button");
    deleteBtn.className = "delete-button";
    deleteBtn.innerHTML = ICON_DELETE;
    deleteBtn.title = "Delete";
    deleteBtn.addEventListener("click", async () => {
      await invoke("delete_item", { id });
      await refreshHistory();
    });

    row.appendChild(wrapper);
    row.appendChild(deleteBtn);
    container.appendChild(row);
  }
}

// Clear all history
document.addEventListener("DOMContentLoaded", () => {
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
    const history = await invoke("get_history");
    for (const [id] of history) {
      await invoke("delete_item", { id });
    }
    lastHistoryJSON = "";
    await refreshHistory();
  });

  document.getElementById("search-bar").addEventListener("input", () => {
    refreshHistory();
  });

  refreshHistory();
  setInterval(refreshHistory, 500);
});