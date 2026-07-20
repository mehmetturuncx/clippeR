const { invoke } = window.__TAURI__.core;

let lastHistoryJSON = "";

async function refreshHistory() {
  const history = await invoke("get_history");
  const historyJSON = JSON.stringify(history);

  if (historyJSON === lastHistoryJSON) {
    return; // nothing changed, do not touch the screen
  }
  lastHistoryJSON = historyJSON;

  const container = document.querySelector("#history-list");
  container.innerHTML = "";

  for (const [id, content] of history) {
    const row = document.createElement("div");
    row.className = "item-row";

    const wrapper = document.createElement("div");
    wrapper.className = "item-content-wrapper";

    const button = document.createElement("button");
    button.className = "item-button";
    button.textContent = content;
    button.addEventListener("click", async () => {
      await invoke("copy_item", { content });
    });
    
    wrapper.appendChild(button);

    // If text is long (e.g. > 120 chars or contains multiple newlines), add expand button
    if (content.length > 120 || (content.match(/\n/g) || []).length >= 2) {
      const expandBtn = document.createElement("button");
      expandBtn.className = "expand-btn";
      expandBtn.textContent = "▼";
      expandBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        button.classList.toggle("expanded");
        expandBtn.textContent = button.classList.contains("expanded") ? "▲" : "▼";
      });
      wrapper.appendChild(expandBtn);
    }

    const deleteBtn = document.createElement("button");
    deleteBtn.className = "delete-button";
    deleteBtn.textContent = "✖";
    deleteBtn.addEventListener("click", async () => {
      await invoke("delete_item", { id });
      await refreshHistory();
    });

    row.appendChild(wrapper);
    row.appendChild(deleteBtn);
    container.appendChild(row);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  refreshHistory();
  setInterval(refreshHistory, 500);
});