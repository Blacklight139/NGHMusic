// 排行榜页：排行榜列表（占位卡片）
export function renderLeaderboard(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">排行榜</h2>
      <p class="page-subtitle">各音源提供的排行榜</p>
      <div class="card-grid" id="lb-grid">
        <div class="empty-state" style="grid-column: 1 / -1">暂无可用排行榜，启用音源后将自动加载</div>
      </div>
    </div>`;
}
