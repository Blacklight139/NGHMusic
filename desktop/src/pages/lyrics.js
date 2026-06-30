// 歌词页：歌词滚动区（占位）
export function renderLyrics(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">歌词</h2>
      <div class="lyrics-view" id="lyrics-view">
        <div class="lyrics-line">暂无歌词</div>
        <div class="lyrics-line">播放歌曲后此处将同步滚动显示歌词</div>
      </div>
    </div>`;
}
