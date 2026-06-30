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
const MODE_ICON = { sequential: "\u21BB", single_loop: "\u21BB\u00B9", random: "\u21C4" };

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

  // 淡出 → 切换内容 → 淡入（与 styles.css 的 transition 时长匹配）
  el.classList.remove("page-visible");
  window.setTimeout(() => {
    pages[page](el);
    document.querySelectorAll(".nav-item").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.page === page);
    });
    // 下一帧再触发淡入，确保浏览器应用初始状态
    requestAnimationFrame(() => el.classList.add("page-visible"));
  }, 180);
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
    el.textContent = `v${v}`;
  } catch (e) {
    el.textContent = "v?";
  }
}

function initPlayer() {
  const playBtn = document.getElementById("btn-play");
  if (playBtn) {
    playBtn.addEventListener("click", () => {
      const playing = playBtn.classList.toggle("playing");
      playBtn.textContent = playing ? "\u23F8" : "\u25B6";
    });
  }

  const lyricsBtn = document.getElementById("btn-lyrics");
  if (lyricsBtn) {
    lyricsBtn.addEventListener("click", () => navigate("lyrics"));
  }

  const modeBtn = document.getElementById("btn-mode");
  if (modeBtn) {
    let i = 0;
    modeBtn.addEventListener("click", () => {
      i = (i + 1) % PLAY_MODES.length;
      const mode = PLAY_MODES[i];
      modeBtn.dataset.mode = mode;
      modeBtn.textContent = MODE_ICON[mode];
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
