import "./style.css";
import { invoke } from "@tauri-apps/api/core";

type HostEntry = {
  id: string;
  ip: string;
  domain: string;
  enabled: boolean;
};

type BackupItem = {
  name: string;
  path: string;
  created_at: string;
};

const DEFAULT_REMOTE_HOSTS_URL = "";
const URL_HISTORY_KEY = "xhosts.remoteHostsUrlHistory";

// Icons as SVG strings
const Icons = {
  list: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M2 3h12v1H2V3zm0 3h12v1H2V6zm0 3h12v1H2V9zm0 3h12v1H2v-1z"/></svg>`,
  edit: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M11.013 1.427a1.75 1.75 0 012.474 0l1.086 1.086a1.75 1.75 0 010 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 01-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61zm1.414 1.06a.25.25 0 00-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 000-.354l-1.086-1.086zM11.189 6.25L9.75 4.81l-6.286 6.287a.25.25 0 00-.067.108l-1.039 3.638 3.638-1.04a.25.25 0 00.108-.067L11.19 6.25z"/></svg>`,
  backup: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M2.75 2.5a.75.75 0 00-.75.75v9.5c0 .414.336.75.75.75h10.5a.75.75 0 00.75-.75v-9.5a.75.75 0 00-.75-.75H2.75zM2 1.75C2 .784 2.784 0 3.75 0h8.5C13.216 0 14 .784 14 1.75v12.5c0 .966-.784 1.75-1.75 1.75h-8.5C2.784 16 2 15.216 2 14.25V1.75zM5 7a1 1 0 100-2 1 1 0 000 2zm1.5 4.5a.5.5 0 01-.5-.5v-2a.5.5 0 011 0v2a.5.5 0 01-.5.5zM5 11a1 1 0 100-2 1 1 0 000 2zm5-7a1 1 0 11-2 0 1 1 0 012 0zm-1.5 4.5a.5.5 0 01-.5-.5v-2a.5.5 0 011 0v2a.5.5 0 01-.5.5zM9 11a1 1 0 100-2 1 1 0 000 2z"/></svg>`,
  refresh: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M1.705 8.005a.75.75 0 01.834.656 5.5 5.5 0 009.592 2.97l-1.204-1.204a.25.25 0 01.177-.427h3.646a.25.25 0 01.25.25v3.646a.25.25 0 01-.427.177l-1.38-1.38A7.002 7.002 0 011.437 7.17a.75.75 0 01.656-.834zM8 1.5a6.5 6.5 0 00-6.022 4.095.75.75 0 11-1.375-.59 8 8 0 0114.005 2.944l1.38 1.38A.25.25 0 0115.75 8V4.354a.25.25 0 01.427-.177l1.204 1.204A7 7 0 008 1.5z"/></svg>`,
  plus: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M7.75 2a.75.75 0 01.75.75V7h4.25a.75.75 0 110 1.5H8.5v4.25a.75.75 0 11-1.5 0V8.5H2.75a.75.75 0 010-1.5H7V2.75A.75.75 0 017.75 2z"/></svg>`,
  trash: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M11 1.75V3h2.25a.75.75 0 010 1.5H2.75a.75.75 0 010-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75zM4.496 6.675a.75.75 0 10-1.492.15l.66 6.6A1.75 1.75 0 005.405 15h5.19c.9 0 1.652-.68 1.741-1.575l.66-6.6a.75.75 0 00-1.492-.149l-.66 6.6a.25.25 0 01-.249.225h-5.19a.25.25 0 01-.249-.225l-.66-6.6z"/></svg>`,
  check: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z"/></svg>`,
  warning: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0114.082 15H1.918a1.75 1.75 0 01-1.543-2.575L6.457 1.047zM8 5a.75.75 0 00-.75.75v3.5a.75.75 0 001.5 0v-3.5A.75.75 0 008 5zm0 7a1 1 0 100-2 1 1 0 000 2z"/></svg>`,
  dns: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M8 0a8 8 0 100 16A8 8 0 008 0zM1.5 8a6.5 6.5 0 1111.596 4.035l-1.781-1.78a.75.75 0 00-1.06 1.06l1.78 1.781A6.5 6.5 0 011.5 8zm3.535-2.536a.75.75 0 001.06-1.06l-1.78-1.781A6.5 6.5 0 0114.5 8a.75.75 0 00-1.5 0 5 5 0 11-7.965-3.535z"/></svg>`,
  save: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M11.28 1.78a.75.75 0 00-1.06 0L7.25 4.75 6.22 3.72a.75.75 0 00-1.06 1.06l1.75 1.75a.75.75 0 001.06 0l3.25-3.25a.75.75 0 000-1.06zM4 7.75a.75.75 0 01.75-.75h6.5a.75.75 0 010 1.5h-6.5A.75.75 0 014 7.75zm0 3a.75.75 0 01.75-.75h6.5a.75.75 0 010 1.5h-6.5A.75.75 0 014 10.75zM1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0114.25 15H1.75A1.75 1.75 0 010 13.25V2.75C0 1.784.784 1 1.75 1zM1.5 2.75v10.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25V2.75a.25.25 0 00-.25-.25H1.75a.25.25 0 00-.25.25z"/></svg>`,
  restore: `<svg viewBox="0 0 16 16" fill="currentColor"><path d="M1.705 8.005a.75.75 0 01.834.656 5.5 5.5 0 009.592 2.97l-1.204-1.204a.25.25 0 01.177-.427h3.646a.25.25 0 01.25.25v3.646a.25.25 0 01-.427.177l-1.38-1.38A7.002 7.002 0 011.437 7.17a.75.75 0 01.656-.834z"/></svg>`,
};

// App HTML Structure
const appHTML = `
  <div class="app-container">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <div class="app-title">x-hosts</div>
        <div class="app-subtitle">Hosts File Manager</div>
      </div>
      
      <nav class="sidebar-nav">
        <div class="nav-section">
          <div class="nav-section-title">Editing Mode</div>
          <button class="nav-item" data-mode="table" id="nav-table">
            <span class="icon">${Icons.list}</span>
            <span>Table View</span>
          </button>
          <button class="nav-item active" data-mode="raw" id="nav-raw">
            <span class="icon">${Icons.edit}</span>
            <span>Raw Text</span>
          </button>
        </div>
        
        <div class="nav-section">
          <div class="nav-section-title">🛠️ Toolbox</div>
          <div class="form-group" style="padding: 4px 8px 10px 8px; margin-bottom: 8px;">
            <label class="form-label" for="input-remote-url">🌐 Remote Hosts URL</label>
            <input id="input-remote-url" class="form-input" type="text" placeholder="https://.../hosts" />
            <select id="select-url-history" class="form-input" style="margin-top: 8px; display: none;">
              <option value="">📋 Select a saved URL...</option>
            </select>
          </div>
          <button class="nav-item" id="btn-sync-remote-hosts">
            <span class="icon">${Icons.refresh}</span>
            <span>🔄 Sync Remote Hosts</span>
          </button>
          <button class="nav-item" id="btn-create-backup">
            <span class="icon">${Icons.backup}</span>
            <span>💾 Create Backup</span>
          </button>
          <button class="nav-item" id="btn-flush-dns">
            <span class="icon">${Icons.dns}</span>
            <span>🧹 Flush DNS Cache</span>
          </button>
        </div>
      </nav>
      
      <div class="sidebar-footer">
        <div id="admin-warning" class="admin-warning" style="display: none;">
          <span class="icon">${Icons.warning}</span>
          <span>⚠️ Administrator access required</span>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <!-- Toolbar -->
      <header class="toolbar">
        <div class="toolbar-section">
          <h2 class="toolbar-title" id="view-title">📝 Raw Text Editor</h2>
          <span class="toolbar-subtitle" id="entry-count">⏳ Loading...</span>
        </div>
        <div class="toolbar-section">
          <div class="mode-switcher" id="mode-switcher">
            <button class="mode-btn" data-mode="table">Table</button>
            <button class="mode-btn active" data-mode="raw">Raw</button>
          </div>
        </div>
      </header>

      <!-- Content Area -->
      <div class="content-area">
        <!-- Table View -->
        <div class="view-container" id="view-table">
          <div class="content-panel">
            <!-- Add Entry Form -->
            <div class="add-entry-section">
              <div class="add-entry-title">
                <span class="icon">${Icons.plus}</span>
                ➕ Add New Entry
              </div>
              <div class="add-entry-form">
                <div class="form-group">
                  <label class="form-label">🌐 IP Address</label>
                  <input type="text" class="form-input" id="input-ip" placeholder="e.g. 127.0.0.1">
                </div>
                <div class="form-group">
                  <label class="form-label">🔗 Domain</label>
                  <input type="text" class="form-input" id="input-domain" placeholder="e.g. example.local">
                </div>
                <button class="btn btn-primary" id="btn-add-entry">
                  <span class="icon">${Icons.plus}</span>
                  ✨ Add
                </button>
              </div>
            </div>
            
            <!-- Entries Table -->
            <div class="table-container">
              <table class="data-table" id="entries-table">
                <thead>
                  <tr>
                    <th class="col-status">✅ Status</th>
                    <th>🌐 IP Address</th>
                    <th>🔗 Domain</th>
                    <th class="col-actions">⚙️ Actions</th>
                  </tr>
                </thead>
                <tbody id="entries-tbody">
                  <!-- Dynamic content -->
                </tbody>
              </table>
              <div class="empty-state" id="empty-state" style="display: none;">
                <div class="empty-state-icon">📭</div>
                <div class="empty-state-title">🤔 No hosts entries yet</div>
                <div class="empty-state-desc">💡 Add an IP address and a domain above to create a new mapping</div>
              </div>
            </div>
          </div>
          
          <!-- Backups Section -->
          <div class="content-panel backup-section">
            <div class="backup-header">
              <span class="backup-title">📦 Backup Management</span>
              <button class="btn btn-ghost btn-icon" id="btn-reload-backups" title="🔄 Refresh backup list">
                <span class="icon">${Icons.refresh}</span>
              </button>
            </div>
            <div class="backup-list" id="backup-list">
              <!-- Dynamic content -->
            </div>
            <div class="empty-state" id="backup-empty" style="display: none; padding: 30px;">
              <div class="empty-state-icon">📂</div>
              <div class="empty-state-title">🤷 No backup files yet</div>
              <div class="empty-state-desc">💾 Click "Create Backup" above to save the current configuration</div>
            </div>
          </div>
        </div>

        <!-- Raw Editor View -->
        <div class="view-container active" id="view-raw">
          <div class="content-panel" style="height: 100%;">
            <div class="raw-editor-container">
              <div class="raw-editor-toolbar">
                <span class="raw-editor-info">📝 Edit the hosts file directly. Batch editing is supported.</span>
                <div class="toolbar-section">
                  <button class="btn btn-secondary" id="btn-reload-raw">
                    <span class="icon">${Icons.refresh}</span>
                    🔄 Reload
                  </button>
                  <button class="btn btn-primary" id="btn-save-raw">
                    <span class="icon">${Icons.save}</span>
                    💾 Save Changes
                  </button>
                </div>
              </div>
              <textarea class="raw-textarea" id="raw-textarea" placeholder="# 📝 Edit the hosts file content here...
# 📋 Format: IP_ADDRESS DOMAIN
# 💡 Example:
# 127.0.0.1  localhost
# 127.0.0.1  example.local"></textarea>
            </div>
          </div>
        </div>
      </div>

      <!-- Status Bar -->
      <footer class="status-bar">
        <div class="status-bar-left">
          <div class="status-message">
            <span class="status-indicator" id="status-indicator"></span>
            <span id="status-text">✅ Ready</span>
          </div>
        </div>
        <div class="status-bar-right">
          <span id="hosts-path">⏳ Loading...</span>
        </div>
      </footer>
    </main>
  </div>
`;

// Initialize App
const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("App root not found");
}
app.innerHTML = appHTML;

// State
let currentMode: "table" | "raw" = "raw";
let hasAdminPermission = false;
let entriesData: HostEntry[] = [];
let urlHistory: string[] = [];
let rawLoadSequence = 0;

function summarizeContent(content: string): string {
  const lines = content.length === 0 ? 0 : content.split(/\r?\n/).length;
  const githubStart = (content.match(/#Github Hosts Start/g) || []).length;
  const githubEnd = (content.match(/#Github Hosts End/g) || []).length;
  const firstLine = (content.split(/\r?\n/).find(line => line.trim().length > 0) || "")
    .slice(0, 80)
    .replace(/"/g, "'");

  return `len=${content.length} lines=${lines} github_start=${githubStart} github_end=${githubEnd} first_line="${firstLine}"`;
}

async function writeDiagnosticLog(component: string, action: string, details: string): Promise<void> {
  try {
    await invoke("append_diagnostic_log", { component, action, details });
  } catch (error) {
    console.warn("Failed to write diagnostic log:", error);
  }
}

async function logFrontendEvent(action: string, details: string): Promise<void> {
  await writeDiagnosticLog("main", action, details);
}

// DOM Elements
const els = {
  // Navigation
  navTable: document.getElementById("nav-table") as HTMLButtonElement,
  navRaw: document.getElementById("nav-raw") as HTMLButtonElement,
  modeBtns: document.querySelectorAll<HTMLButtonElement>(".mode-btn"),
  viewContainers: document.querySelectorAll<HTMLElement>(".view-container"),
  viewTitle: document.getElementById("view-title") as HTMLElement,
  
  // Table view
  entriesTbody: document.getElementById("entries-tbody") as HTMLTableSectionElement,
  emptyState: document.getElementById("empty-state") as HTMLDivElement,
  entryCount: document.getElementById("entry-count") as HTMLElement,
  inputIp: document.getElementById("input-ip") as HTMLInputElement,
  inputDomain: document.getElementById("input-domain") as HTMLInputElement,
  btnAddEntry: document.getElementById("btn-add-entry") as HTMLButtonElement,
  
  // Raw editor
  rawTextarea: document.getElementById("raw-textarea") as HTMLTextAreaElement,
  btnReloadRaw: document.getElementById("btn-reload-raw") as HTMLButtonElement,
  btnSaveRaw: document.getElementById("btn-save-raw") as HTMLButtonElement,
  
  // Backups
  backupList: document.getElementById("backup-list") as HTMLDivElement,
  backupEmpty: document.getElementById("backup-empty") as HTMLDivElement,
  btnReloadBackups: document.getElementById("btn-reload-backups") as HTMLButtonElement,
  
  // Sidebar tools
  btnCreateBackup: document.getElementById("btn-create-backup") as HTMLButtonElement,
  btnFlushDns: document.getElementById("btn-flush-dns") as HTMLButtonElement,
  btnSyncRemoteHosts: document.getElementById("btn-sync-remote-hosts") as HTMLButtonElement,
  inputRemoteUrl: document.getElementById("input-remote-url") as HTMLInputElement,
  selectUrlHistory: document.getElementById("select-url-history") as HTMLSelectElement,
  adminWarning: document.getElementById("admin-warning") as HTMLDivElement,
  
  // Status
  statusText: document.getElementById("status-text") as HTMLElement,
  statusIndicator: document.getElementById("status-indicator") as HTMLElement,
  hostsPath: document.getElementById("hosts-path") as HTMLElement,
};

// Status Management
function setStatus(message: string, type: "info" | "success" | "error" | "processing" = "info"): void {
  els.statusText.textContent = message;
  els.statusIndicator.className = "status-indicator";
  
  if (type === "success") {
    els.statusIndicator.classList.add("success");
  } else if (type === "error") {
    els.statusIndicator.classList.add("error");
  } else if (type === "processing") {
    els.statusIndicator.classList.add("processing");
  }
  
  // Clear success/error status after 3 seconds
  if (type === "success" || type === "error") {
    setTimeout(() => {
      els.statusText.textContent = "✅ Ready";
      els.statusIndicator.className = "status-indicator";
    }, 3000);
  }
}

// Mode Switching
function switchMode(mode: "table" | "raw"): void {
  currentMode = mode;
  void logFrontendEvent("switch_mode", `mode=${mode}`);

  // Update nav items
  els.navTable.classList.toggle("active", mode === "table");
  els.navRaw.classList.toggle("active", mode === "raw");

  // Update mode buttons
  els.modeBtns.forEach(btn => {
    btn.classList.toggle("active", btn.dataset.mode === mode);
  });

  // Update view containers
  els.viewContainers.forEach(container => {
    container.classList.toggle("active", container.id === `view-${mode}`);
  });

  // Update title
  els.viewTitle.textContent = mode === "table" ? "📋 Hosts Entries" : "📝 Raw Text Editor";

  // Load raw content when switching to raw mode
  if (mode === "raw") {
    void loadRawContent("switch_mode");
  }
}

// Admin Check
async function checkAdmin(): Promise<void> {
  try {
    hasAdminPermission = await invoke<boolean>("ensure_admin");
    await logFrontendEvent("check_admin:ok", `elevated=${hasAdminPermission}`);
    if (!hasAdminPermission) {
      els.adminWarning.style.display = "flex";
      setStatus("⚠️ Administrator access is required to save changes", "error");
    } else {
      els.adminWarning.style.display = "none";
    }
  } catch (error) {
    console.error("Admin check failed:", error);
    hasAdminPermission = false;
    els.adminWarning.style.display = "flex";
    await logFrontendEvent("check_admin:error", String(error));
  }
}

// Load Entries
async function loadEntries(): Promise<void> {
  setStatus("⏳ Loading entries...", "processing");

  try {
    entriesData = await invoke<HostEntry[]>("list_entries");
    renderEntries();
    setStatus(`✅ Loaded ${entriesData.length} entries`, "success");
  } catch (error) {
    console.error("Failed to load entries:", error);
    setStatus("❌ Failed to load entries", "error");
  }
}

// Render Entries Table
function renderEntries(): void {
  els.entriesTbody.innerHTML = "";
  els.entryCount.textContent = `📊 ${entriesData.length} entries`;
  
  if (entriesData.length === 0) {
    els.emptyState.style.display = "block";
    return;
  }
  
  els.emptyState.style.display = "none";
  
  for (const entry of entriesData) {
    const tr = document.createElement("tr");
    tr.className = "fade-in";
    tr.innerHTML = `
      <td class="col-status">
        <div class="checkbox-wrapper">
          <div class="custom-checkbox ${entry.enabled ? "checked" : ""}" 
               data-action="toggle" 
               data-id="${entry.id}"
               title="${entry.enabled ? "🚫 Click to disable" : "✅ Click to enable"}">
          </div>
        </div>
      </td>
      <td><code>${escapeHtml(entry.ip)}</code></td>
      <td>${escapeHtml(entry.domain)}</td>
      <td class="col-actions">
        <button class="btn btn-danger btn-icon" data-action="delete" data-id="${entry.id}" title="🗑️ Delete">
          <span class="icon">${Icons.trash}</span>
        </button>
      </td>
    `;
    els.entriesTbody.appendChild(tr);
  }
}

// Escape HTML
function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

// Add Entry
async function addEntry(): Promise<void> {
  const ip = els.inputIp.value.trim();
  const domain = els.inputDomain.value.trim();

  if (!ip || !domain) {
    setStatus("⚠️ IP address and domain are required", "error");
    return;
  }

  // Basic IP validation
  const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/;
  if (!ipRegex.test(ip)) {
    setStatus("⚠️ Invalid IP address format", "error");
    return;
  }

  setStatus("⏳ Adding entry...", "processing");

  try {
    await invoke("add_entry", { ip, domain });
    els.inputIp.value = "";
    els.inputDomain.value = "";
    await loadEntries();
    setStatus("✅ Entry added successfully", "success");
  } catch (error) {
    console.error("Failed to add entry:", error);
    setStatus("❌ Failed to add entry", "error");
  }
}

// Toggle Entry
async function toggleEntry(id: string, enabled: boolean): Promise<void> {
  setStatus("⏳ Updating entry state...", "processing");

  try {
    await invoke("toggle_entry", { id, enabled });
    await loadEntries();
    setStatus(enabled ? "✅ Entry enabled" : "🚫 Entry disabled", "success");
  } catch (error) {
    console.error("Failed to toggle entry:", error);
    setStatus("❌ Failed to update the entry state", "error");
  }
}

// Delete Entry
async function deleteEntry(id: string): Promise<void> {
  if (!confirm("🗑️ Delete this entry?\n\n⚠️ This action cannot be undone.")) {
    return;
  }

  setStatus("⏳ Deleting entry...", "processing");

  try {
    await invoke("delete_entry", { id });
    await loadEntries();
    setStatus("✅ Entry deleted", "success");
  } catch (error) {
    console.error("Failed to delete entry:", error);
    setStatus("❌ Failed to delete the entry", "error");
  }
}

// Load Backups
async function loadBackups(): Promise<void> {
  try {
    const backups = await invoke<BackupItem[]>("list_backups");
    renderBackups(backups);
  } catch (error) {
    console.error("Failed to load backups:", error);
    setStatus("❌ Failed to load the backup list", "error");
  }
}

// Render Backups
function renderBackups(backups: BackupItem[]): void {
  els.backupList.innerHTML = "";
  
  if (backups.length === 0) {
    els.backupEmpty.style.display = "block";
    return;
  }
  
  els.backupEmpty.style.display = "none";
  
  for (const backup of backups) {
    const item = document.createElement("div");
    item.className = "backup-item";
    item.innerHTML = `
      <div class="backup-info">
        <span class="backup-name">${escapeHtml(backup.name)}</span>
        <span class="backup-time">${formatDate(backup.created_at)}</span>
      </div>
        <button class="btn btn-secondary btn-icon" data-action="restore" data-path="${escapeHtml(backup.path)}" title="⏪ Restore this backup">
        <span class="icon">${Icons.restore}</span>
      </button>
    `;
    els.backupList.appendChild(item);
  }
}

// Format Date
function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString("en-US", {
      year: "numeric",
      month: "short",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  } catch {
    return dateStr;
  }
}

// Create Backup
async function createBackup(): Promise<void> {
  setStatus("⏳ Creating backup...", "processing");

  try {
    await invoke("create_backup");
    await loadBackups();
    setStatus("✅ Backup created successfully", "success");
  } catch (error) {
    console.error("Failed to create backup:", error);
    setStatus("❌ Failed to create the backup", "error");
  }
}

// Restore Backup
async function restoreBackup(path: string): Promise<void> {
  if (!confirm("⏪ Restore this backup?\n\n⚠️ The current configuration will be overwritten.")) {
    return;
  }

  setStatus("⏳ Restoring backup...", "processing");

  try {
    await invoke("restore_backup", { path });
    await loadEntries();
    setStatus("✅ Backup restored successfully", "success");
  } catch (error) {
    console.error("Failed to restore backup:", error);
    setStatus("❌ Failed to restore the backup", "error");
  }
}

// Flush DNS
async function flushDns(): Promise<void> {
  setStatus("⏳ Flushing the DNS cache...", "processing");

  try {
    const output = await invoke<string>("flush_dns");
    setStatus("✅ DNS cache flushed", "success");
    console.log("DNS flush output:", output);
  } catch (error) {
    console.error("Failed to flush DNS:", error);
    setStatus("❌ Failed to flush the DNS cache", "error");
  }
}

// URL History Management
function loadUrlHistory(): string[] {
  try {
    const stored = localStorage.getItem(URL_HISTORY_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function saveUrlHistory(url: string): void {
  if (!url || url === DEFAULT_REMOTE_HOSTS_URL) return;

  // Remove if exists, add to front, keep max 10
  urlHistory = urlHistory.filter(u => u !== url);
  urlHistory.unshift(url);
  urlHistory = urlHistory.slice(0, 10);

  try {
    localStorage.setItem(URL_HISTORY_KEY, JSON.stringify(urlHistory));
  } catch {
    // Ignore storage errors
  }

  renderUrlHistory();
}

function renderUrlHistory(): void {
  const select = els.selectUrlHistory;
  select.innerHTML = '<option value="">📋 Select a saved URL...</option>';

  if (urlHistory.length === 0) {
    select.style.display = "none";
    return;
  }

  select.style.display = "block";

  for (const url of urlHistory) {
    const option = document.createElement("option");
    option.value = url;
    // Truncate long URLs for display
    const displayUrl = url.length > 35 ? url.substring(0, 32) + "..." : url;
    option.textContent = `📌 ${displayUrl}`;
    select.appendChild(option);
  }
}

function handleUrlHistorySelect(event: Event): void {
  const select = event.target as HTMLSelectElement;
  const url = select.value;
  if (url) {
    els.inputRemoteUrl.value = url;
    select.value = ""; // Reset to placeholder
  }
}

async function syncRemoteHosts(): Promise<void> {
  if (!hasAdminPermission) {
    setStatus("⚠️ Administrator access is required to sync remote hosts", "error");
    await logFrontendEvent("sync_remote_hosts:blocked", "reason=not_admin");
    return;
  }

  const url = els.inputRemoteUrl.value.trim() || DEFAULT_REMOTE_HOSTS_URL;
  if (!url) {
    setStatus("⚠️ Enter a remote hosts URL first", "error");
    await logFrontendEvent("sync_remote_hosts:blocked", "reason=missing_url");
    return;
  }

  // Save current URL to history (if not default)
  if (url !== DEFAULT_REMOTE_HOSTS_URL) {
    saveUrlHistory(url);
  }

  // Always save current URL as last used
  localStorage.setItem("xhosts.remoteHostsUrl", url);

  setStatus("⏳ Syncing remote hosts...", "processing");
  await logFrontendEvent("sync_remote_hosts:start", `url=${url}`);

  try {
    const message = await invoke<string>("sync_remote_hosts", { url });
    const persisted = await invoke<string>("get_hosts_text");
    els.rawTextarea.value = persisted;
    await logFrontendEvent("sync_remote_hosts:post_read", summarizeContent(persisted));
    await loadEntries();
    await loadBackups();
    setStatus(message || "✅ Remote hosts synced and saved to the hosts file", "success");
  } catch (error) {
    console.error("Failed to sync remote hosts:", error);
    await logFrontendEvent("sync_remote_hosts:error", String(error));
    setStatus("❌ Failed to sync remote hosts", "error");
  }
}

// Raw Editor Functions
async function loadRawContent(reason = "manual"): Promise<void> {
  const requestId = ++rawLoadSequence;
  setStatus("⏳ Loading raw content...", "processing");
  await logFrontendEvent("load_raw_content:start", `request_id=${requestId} reason=${reason}`);

  try {
    const content = await invoke<string>("get_hosts_text");
    if (requestId !== rawLoadSequence) {
      await logFrontendEvent(
        "load_raw_content:stale",
        `request_id=${requestId} latest_request_id=${rawLoadSequence}`,
      );
      return;
    }

    els.rawTextarea.value = content;
    await logFrontendEvent(
      "load_raw_content:ok",
      `request_id=${requestId} ${summarizeContent(content)}`,
    );
    setStatus("✅ Raw content loaded", "success");
  } catch (error) {
    console.error("Failed to load raw content:", error);
    await logFrontendEvent("load_raw_content:error", `request_id=${requestId} error=${String(error)}`);
    setStatus("❌ Failed to load raw content", "error");
  }
}

async function saveRawContent(): Promise<void> {
  if (!hasAdminPermission) {
    setStatus("⚠️ Administrator access is required to save", "error");
    await logFrontendEvent("save_raw_content:blocked", "reason=not_admin");
    return;
  }

  const content = els.rawTextarea.value;

  if (!confirm("💾 Save these changes?\n\n⚠️ This will overwrite the current hosts file.")) {
    await logFrontendEvent("save_raw_content:cancelled", summarizeContent(content));
    return;
  }

  setStatus("⏳ Saving changes...", "processing");
  await logFrontendEvent("save_raw_content:start", summarizeContent(content));

  try {
    await invoke("save_hosts_text", { content });
    const persisted = await invoke<string>("get_hosts_text");
    els.rawTextarea.value = persisted;
    await logFrontendEvent("save_raw_content:post_read", summarizeContent(persisted));
    setStatus("✅ Saved and reloaded from disk", "success");
    await loadEntries();
    await loadBackups();
  } catch (error) {
    console.error("Failed to save raw content:", error);
    await logFrontendEvent("save_raw_content:error", String(error));
    setStatus("❌ Failed to save changes", "error");
  }
}

// Event Listeners

// Mode switching
els.navTable.addEventListener("click", () => switchMode("table"));
els.navRaw.addEventListener("click", () => switchMode("raw"));

els.modeBtns.forEach(btn => {
  btn.addEventListener("click", () => {
    const mode = btn.dataset.mode as "table" | "raw";
    switchMode(mode);
  });
});

// Add entry
els.btnAddEntry.addEventListener("click", addEntry);
els.inputIp.addEventListener("keypress", (e) => {
  if (e.key === "Enter") els.inputDomain.focus();
});
els.inputDomain.addEventListener("keypress", (e) => {
  if (e.key === "Enter") addEntry();
});

// Table actions
els.entriesTbody.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  const actionEl = target.closest("[data-action]") as HTMLElement;
  
  if (!actionEl) return;
  
  const action = actionEl.dataset.action;
  const id = actionEl.dataset.id;
  
  if (action === "toggle" && id) {
    const checkbox = actionEl;
    const isChecked = checkbox.classList.contains("checked");
    toggleEntry(id, !isChecked);
  } else if (action === "delete" && id) {
    deleteEntry(id);
  }
});

// Backup actions
els.btnCreateBackup.addEventListener("click", createBackup);
els.btnReloadBackups.addEventListener("click", () => {
  loadBackups();
  setStatus("🔄 Backup list refreshed", "success");
});

els.backupList.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  const actionEl = target.closest("[data-action]") as HTMLElement;
  
  if (!actionEl) return;
  
  const action = actionEl.dataset.action;
  const path = actionEl.dataset.path;
  
  if (action === "restore" && path) {
    restoreBackup(path);
  }
});

// Raw editor actions
els.btnReloadRaw.addEventListener("click", () => {
  void loadRawContent();
});
els.btnSaveRaw.addEventListener("click", saveRawContent);

// Sidebar tools
els.btnFlushDns.addEventListener("click", flushDns);
els.btnSyncRemoteHosts.addEventListener("click", syncRemoteHosts);
els.selectUrlHistory.addEventListener("change", handleUrlHistorySelect);

// Bootstrap
async function bootstrap(): Promise<void> {
  await logFrontendEvent("bootstrap:start", "");

  // Load URL history first
  urlHistory = loadUrlHistory();
  renderUrlHistory();

  // Set current URL (last used or default)
  els.inputRemoteUrl.value =
    localStorage.getItem("xhosts.remoteHostsUrl") || DEFAULT_REMOTE_HOSTS_URL;
  await logFrontendEvent("bootstrap:url", `url=${els.inputRemoteUrl.value}`);

  await checkAdmin();
  await loadEntries();
  await loadBackups();
  switchMode("raw");

  try {
    const path = await invoke<string>("get_hosts_path");
    els.hostsPath.textContent = `📁 ${path}`;
    await logFrontendEvent("bootstrap:hosts_path", `path=${path}`);
  } catch (error) {
    els.hostsPath.textContent = "📁 Hosts path unavailable";
    await logFrontendEvent("bootstrap:hosts_path_error", String(error));
  }
}

void bootstrap();
