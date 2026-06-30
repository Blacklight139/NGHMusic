// 收藏页：收藏夹分组（占位卡片）
export function renderFavorites(root) {
  root.innerHTML = `
    <div class="page">
      <h2 class="page-title">收藏</h2>
      <p class="page-subtitle">收藏夹分组</p>
      <div class="card-grid" id="fav-grid">
        <div class="empty-state" style="grid-column: 1 / -1">暂无收藏分组，点击歌曲收藏后会出现在这里</div>
      </div>
    </div>`;
}
