// 播放列表页：当前播放列表（占位）
export function renderPlaylist(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">播放列表</h2>
      <p class="page-subtitle">当前播放队列</p>
      <div id="playlist-list" class="song-list">
        <div class="empty-state">播放列表为空</div>
      </div>
    </div>`;
}
