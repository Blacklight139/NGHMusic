// 设置页：音源导入（文件选择 + JSON 文本框 + 导入按钮）+ 缓存管理
export function renderSettings(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">设置</h2>

      <section style="margin-bottom: var(--space-6)">
        <h3 class="page-subtitle">音源导入</h3>
        <div class="row" style="margin-bottom: var(--space-3)">
          <input type="file" id="source-file" accept="application/json,.json" />
          <button id="load-file-btn" class="btn btn-secondary">载入文件</button>
        </div>
        <div class="form-group">
          <label class="form-label" for="source-json">音源 JSON（标准格式 / 社区格式 A / 社区格式 B）</label>
          <textarea id="source-json" class="textarea" placeholder='例如：{"name":"示例","search_url":"https://...","song_url":"https://..."}'></textarea>
        </div>
        <div class="row">
          <button id="import-btn" class="btn">导入并校验</button>
          <span class="hint">导入后会调用 music-core 适配 + schema 严格校验</span>
        </div>
        <pre id="import-result" class="result-box" style="display:none"></pre>
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
  const jsonArea = root.querySelector("#source-json");
  const fileInput = root.querySelector("#source-file");
  const result = root.querySelector("#import-result");

  function show(text, ok) {
    result.style.display = "block";
    result.textContent = text;
    result.className = "result-box " + (ok ? "ok" : "error");
  }

  root.querySelector("#load-file-btn").addEventListener("click", () => {
    const file = fileInput.files && fileInput.files[0];
    if (!file) {
      show("请先选择 JSON 文件", false);
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      jsonArea.value = String(reader.result || "");
    };
    reader.onerror = () => show("文件读取失败", false);
    reader.readAsText(file);
  });

  async function doImport() {
    const json = jsonArea.value.trim();
    if (!json) {
      show("请输入或载入音源 JSON", false);
      return;
    }
    show("导入中…", true);
    try {
      const out = await invoke("import_source", { json });
      show("导入成功，适配后的标准配置：\n" + out, true);
    } catch (e) {
      show("导入失败：" + escapeHtml(String(e)), false);
    }
  }

  root.querySelector("#import-btn").addEventListener("click", doImport);

  root.querySelector("#clear-cache-btn").addEventListener("click", () => {
    // 占位：实际接入后调用缓存清理 command
    root.querySelector("#cache-hint").textContent = "缓存已清除（占位）";
  });
}
