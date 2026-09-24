// Appify Modern Manager Dashboard JavaScript Controller
(() => {
  "use strict";

  // Prevent browser contextmenu across manager window
  document.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    return false;
  }, { capture: true });

  // Prevent dragstart on images and cards
  document.addEventListener("dragstart", (e) => {
    if (e.target.tagName === "IMG" || e.target.closest(".app-card") || e.target.closest(".preset-card")) {
      e.preventDefault();
    }
  });

  // Tauri IPC Invoke Helper
  const invoke = (cmd, args = {}) => {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    console.warn(`[Appify IPC] Tauri runtime not detected for command: ${cmd}`, args);
    return Promise.reject(new Error("Tauri IPC bridge not ready"));
  };

  // State
  let installedApps = [];
  let availablePresets = [];
  let registryExtensions = { plugins: [], themes: [] };
  let marketplaceCatalog = null;
  let activeTab = "installed";
  let activeEditHash = null;
  let activeExtFilter = "all";
  let activeMarketFilter = "all";

  // App Extension Studio State (inside Edit Modal)
  let appActivePlugins = [];
  let appActiveThemes = [];
  let activeStudioTab = "plugin"; // "plugin" | "theme"
  let selectedStudioItemId = null;

  // Monaco & Component Instances
  let studioEditor = null;
  let registryEditor = null;
  let marketPreviewEditor = null;
  let activePreviewItem = null;
  let extCategoryDropdown = null;

  // Monaco Editor Factory
  const initMonaco = (containerId, initialValue = "", lang = "javascript", extraOptions = {}) => {
    const container = document.getElementById(containerId);
    if (!container || !window.monaco || !window.monaco.editor) return null;
    return window.monaco.editor.create(container, {
      value: initialValue,
      language: lang,
      theme: "vs-dark",
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      fontSize: 13,
      lineNumbersMinChars: 3,
      glyphMargin: false,
      folding: true,
      lineDecorationsWidth: 6,
      renderLineHighlight: "all",
      tabSize: 2,
      overviewRulerBorder: false,
      overviewRulerLanes: 0,
      hideCursorInOverviewRuler: true,
      scrollbar: {
        vertical: "visible",
        horizontal: "auto",
        verticalScrollbarSize: 8,
        horizontalScrollbarSize: 8
      },
      ...extraOptions
    });
  };

  const setEditorContentAndLang = (editor, content, lang) => {
    if (!editor) return;
    editor.setValue(content || "");
    if (lang && window.monaco && window.monaco.editor) {
      const model = editor.getModel();
      if (model) {
        window.monaco.editor.setModelLanguage(model, lang);
      }
    }
    setTimeout(() => editor.layout(), 60);
  };

  const insertSnippetToEditor = (editor, snippetText) => {
    if (!editor) return;
    const selection = editor.getSelection();
    const id = { major: 1, minor: 1 };
    const op = { identifier: id, range: selection, text: snippetText, forceMoveMarkers: true };
    editor.executeEdits("snippet", [op]);
    editor.focus();
  };

  // Unified Custom Dropdown Controller
  const setupDropdown = (dropdownId, initialValue = "plugin", onChange = null) => {
    const dropdown = document.getElementById(dropdownId);
    if (!dropdown) return { setValue: () => {} };

    const trigger = dropdown.querySelector(".dropdown-trigger");
    const menu = dropdown.querySelector(".dropdown-menu");
    const hiddenInput = dropdown.querySelector("input[type='hidden']");
    const label = dropdown.querySelector(".dropdown-selected-label");
    const icon = dropdown.querySelector(".dropdown-badge-icon");

    const setValue = (val, triggerChange = true) => {
      val = val === "theme" ? "theme" : "plugin";
      if (hiddenInput) hiddenInput.value = val;
      dropdown.dataset.value = val;
      if (label) label.textContent = val === "theme" ? "Theme (CSS)" : "Plugin (JavaScript)";
      if (icon) {
        icon.textContent = val === "theme" ? "CSS" : "JS";
        icon.className = `dropdown-badge-icon ${val === "theme" ? "badge-css" : "badge-js"}`;
      }
      menu.querySelectorAll(".dropdown-option").forEach(opt => {
        opt.classList.toggle("selected", opt.dataset.value === val);
      });
      if (triggerChange && typeof onChange === "function") {
        onChange(val);
      }
    };

    trigger.addEventListener("click", (e) => {
      e.stopPropagation();
      if (trigger.disabled) return;
      const isOpen = dropdown.classList.contains("open");
      dropdown.classList.toggle("open", !isOpen);
      menu.classList.toggle("hidden", isOpen);
      trigger.setAttribute("aria-expanded", !isOpen);
    });

    menu.addEventListener("click", (e) => {
      const option = e.target.closest(".dropdown-option");
      if (!option) return;
      setValue(option.dataset.value, true);
      dropdown.classList.remove("open");
      menu.classList.add("hidden");
      trigger.setAttribute("aria-expanded", "false");
    });

    document.addEventListener("click", (e) => {
      if (!e.target.closest(`#${dropdownId}`)) {
        dropdown.classList.remove("open");
        menu.classList.add("hidden");
        trigger.setAttribute("aria-expanded", "false");
      }
    });

    return { setValue };
  };

  // Modern Number Steppers Controller
  const setupNumberSteppers = () => {
    const stepValue = (input, direction) => {
      const step = parseFloat(input.step) || 1;
      const min = input.min !== "" ? parseFloat(input.min) : -Infinity;
      const max = input.max !== "" ? parseFloat(input.max) : Infinity;
      let current = parseFloat(input.value);
      if (isNaN(current)) {
        current = input.placeholder ? parseFloat(input.placeholder) || 0 : 0;
      }
      let next = current + direction * step;
      const precision = (step.toString().split(".")[1] || "").length;
      next = parseFloat(next.toFixed(precision));
      if (next < min) next = min;
      if (next > max) next = max;
      input.value = next;
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    };

    let stepperInterval = null;
    let stepperTimeout = null;

    const stopStepper = () => {
      if (stepperTimeout) clearTimeout(stepperTimeout);
      if (stepperInterval) clearInterval(stepperInterval);
      stepperTimeout = null;
      stepperInterval = null;
    };

    document.addEventListener("mousedown", (e) => {
      const btn = e.target.closest(".stepper-btn");
      if (!btn) return;
      e.preventDefault();
      const wrapper = btn.closest(".modern-number-stepper");
      if (!wrapper) return;
      const input = wrapper.querySelector("input[type='number']");
      if (!input || input.disabled) return;

      const direction = btn.classList.contains("stepper-up") ? 1 : -1;
      stepValue(input, direction);

      stepperTimeout = setTimeout(() => {
        stepperInterval = setInterval(() => {
          stepValue(input, direction);
        }, 75);
      }, 300);
    });

    document.addEventListener("mouseup", stopStepper);
    document.addEventListener("mouseleave", stopStepper);
  };

  // DOM Elements
  const tabButtons = document.querySelectorAll(".nav-item");
  const tabViews = document.querySelectorAll(".tab-view");
  const viewTitle = document.getElementById("view-title");
  const viewDesc = document.getElementById("view-desc");
  const installedCount = document.getElementById("installed-count");
  const searchInput = document.getElementById("search-input");
  const installedGrid = document.getElementById("installed-grid");
  const installedEmpty = document.getElementById("installed-empty");
  const presetsGrid = document.getElementById("presets-grid");
  const extensionsGrid = document.getElementById("extensions-grid");
  const marketplaceGrid = document.getElementById("marketplace-grid");
  const marketplaceSearchInput = document.getElementById("marketplace-search-input");
  const marketPreviewModal = document.getElementById("modal-marketplace-preview");
  const installForm = document.getElementById("install-form");
  const toastContainer = document.getElementById("toast-container");
  const editModal = document.getElementById("modal-edit");
  const extModal = document.getElementById("modal-extension");
  const attachPickerModal = document.getElementById("modal-attach-picker");

  // Tab Definitions
  const tabsMeta = {
    installed: {
      title: "Installed Applications",
      desc: "Manage, launch, and configure your isolated desktop web apps."
    },
    presets: {
      title: "Preset Store",
      desc: "Curated web applications with ecosystem isolation, tray controls, and autostart."
    },
    extensions: {
      title: "Plugins & Themes Studio",
      desc: "Extend and customize applications with modular JavaScript plugins and CSS themes."
    },
    marketplace: {
      title: "Community Marketplace",
      desc: "Discover, preview, and install community plugins and themes hosted on cdn.appify.aerovex.net."
    },
    "new-app": {
      title: "Install Custom Application",
      desc: "Turn any website into a standalone desktop application."
    },
    settings: {
      title: "System & Information",
      desc: "Manage Appify binary path, autostart integrations, and shortcuts."
    }
  };

  // Show Toast
  const showToast = (message, type = "info") => {
    const toast = document.createElement("div");
    toast.className = `toast toast-${type}`;
    toast.innerHTML = `
      <div class="toast-content">${escapeHtml(message)}</div>
    `;
    toastContainer.appendChild(toast);
    setTimeout(() => {
      toast.style.opacity = "0";
      toast.style.transform = "translateX(30px)";
      toast.style.transition = "all 0.3s ease";
      setTimeout(() => toast.remove(), 300);
    }, 4000);
  };

  const escapeHtml = (str) => {
    if (!str) return "";
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  };

  // Tab Switcher
  const switchTab = (tabId) => {
    activeTab = tabId;
    tabButtons.forEach(btn => {
      btn.classList.toggle("active", btn.dataset.tab === tabId);
    });
    tabViews.forEach(view => {
      view.classList.toggle("active", view.id === `tab-${tabId}`);
    });

    if (tabsMeta[tabId]) {
      viewTitle.textContent = tabsMeta[tabId].title;
      viewDesc.textContent = tabsMeta[tabId].desc;
    }

    if (tabId === "installed") {
      searchInput.placeholder = "Search installed apps...";
      loadInstalledApps();
    } else if (tabId === "presets") {
      searchInput.placeholder = "Search preset store...";
      loadPresets();
    } else if (tabId === "extensions") {
      searchInput.placeholder = "Search plugins and themes...";
      loadRegistry();
    } else if (tabId === "marketplace") {
      searchInput.placeholder = "Search marketplace...";
      loadMarketplace();
    } else if (tabId === "settings") {
      searchInput.placeholder = "Search...";
      loadSystemInfo();
    } else {
      searchInput.placeholder = "Search...";
    }
  };

  tabButtons.forEach(btn => {
    btn.addEventListener("click", () => switchTab(btn.dataset.tab));
  });

  document.getElementById("btn-quick-new").addEventListener("click", () => switchTab("new-app"));
  document.getElementById("empty-go-presets").addEventListener("click", () => switchTab("presets"));
  document.getElementById("empty-go-new").addEventListener("click", () => switchTab("new-app"));

  // Fetch & Render Installed Apps
  const loadInstalledApps = async () => {
    try {
      const apps = await invoke("get_installed_apps");
      installedApps = Array.isArray(apps) ? apps : [];
      renderInstalledApps();
    } catch (err) {
      console.error("Failed to load apps:", err);
      showToast(`Failed to load installed apps: ${err}`, "error");
    }
  };

  const renderInstalledApps = () => {
    const filter = (searchInput.value || "").toLowerCase().trim();
    const filtered = installedApps.filter(app => {
      if (!filter) return true;
      return (
        app.name.toLowerCase().includes(filter) ||
        app.url.toLowerCase().includes(filter) ||
        app.wm_class.toLowerCase().includes(filter)
      );
    });

    installedCount.textContent = installedApps.length;

    if (filtered.length === 0) {
      installedGrid.innerHTML = "";
      if (installedApps.length === 0) {
        installedEmpty.classList.remove("hidden");
      } else {
        installedEmpty.classList.add("hidden");
        installedGrid.innerHTML = `<div class="empty-filter" style="grid-column: 1 / -1; text-align: center; padding: 40px 20px; color: var(--text-dim);">No apps matching "${escapeHtml(filter)}"</div>`;
      }
      return;
    }

    installedEmpty.classList.add("hidden");
    installedGrid.innerHTML = filtered.map(app => {
      const iconSrc = app.icon_base64 
        ? `data:image/png;base64,${app.icon_base64}` 
        : (app.icon_path || "");

      const badges = [];
      if (app.single_instance) badges.push(`<span class="tag-badge active-blue">Single Instance</span>`);
      if (app.hide_on_close) badges.push(`<span class="tag-badge active-green">Hide on Close</span>`);
      if (app.tray) badges.push(`<span class="tag-badge active-blue">System Tray</span>`);
      if (app.autostart || app.autostart_hidden) {
        badges.push(`<span class="tag-badge active-green">Autostart${app.autostart_hidden ? ' (Tray)' : ''}</span>`);
      }
      if (app.has_custom_scripts) badges.push(`<span class="tag-badge">Script Injected</span>`);

      return `
        <div class="app-card" data-hash="${escapeHtml(app.hash)}">
          <div class="app-card-quick-actions">
            <button class="btn-card-action" data-action="configure" data-hash="${escapeHtml(app.hash)}" title="Options (Settings, .desktop, zoom, autostart, scripts, reinstall)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
            </button>
            <button class="btn-card-action danger" data-action="uninstall" data-hash="${escapeHtml(app.hash)}" data-name="${escapeHtml(app.name)}" title="Delete / Uninstall Application">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6"></polyline>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
              </svg>
            </button>
          </div>

          <div class="app-card-header">
            <img class="app-card-icon" src="${iconSrc}" alt="${escapeHtml(app.name)}" onerror="this.src='default_icon.png'">
            <div class="app-card-title-wrap">
              <div class="app-card-name" title="${escapeHtml(app.name)}">${escapeHtml(app.name)}</div>
              <div class="app-card-url" title="${escapeHtml(app.url)}">${escapeHtml(app.url)}</div>
            </div>
          </div>

          <div class="app-card-tags">
            ${badges.join("")}
          </div>

          <div class="app-card-actions">
            <button class="btn btn-primary btn-launch" data-action="launch" data-hash="${escapeHtml(app.hash)}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="5 3 19 12 5 21 5 3"></polygon>
              </svg>
              <span>Launch</span>
            </button>
          </div>
        </div>
      `;
    }).join("");
  };

  searchInput.addEventListener("input", () => {
    if (activeTab === "presets") {
      renderPresets();
    } else if (activeTab === "installed") {
      renderInstalledApps();
    } else if (activeTab === "extensions") {
      renderExtensions();
    } else if (activeTab === "marketplace") {
      if (marketplaceSearchInput) {
        marketplaceSearchInput.value = searchInput.value;
      }
      renderMarketplace();
    }
  });

  // App Card Action Handlers
  installedGrid.addEventListener("click", async (e) => {
    const btn = e.target.closest("button[data-action]");
    if (!btn) return;
    const action = btn.dataset.action;
    const hash = btn.dataset.hash;

    if (action === "launch") {
      try {
        btn.disabled = true;
        showToast("Launching application...", "info");
        await invoke("launch_app_cmd", { hash });
        setTimeout(() => { btn.disabled = false; }, 1000);
      } catch (err) {
        btn.disabled = false;
        showToast(`Failed to launch: ${err}`, "error");
      }
    } else if (action === "configure") {
      openEditModal(hash);
    } else if (action === "uninstall") {
      const name = btn.dataset.name || "this application";
      if (confirm(`Are you sure you want to uninstall '${name}'? This will remove its desktop launcher and cache.`)) {
        try {
          await invoke("uninstall_app_cmd", { hash });
          showToast(`Successfully uninstalled '${name}'`, "success");
          loadInstalledApps();
        } catch (err) {
          showToast(`Failed to uninstall: ${err}`, "error");
        }
      }
    }
  });

  // Presets Catalog
  const loadPresets = async () => {
    try {
      const presets = await invoke("get_presets");
      availablePresets = Array.isArray(presets) ? presets : [];
      renderPresets();
    } catch (err) {
      console.error("Failed to load presets:", err);
      showToast(`Failed to load presets: ${err}`, "error");
    }
  };

  const renderPresets = () => {
    // Brand SVGs
    const whatsappSvg = `<svg viewBox="0 0 24 24"><path d="M12.04 2C6.58 2 2.13 6.45 2.13 11.91C2.13 13.66 2.59 15.36 3.45 16.86L2.05 22L7.3 20.62C8.75 21.41 10.38 21.83 12.04 21.83C17.5 21.83 21.95 17.38 21.95 11.92C21.95 9.27 20.92 6.78 19.05 4.91C17.18 3.03 14.69 2 12.04 2M12.05 3.67C14.25 3.67 16.31 4.53 17.87 6.09C19.42 7.65 20.28 9.72 20.28 11.92C20.28 16.46 16.58 20.15 12.04 20.15C10.56 20.15 9.11 19.76 7.85 19L7.55 18.83L4.43 19.65L5.26 16.61L5.06 16.29C4.24 15 3.8 13.47 3.8 11.91C3.81 7.37 7.5 3.67 12.05 3.67M9.27 7.33C9.08 7.33 8.77 7.4 8.5 7.7C8.25 8 7.5 8.71 7.5 10.15C7.5 11.6 8.55 13 8.7 13.2C8.85 13.4 10.73 16.3 13.65 17.56C14.35 17.86 14.9 18.04 15.33 18.17C16.03 18.4 16.67 18.36 17.18 18.29C17.75 18.2 18.93 17.57 19.18 16.87C19.42 16.17 19.42 15.57 19.35 15.45C19.28 15.33 19.08 15.26 18.78 15.11C18.47 14.96 17.03 14.25 16.76 14.15C16.5 14.05 16.3 14 16.1 14.3C15.9 14.6 15.35 15.26 15.17 15.45C15 15.65 14.82 15.68 14.52 15.53C14.22 15.38 12.24 14.73 11.2 13.8C10.39 13.08 9.84 12.19 9.69 11.89C9.54 11.59 9.68 11.42 9.83 11.27C9.96 11.14 10.12 10.92 10.27 10.75C10.42 10.57 10.47 10.45 10.57 10.25C10.67 10.05 10.62 9.87 10.54 9.72C10.47 9.57 9.87 8.09 9.62 7.5C9.38 6.92 9.13 7 8.95 7L8.5 7C8.5 7 8.27 7.03 8.04 7.26C7.8 7.5 7.5 7.78 7.5 8.35L9.27 7.33Z"/></svg>`;
    const discordSvg = `<svg viewBox="0 0 24 24"><path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994.021-.041.001-.09-.041-.106a13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.929 1.793 8.18 1.793 12.061 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.894.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.028zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>`;
    const telegramSvg = `<svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 0 0-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.37.74-.56 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.07.01.25 0 .36z"/></svg>`;
    const spotifySvg = `<svg viewBox="0 0 24 24"><path d="M12 2C6.477 2 2 6.477 2 12s4.477 10 10 10 10-4.477 10-10S17.523 2 12 2zm4.586 14.424c-.18.295-.563.387-.857.207-2.35-1.435-5.308-1.76-8.795-.963-.336.077-.67-.133-.747-.469-.077-.336.133-.67.469-.747 3.812-.871 7.085-.503 9.723 1.115.294.18.386.562.207.857zm1.18-2.627c-.226.368-.71.482-1.078.256-2.69-1.653-6.79-2.134-9.97-1.169-.413.125-.853-.11-.978-.523-.125-.413.11-.853.523-.978 3.633-1.102 8.147-.568 11.247 1.336.368.226.482.71.256 1.078zm.106-2.767c-3.226-1.915-8.54-2.092-11.611-1.16-.494.15-1.02-.132-1.17-.626-.15-.494.132-1.02.626-1.17 3.535-1.073 9.406-.867 13.121 1.338.445.264.59.838.326 1.283-.264.444-.838.59-1.282.325z"/></svg>`;

    const filter = (searchInput.value || "").toLowerCase().trim();
    const filtered = availablePresets.filter(preset => {
      if (!filter) return true;
      return (
        preset.name.toLowerCase().includes(filter) ||
        preset.description.toLowerCase().includes(filter) ||
        preset.category.toLowerCase().includes(filter) ||
        preset.url.toLowerCase().includes(filter) ||
        preset.id.toLowerCase().includes(filter)
      );
    });

    if (filtered.length === 0) {
      presetsGrid.innerHTML = `<div class="empty-filter" style="grid-column: 1 / -1; text-align: center; padding: 40px 20px; color: var(--text-dim);">No presets matching "${escapeHtml(filter)}"</div>`;
      return;
    }

    presetsGrid.innerHTML = filtered.map(preset => {
      let svgIcon = whatsappSvg;
      let brandClass = "whatsapp";
      if (preset.id === "discord") {
        svgIcon = discordSvg;
        brandClass = "discord";
      } else if (preset.id === "telegram") {
        svgIcon = telegramSvg;
        brandClass = "telegram";
      } else if (preset.id === "spotify") {
        svgIcon = spotifySvg;
        brandClass = "spotify";
      }
      const isInstalled = preset.is_installed;

      return `
        <div class="preset-card" data-preset-id="${escapeHtml(preset.id)}">
          <div class="app-card-header">
            <div class="preset-brand-icon ${brandClass}">
              ${svgIcon}
            </div>
            <div class="app-card-title-wrap">
              <span class="preset-category">${escapeHtml(preset.category)}</span>
              <div class="app-card-name">${escapeHtml(preset.name)}</div>
              <div class="app-card-url">${escapeHtml(preset.url)}</div>
            </div>
          </div>

          <p class="preset-desc">${escapeHtml(preset.description)}</p>

          <div class="preset-features">
            <span class="tag-badge active-blue">Domain Isolation</span>
            <span class="tag-badge active-green">Single Instance</span>
            <span class="tag-badge active-blue">System Tray</span>
            <span class="tag-badge">Background Close</span>
          </div>

          <div class="preset-card-footer">
            ${isInstalled 
              ? `<span class="tag-badge active-green">Installed</span>
                 <div style="display:flex;gap:8px;">
                   <button class="btn btn-secondary btn-preset-action" data-action="configure" data-hash="${escapeHtml(preset.installed_hash || '')}">Options</button>
                   <button class="btn btn-primary btn-preset-action" data-action="launch" data-hash="${escapeHtml(preset.installed_hash || '')}">Launch</button>
                 </div>`
              : `<button class="btn btn-primary btn-preset-action" data-action="install" data-preset="${escapeHtml(preset.id)}">
                   <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                     <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                     <polyline points="7 10 12 15 17 10"></polyline>
                     <line x1="12" y1="15" x2="12" y2="3"></line>
                   </svg>
                   <span>1-Click Install</span>
                 </button>`
            }
          </div>
        </div>
      `;
    }).join("");
  };

  presetsGrid.addEventListener("click", async (e) => {
    const btn = e.target.closest("button.btn-preset-action");
    if (!btn) return;
    const action = btn.dataset.action;

    if (action === "install") {
      const presetId = btn.dataset.preset;
      try {
        btn.disabled = true;
        btn.innerHTML = `<span class="btn-spinner"></span> <span>Installing...</span>`;
        showToast(`Installing preset '${presetId}'...`, "info");
        await invoke("install_preset_cmd", { id: presetId, options: null });
        showToast(`Successfully installed '${presetId}'!`, "success");
        await loadPresets();
        await loadInstalledApps();
      } catch (err) {
        btn.disabled = false;
        btn.innerHTML = `<span>1-Click Install</span>`;
        showToast(`Installation failed: ${err}`, "error");
      }
    } else if (action === "launch") {
      const hash = btn.dataset.hash;
      try {
        await invoke("launch_app_cmd", { hash });
        showToast("Launching app...", "info");
      } catch (err) {
        showToast(`Failed to launch: ${err}`, "error");
      }
    } else if (action === "configure") {
      const hash = btn.dataset.hash;
      openEditModal(hash);
    }
  });

  // ==========================================
  // Global Registry: Plugins & Themes
  // ==========================================
  const loadRegistry = async () => {
    try {
      const reg = await invoke("get_registry_cmd");
      registryExtensions = reg || { plugins: [], themes: [] };
      renderExtensions();
      if (activeTab === "marketplace") {
        renderMarketplace();
      }
    } catch (err) {
      console.error("Failed to load registry:", err);
      showToast(`Failed to load registry: ${err}`, "error");
    }
  };

  const renderExtensions = () => {
    const plugins = registryExtensions.plugins || [];
    const themes = registryExtensions.themes || [];
    const totalCount = plugins.length + themes.length;

    const btnAll = document.getElementById("ext-filter-all");
    const btnPlug = document.getElementById("ext-filter-plugins");
    const btnThm = document.getElementById("ext-filter-themes");
    if (btnAll) btnAll.textContent = `All Extensions (${totalCount})`;
    if (btnPlug) btnPlug.textContent = `Plugins (JS) (${plugins.length})`;
    if (btnThm) btnThm.textContent = `Themes (CSS) (${themes.length})`;

    let combined = [];
    if (activeExtFilter === "all" || activeExtFilter === "plugin") {
      combined = combined.concat(plugins);
    }
    if (activeExtFilter === "all" || activeExtFilter === "theme") {
      combined = combined.concat(themes);
    }

    const filter = (searchInput.value || "").toLowerCase().trim();
    const filtered = combined.filter(item => {
      if (!filter) return true;
      return (
        item.name.toLowerCase().includes(filter) ||
        (item.description && item.description.toLowerCase().includes(filter)) ||
        (item.author && item.author.toLowerCase().includes(filter)) ||
        item.id.toLowerCase().includes(filter)
      );
    });

    if (filtered.length === 0) {
      extensionsGrid.innerHTML = `<div class="empty-filter" style="grid-column: 1 / -1; text-align: center; padding: 40px 20px; color: var(--text-dim);">No extensions matching "${escapeHtml(filter)}"</div>`;
      return;
    }

    extensionsGrid.innerHTML = filtered.map(item => {
      const isPlugin = item.category === "plugin";
      const isBuiltin = !!item.is_builtin;

      return `
        <div class="extension-card" data-id="${escapeHtml(item.id)}" data-type="${escapeHtml(item.category)}">
          <div class="extension-card-header">
            <div class="ext-header-info">
              <span class="extension-name">${escapeHtml(item.name)}</span>
              <span class="extension-author">by ${escapeHtml(item.author || "Appify")} &bull; v${escapeHtml(item.version || "1.0.0")}</span>
            </div>
            <div style="display:flex; gap:6px;">
              <span class="tag-badge ${isPlugin ? 'active-blue' : 'active-green'}">${isPlugin ? 'Plugin (JS)' : 'Theme (CSS)'}</span>
              ${isBuiltin ? `<span class="tag-badge badge-builtin">Built-in</span>` : `<span class="tag-badge">Custom</span>`}
            </div>
          </div>

          <p class="extension-desc">${escapeHtml(item.description || "No description provided.")}</p>

          <div class="extension-footer">
            <button class="btn btn-secondary btn-sm btn-ext-action" data-action="view" data-id="${escapeHtml(item.id)}">
              ${isBuiltin ? 'View Code' : 'Edit Extension'}
            </button>
            ${isBuiltin 
              ? `<span class="tag-badge" title="Built-in extensions are protected from deletion">Protected</span>` 
              : `<button class="btn btn-secondary btn-sm text-danger btn-ext-action" data-action="delete" data-id="${escapeHtml(item.id)}" data-name="${escapeHtml(item.name)}">Delete</button>`
            }
          </div>
        </div>
      `;
    }).join("");
  };

  // Filter tabs in Registry View
  document.querySelectorAll("#tab-extensions .ext-tab-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      document.querySelectorAll("#tab-extensions .ext-tab-btn").forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      activeExtFilter = btn.dataset.exttype;
      renderExtensions();
    });
  });

  // Card click actions in Registry View
  extensionsGrid.addEventListener("click", async (e) => {
    const btn = e.target.closest("button.btn-ext-action");
    if (!btn) return;
    const action = btn.dataset.action;
    const id = btn.dataset.id;

    const all = [...(registryExtensions.plugins || []), ...(registryExtensions.themes || [])];
    const item = all.find(x => x.id === id);
    if (!item) return;

    if (action === "view") {
      openExtensionModal(item);
    } else if (action === "delete") {
      if (item.is_builtin) {
        showToast("Built-in extensions cannot be deleted.", "error");
        return;
      }
      if (confirm(`Are you sure you want to delete '${item.name}' from the registry?`)) {
        try {
          await invoke("delete_registry_item_cmd", { id });
          showToast(`Deleted extension '${item.name}'`, "success");
          await loadRegistry();
        } catch (err) {
          showToast(`Failed to delete: ${err}`, "error");
        }
      }
    }
  });

  // New Extension Button in Registry View
  document.getElementById("btn-create-extension").addEventListener("click", () => {
    openExtensionModal(null);
  });

  // Extension Modal: Open / Close / Save
  // Extension Modal: Open / Close / Save
  const openExtensionModal = (item) => {
    const isEdit = !!item;
    document.getElementById("modal-ext-title").textContent = isEdit ? (item.is_builtin ? "View Extension" : "Edit Extension") : "New Extension";
    document.getElementById("modal-ext-subtitle").textContent = isEdit 
      ? (item.is_builtin ? "Protected built-in system extension" : "Modify custom registry extension")
      : "Create a modular plugin or theme for the registry";

    document.getElementById("ext-form-id").value = item ? item.id : `user-${Date.now()}`;
    document.getElementById("ext-form-is-builtin").value = item ? (item.is_builtin ? "true" : "false") : "false";
    document.getElementById("ext-form-name").value = item ? item.name : "";
    document.getElementById("ext-form-desc").value = item ? (item.description || "") : "";
    document.getElementById("ext-form-author").value = item ? (item.author || "") : "User";
    document.getElementById("ext-form-version").value = item ? (item.version || "") : "1.0.0";
    
    // Category Dropdown
    if (!extCategoryDropdown) {
      extCategoryDropdown = setupDropdown("dropdown-ext-category", "plugin", (newCat) => {
        if (registryEditor && window.monaco && window.monaco.editor) {
          const model = registryEditor.getModel();
          if (model) {
            window.monaco.editor.setModelLanguage(model, newCat === "theme" ? "css" : "javascript");
          }
        }
      });
    }

    const targetCat = item ? item.category : (activeExtFilter === "theme" ? "theme" : "plugin");
    extCategoryDropdown.setValue(targetCat, false);

    const dropdownTrigger = document.getElementById("dropdown-trigger-ext-category");
    if (dropdownTrigger) {
      dropdownTrigger.disabled = isEdit;
      dropdownTrigger.style.opacity = isEdit ? "0.6" : "1";
      dropdownTrigger.style.cursor = isEdit ? "not-allowed" : "pointer";
    }

    // Monaco Editor
    const code = item ? item.content : "";
    const lang = targetCat === "theme" ? "css" : "javascript";
    if (!registryEditor) {
      registryEditor = initMonaco("monaco-ext-editor", code, lang);
    } else {
      setEditorContentAndLang(registryEditor, code, lang);
    }

    const btnSave = document.getElementById("btn-ext-save");
    const isBuiltin = item && item.is_builtin;
    btnSave.style.display = isBuiltin ? "none" : "inline-flex";

    const deleteBtn = document.getElementById("btn-ext-delete");
    if (isEdit && !item.is_builtin) {
      deleteBtn.classList.remove("hidden");
      deleteBtn.onclick = async () => {
        if (confirm(`Delete '${item.name}' from the registry?`)) {
          try {
            await invoke("delete_registry_item_cmd", { id: item.id });
            showToast("Extension deleted.", "success");
            extModal.classList.add("hidden");
            await loadRegistry();
          } catch (err) {
            showToast(`Error: ${err}`, "error");
          }
        }
      };
    } else {
      deleteBtn.classList.add("hidden");
    }

    extModal.classList.remove("hidden");
    setTimeout(() => {
      if (registryEditor) registryEditor.layout();
    }, 60);
  };

  document.getElementById("btn-modal-ext-close").addEventListener("click", () => extModal.classList.add("hidden"));
  document.getElementById("btn-ext-cancel").addEventListener("click", () => extModal.classList.add("hidden"));

  // Extension Modal Save
  document.getElementById("btn-ext-save").addEventListener("click", async () => {
    const id = document.getElementById("ext-form-id").value;
    const name = document.getElementById("ext-form-name").value.trim();
    if (!name) {
      showToast("Extension name is required", "error");
      return;
    }
    const category = document.getElementById("ext-form-category").value;
    const description = document.getElementById("ext-form-desc").value.trim();
    const author = document.getElementById("ext-form-author").value.trim() || "User";
    const version = document.getElementById("ext-form-version").value.trim() || "1.0.0";
    const content = registryEditor ? registryEditor.getValue() : (document.getElementById("ext-form-content").value || "");

    const item = {
      id,
      name,
      description,
      category,
      content,
      author,
      version,
      is_builtin: false
    };

    try {
      await invoke("save_registry_item_cmd", { item });
      showToast(`Saved '${name}' to registry!`, "success");
      extModal.classList.add("hidden");
      await loadRegistry();
    } catch (err) {
      showToast(`Failed to save: ${err}`, "error");
    }
  });

  // Snippets in Extension Modal
  document.querySelectorAll("#ext-snippets-bar .btn-snippet").forEach(btn => {
    btn.addEventListener("click", () => {
      const snippetType = btn.dataset.snippet;
      if (!registryEditor) return;
      let snippetText = "";
      if (snippetType === "log") {
        snippetText = `\nconsole.log("[Extension] Active:", window.location.href);\n`;
      } else if (snippetType === "dark") {
        snippetText = `\nwindow.matchMedia = window.matchMedia || function() { return { matches: true, addListener: function() {}, removeListener: function() {} }; };\n`;
      } else if (snippetType === "hide-scroll") {
        snippetText = `\n::-webkit-scrollbar { display: none !important; width: 0 !important; height: 0 !important; }\n`;
      }
      insertSnippetToEditor(registryEditor, snippetText);
    });
  });

  // ==========================================
  // Community Marketplace View
  // ==========================================
  const isItemInstalled = (id) => {
    const allReg = [...(registryExtensions.plugins || []), ...(registryExtensions.themes || [])];
    return allReg.some(reg => reg.id === id);
  };

  const loadMarketplace = async () => {
    try {
      const catalog = await invoke("get_marketplace_catalog_cmd");
      marketplaceCatalog = catalog || { version: 1, updated_at: "", items: [] };
      renderMarketplace();
    } catch (err) {
      console.error("Failed to load marketplace catalog:", err);
      showToast(`Failed to load marketplace: ${err}`, "error");
    }
  };

  const renderMarketplace = () => {
    if (!marketplaceGrid) return;
    const items = marketplaceCatalog?.items || [];
    const plugins = items.filter(i => i.category === "plugin");
    const themes = items.filter(i => i.category === "theme");

    const btnAll = document.getElementById("market-filter-all");
    const btnPlug = document.getElementById("market-filter-plugins");
    const btnThm = document.getElementById("market-filter-themes");
    if (btnAll) btnAll.textContent = `All (${items.length})`;
    if (btnPlug) btnPlug.textContent = `Plugins (JS) (${plugins.length})`;
    if (btnThm) btnThm.textContent = `Themes (CSS) (${themes.length})`;

    let combined = [];
    if (activeMarketFilter === "all" || activeMarketFilter === "plugin") {
      combined = combined.concat(plugins);
    }
    if (activeMarketFilter === "all" || activeMarketFilter === "theme") {
      combined = combined.concat(themes);
    }

    const filter = ((marketplaceSearchInput && marketplaceSearchInput.value) || searchInput.value || "").toLowerCase().trim();
    const filtered = combined.filter(item => {
      if (!filter) return true;
      const tagMatch = Array.isArray(item.tags) && item.tags.some(t => t.toLowerCase().includes(filter));
      return (
        item.name.toLowerCase().includes(filter) ||
        (item.description && item.description.toLowerCase().includes(filter)) ||
        (item.author && item.author.toLowerCase().includes(filter)) ||
        item.id.toLowerCase().includes(filter) ||
        tagMatch
      );
    });

    if (filtered.length === 0) {
      marketplaceGrid.innerHTML = `<div class="empty-filter" style="grid-column: 1 / -1; text-align: center; padding: 40px 20px; color: var(--text-dim);">No extensions matching "${escapeHtml(filter)}"</div>`;
      return;
    }

    marketplaceGrid.innerHTML = filtered.map(item => {
      const isInstalled = isItemInstalled(item.id);
      const isPlugin = item.category === "plugin";
      const tagsHtml = (item.tags || []).map(t => `<span class="market-card-tag">${escapeHtml(t)}</span>`).join("");

      return `
        <div class="marketplace-card" data-id="${escapeHtml(item.id)}" data-type="${escapeHtml(item.category)}">
          <div class="market-card-top">
            <div class="market-card-title-group">
              <span class="market-card-title">${escapeHtml(item.name)}</span>
              <span class="market-card-author">by ${escapeHtml(item.author || "Community")} &bull; v${escapeHtml(item.version || "1.0.0")}</span>
            </div>
            <span class="tag-badge ${isPlugin ? 'active-blue' : 'active-green'}">${isPlugin ? 'Plugin (JS)' : 'Theme (CSS)'}</span>
          </div>

          <p class="market-card-desc">${escapeHtml(item.description || "No description provided.")}</p>

          ${tagsHtml ? `<div class="market-card-tags">${tagsHtml}</div>` : ''}

          <div class="market-card-footer">
            <span style="font-size: 0.75rem; color: var(--text-dim); font-family: monospace;">${escapeHtml(item.id)}</span>
            <div class="market-card-footer-actions">
              <button type="button" class="btn btn-secondary btn-sm btn-market-preview" data-id="${escapeHtml(item.id)}">Preview</button>
              ${isInstalled 
                ? `<button type="button" class="btn btn-secondary btn-sm" disabled style="opacity: 0.7; cursor: default;">Installed</button>`
                : `<button type="button" class="btn btn-primary btn-sm btn-market-install" data-id="${escapeHtml(item.id)}">Install</button>`
              }
            </div>
          </div>
        </div>
      `;
    }).join("");
  };

  const installMarketplaceExtension = async (item, btnElement) => {
    if (!item) return;
    try {
      if (btnElement) {
        btnElement.disabled = true;
        btnElement.textContent = "Installing...";
      }
      await invoke("install_marketplace_item_cmd", { id: item.id });
      await loadRegistry();
      showToast(`Installed "${item.name}" to registry!`, "success");
      renderMarketplace();
      if (activePreviewItem && activePreviewItem.id === item.id) {
        updateMarketPreviewInstallButton(true);
      }
    } catch (err) {
      console.error("Failed to install marketplace extension:", err);
      showToast(`Installation failed: ${err}`, "error");
      if (btnElement) {
        btnElement.disabled = false;
        btnElement.textContent = "Install";
      }
    }
  };

  const openMarketplacePreview = (item) => {
    activePreviewItem = item;
    const titleEl = document.getElementById("market-preview-title");
    const metaEl = document.getElementById("market-preview-meta");
    const descEl = document.getElementById("market-preview-desc");
    const badgeEl = document.getElementById("market-preview-badge");

    if (titleEl) titleEl.textContent = item.name;
    if (metaEl) metaEl.textContent = `by ${item.author || "Community"} • v${item.version || "1.0.0"}`;
    if (descEl) descEl.textContent = item.description || "No description provided.";
    if (badgeEl) {
      badgeEl.textContent = item.category === "theme" ? "Theme (CSS)" : "Plugin (JS)";
      badgeEl.className = `tag-badge ${item.category === 'theme' ? 'active-green' : 'active-blue'}`;
    }

    const installed = isItemInstalled(item.id);
    updateMarketPreviewInstallButton(installed);

    const lang = item.category === "theme" ? "css" : "javascript";
    const code = item.code || "// No preview code available";

    if (!marketPreviewEditor) {
      marketPreviewEditor = initMonaco("monaco-market-preview", code, lang, { readOnly: true });
    } else {
      setEditorContentAndLang(marketPreviewEditor, code, lang);
      marketPreviewEditor.updateOptions({ readOnly: true });
    }

    if (marketPreviewModal) marketPreviewModal.classList.remove("hidden");
    setTimeout(() => {
      if (marketPreviewEditor) marketPreviewEditor.layout();
    }, 60);
  };

  const updateMarketPreviewInstallButton = (installed) => {
    const installBtn = document.getElementById("btn-market-preview-install");
    if (!installBtn) return;
    if (installed) {
      installBtn.textContent = "Installed";
      installBtn.disabled = true;
      installBtn.classList.remove("btn-primary");
      installBtn.classList.add("btn-secondary");
    } else {
      installBtn.textContent = "Install to Registry";
      installBtn.disabled = false;
      installBtn.classList.add("btn-primary");
      installBtn.classList.remove("btn-secondary");
    }
  };

  const closeMarketPreviewModal = () => {
    if (marketPreviewModal) marketPreviewModal.classList.add("hidden");
    activePreviewItem = null;
  };

  // Filter tabs in Marketplace View
  document.querySelectorAll("#tab-marketplace .ext-tab-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      document.querySelectorAll("#tab-marketplace .ext-tab-btn").forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      activeMarketFilter = btn.dataset.marketfilter || "all";
      renderMarketplace();
    });
  });

  if (marketplaceSearchInput) {
    marketplaceSearchInput.addEventListener("input", () => {
      if (searchInput) searchInput.value = marketplaceSearchInput.value;
      renderMarketplace();
    });
  }

  if (marketplaceGrid) {
    marketplaceGrid.addEventListener("click", async (e) => {
      const previewBtn = e.target.closest(".btn-market-preview");
      if (previewBtn) {
        const id = previewBtn.dataset.id;
        const item = (marketplaceCatalog?.items || []).find(i => i.id === id);
        if (item) openMarketplacePreview(item);
        return;
      }

      const installBtn = e.target.closest(".btn-market-install");
      if (installBtn) {
        const id = installBtn.dataset.id;
        const item = (marketplaceCatalog?.items || []).find(i => i.id === id);
        if (item) {
          await installMarketplaceExtension(item, installBtn);
        }
        return;
      }
    });
  }

  const btnMarketPreviewClose = document.getElementById("btn-market-preview-close");
  if (btnMarketPreviewClose) btnMarketPreviewClose.addEventListener("click", closeMarketPreviewModal);

  const btnMarketPreviewCancel = document.getElementById("btn-market-preview-cancel");
  if (btnMarketPreviewCancel) btnMarketPreviewCancel.addEventListener("click", closeMarketPreviewModal);

  if (marketPreviewModal) {
    marketPreviewModal.addEventListener("click", (e) => {
      if (e.target === marketPreviewModal) closeMarketPreviewModal();
    });
  }

  const btnMarketPreviewInstall = document.getElementById("btn-market-preview-install");
  if (btnMarketPreviewInstall) {
    btnMarketPreviewInstall.addEventListener("click", async () => {
      if (!activePreviewItem) return;
      await installMarketplaceExtension(activePreviewItem, btnMarketPreviewInstall);
    });
  }

  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && marketPreviewModal && !marketPreviewModal.classList.contains("hidden")) {
      closeMarketPreviewModal();
    }
  });

  // Auto-Detect Target in New App Form
  const formUrlInput = document.getElementById("form-url");
  const formNameInput = document.getElementById("form-name");
  const formWmClassInput = document.getElementById("form-wmclass");

  const autoDetectUrl = () => {
    const raw = formUrlInput.value.trim();
    if (!raw) return;

    if (raw.toLowerCase() === "whatsapp") {
      formUrlInput.value = "https://web.whatsapp.com";
      formNameInput.value = "WhatsApp";
      formWmClassInput.value = "whatsapp-desktop";
      return;
    }
    if (raw.toLowerCase() === "discord") {
      formUrlInput.value = "https://discord.com/app";
      formNameInput.value = "Discord";
      formWmClassInput.value = "discord-app";
      return;
    }
    if (raw.toLowerCase() === "telegram") {
      formUrlInput.value = "https://web.telegram.org";
      formNameInput.value = "Telegram";
      formWmClassInput.value = "telegram-web";
      return;
    }
    if (raw.toLowerCase() === "spotify") {
      formUrlInput.value = "https://open.spotify.com";
      formNameInput.value = "Spotify";
      formWmClassInput.value = "spotify-web";
      return;
    }

    try {
      let urlStr = raw;
      if (!urlStr.startsWith("http://") && !urlStr.startsWith("https://")) {
        urlStr = "https://" + urlStr;
      }
      const u = new URL(urlStr);
      let host = u.hostname.replace(/^www\./, "");
      let parts = host.split(".");
      let candidate = parts.length > 1 ? parts[0] : host;
      let capitalized = candidate.charAt(0).toUpperCase() + candidate.slice(1);

      if (!formNameInput.value) {
        formNameInput.value = capitalized;
      }
      if (!formWmClassInput.value) {
        formWmClassInput.value = `${candidate}-desktop`;
      }
    } catch (_) {}
  };

  document.getElementById("btn-detect-url").addEventListener("click", autoDetectUrl);
  formUrlInput.addEventListener("blur", autoDetectUrl);

  // New App Form Submit
  installForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    const btnSubmit = document.getElementById("btn-submit-install");

    const url = formUrlInput.value.trim();
    if (!url) {
      showToast("Please enter a target URL or alias", "error");
      return;
    }

    const domains = document.getElementById("form-domains").value
      .split(",")
      .map(s => s.trim())
      .filter(s => s.length > 0);

    const req = {
      url,
      name: document.getElementById("form-name").value.trim() || null,
      wm_class: document.getElementById("form-wmclass").value.trim() || null,
      custom_icon: document.getElementById("form-icon").value.trim() || null,
      custom_allowed_domains: domains,
      hide_on_close: document.getElementById("form-hide-close").checked,
      single_instance: document.getElementById("form-single-instance").checked,
      tray: document.getElementById("form-tray").checked,
      start_hidden: document.getElementById("form-start-hidden").checked,
      maximize: false,
      zoom: parseFloat(document.getElementById("form-zoom").value) || null,
      user_agent: document.getElementById("form-ua").value.trim() || null,
      width: parseFloat(document.getElementById("form-width").value) || null,
      height: parseFloat(document.getElementById("form-height").value) || null,
      autostart: document.getElementById("form-autostart").checked,
      autostart_hidden: document.getElementById("form-autostart-hidden").checked,
      inject_js: document.getElementById("form-custom-js").value || null,
      inject_css: document.getElementById("form-custom-css").value || null,
    };

    try {
      btnSubmit.disabled = true;
      btnSubmit.innerHTML = `<span>Installing & Fetching Icon...</span>`;
      showToast("Installing desktop application...", "info");

      await invoke("install_custom_cmd", { req });

      showToast(`Successfully installed '${req.name || url}'!`, "success");
      installForm.reset();
      switchTab("installed");
    } catch (err) {
      showToast(`Installation failed: ${err}`, "error");
    } finally {
      btnSubmit.disabled = false;
      btnSubmit.innerHTML = `
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
        <span>Install Desktop Application</span>
      `;
    }
  });

  // Modal Configuration & Scripting
  // Modal Configuration & Extension Studio
  const openEditModal = async (hash) => {
    activeEditHash = hash;
    const app = installedApps.find(a => a.hash === hash);
    if (!app) return;

    document.getElementById("modal-app-hash").value = hash;
    document.getElementById("modal-app-name").textContent = app.name;
    document.getElementById("modal-app-url").textContent = app.url;
    document.getElementById("modal-app-icon").src = app.icon_base64 
      ? `data:image/png;base64,${app.icon_base64}` 
      : (app.icon_path || "");

    // Populate Settings
    document.getElementById("edit-name").value = app.name || "";
    document.getElementById("edit-wmclass").value = app.wm_class || "";
    document.getElementById("edit-hide-close").checked = !!app.hide_on_close;
    document.getElementById("edit-single-instance").checked = !!app.single_instance;
    document.getElementById("edit-tray").checked = !!app.tray;
    document.getElementById("edit-start-hidden").checked = !!app.start_hidden;
    document.getElementById("edit-autostart").checked = !!app.autostart;
    document.getElementById("edit-autostart-hidden").checked = !!app.autostart_hidden;
    document.getElementById("edit-zoom").value = app.zoom || "";
    document.getElementById("edit-width").value = app.width || "";
    document.getElementById("edit-height").value = app.height || "";
    document.getElementById("edit-domains").value = (app.custom_allowed_domains || []).join(", ");

    // Ensure Registry is loaded
    if ((!registryExtensions.plugins || registryExtensions.plugins.length === 0) &&
        (!registryExtensions.themes || registryExtensions.themes.length === 0)) {
      await loadRegistry();
    }

    // Load App Extensions & Scripts
    try {
      const scripts = await invoke("get_app_scripts_cmd", { hash });
      appActivePlugins = (scripts.plugins || []).map(p => ({ ...p }));
      appActiveThemes = (scripts.themes || []).map(t => ({ ...t }));
    } catch (_) {
      appActivePlugins = [];
      appActiveThemes = [];
    }

    activeStudioTab = "plugin";
    selectedStudioItemId = appActivePlugins.length > 0 ? appActivePlugins[0].id : null;
    renderStudio();

    editModal.classList.remove("hidden");
  };

  const closeEditModal = () => {
    editModal.classList.add("hidden");
    activeEditHash = null;
  };

  document.getElementById("btn-modal-close").addEventListener("click", closeEditModal);
  document.getElementById("btn-modal-cancel").addEventListener("click", closeEditModal);

  // Modal Tabs (Settings vs Plugins & Themes Studio)
  const modalTabButtons = document.querySelectorAll(".modal-tab-btn");
  const modalTabViews = document.querySelectorAll(".modal-tab-view");

  modalTabButtons.forEach(btn => {
    btn.addEventListener("click", () => {
      const target = btn.dataset.modaltab;
      modalTabButtons.forEach(b => b.classList.toggle("active", b === btn));
      modalTabViews.forEach(v => v.classList.toggle("active", v.id === `modal-tab-${target}`));
      if (target === "scripts" && studioEditor) {
        setTimeout(() => studioEditor.layout(), 60);
      }
    });
  });

  // Extension Studio Logic
  const getResolvedItemContent = (item) => {
    if (item.custom_content !== null && item.custom_content !== undefined) {
      return item.custom_content;
    }
    const allReg = [...(registryExtensions.plugins || []), ...(registryExtensions.themes || [])];
    const reg = allReg.find(r => r.id === item.id);
    return reg ? reg.content : "";
  };

  const renderStudio = () => {
    const tabBtnPlugins = document.getElementById("tab-btn-app-plugins");
    const tabBtnThemes = document.getElementById("tab-btn-app-themes");
    tabBtnPlugins.textContent = `Plugins (JS) (${appActivePlugins.length})`;
    tabBtnThemes.textContent = `Themes (CSS) (${appActiveThemes.length})`;
    tabBtnPlugins.classList.toggle("active", activeStudioTab === "plugin");
    tabBtnThemes.classList.toggle("active", activeStudioTab === "theme");

    const listTitle = document.getElementById("studio-list-title");
    listTitle.textContent = activeStudioTab === "plugin" ? "Active Plugins" : "Active Themes";

    const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
    
    if (!currentItems.some(x => x.id === selectedStudioItemId)) {
      selectedStudioItemId = currentItems.length > 0 ? currentItems[0].id : null;
    }

    const listEl = document.getElementById("studio-items-list");
    if (currentItems.length === 0) {
      listEl.innerHTML = `
        <div class="studio-empty-state">
          <p>No ${activeStudioTab === "plugin" ? "plugins" : "themes"} attached yet.</p>
          <span class="studio-empty-hint">Click <b>Attach from Registry</b> or <b>Add Custom</b> above to customize.</span>
        </div>
      `;
    } else {
      listEl.innerHTML = currentItems.map((item, idx) => {
        const isBuiltin = item.is_builtin || item.id.startsWith("builtin-");
        const isCustom = item.id.startsWith("custom-");
        const isSelected = item.id === selectedStudioItemId;
        return `
          <div class="studio-item-card ${isSelected ? "active" : ""} ${item.enabled ? "" : "disabled"}" data-id="${item.id}" data-idx="${idx}" draggable="true">
            <div class="drag-handle" title="Drag to reorder execution sequence">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="9" cy="5" r="1.5"></circle>
                <circle cx="9" cy="12" r="1.5"></circle>
                <circle cx="9" cy="19" r="1.5"></circle>
                <circle cx="15" cy="5" r="1.5"></circle>
                <circle cx="15" cy="12" r="1.5"></circle>
                <circle cx="15" cy="19" r="1.5"></circle>
              </svg>
            </div>
            <label class="studio-card-toggle" title="${item.enabled ? "Enabled" : "Disabled"}" onclick="event.stopPropagation();">
              <input type="checkbox" class="studio-item-enable" data-id="${item.id}" ${item.enabled ? "checked" : ""}>
              <span class="studio-slider"></span>
            </label>
            <div class="studio-card-info">
              <div class="studio-card-name">${escapeHtml(item.name)}</div>
              <div class="studio-card-meta">
                <span class="order-idx">#${idx + 1}</span>
                ${isBuiltin ? '<span class="mini-tag builtin">Built-in</span>' : (isCustom ? '<span class="mini-tag custom">Custom</span>' : '<span class="mini-tag registry">Registry</span>')}
              </div>
            </div>
            <button type="button" class="btn-detach" data-id="${item.id}" title="Detach from this app">✕</button>
          </div>
        `;
      }).join("");
    }

    // Editor Pane
    const emptyPane = document.getElementById("studio-editor-empty");
    const activePane = document.getElementById("studio-editor-active");
    if (!selectedStudioItemId) {
      emptyPane.classList.remove("hidden");
      activePane.classList.add("hidden");
    } else {
      emptyPane.classList.add("hidden");
      activePane.classList.remove("hidden");
      const selectedItem = currentItems.find(x => x.id === selectedStudioItemId);
      if (selectedItem) {
        const selectedIdx = currentItems.indexOf(selectedItem);
        document.getElementById("studio-item-name").value = selectedItem.name;
        
        const orderBadge = document.getElementById("studio-item-order-badge");
        orderBadge.textContent = `Executes #${selectedIdx + 1} in order`;

        const sourceBadge = document.getElementById("studio-item-source-badge");
        const isBuiltin = selectedItem.is_builtin || selectedItem.id.startsWith("builtin-");
        const isCustom = selectedItem.id.startsWith("custom-");
        sourceBadge.textContent = isBuiltin ? "Built-in System" : (isCustom ? "Custom Script" : "Registry Item");
        sourceBadge.className = `tag-badge ${isBuiltin ? "badge-purple" : "badge-blue"}`;

        const content = getResolvedItemContent(selectedItem);
        const lang = activeStudioTab === "plugin" ? "javascript" : "css";
        if (!studioEditor) {
          studioEditor = initMonaco("monaco-studio-editor", content, lang);
          if (studioEditor) {
            studioEditor.onDidChangeModelContent(() => {
              const current = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
              const curItem = current.find(x => x.id === selectedStudioItemId);
              if (curItem) {
                curItem.custom_content = studioEditor.getValue();
              }
            });
          }
        } else {
          setEditorContentAndLang(studioEditor, content, lang);
        }
      }
    }
  };

  // Switch Studio Subtabs (Plugins vs Themes)
  document.getElementById("tab-btn-app-plugins").addEventListener("click", () => {
    if (selectedStudioItemId && studioEditor) {
      const current = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
      const curItem = current.find(x => x.id === selectedStudioItemId);
      if (curItem) curItem.custom_content = studioEditor.getValue();
    }
    activeStudioTab = "plugin";
    selectedStudioItemId = appActivePlugins.length > 0 ? appActivePlugins[0].id : null;
    renderStudio();
  });

  document.getElementById("tab-btn-app-themes").addEventListener("click", () => {
    if (selectedStudioItemId && studioEditor) {
      const current = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
      const curItem = current.find(x => x.id === selectedStudioItemId);
      if (curItem) curItem.custom_content = studioEditor.getValue();
    }
    activeStudioTab = "theme";
    selectedStudioItemId = appActiveThemes.length > 0 ? appActiveThemes[0].id : null;
    renderStudio();
  });

  // Studio Drag and Drop Reordering
  const studioListEl = document.getElementById("studio-items-list");
  let dragSrcIdx = null;

  studioListEl.addEventListener("dragstart", (e) => {
    const card = e.target.closest(".studio-item-card");
    if (!card) return;
    dragSrcIdx = parseInt(card.dataset.idx, 10);
    card.classList.add("dragging");
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", dragSrcIdx);
  });

  studioListEl.addEventListener("dragover", (e) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    const targetCard = e.target.closest(".studio-item-card");
    if (!targetCard) return;

    const rect = targetCard.getBoundingClientRect();
    const mid = rect.top + rect.height / 2;
    targetCard.classList.remove("drag-over-top", "drag-over-bottom");
    if (e.clientY < mid) {
      targetCard.classList.add("drag-over-top");
    } else {
      targetCard.classList.add("drag-over-bottom");
    }
  });

  studioListEl.addEventListener("dragleave", (e) => {
    const targetCard = e.target.closest(".studio-item-card");
    if (targetCard) {
      targetCard.classList.remove("drag-over-top", "drag-over-bottom");
    }
  });

  studioListEl.addEventListener("drop", (e) => {
    e.preventDefault();
    const targetCard = e.target.closest(".studio-item-card");
    if (!targetCard || dragSrcIdx === null) return;

    const targetIdx = parseInt(targetCard.dataset.idx, 10);
    const isTop = targetCard.classList.contains("drag-over-top");
    targetCard.classList.remove("drag-over-top", "drag-over-bottom");

    const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
    if (dragSrcIdx === targetIdx) return;

    // Save active editor value first
    if (selectedStudioItemId && studioEditor) {
      const activeItem = currentItems.find(x => x.id === selectedStudioItemId);
      if (activeItem) {
        activeItem.custom_content = studioEditor.getValue();
      }
    }

    const [moved] = currentItems.splice(dragSrcIdx, 1);
    let insertIdx = targetIdx;
    if (dragSrcIdx < targetIdx && isTop) {
      insertIdx = targetIdx - 1;
    } else if (dragSrcIdx > targetIdx && !isTop) {
      insertIdx = targetIdx + 1;
    }
    currentItems.splice(insertIdx, 0, moved);
    selectedStudioItemId = moved.id;
    renderStudio();
  });

  studioListEl.addEventListener("dragend", () => {
    studioListEl.querySelectorAll(".studio-item-card").forEach(c => {
      c.classList.remove("dragging", "drag-over-top", "drag-over-bottom");
    });
    dragSrcIdx = null;
  });

  // Studio List Interactions: Select, Detach
  studioListEl.addEventListener("click", (e) => {
    if (e.target.closest(".studio-card-toggle")) return;

    // Detach button
    const btnDetach = e.target.closest(".btn-detach");
    if (btnDetach) {
      e.stopPropagation();
      const id = btnDetach.dataset.id;
      const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
      const pos = currentItems.findIndex(x => x.id === id);
      if (pos !== -1) {
        currentItems.splice(pos, 1);
        if (selectedStudioItemId === id) {
          selectedStudioItemId = currentItems.length > 0 ? currentItems[Math.max(0, pos - 1)].id : null;
        }
        renderStudio();
      }
      return;
    }

    // Select Card
    const card = e.target.closest(".studio-item-card");
    if (card) {
      if (selectedStudioItemId && studioEditor) {
        const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
        const prevItem = currentItems.find(x => x.id === selectedStudioItemId);
        if (prevItem) {
          prevItem.custom_content = studioEditor.getValue();
        }
      }
      selectedStudioItemId = card.dataset.id;
      renderStudio();
    }
  });

  // Enable/Disable Toggle Change
  studioListEl.addEventListener("change", (e) => {
    if (e.target.classList.contains("studio-item-enable")) {
      const id = e.target.dataset.id;
      const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
      const item = currentItems.find(x => x.id === id);
      if (item) {
        item.enabled = e.target.checked;
        const card = e.target.closest(".studio-item-card");
        if (card) {
          card.classList.toggle("disabled", !item.enabled);
        }
      }
    }
  });

  // Studio Editor Inputs
  document.getElementById("studio-item-name").addEventListener("input", (e) => {
    const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
    const item = currentItems.find(x => x.id === selectedStudioItemId);
    if (item) {
      item.name = e.target.value;
      const cardTitle = document.querySelector(`.studio-item-card[data-id="${item.id}"] .studio-card-name`);
      if (cardTitle) {
        cardTitle.textContent = item.name;
      }
    }
  });

  // Snippets in Studio
  document.querySelectorAll("#studio-snippets-bar .btn-snippet").forEach(btn => {
    btn.addEventListener("click", () => {
      const snippet = btn.dataset.snippet;
      if (!studioEditor) return;
      let snippetText = "";
      if (snippet === "log") {
        snippetText = `\nconsole.log("[Appify] Extension active:", window.location.href);\n`;
      } else if (snippet === "dark") {
        snippetText = `\nwindow.matchMedia = window.matchMedia || function() { return { matches: true, addListener: function() {}, removeListener: function() {} }; };\n`;
      } else if (snippet === "hide-scroll") {
        snippetText = `\n::-webkit-scrollbar { display: none !important; width: 0 !important; height: 0 !important; }\n`;
      }
      insertSnippetToEditor(studioEditor, snippetText);
    });
  });

  // Add Custom Extension to App
  document.getElementById("btn-app-add-custom").addEventListener("click", () => {
    const isPlugin = activeStudioTab === "plugin";
    const newItem = {
      id: `custom-${isPlugin ? "js" : "css"}-${Date.now()}`,
      name: isPlugin ? `Custom Script ${appActivePlugins.length + 1}` : `Custom Style ${appActiveThemes.length + 1}`,
      category: isPlugin ? "plugin" : "theme",
      enabled: true,
      custom_content: isPlugin 
        ? `// Custom JavaScript\nconsole.log("[Appify] Running custom script");\n`
        : `/* Custom CSS */\n`
    };
    if (isPlugin) {
      appActivePlugins.push(newItem);
    } else {
      appActiveThemes.push(newItem);
    }
    selectedStudioItemId = newItem.id;
    renderStudio();
    setTimeout(() => {
      if (studioEditor) {
        studioEditor.focus();
      } else {
        const codeArea = document.getElementById("studio-item-code");
        if (codeArea) codeArea.focus();
      }
    }, 50);
  });

  // Attach from Registry Modal State & Infinite Scrolling
  let pickerFilteredItems = [];
  let pickerRenderedCount = 0;
  const PICKER_PAGE_SIZE = 20;

  const openAttachPicker = () => {
    const isPlugin = activeStudioTab === "plugin";
    const available = isPlugin ? (registryExtensions.plugins || []) : (registryExtensions.themes || []);
    document.getElementById("modal-picker-title").textContent = isPlugin ? "Attach Plugin from Registry" : "Attach Theme from Registry";
    const descEl = document.getElementById("modal-picker-desc");
    if (descEl) {
      descEl.textContent = `Pick an installed ${isPlugin ? "plugin" : "theme"} to add to this app`;
    }
    document.getElementById("picker-search-input").value = "";
    initAttachPickerList("");
    attachPickerModal.classList.remove("hidden");
  };

  const renderPickerCardHtml = (ext, isAttached) => {
    return `
      <div class="picker-item ${isAttached ? "already-attached" : ""}" data-id="${ext.id}">
        <div class="picker-item-info">
          <div class="picker-item-header">
            <span class="picker-item-name">${escapeHtml(ext.name)}</span>
            ${ext.is_builtin ? '<span class="tag-badge badge-purple">Built-in</span>' : '<span class="tag-badge">Custom</span>'}
          </div>
          <p class="picker-item-desc">${escapeHtml(ext.description || "No description provided.")}</p>
        </div>
        <div class="picker-item-action">
          ${isAttached 
            ? `<span class="badge-attached">
                 <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="attached-check-icon">
                   <polyline points="20 6 9 17 4 12"></polyline>
                 </svg>
                 Attached
               </span>` 
            : `<button type="button" class="btn btn-secondary btn-sm btn-pick-attach">
                 <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="btn-attach-icon">
                   <path d="M12 5v14M5 12h14"></path>
                 </svg>
                 Attach
               </button>`}
        </div>
      </div>
    `;
  };

  const initAttachPickerList = (query = "") => {
    const isPlugin = activeStudioTab === "plugin";
    const available = isPlugin ? (registryExtensions.plugins || []) : (registryExtensions.themes || []);
    const attachedIds = new Set((isPlugin ? appActivePlugins : appActiveThemes).map(x => x.id));
    const q = query.toLowerCase().trim();

    const filtered = available.filter(ext => {
      if (q && !ext.name.toLowerCase().includes(q) && !(ext.description || "").toLowerCase().includes(q)) {
        return false;
      }
      return true;
    });

    // Sort: Unattached items at the top, already-attached items grayed out at the bottom!
    filtered.sort((a, b) => {
      const aAttached = attachedIds.has(a.id);
      const bAttached = attachedIds.has(b.id);
      if (aAttached !== bAttached) {
        return aAttached ? 1 : -1;
      }
      return a.name.localeCompare(b.name);
    });

    pickerFilteredItems = filtered;
    pickerRenderedCount = 0;
    const list = document.getElementById("picker-items-list");
    list.innerHTML = "";

    if (filtered.length === 0) {
      list.innerHTML = `<div class="picker-empty">No matching installed extensions found.</div>`;
      return;
    }

    appendMorePickerItems();
  };

  const appendMorePickerItems = () => {
    if (pickerRenderedCount >= pickerFilteredItems.length) return;

    const isPlugin = activeStudioTab === "plugin";
    const attachedIds = new Set((isPlugin ? appActivePlugins : appActiveThemes).map(x => x.id));
    const list = document.getElementById("picker-items-list");

    const existingSentinel = list.querySelector(".picker-load-sentinel");
    if (existingSentinel) existingSentinel.remove();

    const nextBatch = pickerFilteredItems.slice(pickerRenderedCount, pickerRenderedCount + PICKER_PAGE_SIZE);
    const html = nextBatch.map(ext => renderPickerCardHtml(ext, attachedIds.has(ext.id))).join("");
    list.insertAdjacentHTML("beforeend", html);
    pickerRenderedCount += nextBatch.length;

    if (pickerRenderedCount < pickerFilteredItems.length) {
      list.insertAdjacentHTML("beforeend", `
        <div class="picker-load-sentinel">
          <span>Loaded ${pickerRenderedCount} of ${pickerFilteredItems.length} installed items — scroll for more</span>
        </div>
      `);
    }
  };

  const pickerListEl = document.getElementById("picker-items-list");
  pickerListEl.addEventListener("scroll", () => {
    if (pickerListEl.scrollTop + pickerListEl.clientHeight >= pickerListEl.scrollHeight - 50) {
      appendMorePickerItems();
    }
  });

  document.getElementById("btn-app-attach-registry").addEventListener("click", openAttachPicker);
  document.getElementById("picker-search-input").addEventListener("input", (e) => {
    initAttachPickerList(e.target.value);
  });
  document.getElementById("btn-picker-close").addEventListener("click", () => attachPickerModal.classList.add("hidden"));
  document.getElementById("btn-picker-cancel").addEventListener("click", () => attachPickerModal.classList.add("hidden"));

  document.getElementById("picker-items-list").addEventListener("click", (e) => {
    const btn = e.target.closest(".btn-pick-attach");
    if (!btn) return;
    const card = e.target.closest(".picker-item");
    if (!card) return;
    const id = card.dataset.id;
    const isPlugin = activeStudioTab === "plugin";
    const registryList = isPlugin ? (registryExtensions.plugins || []) : (registryExtensions.themes || []);
    const regItem = registryList.find(x => x.id === id);
    if (!regItem) return;

    const newRef = {
      id: regItem.id,
      name: regItem.name,
      category: isPlugin ? "plugin" : "theme",
      enabled: true,
      custom_content: null
    };

    if (isPlugin) {
      appActivePlugins.push(newRef);
    } else {
      appActiveThemes.push(newRef);
    }

    selectedStudioItemId = newRef.id;
    renderStudio();
    initAttachPickerList(document.getElementById("picker-search-input").value);
    showToast(`Attached '${regItem.name}' to this app.`, "success");
  });

  // Modal Save Changes
  document.getElementById("btn-modal-save").addEventListener("click", async () => {
    const hash = activeEditHash;
    if (!hash) return;

    // Flush current editor fields to selected item
    if (selectedStudioItemId) {
      const currentItems = activeStudioTab === "plugin" ? appActivePlugins : appActiveThemes;
      const item = currentItems.find(x => x.id === selectedStudioItemId);
      if (item) {
        item.name = document.getElementById("studio-item-name").value.trim() || item.name;
        if (studioEditor) {
          item.custom_content = studioEditor.getValue();
        } else {
          item.custom_content = document.getElementById("studio-item-code") ? document.getElementById("studio-item-code").value : "";
        }
      }
    }

    const domains = document.getElementById("edit-domains").value
      .split(",")
      .map(s => s.trim())
      .filter(s => s.length > 0);

    const updateReq = {
      name: document.getElementById("edit-name").value.trim(),
      wm_class: document.getElementById("edit-wmclass").value.trim(),
      custom_allowed_domains: domains,
      hide_on_close: document.getElementById("edit-hide-close").checked,
      single_instance: document.getElementById("edit-single-instance").checked,
      tray: document.getElementById("edit-tray").checked,
      start_hidden: document.getElementById("edit-start-hidden").checked,
      maximize: false,
      zoom: parseFloat(document.getElementById("edit-zoom").value) || null,
      user_agent: null,
      width: parseFloat(document.getElementById("edit-width").value) || null,
      height: parseFloat(document.getElementById("edit-height").value) || null,
      autostart: document.getElementById("edit-autostart").checked,
      autostart_hidden: document.getElementById("edit-autostart-hidden").checked,
      inject_js: null,
      inject_css: null,
      app_plugins: appActivePlugins,
      app_themes: appActiveThemes,
    };

    try {
      showToast("Saving app configuration & compiling extensions...", "info");
      await invoke("update_app_config_cmd", { hash, req: updateReq });
      await invoke("save_app_scripts_cmd", { 
        hash, 
        js: "", 
        css: "",
        plugins: appActivePlugins,
        themes: appActiveThemes
      });

      showToast("Settings and extensions saved! Launch to apply.", "success");
      closeEditModal();
      loadInstalledApps();
    } catch (err) {
      showToast(`Failed to update settings: ${err}`, "error");
    }
  });

  // Modal Reinstall & Refresh Icon
  document.getElementById("btn-modal-reinstall").addEventListener("click", async () => {
    const hash = activeEditHash;
    if (!hash) return;
    const app = installedApps.find(a => a.hash === hash);
    if (!app) return;

    try {
      showToast(`Refreshing icon and updating launcher for '${app.name}'...`, "info");
      await invoke("update_app_config_cmd", { 
        hash, 
        req: {
          name: app.name,
          wm_class: app.wm_class,
          custom_allowed_domains: Array.isArray(app.custom_allowed_domains) ? app.custom_allowed_domains : [],
          hide_on_close: Boolean(app.hide_on_close),
          single_instance: Boolean(app.single_instance),
          tray: Boolean(app.tray),
          start_hidden: Boolean(app.start_hidden),
          maximize: Boolean(app.maximize),
          zoom: (app.zoom !== undefined && app.zoom !== null && !isNaN(app.zoom)) ? Number(app.zoom) : null,
          user_agent: app.user_agent || null,
          width: (app.width !== undefined && app.width !== null && !isNaN(app.width)) ? Number(app.width) : null,
          height: (app.height !== undefined && app.height !== null && !isNaN(app.height)) ? Number(app.height) : null,
          autostart: Boolean(app.autostart),
          autostart_hidden: Boolean(app.autostart_hidden),
          inject_js: null,
          inject_css: null,
          app_plugins: appActivePlugins || [],
          app_themes: appActiveThemes || [],
        },
        reinstall_icon: true 
      });
      showToast("Reinstall complete!", "success");
      closeEditModal();
      loadInstalledApps();
    } catch (err) {
      showToast(`Reinstall failed: ${err}`, "error");
    }
  });

  // Modal Clear Cache
  document.getElementById("btn-modal-clear-cache").addEventListener("click", async () => {
    const hash = activeEditHash;
    if (!hash) return;
    const app = installedApps.find(a => a.hash === hash);
    if (!app) return;

    if (confirm(`Are you sure you want to clear session cache and cookies for '${app.name}'? You will need to log in again.`)) {
      try {
        await invoke("clear_app_cache_cmd", { hash });
        showToast("Session cache cleared successfully.", "success");
      } catch (err) {
        showToast(`Failed to clear cache: ${err}`, "error");
      }
    }
  });

  // Modal Open App Folder
  document.getElementById("btn-modal-open-folder").addEventListener("click", async () => {
    const hash = activeEditHash;
    if (!hash) return;
    try {
      await invoke("open_app_folder_cmd", { hash });
      showToast("Opened app folder in file manager.", "info");
    } catch (err) {
      showToast(`Failed to open folder: ${err}`, "error");
    }
  });

  // System Info & Self Install
  const loadSystemInfo = async () => {
    try {
      const info = await invoke("get_system_info_cmd");
      if (info) {
        if (info.bin_path) document.getElementById("sys-bin-path").textContent = info.bin_path;
        if (info.profiles_dir) document.getElementById("sys-profiles-path").textContent = info.profiles_dir;
        if (info.applications_dir) document.getElementById("sys-desktop-path").textContent = info.applications_dir;
        if (info.autostart_dir) document.getElementById("sys-autostart-path").textContent = info.autostart_dir;
      }
    } catch (_) {}
  };

  document.getElementById("btn-self-install").addEventListener("click", async () => {
    const btn = document.getElementById("btn-self-install");
    try {
      btn.disabled = true;
      showToast("Registering Appify in system applications...", "info");
      const res = await invoke("self_install_binary_cmd");
      showToast(res || "Appify registered successfully!", "success");
      loadSystemInfo();
    } catch (err) {
      showToast(`Self-installation failed: ${err}`, "error");
    } finally {
      btn.disabled = false;
    }
  });

  // Initialize
  document.addEventListener("DOMContentLoaded", () => {
    setupNumberSteppers();
    loadInstalledApps();
    loadPresets();
    loadRegistry();
    loadMarketplace();
  });
})();
