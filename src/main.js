import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

let cacheDir = null;
let videos = [];
let converting = false;

function fmtSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
}

async function doScan(dir) {
  try {
    const list = await invoke('scan_cache', { dir });
    videos = list || [];
    cacheDir = dir;
    renderList();
    return list;
  } catch (e) {
    alert('扫描失败: ' + e);
    return [];
  }
}

async function scanDefault() {
  return doScan(null);
}

const QUALITY_ORDER = ['1080P+', '1080P60', '1080P', '720P60', '720P', '480P', '360P', '240P', '未知'];

function getFilteredVideos() {
  const search = document.getElementById('search-input').value?.trim().toLowerCase() || '';
  const qualityFilter = document.getElementById('filter-quality').value || '';
  let list = videos;
  if (search) {
    list = list.filter(v => (v.title || '').toLowerCase().includes(search));
  }
  if (qualityFilter) list = list.filter(v => v.quality === qualityFilter);
  return list;
}

function renderFilterOptions() {
  const qualityEl = document.getElementById('filter-quality');
  const prevQuality = qualityEl.value;

  const qualities = [...new Set(videos.map(v => v.quality).filter(Boolean))];
  qualities.sort((a, b) => {
    const ia = QUALITY_ORDER.indexOf(a);
    const ib = QUALITY_ORDER.indexOf(b);
    if (ia >= 0 && ib >= 0) return ia - ib;
    if (ia >= 0) return -1;
    if (ib >= 0) return 1;
    return (a || '').localeCompare(b || '');
  });

  qualityEl.innerHTML = '<option value="">全部清晰度</option>' +
    qualities.map(q => `<option value="${escapeHtml(q)}">${escapeHtml(q)}</option>`).join('');

  if (qualities.includes(prevQuality)) qualityEl.value = prevQuality;
}

function renderList() {
  const filtered = getFilteredVideos();
  const tbody = document.getElementById('video-tbody');
  const table = document.getElementById('video-table');
  const empty = document.getElementById('list-empty');

  if (videos.length === 0) {
    table.style.display = 'none';
    empty.style.display = 'flex';
    return;
  }
  renderFilterOptions();
  if (filtered.length === 0) {
    table.style.display = 'table';
    empty.style.display = 'none';
    tbody.innerHTML = '<tr><td colspan="5" class="no-results">无匹配视频</td></tr>';
    document.getElementById('check-all').checked = false;
    document.getElementById('check-all').indeterminate = false;
    updateConvertState();
    return;
  }
  empty.style.display = 'none';
  table.style.display = 'table';

  const sorted = [...filtered].sort((a, b) => (a.page || 1) - (b.page || 1));

  tbody.innerHTML = sorted.map((v, i) => `
    <tr data-idx="${i}" class="video-row">
      <td><input type="checkbox" class="row-check" data-idx="${i}"></td>
      <td title="${escapeHtml(v.title)}">${escapeHtml(v.title)}</td>
      <td>${escapeHtml(v.quality)}</td>
      <td>${fmtSize(v.size_bytes)}</td>
      <td>${v.cached_at || '-'}</td>
    </tr>
  `).join('');
  tbody._flatList = sorted;

  document.querySelectorAll('.row-check').forEach(cb => {
    cb.addEventListener('change', () => { updateConvertState(); });
  });
  document.getElementById('check-all').checked = false;
  document.getElementById('check-all').indeterminate = false;
  updateConvertState();
}

function escapeHtml(s) {
  const div = document.createElement('div');
  div.textContent = s;
  return div.innerHTML;
}

function getSelectedItems() {
  const flatList = document.getElementById('video-tbody')._flatList;
  if (!flatList) return [];
  const checked = [...document.querySelectorAll('.row-check:checked')]
    .map(cb => parseInt(cb.dataset.idx, 10))
    .filter(n => !isNaN(n) && n >= 0);
  return checked.map(i => flatList[i]).filter(Boolean);
}

function updateConvertState() {
  const n = getSelectedItems().length;
  const btn = document.getElementById('btn-convert');
  btn.disabled = converting; // 不再因未勾选而禁用，点击时由 doConvert 提示
  btn.textContent = n > 0 ? `开始转换 (${n})` : '开始转换';
  const checkAll = document.getElementById('check-all');
  const total = document.querySelectorAll('.row-check').length;
  const checked = document.querySelectorAll('.row-check:checked').length;
  checkAll.checked = total > 0 && checked === total;
  checkAll.indeterminate = checked > 0 && checked < total;
}

document.getElementById('btn-scan').addEventListener('click', () => scanDefault());
document.getElementById('btn-refresh').addEventListener('click', () => doScan(cacheDir));

document.getElementById('btn-select-dir').addEventListener('click', async () => {
  try {
  const defaultPath = await invoke('default_cache_dialog_path');
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择 B 站缓存目录',
    defaultPath: defaultPath || undefined
  });
  if (selected) {
    const path = Array.isArray(selected) ? selected[0] : selected;
    await doScan(path);
  }
  } catch (e) {
    alert('选择目录失败: ' + e);
  }
});

document.getElementById('btn-browse').addEventListener('click', async () => {
  try {
  const defaultPath = await invoke('default_output_dir');
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择输出目录',
    defaultPath: defaultPath || undefined
  });
  if (selected) {
    const path = Array.isArray(selected) ? selected[0] : selected;
    document.getElementById('output-path').value = path;
  }
  } catch (e) {
    alert('选择目录失败: ' + e);
  }
});

document.getElementById('check-all').addEventListener('change', (e) => {
  document.querySelectorAll('.row-check').forEach(cb => {
    cb.checked = e.target.checked;
  });
  updateConvertState();
});

document.getElementById('video-tbody').addEventListener('click', (e) => {
  const tr = e.target.closest('tr.video-row');
  if (!tr) return;
  if (e.target.classList.contains('row-check')) return;
  const cb = tr.querySelector('.row-check');
  if (cb) { cb.checked = !cb.checked; updateConvertState(); }
});

document.getElementById('filter-quality').addEventListener('change', renderList);
document.getElementById('conflict-strategy').addEventListener('change', async (e) => {
  try {
    const config = await invoke('get_config');
    await invoke('set_config', { config: { ...config, conflict_strategy: e.target.value } });
  } catch (_) {}
});
document.getElementById('search-input').addEventListener('input', debounce(renderList, 200));
document.getElementById('search-input').addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    document.getElementById('search-input').value = '';
    renderList();
  }
});

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(null, args), ms);
  };
}

async function doConvert() {
  const outDir = document.getElementById('output-path').value?.trim();
  if (!outDir) {
    alert('请先选择输出目录');
    return;
  }
  const items = getSelectedItems();
  if (items.length === 0) {
    alert('请勾选要转换的视频');
    return;
  }

  converting = true;
  document.getElementById('btn-convert').style.display = 'none';
  document.getElementById('btn-cancel').style.display = 'inline-block';
  document.getElementById('progress-area').style.display = 'block';
  document.getElementById('progress-fill').style.width = '0%';
  document.getElementById('progress-text').textContent = '准备中...';
  if (document.getElementById('log-mode').checked) {
    document.getElementById('log-area').style.display = 'flex';
    const logEl = document.getElementById('log-content');
    if (logEl) logEl.innerHTML = '';
    appendLog('info', '准备转换...');
  }

  try {
    const config = await invoke('get_config');
    const conflictStrategy = document.getElementById('conflict-strategy').value;
    await invoke('set_config', {
      config: { ...config, output_dir: outDir, conflict_strategy: conflictStrategy }
    });

    const paths = await invoke('convert', { items, outDir });
    document.getElementById('progress-fill').style.width = '100%';
    document.getElementById('progress-text').textContent = `完成，共 ${paths.length} 个文件`;
    const cfg = await invoke('get_config');
    if (cfg.on_complete === 'open_folder' || !cfg.on_complete) {
      if (paths.length > 0) {
        try {
          await invoke('open_folder', { path: outDir });
        } catch (e) {
          appendLog('warn', '打开输出文件夹失败: ' + String(e));
        }
      }
    }
  } catch (e) {
    if (document.getElementById('log-mode').checked) appendLog('error', '转换失败: ' + String(e));
    alert('转换失败: ' + String(e));
  } finally {
    converting = false;
    document.getElementById('btn-convert').style.display = 'inline-block';
    document.getElementById('btn-cancel').style.display = 'none';
    document.getElementById('progress-area').style.display = 'none';
    updateConvertState();
  }
}

document.getElementById('btn-convert').addEventListener('click', () => {
  if (document.getElementById('btn-convert').disabled) return;
  doConvert();
});

document.getElementById('btn-cancel').addEventListener('click', async () => {
  await invoke('cancel_convert');
});

document.getElementById('log-mode').addEventListener('change', (e) => {
  document.getElementById('log-area').style.display = e.target.checked ? 'flex' : 'none';
});

document.getElementById('btn-clear-log').addEventListener('click', () => {
  const el = document.getElementById('log-content');
  if (el) el.innerHTML = '';
});

function appendLog(level, message) {
  if (!document.getElementById('log-mode').checked) return;
  const el = document.getElementById('log-content');
  if (!el) return;
  const line = document.createElement('div');
  line.className = 'log-line' + (level === 'error' ? ' error' : level === 'warn' ? ' warn' : '');
  line.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
  el.appendChild(line);
  el.scrollTop = el.scrollHeight;
}

listen('convert-log', (e) => {
  const { level, message } = e.payload || {};
  appendLog(level || 'info', message || String(e.payload));
});

listen('convert-progress', (e) => {
  const p = e.payload;
  const percent = p.percent ?? 0;
  document.getElementById('progress-fill').style.width = percent + '%';
  document.getElementById('progress-text').textContent =
    `${p.current_file || ''} (${p.current_index || 0}/${p.total || 1}) ${percent}%`;
});

// TAURI_TEST_CONVERT=1 模式：自动执行完整转换流程（用于 CLI 验证）
listen('run-test-convert', async () => {
  try {
    const list = await invoke('scan_cache', { dir: null });
    if (!list?.length) {
      await invoke('report_test_result', { success: false, message: '无视频可转换' });
      return;
    }
    const items = list.slice(0, 1);
    const outDir = await invoke('default_output_dir');
    const out = outDir || '/tmp/bili2mp4-test';
    const config = await invoke('get_config');
    await invoke('set_config', { config: { ...config, output_dir: out } });
    const paths = await invoke('convert', { items, outDir: out });
    await invoke('report_test_result', { success: true, message: paths?.length ? paths.join(',') : '0 files' });
  } catch (e) {
    await invoke('report_test_result', { success: false, message: String(e) });
  }
});

// Init
(async () => {
  try {
    const config = await invoke('get_config');
    if (config.output_dir) {
      document.getElementById('output-path').value = config.output_dir;
    } else {
      const out = await invoke('default_output_dir');
      if (out) document.getElementById('output-path').value = out;
    }
    const cs = document.getElementById('conflict-strategy');
    if (cs && config.conflict_strategy) cs.value = config.conflict_strategy;
    await scanDefault();
    updateConvertState();
  } catch (e) {
    alert('初始化失败: ' + String(e));
  }
})();
