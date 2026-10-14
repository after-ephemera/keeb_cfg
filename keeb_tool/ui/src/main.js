import { Keyboard3D } from "./keyboard3d.js";

// Tauri v2 invoke. When opened in a plain browser (no Tauri), degrade to a
// clear message instead of crashing.
let invoke;
try {
  ({ invoke } = await import("@tauri-apps/api/core"));
} catch {
  invoke = async () => {
    throw new Error("Not running inside Tauri — launch with `npm run tauri dev`.");
  };
}

const state = {
  view: null, // ConfigView from Rust
  activeLayer: 0,
  selected: -1,
};

const kb = new Keyboard3D(document.getElementById("canvas-wrap"));
kb.onSelect = (idx) => selectKey(idx);

const $ = (id) => document.getElementById(id);
function setStatus(msg, kind = "") {
  const el = $("status");
  el.textContent = msg;
  el.className = "status " + kind;
}

// ---------- load / save / preview ----------

$("btn-load").onclick = async () => {
  const path = $("config-path").value.trim();
  if (!path) return setStatus("Enter the path to your lily58/main.py", "err");
  try {
    state.view = await invoke("load_config", { path });
    state.activeLayer = 0;
    state.selected = -1;
    kb.setLayout(state.view.layout);
    renderLayerTabs();
    renderLayer();
    renderPalette();
    renderEditor();
    setStatus(
      `Loaded ${state.view.keymap.layers.length} layers, ${state.view.keymap.key_count ?? countKeys()} keys.`,
      "ok"
    );
  } catch (e) {
    setStatus(String(e), "err");
  }
};

$("btn-save").onclick = async () => {
  if (!state.view) return;
  const path = $("config-path").value.trim();
  try {
    await invoke("save_config", { path, keymap: state.view.keymap });
    setStatus(`Saved to ${path}`, "ok");
  } catch (e) {
    setStatus(String(e), "err");
  }
};

$("btn-preview").onclick = async () => {
  if (!state.view) return;
  try {
    const text = await invoke("preview", { keymap: state.view.keymap });
    $("preview-text").textContent = text;
    $("preview-modal").hidden = false;
  } catch (e) {
    setStatus(String(e), "err");
  }
};
$("btn-close-preview").onclick = () => ($("preview-modal").hidden = true);

// ---------- layers ----------

function renderLayerTabs() {
  const tabs = $("layer-tabs");
  tabs.innerHTML = "";
  state.view.keymap.layers.forEach((layer, i) => {
    const b = document.createElement("button");
    b.textContent = layer.name ? `${i}: ${layer.name}` : `Layer ${i}`;
    b.className = i === state.activeLayer ? "active" : "";
    b.onclick = () => {
      state.activeLayer = i;
      renderLayerTabs();
      renderLayer();
    };
    tabs.appendChild(b);
  });
}

function renderLayer() {
  const keys = state.view.keymap.layers[state.activeLayer].keys;
  kb.setLayer(keys);
}

function countKeys() {
  return state.view.keymap.layers[0]?.keys.length ?? 0;
}

// ---------- key editor ----------

function selectKey(idx) {
  state.selected = idx;
  kb.select(idx);
  renderEditor();
}

function renderEditor() {
  const body = $("sel-body");
  const empty = $("sel-empty");
  if (!state.view || state.selected < 0) {
    body.hidden = true;
    empty.hidden = false;
    return;
  }
  body.hidden = false;
  empty.hidden = true;
  const key = currentKey();
  const slot = state.view.layout.find((s) => s.index === state.selected);
  $("sel-pos").textContent = slot
    ? `#${state.selected} · ${slot.side} r${slot.row}${slot.thumb ? " thumb" : ""}`
    : `#${state.selected}`;
  $("sel-label").textContent = key.label || "∅";
  $("sel-raw").value = key.raw;
}

function currentKey() {
  return state.view.keymap.layers[state.activeLayer].keys[state.selected];
}

$("btn-apply-raw").onclick = () => applyRaw($("sel-raw").value.trim());
$("sel-raw").addEventListener("keydown", (e) => {
  if (e.key === "Enter") applyRaw($("sel-raw").value.trim());
});

function applyRaw(raw) {
  if (!state.view || state.selected < 0 || !raw) return;
  const key = currentKey();
  key.raw = raw;
  key.label = quickLabel(raw);
  key.is_transparent = raw === "KC.TRNS" || raw === "KC.NO";
  key.is_alias = /^[A-Za-z_][A-Za-z0-9_]*$/.test(raw) && !raw.startsWith("KC.");
  renderLayer();
  renderEditor();
  setStatus(`Set #${state.selected} on layer ${state.activeLayer} → ${raw}`, "ok");
}

// Palette
function renderPalette() {
  const pal = $("palette");
  pal.innerHTML = "";
  let group = "";
  for (const p of state.view.palette) {
    if (p.group !== group) {
      group = p.group;
      const g = document.createElement("div");
      g.className = "grp";
      g.textContent = group;
      pal.appendChild(g);
    }
    const b = document.createElement("button");
    b.textContent = p.label;
    b.title = p.code;
    b.onclick = () => {
      $("sel-raw").value = p.code;
      applyRaw(p.code);
    };
    pal.appendChild(b);
  }
}

// A lightweight client-side label so edits feel instant; the authoritative
// label is recomputed by Rust on the next load.
function quickLabel(raw) {
  const map = {
    "KC.TRNS": "▽", "KC.NO": "", "KC.SPC": "␣", "KC.ENT": "⏎", "KC.BSPC": "⌫",
    "KC.TAB": "⇥", "KC.ESC": "⎋", "KC.DEL": "⌦", "KC.UP": "↑", "KC.DOWN": "↓",
    "KC.LEFT": "←", "KC.RIGHT": "→", "KC.LSFT": "⇧", "KC.LCTL": "⌃",
    "KC.LALT": "⌥", "KC.LGUI": "⌘", "KC.RGUI": "⌘",
  };
  if (map[raw] !== undefined) return map[raw];
  if (raw.startsWith("KC.N")) return raw.slice(4);
  if (raw.startsWith("KC.") && raw.includes("(")) {
    const head = raw.slice(3, raw.indexOf("("));
    return { TD: "TapDance", HT: "Hold/Tap", MT: "Mod/Tap", LT: "Layer/Tap" }[head] || head;
  }
  if (raw.startsWith("KC.")) return raw.slice(3);
  return raw.length > 8 ? raw.slice(0, 7) + "…" : raw;
}

// ---------- flashing ----------

$("btn-scan").onclick = async () => {
  try {
    const drives = await invoke("list_drives");
    renderDrives(drives);
  } catch (e) {
    setStatus(String(e), "err");
  }
};

function renderDrives(drives) {
  const box = $("drives");
  box.innerHTML = "";
  if (!drives.length) {
    box.innerHTML = `<div class="muted">No CIRCUITPY volume found. Plug in the keyboard and rescan.</div>`;
    return;
  }
  for (const d of drives) {
    const el = document.createElement("div");
    el.className = "drive";
    el.innerHTML = `<div class="p">${d.path}</div>
      <div class="i">${d.info ? escapeHtml(d.info) : "(no boot_out.txt)"}</div>`;
    const btn = document.createElement("button");
    btn.className = "primary";
    btn.textContent = d.has_keymap ? "Flash (overwrites main.py)" : "Flash main.py";
    btn.onclick = () => flashTo(d.path);
    el.appendChild(btn);
    box.appendChild(el);
  }
}

async function flashTo(drive) {
  if (!state.view) return setStatus("Load a config first.", "err");
  try {
    const written = await invoke("flash", { drive, keymap: state.view.keymap });
    setStatus(`Flashed → ${written} (CircuitPython will auto-reload).`, "ok");
  } catch (e) {
    setStatus(String(e), "err");
  }
}

// ---------- Ferris / Corne suggestion ----------

$("btn-suggest").onclick = async () => {
  if (!state.view) return setStatus("Load a config first.", "err");
  try {
    const s = await invoke("suggest", {
      keymap: state.view.keymap,
      target: $("target-board").value,
    });
    renderSuggestion(s);
  } catch (e) {
    setStatus(String(e), "err");
  }
};

function renderSuggestion(s) {
  const out = $("suggestion");
  const gridCols = s.cols_per_half * 2 + 1;
  const caps = (rowsL, rowsR) => {
    let html = `<div class="grid" style="grid-template-columns:repeat(${gridCols}, 1fr)">`;
    for (let r = 0; r < rowsL.length; r++) {
      for (const k of rowsL[r]) html += `<div class="cap">${escapeHtml(k.label)}</div>`;
      html += `<div></div>`; // split gutter
      for (const k of rowsR[r]) html += `<div class="cap">${escapeHtml(k.label)}</div>`;
    }
    html += `</div>`;
    return html;
  };
  const thumbs = `<div class="grid" style="grid-template-columns:repeat(${s.thumbs_per_half * 2}, 1fr)">` +
    [...s.left_thumbs, ...s.right_thumbs].map((k) => `<div class="cap">${escapeHtml(k.label)}</div>`).join("") +
    `</div>`;

  let html = `<h3>${escapeHtml(s.target)} — ${s.total_keys} keys</h3>`;
  html += caps(s.left_rows, s.right_rows);
  html += `<div class="note">Thumbs kept:</div>` + thumbs;
  html += `<h3>Relocate these (${s.relocations.length})</h3>`;
  for (const r of s.relocations) {
    html += `<div class="reloc"><span class="k">${escapeHtml(r.label)}</span>
      <span class="tag">${escapeHtml(r.raw)}</span>
      <div class="why">${escapeHtml(r.reason)}</div>
      <div>${escapeHtml(r.suggestion).replace(/`([^`]+)`/g, "<code>$1</code>")}</div></div>`;
  }
  for (const n of s.notes) html += `<div class="note">💡 ${escapeHtml(n)}</div>`;
  out.innerHTML = html;
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[c]));
}
