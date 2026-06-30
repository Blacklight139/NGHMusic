// 搜索页：搜索框 + 结果列表（标题 / 艺术家 / 来源标签）
export function renderSearch(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">搜索</h2>
      <div class="search-bar">
        <input id="search-input" class="search-input" type="text"
               placeholder="输入歌曲 / 艺术家 / 专辑…" />
        <button id="search-btn" class="btn">搜索</button>
      </div>
      <div id="search-results" class="song-list"></div>
    </div>`;

  const input = root.querySelector("#search-input");
  const btn = root.querySelector("#search-btn");
  const results = root.querySelector("#search-results");
  const { invoke, escapeHtml } = window.__app;

  results.innerHTML = `<div class="empty-state">输入关键词后点击搜索</div>`;

  async function doSearch() {
    const keyword = input.value.trim();
    if (!keyword) return;
    btn.disabled = true;
    results.innerHTML = `<div class="empty-state">搜索中…</div>`;
    try {
      const res = await invoke("search", { keyword, page: 1, page_size: 20 });
      renderResults(res);
    } catch (e) {
      results.innerHTML = `<div class="empty-state">搜索失败：${escapeHtml(String(e))}</div>`;
    } finally {
      btn.disabled = false;
    }
  }

  function renderResults(res) {
    const songs = (res && res.songs) || [];
    if (!songs.length) {
      results.innerHTML = `<div class="empty-state">未找到「${escapeHtml(
        (res && res.keyword) || ""
      )}」相关结果</div>`;
      return;
    }
    results.innerHTML = songs
      .map(
        (s, i) => `
      <div class="song-item">
        <div class="song-index">${i + 1}</div>
        <div class="song-meta">
          <div class="song-title">${escapeHtml(s.title || "")}</div>
          <div class="song-artist">${escapeHtml((s.artists || []).join(" / "))}</div>
        </div>
        <span class="tag">${escapeHtml(s.source_id || "source")}</span>
      </div>`
      )
      .join("");
  }

  btn.addEventListener("click", doSearch);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") doSearch();
  });
}
