// 前端入口：导航路由 + Tauri command 调用 + 播放栏交互
import { renderSearch } from "./pages/search.js";
import { renderPlaylist } from "./pages/playlist.js";
import { renderFavorites } from "./pages/favorites.js";
import { renderLeaderboard } from "./pages/leaderboard.js";
import { renderLocal } from "./pages/local.js";
import { renderLyrics } from "./pages/lyrics.js";
import { renderSettings } from "./pages/settings.js";

const pages = {
  search: renderSearch,
  playlist: renderPlaylist,
  favorites: renderFavorites,
  leaderboard: renderLeaderboard,
  local: renderLocal,
  lyrics: renderLyrics,
  settings: renderSettings,
};

const PLAY_MODES = ["sequential", "single_loop", "random"];
const MODE_LABELS = { sequential: "顺序", single_loop: "单曲", random: "随机" };

// 内联 Lucide SVG 图标（24x24 viewBox，stroke=currentColor，离线可用）
const svg = (paths) =>
  `<svg class="icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;

const ICON = {
  play: svg('<polygon points="6 3 20 12 6 21 6 3"/>'),
  pause:
    svg('<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>'),
  repeat:
    svg('<path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/>'),
  repeat1:
    svg('<path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/><path d="M11 10h1v4"/>'),
  shuffle:
    svg('<path d="M2 18h1.4c1.3 0 2.5-.6 3.3-1.7l6.1-8.6c.7-1.1 2-1.7 3.3-1.7H22"/><path d="m18 2 4 4-4 4"/><path d="M2 6h1.9c1.5 0 2.9.9 3.6 2.2"/><path d="M22 18h-5.9c-1.3 0-2.6-.7-3.3-1.8l-.5-.8"/><path d="m18 14 4 4-4 4"/>'),
};

// 播放模式 SVG 图标（顺序=列表循环 / 单曲=repeat-1 / 随机=shuffle）
const MODE_ICON = {
  sequential: ICON.repeat,
  single_loop: ICON.repeat1,
  random: ICON.shuffle,
};

// 调用 Tauri command；非 Tauri 环境（如直接用浏览器打开）优雅降级
function invoke(cmd, args) {
  const tauri = window.__TAURI__;
  if (tauri && tauri.core && typeof tauri.core.invoke === "function") {
    return tauri.core.invoke(cmd, args);
  }
  return Promise.reject(new Error("Tauri invoke 不可用（当前非 Tauri 运行环境）"));
}

function escapeHtml(s) {
  return String(s == null ? "" : s).replace(/[&<>"']/g, (c) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  }[c]));
}

// 暴露给各页面模块复用，避免重复实现
window.__app = { invoke, escapeHtml };

const view = () => document.getElementById("page-view");

function navigate(page) {
  const el = view();
  if (!el || !pages[page]) return;

  // 离场 150ms ease-in -> 切换内容 -> 进场 200ms cubic-bezier（见 styles.css）
  el.classList.remove("page-visible");
  window.setTimeout(() => {
    pages[page](el);
    document.querySelectorAll(".nav-item").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.page === page);
    });
    // 下一帧再触发淡入，确保浏览器应用初始状态
    requestAnimationFrame(() => el.classList.add("page-visible"));
  }, 150);
}

function initNav() {
  document.querySelectorAll(".nav-item").forEach((btn) => {
    btn.addEventListener("click", () => navigate(btn.dataset.page));
  });
}

async function initVersion() {
  const el = document.getElementById("app-version");
  if (!el) return;
  try {
    const v = await invoke("app_version");
    el.textContent = v ? `NGHMusic v${v}` : "NGHMusic";
  } catch (e) {
    el.textContent = "NGHMusic";
  }
}

function initPlayer() {
  const playBtn = document.getElementById("btn-play");
  if (playBtn) {
    // 初始为「播放」态，点击后切换 SVG 图标
    playBtn.innerHTML = ICON.play;
    playBtn.addEventListener("click", () => {
      const playing = playBtn.classList.toggle("playing");
      playBtn.innerHTML = playing ? ICON.pause : ICON.play;
      playBtn.title = playing ? "暂停" : "播放";
    });
  }

  const lyricsBtn = document.getElementById("btn-lyrics");
  if (lyricsBtn) {
    lyricsBtn.addEventListener("click", () => navigate("lyrics"));
  }

  const modeBtn = document.getElementById("btn-mode");
  if (modeBtn) {
    let i = 0;
    modeBtn.innerHTML = MODE_ICON[PLAY_MODES[i]];
    modeBtn.addEventListener("click", () => {
      i = (i + 1) % PLAY_MODES.length;
      const mode = PLAY_MODES[i];
      modeBtn.dataset.mode = mode;
      modeBtn.innerHTML = MODE_ICON[mode];
      modeBtn.title = `播放模式：${MODE_LABELS[mode]}`;
    });
  }

  const volume = document.getElementById("volume");
  if (volume) {
    volume.addEventListener("input", () => {
      // 占位：实际播放器接入后在此设置音量
    });
  }

  const progress = document.getElementById("progress");
  if (progress) {
    progress.addEventListener("input", () => {
      // 占位：实际播放器接入后在此 seek
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  initNav();
  initPlayer();
  initVersion();
  navigate("search");
});
