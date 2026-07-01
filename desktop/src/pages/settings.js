// 设置页：音源管理（LXMusic 风格列表 + 文件上传导入）+ 缓存管理
//
// 关于文件路径获取与导入兼容性（见 Task 4 约束）：
//   Tauri 2 webview 中 <input type="file"> 选取的 File 对象通常没有 .path 属性。
//   本页策略：
//     1. 优先尝试 file.path -> 命中则调用 import_source_file({ filePath })（会注册到 SourceManager）。
//     2. 若 file.path 不可用（如浏览器直开 / 部分 webview），退回到 FileReader/file.text()
//        读取内容后调用旧 import_source({ json }) command 做适配 + schema 校验。
//        注意：import_source 仅做适配与校验，不会将音源注册进 SourceManager，
//        故该兼容路径下列表不会新增条目；此时向用户提示需在 Tauri 环境内通过
//        文件路径导入或拖拽文件。若后续需要「由 JSON 字符串注册」，需在 lib.rs
//        新增 import_source_json(json_str) command（本任务范围外，暂不新增）。

const svgIcon = (paths) =>
  `<svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;

const ICON = {
  upload: svgIcon(
    '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/>'
  ),
  grip: svgIcon(
    '<circle cx="9" cy="6" r="1"/><circle cx="9" cy="12" r="1"/><circle cx="9" cy="18" r="1"/><circle cx="15" cy="6" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="15" cy="18" r="1"/>'
  ),
  up: svgIcon('<path d="m18 15-6-6-6 6"/>'),
  down: svgIcon('<path d="m6 9 6 6 6-6"/>'),
  trash: svgIcon(
    '<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/>'
  ),
  package: svgIcon(
    '<path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/>'
  ),
  puzzle: svgIcon(
    '<path d="M19.439 7.85c-.049.322.059.648.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.877-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 0-3.237-3.237c-.464.18-.894.527-.967 1.02a1.026 1.026 0 0 1-.29.877l-1.568 1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.23 8.77c.24-.24.581-.353.917-.303.515.077.877.528 1.073 1.01a2.5 2.5 0 1 0 3.259-3.259c-.482-.196-.933-.558-1.01-1.073-.05-.336.062-.676.303-.917l1.525-1.525A2.402 2.402 0 0 1 12 1.998c.617 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.877.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 0 3.237 3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 1 .29-.877l1.568-1.568a2.402 2.402 0 0 1 1.704-.706c.617 0 1.234.236 1.704.706l1.568 1.568c.23.23.338.556.29.877Z"/>'
  ),
  drive: svgIcon(
    '<line x1="22" x2="2" y1="12" y2="12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/><line x1="6" x2="6.01" y1="16" y2="16"/><line x1="10" x2="10.01" y1="16" y2="16"/>'
  ),
};

// 将后端 source_type 字符串映射为来源标签 + 图标（json / community / local）
function sourceTypeMeta(t) {
  const s = String(t || "").toLowerCase();
  if (s.includes("comm")) return { label: "community", icon: ICON.puzzle };
  if (s.includes("local")) return { label: "local", icon: ICON.drive };
  return { label: "json", icon: ICON.package };
}

export function renderSettings(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">设置</h2>

      <section style="margin-bottom: var(--space-6)">
        <div class="source-header">
          <div>
            <h3 class="page-subtitle" style="margin-bottom: var(--space-1)">音源管理</h3>
            <span class="hint" id="source-subtitle">已导入 0 个音源，按顺序聚合搜索</span>
          </div>
          <button id="import-btn" class="btn" type="button" title="导入音源">
            ${ICON.upload}<span>导入音源</span>
          </button>
          <input type="file" id="source-file" accept=".json,application/json" hidden />
        </div>
        <pre id="import-result" class="result-box" style="display:none"></pre>
        <div class="source-list" id="source-list"></div>
      </section>

      <section>
        <h3 class="page-subtitle">缓存管理</h3>
        <div class="row">
          <button id="clear-cache-btn" class="btn btn-secondary">清除缓存</button>
          <span class="hint" id="cache-hint">缓存占用：未知</span>
        </div>
      </section>
    </div>`;

  const { invoke, escapeHtml } = window.__app;
  const listEl = root.querySelector("#source-list");
  const subtitleEl = root.querySelector("#source-subtitle");
  const result = root.querySelector("#import-result");
  const fileInput = root.querySelector("#source-file");
  const importBtn = root.querySelector("#import-btn");

  // 当前音源列表（按优先级降序，与后端 list_sources 一致）；展开详情用 Set 记录 id
  let sources = [];
  const expanded = new Set();

  function show(text, ok) {
    result.style.display = "block";
    result.textContent = text;
    result.className = "result-box " + (ok ? "ok" : "error");
  }
  function clearShow() {
    result.style.display = "none";
    result.textContent = "";
  }

  function renderItem(s, i) {
    const meta = sourceTypeMeta(s.source_type);
    const isOpen = expanded.has(s.id);
    const desc = s.description ? escapeHtml(s.description) : "—";
    return `
      <div class="source-item" data-id="${escapeHtml(s.id)}">
        <span class="source-handle" title="拖动以调整顺序">${ICON.grip}</span>
        <div class="source-main" data-action="toggle">
          <div class="source-line">
            <span class="source-name">${escapeHtml(s.name)}</span>
            <span class="tag">${meta.icon}<span>${escapeHtml(meta.label)}</span></span>
            <span class="source-meta">v${escapeHtml(s.version)}</span>
          </div>
          ${
            isOpen
              ? `<div class="source-detail">
              <div>id: ${escapeHtml(s.id)}</div>
              <div>source_type: ${escapeHtml(s.source_type)}</div>
              <div>version: ${escapeHtml(s.version)}</div>
              <div>description: ${desc}</div>
            </div>`
              : ""
          }
        </div>
        <button class="toggle-switch ${s.enabled ? "on" : ""}" data-action="toggle-enabled"
          role="switch" aria-checked="${s.enabled}" type="button"
          title="${s.enabled ? "已启用" : "已禁用"}"><span class="toggle-knob"></span></button>
        <div class="source-actions">
          <button class="icon-btn" data-action="up" type="button" title="上移"
            ${i === 0 ? "disabled" : ""}>${ICON.up}</button>
          <button class="icon-btn" data-action="down" type="button" title="下移"
            ${i === sources.length - 1 ? "disabled" : ""}>${ICON.down}</button>
          <button class="icon-btn danger" data-action="delete" type="button" title="删除">${ICON.trash}</button>
        </div>
      </div>`;
  }

  function renderList() {
    subtitleEl.textContent = `已导入 ${sources.length} 个音源，按顺序聚合搜索`;
    if (sources.length === 0) {
      listEl.innerHTML = `<div class="empty-state">暂无音源，点击右上角导入音源</div>`;
      return;
    }
    listEl.innerHTML = sources.map((s, i) => renderItem(s, i)).join("");
  }

  async function loadSources() {
    try {
      sources = await invoke("list_sources");
    } catch (e) {
      sources = [];
      show("加载音源列表失败：" + escapeHtml(String(e)), false);
    }
    renderList();
  }

  // 单击音源项主体（非按钮区域）-> 展开 / 收起详情
  async function toggleEnabled(idx, btnEl) {
    const s = sources[idx];
    const next = !s.enabled;
    try {
      await invoke("set_source_enabled", { id: s.id, enabled: next });
      sources[idx] = { ...s, enabled: next };
      if (btnEl) {
        btnEl.classList.toggle("on", next);
        btnEl.setAttribute("aria-checked", String(next));
        btnEl.title = next ? "已启用" : "已禁用";
      }
    } catch (e) {
      show("切换启用状态失败：" + escapeHtml(String(e)), false);
    }
  }

  // 上移 / 下移：本地交换后用新顺序的 id 数组调用 reorder_sources
  async function move(from, to) {
    if (from === to || to < 0 || to >= sources.length) return;
    const next = sources.slice();
    const [it] = next.splice(from, 1);
    next.splice(to, 0, it);
    const orderedIds = next.map((s) => s.id);
    try {
      await invoke("reorder_sources", { orderedIds });
      sources = next;
      renderList();
    } catch (e) {
      show("调整顺序失败：" + escapeHtml(String(e)), false);
    }
  }

  async function doDelete(idx) {
    const s = sources[idx];
    if (!window.confirm(`确定删除音源「${s.name}」？`)) return;
    try {
      await invoke("delete_source", { id: s.id });
      expanded.delete(s.id);
      sources.splice(idx, 1);
      renderList();
    } catch (e) {
      show("删除失败：" + escapeHtml(String(e)), false);
    }
  }

  // 列表事件委托：根据 data-action 分发到对应处理函数
  listEl.addEventListener("click", (e) => {
    const actionEl = e.target.closest("[data-action]");
    if (!actionEl || actionEl.disabled) return;
    const item = actionEl.closest(".source-item");
    if (!item) return;
    const id = item.dataset.id;
    const idx = sources.findIndex((s) => s.id === id);
    if (idx < 0) return;
    const action = actionEl.dataset.action;
    if (action === "toggle") {
      if (expanded.has(id)) expanded.delete(id);
      else expanded.add(id);
      renderList();
    } else if (action === "toggle-enabled") {
      toggleEnabled(idx, actionEl);
    } else if (action === "up") {
      move(idx, idx - 1);
    } else if (action === "down") {
      move(idx, idx + 1);
    } else if (action === "delete") {
      doDelete(idx);
    }
  });

  // 导入音源：点击按钮 -> 触发隐藏 file input
  importBtn.addEventListener("click", () => fileInput.click());

  fileInput.addEventListener("change", async (e) => {
    const file = e.target.files && e.target.files[0];
    if (!file) return;
    clearShow();
    try {
      // 优先走 file.path（Tauri 环境内可获取绝对路径，注册到 SourceManager）
      const filePath = file.path;
      if (typeof filePath === "string" && filePath.length > 0) {
        await invoke("import_source_file", { filePath });
        show("导入成功：" + file.name, true);
      } else {
        // 兼容路径：file.path 不可用时读取文本，调用 import_source 做适配 + 校验。
        // 注意：import_source 不会注册到 SourceManager，列表不会新增条目。
        const text = await file.text();
        const out = await invoke("import_source", { json: text });
        show(
          "已校验音源「" +
            file.name +
            "」（当前环境无法获取文件路径，仅做适配校验、未注册到列表）。\n如需注册请在 Tauri 桌面端通过文件路径导入或拖拽文件。\n\n适配后配置：\n" +
            out,
          true
        );
      }
      await loadSources();
    } catch (err) {
      show("导入失败：" + escapeHtml(String(err)), false);
    } finally {
      // 重置 value 以便同一文件可重复选择
      fileInput.value = "";
    }
  });

  root.querySelector("#clear-cache-btn").addEventListener("click", () => {
    // 占位：实际接入后调用缓存清理 command
    root.querySelector("#cache-hint").textContent = "缓存已清除（占位）";
  });

  loadSources();
}
