// 本地音乐页：添加目录按钮 + 歌曲列表，调用 list_local_songs command
export function renderLocal(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">本地音乐</h2>
      <div class="row" style="margin-bottom: var(--space-4)">
        <button id="add-dir-btn" class="btn">添加目录</button>
        <button id="refresh-btn" class="btn btn-secondary">刷新</button>
        <span class="hint">扫描本地音乐文件</span>
      </div>
      <div id="local-list" class="song-list">
        <div class="empty-state">本地音乐库为空</div>
      </div>
    </div>`;

  const list = root.querySelector("#local-list");
  const { invoke, escapeHtml } = window.__app;

  async function load() {
    list.innerHTML = `<div class="empty-state">加载中…</div>`;
    try {
      const songs = await invoke("list_local_songs");
      if (!Array.isArray(songs) || !songs.length) {
        list.innerHTML = `<div class="empty-state">本地音乐库为空</div>`;
        return;
      }
      list.innerHTML = songs
        .map(
          (s, i) => `
        <div class="song-item">
          <div class="song-index">${i + 1}</div>
          <div class="song-meta">
            <div class="song-title">${escapeHtml(s.title || "")}</div>
            <div class="song-artist">${escapeHtml((s.artists || []).join(" / "))}</div>
          </div>
          <span class="tag">本地</span>
        </div>`
        )
        .join("");
    } catch (e) {
      list.innerHTML = `<div class="empty-state">加载失败：${escapeHtml(String(e))}</div>`;
    }
  }

  root.querySelector("#refresh-btn").addEventListener("click", load);
  root.querySelector("#add-dir-btn").addEventListener("click", () => {
    // 占位：实际接入后调用对话框选择目录并触发扫描
    list.innerHTML = `<div class="empty-state">目录选择尚未接入（占位）</div>`;
  });

  load();
}
