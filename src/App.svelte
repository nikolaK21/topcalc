<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import Graph from "./Graph.svelte";
  import MathView from "./MathView.svelte";
  import {
    activate,
    getSettings,
    hideWindow,
    inTauri,
    openDesk,
    placeWindow,
    quitApp,
    resumeHotkey,
    runQuery,
    saveSettings,
    suspendHotkey,
  } from "./lib/api";
  import { deskExpression, wantsDesk } from "./lib/desk";
  import { abandonEmpty, closePair, erasePair, expandTemplate, mathGroups, spliceTemplate } from "./lib/mathpad";
  import {
    ANCHOR_OPTIONS,
    THEME_OPTIONS,
    clampPercent,
    cssVars,
    isLightBackground,
    presetColors,
    type ThemeColors,
  } from "./lib/theme";
  import type { QueryResponse, Row, SettingsResponse } from "./lib/types";

  let query = $state("");
  let mode = $state("empty");
  let rows = $state<Row[]>([]);
  let math = $state<QueryResponse["math"]>(null);
  let selected = $state(0);
  let shellArmed = $state(false);
  let runExplicit = $state(false);
  let output = $state<string | null>(null);
  let error = $state("");
  let graphOpen = $state(false);
  let graphDismissed = $state(false);
  let settingsOpen = $state(false);
  let keysOpen = $state(false);
  let keyGroup = $state(0);
  let recording = $state(false);
  let hotkeyDraft = $state("Alt+Space");
  let searchDraft = $state("https://duckduckgo.com/?q={query}");
  let themeDraft = $state("dark");
  let colorsDraft = $state<ThemeColors>(presetColors("dark"));
  let anchorDraft = $state("top");
  let xDraft = $state(50);
  let yDraft = $state(12);
  let savedLook = $state({
    theme: "dark",
    colors: presetColors("dark"),
    anchor: "top",
    xPercent: 50,
    yPercent: 12,
  });
  let lookReady = $state(!inTauri());
  let settingsError = $state("");
  let configPath = $state("");
  let inputEl = $state<HTMLInputElement | null>(null);
  let hiding = false;
  let shownAt = 0;
  let token = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let mathSig = "";
  let deskSent = "";
  let holdLauncher = false;

  let shellHint = $derived(
    shellArmed ? "Press Enter again to run the command above." : "",
  );

  function desiredHeight(): number {
    const hint = mode === "empty" && !settingsOpen && !keysOpen ? 34 : 0;
    const pods = math ? 156 + (math.note ? 18 : 0) + (math.resultLatex ? 72 : 0) : 0;
    const list = Math.min(rows.length, 8) * 46;
    const shell = output || shellHint ? 148 : 0;
    const settings = settingsBlock();
    const keys = keysOpen ? 124 : 0;
    const graph = graphOpen && math ? 360 : 0;
    const err = error ? 28 : 0;
    return Math.min(980, 64 + hint + pods + list + shell + settings + keys + graph + err);
  }

  const shownColors = $derived(
    settingsOpen ? (themeDraft === "custom" ? colorsDraft : presetColors(themeDraft)) : savedLook.colors,
  );
  const shownAnchor = $derived(settingsOpen ? anchorDraft : savedLook.anchor);
  const shownX = $derived(settingsOpen ? xDraft : savedLook.xPercent);
  const shownY = $derived(settingsOpen ? yDraft : savedLook.yPercent);
  const panelStyle = $derived(
    `${cssVars(shownColors)};color-scheme:${isLightBackground(shownColors.background) ? "light" : "dark"}`,
  );

  $effect(() => {
    const height = desiredHeight();
    const width = graphOpen ? 760 : 640;
    const anchor = lookReady ? shownAnchor : "";
    const x = clampPercent(shownX);
    const y = clampPercent(shownY);
    void fit(width, height, anchor, x, y);
  });

  $effect(() => {
    const text = query.trim();
    const parsed = math;
    if (!text.startsWith("=") || !parsed) return;
    const expression = deskExpression(text);
    if (!expression || expression === deskSent) return;
    deskSent = expression;
    holdLauncher = true;
    void openDesk(expression, false)
      .then(() => hide())
      .catch((err) => {
        deskSent = "";
        error = err instanceof Error ? err.message : String(err);
      })
      .finally(() => {
        setTimeout(() => {
          holdLauncher = false;
        }, 500);
      });
  });

  $effect(() => {
    const id = rows[selected]?.id;
    if (!id) return;
    document.getElementById(id)?.scrollIntoView({ block: "nearest" });
  });

  function settingsBlock(): number {
    if (!settingsOpen) return 0;
    let height = 400;
    if (themeDraft === "custom") height += 78;
    if (anchorDraft === "custom") height += 96;
    if (settingsError) height += 28;
    return height;
  }

  async function fit(width: number, height: number, anchor: string, xPercent: number, yPercent: number) {
    if (!inTauri()) return;
    try {
      const win = getCurrentWindow();
      if (win.label !== "main") return;
      await win.setSize(new LogicalSize(width, height));
      if (anchor) await placeWindow(anchor, xPercent, yPercent);
    } catch {
      /* The window API is only available inside the desktop shell. */
    }
  }

  function immediate(value: string): boolean {
    const text = value.trim();
    if (!text) return true;
    if (text.startsWith(">") || text.startsWith("$") || text.startsWith("=")) return true;
    if (/^[a-zA-Z]:/.test(text) || /^[~\\/]/.test(text)) return false;
    return (
      /[0-9+\-*/^=]/.test(text) ||
      /^(pi|e|x|sin|cos|tan|sqrt|ln|log|abs|arc|sinh|cosh|tanh|sec|csc|cot|floor|ceil|sum|prod|lim|fact)\b/i.test(text) ||
      /^y\s*=/i.test(text)
    );
  }

  function schedule(text: string) {
    const my = ++token;
    if (timer) clearTimeout(timer);
    const wait = immediate(text) ? 0 : 80;
    timer = setTimeout(() => {
      void loadQuery(my, text);
    }, wait);
  }

  function placeCaret(caret: number) {
    const apply = () => inputEl?.setSelectionRange(caret, caret);
    apply();
    queueMicrotask(apply);
    requestAnimationFrame(apply);
  }

  let autoAt: number[] = [];
  let typed = "";

  function revise(at: number, removed: number, added: number, extra?: number) {
    autoAt = autoAt.filter((index) => index < at || index >= at + removed);
    autoAt = autoAt.map((index) => (index >= at ? index + (added - removed) : index));
    if (extra != null) autoAt = [...autoAt, extra];
  }

  function applyTyped(text: string, caret: number) {
    const expanded = expandTemplate(text, caret);
    if (expanded) autoAt = [];
    query = expanded?.text ?? text;
    typed = query;
    placeCaret(expanded?.caret ?? caret);
    schedule(query);
  }

  function onQueryKey(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    const el = event.currentTarget as HTMLInputElement;
    const start = el.selectionStart ?? query.length;
    const end = el.selectionEnd ?? start;
    if (event.key === "Backspace" && start === end) {
      const erased = erasePair(query, start);
      if (!erased) return;
      event.preventDefault();
      revise(start - 1, 2, 0);
      applyTyped(erased.text, erased.caret);
      return;
    }
    if (event.key.length !== 1 || start !== end) return;
    const paired = closePair(query, start, event.key);
    if (!paired) return;
    event.preventDefault();
    revise(start, end - start, paired.text.length - (query.length - (end - start)), paired.autoCloser);
    applyTyped(paired.text, paired.caret);
  }

  function onQueryInput(event: Event) {
    const el = event.currentTarget as HTMLInputElement;
    let text = el.value;
    const caret = el.selectionStart ?? text.length;
    if (text !== typed) {
      const delta = text.length - typed.length;
      const added = Math.max(delta, 0);
      const removed = Math.max(-delta, 0);
      revise(delta >= 0 ? caret - added : caret, removed, added);
      typed = text;
    }
    const expanded = expandTemplate(text, caret);
    if (expanded) {
      autoAt = [];
      text = expanded.text;
      query = text;
      typed = text;
      placeCaret(expanded.caret);
    }
    schedule(text);
  }

  function onQueryBlur() {
    const next = abandonEmpty(query, autoAt, inputEl?.selectionStart ?? query.length);
    autoAt = [];
    if (next.text === query) return;
    query = next.text;
    typed = next.text;
    placeCaret(next.caret);
    schedule(query);
  }

  function insertMath(template: string) {
    const start = inputEl?.selectionStart ?? query.length;
    const end = inputEl?.selectionEnd ?? query.length;
    const next = spliceTemplate(query, start, end, template);
    query = next.text;
    inputEl?.focus();
    placeCaret(next.caret);
    schedule(next.text);
  }

  async function loadQuery(my: number, text: string) {
    try {
      const response = await runQuery(text, localStorage.getItem("topcalc-number-mode") || "decimal");
      if (my !== token) return;
      apply(response);
    } catch (err) {
      if (my !== token) return;
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function apply(response: QueryResponse) {
    const sig = response.math?.series.map((item) => item.expression).join(";") ?? "";
    if (sig !== mathSig) {
      mathSig = sig;
      graphDismissed = false;
    }
    mode = response.mode;
    rows = response.rows;
    math = response.math;
    selected = 0;
    shellArmed = false;
    runExplicit = false;
    output = null;
    error = "";
    if (!response.math) {
      graphOpen = false;
    } else if (response.math.autoGraph && !graphDismissed) {
      graphOpen = true;
    } else if (!response.math.autoGraph) {
      graphOpen = false;
    }
  }

  async function hide() {
    if (hiding) return;
    hiding = true;
    query = "";
    mode = "empty";
    rows = [];
    math = null;
    graphOpen = false;
    graphDismissed = false;
    output = null;
    error = "";
    settingsOpen = false;
    keysOpen = false;
    recording = false;
    shellArmed = false;
    mathSig = "";
    deskSent = "";
    try {
      if (inTauri()) await hideWindow();
    } catch {
      /* ignore */
    } finally {
      hiding = false;
    }
  }

  async function activateIndex(index: number) {
    const row = rows[index];
    if (!row || row.kind === "Hint") return;
    selected = index;
    if (row.kind === "Run" && row.action.type === "runShell") {
      if (!shellArmed && !runExplicit) {
        shellArmed = true;
        return;
      }
    }
    await perform(row);
  }

  async function perform(row: Row) {
    if (row.action.type === "copy") {
      try {
        await navigator.clipboard.writeText(row.action.text);
        output = "Copied";
      } catch {
        output = row.action.text;
      }
      return;
    }
    if (row.action.type === "showGraph") {
      graphOpen = true;
      graphDismissed = false;
      return;
    }
    if (row.action.type === "none") return;
    output = row.action.type === "runShell" ? "Running…" : output;
    try {
      const result = await activate(row.action);
      if (result.output) output = result.output;
      if (result.hide) await hide();
    } catch (err) {
      output = err instanceof Error ? err.message : String(err);
    }
  }

  function onKey(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    const inSettings = target?.dataset.field === "setting";
    if (recording) {
      event.preventDefault();
      captureHotkey(event);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      if (keysOpen) {
        keysOpen = false;
        return;
      }
      if (settingsOpen) {
        settingsOpen = false;
        void resumeHotkey().catch(() => {});
        return;
      }
      void hide();
      return;
    }
    if (inSettings) return;
    if ((event.ctrlKey || event.metaKey) && event.key === ",") {
      event.preventDefault();
      void toggleSettings();
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "g" && !event.altKey) {
      event.preventDefault();
      if (math && math.series.length > 0) {
        graphOpen = !graphOpen;
        graphDismissed = !graphOpen;
      }
      return;
    }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      if (rows.length === 0) return;
      event.preventDefault();
      const delta = event.key === "ArrowDown" ? 1 : -1;
      selected = (selected + delta + rows.length) % rows.length;
      runExplicit = rows[selected]?.kind === "Run";
      return;
    }
    if (event.key === "Enter" && !event.repeat) {
      event.preventDefault();
      if (wantsDesk(query)) {
        const expression = deskExpression(query);
        if (expression) {
          deskSent = "";
          holdLauncher = true;
          void openDesk(expression, true)
            .then(() => hide())
            .catch((err) => {
              error = err instanceof Error ? err.message : String(err);
            })
            .finally(() => {
              setTimeout(() => {
                holdLauncher = false;
              }, 500);
            });
          return;
        }
      }
      if (rows.length === 0) return;
      void activateIndex(selected);
    }
  }

  async function toggleSettings() {
    settingsOpen = !settingsOpen;
    if (settingsOpen) {
      keysOpen = false;
      await loadSettings();
    }
  }

  function applySettings(settings: SettingsResponse) {
    const colors = settings.colors ?? presetColors(settings.theme || "dark");
    savedLook = {
      theme: settings.theme || "dark",
      colors,
      anchor: settings.anchor || "top",
      xPercent: clampPercent(settings.xPercent ?? 50),
      yPercent: clampPercent(settings.yPercent ?? 12),
    };
    themeDraft = savedLook.theme;
    colorsDraft = { ...colors };
    anchorDraft = savedLook.anchor;
    xDraft = savedLook.xPercent;
    yDraft = savedLook.yPercent;
    hotkeyDraft = settings.hotkey;
    searchDraft = settings.searchUrl;
    configPath = settings.configPath;
    settingsError = settings.hotkeyError ?? "";
    lookReady = true;
  }

  async function loadSettings() {
    if (!inTauri()) return;
    try {
      applySettings(await getSettings());
    } catch (err) {
      lookReady = true;
      settingsError = err instanceof Error ? err.message : String(err);
    }
  }

  async function startRecord() {
    recording = true;
    settingsError = "Press a shortcut. It needs a modifier.";
    try {
      await suspendHotkey();
    } catch {
      /* The previous shortcut may already be unregistered. */
    }
  }

  function captureHotkey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      recording = false;
      settingsError = "";
      void resumeHotkey().catch(() => {});
      return;
    }
    if (["Control", "Alt", "Shift", "Meta"].includes(event.key)) return;
    const parts: string[] = [];
    if (event.ctrlKey) parts.push("Ctrl");
    if (event.altKey) parts.push("Alt");
    if (event.shiftKey) parts.push("Shift");
    if (event.metaKey) parts.push("Super");
    if (parts.length === 0) {
      settingsError = "Add Ctrl, Alt, or Shift.";
      return;
    }
    parts.push(event.code);
    hotkeyDraft = parts.join("+");
    recording = false;
    settingsError = "";
  }

  async function save() {
    try {
      const saved = await saveSettings({
        hotkey: hotkeyDraft.trim(),
        searchUrl: searchDraft.trim(),
        theme: themeDraft,
        colors: themeDraft === "custom" ? colorsDraft : presetColors(themeDraft),
        anchor: anchorDraft,
        xPercent: clampPercent(xDraft),
        yPercent: clampPercent(yDraft),
      });
      applySettings(saved);
      recording = false;
      if (!saved.hotkeyError) settingsOpen = false;
    } catch (err) {
      settingsError = err instanceof Error ? err.message : String(err);
    }
  }

  function chooseTheme(next: string) {
    if (next === "custom" && themeDraft !== "custom") colorsDraft = presetColors(themeDraft);
    themeDraft = next;
  }

  function subtitleFor(row: Row): string {
    if (shellArmed && row.kind === "Run" && row.action.type === "runShell") {
      return `Press Enter to run: ${row.action.command}`;
    }
    return row.subtitle;
  }

  function glyph(kind: string): string {
    switch (kind) {
      case "App":
        return "A";
      case "File":
        return "F";
      case "Folder":
        return "D";
      case "Calculate":
        return "=";
      case "Solution":
        return "x";
      case "Graph":
        return "ƒ";
      case "Run":
        return ">";
      case "Terminal":
        return "$_";
      case "Web":
        return "W";
      case "Sleep":
        return "Zz";
      default:
        return "·";
    }
  }

  onMount(() => {
    inputEl?.focus();
    if (!inTauri()) return;
    let unlisten: (() => void) | undefined;
    let unfocus: (() => void) | undefined;
    void listen("shown", () => {
      shownAt = performance.now();
      inputEl?.focus();
      inputEl?.select();
    }).then((stop) => {
      unlisten = stop;
    });
    const launcher = getCurrentWindow();
    if (launcher.label !== "main") return () => {};
    void launcher
      .onFocusChanged(({ payload: focused }) => {
        if (!focused && !holdLauncher && performance.now() - shownAt > 250) void hide();
      })
      .then((stop) => {
        unfocus = stop;
      });
    void loadSettings();
    return () => {
      unlisten?.();
      unfocus?.();
    };
  });
</script>

<svelte:window onkeydown={onKey} />

<div class="panel" style={panelStyle}>
  <div class="input-row">
    <input
      bind:this={inputEl}
      bind:value={query}
      onkeydown={onQueryKey}
      oninput={onQueryInput}
      onblur={onQueryBlur}
      placeholder="TopCalc"
      spellcheck="false"
      autocomplete="off"
      autocapitalize="off"
      aria-label="TopCalc query"
      aria-controls="results"
      aria-activedescendant={rows[selected]?.id}
    />
    <button
      type="button"
      class="gear"
      class:on={keysOpen}
      title="Math keyboard"
      aria-pressed={keysOpen}
      onclick={() => {
        keysOpen = !keysOpen;
        if (keysOpen) settingsOpen = false;
      }}>∑</button
    >
    <button type="button" class="gear" title="Settings (Ctrl+,)" onclick={() => void toggleSettings()}
      >⚙</button
    >
  </div>

  {#if keysOpen}
    <div class="keys">
      <div class="key-tabs" role="tablist">
        {#each mathGroups as group, index (group.name)}
          <button
            type="button"
            class="key-tab"
            class:on={index === keyGroup}
            role="tab"
            aria-selected={index === keyGroup}
            onmousedown={(event) => event.preventDefault()}
            onclick={() => (keyGroup = index)}>{group.name}</button
          >
        {/each}
      </div>
      <div class="key-buttons">
        {#each mathGroups[keyGroup].keys as item (item.title + item.insert)}
          <button
            type="button"
            class="key"
            title={item.title}
            onmousedown={(event) => event.preventDefault()}
            onclick={() => insertMath(item.insert)}>{item.label}</button
          >
        {/each}
      </div>
    </div>
  {/if}

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if mode === "empty" && !settingsOpen}
    <p class="hint">sinx · arcsin · sum · lim · sleep · ∑ keyboard · d/dx · &gt; shell</p>
  {/if}

  {#if math}
    <MathView
      interpretation={math.interpretationLatex}
      resultLabel={math.resultLabel}
      resultLatex={math.resultLatex}
      note={math.note}
    />
  {/if}

  {#if rows.length > 0}
    <div id="results" class="list" role="listbox">
      {#each rows as row, index (row.id)}
        <div
          id={row.id}
          class="row"
          class:active={index === selected}
          role="option"
          tabindex="-1"
          aria-selected={index === selected}
          onmousedown={(event) => event.preventDefault()}
          onmouseenter={() => (selected = index)}
          onclick={() => void activateIndex(index)}
          onkeydown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              void activateIndex(index);
            }
          }}
        >
          <span class="badge" data-kind={row.kind}>{glyph(row.kind)}</span>
          <span class="main">
            <span class="title">{row.title}</span>
            {#if subtitleFor(row)}
              <span class="sub" class:arm={shellArmed && row.kind === "Run"}>{subtitleFor(row)}</span>
            {/if}
          </span>
          <span class="kind">{row.kind}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if shellHint && !output}
    <p class="hint">{shellHint}</p>
  {/if}
  {#if output}
    <pre class="out">{output}</pre>
  {/if}

  {#if settingsOpen}
    <form
      class="settings"
      onsubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      <label>
        Hotkey
        <input data-field="setting" bind:value={hotkeyDraft} spellcheck="false" />
      </label>
      <div class="setting-actions">
        <button type="button" onclick={() => void startRecord()}>
          {recording ? "Listening…" : "Record"}
        </button>
        <span>Example: Alt+Space or Ctrl+Alt+Space</span>
      </div>
      <label>
        Search URL
        <input data-field="setting" bind:value={searchDraft} spellcheck="false" />
      </label>
      <p class="fine">Use {"{query}"} or %s. Saved locally{configPath ? ` at ${configPath}` : ""}.</p>
      <label>
        Color scheme
        <select
          data-field="setting"
          value={themeDraft}
          onchange={(event) => {
            const target = event.currentTarget;
            if (target instanceof HTMLSelectElement) chooseTheme(target.value);
          }}
        >
          {#each THEME_OPTIONS as option (option.id)}
            <option value={option.id}>{option.label}</option>
          {/each}
        </select>
      </label>
      {#if themeDraft === "custom"}
        <div class="swatches">
          <label>
            Background
            <input data-field="setting" type="color" bind:value={colorsDraft.background} />
          </label>
          <label>
            Text
            <input data-field="setting" type="color" bind:value={colorsDraft.text} />
          </label>
          <label>
            Muted
            <input data-field="setting" type="color" bind:value={colorsDraft.muted} />
          </label>
          <label>
            Accent
            <input data-field="setting" type="color" bind:value={colorsDraft.accent} />
          </label>
          <label>
            Surface
            <input data-field="setting" type="color" bind:value={colorsDraft.surface} />
          </label>
        </div>
      {/if}
      <label>
        Position
        <select data-field="setting" bind:value={anchorDraft}>
          {#each ANCHOR_OPTIONS as option (option.id)}
            <option value={option.id}>{option.label}</option>
          {/each}
        </select>
      </label>
      {#if anchorDraft === "custom"}
        <div class="percents">
          <label>
            Horizontal
            <input data-field="setting" type="number" min="0" max="100" bind:value={xDraft} />
          </label>
          <label>
            Vertical
            <input data-field="setting" type="number" min="0" max="100" bind:value={yDraft} />
          </label>
        </div>
        <p class="fine">0 is the left or the top. 100 is the right or the bottom. The bar is centered on that point.</p>
      {/if}
      <p class="fine">The bar moves as you change these. Save keeps them. Esc puts them back.</p>
      {#if settingsError}
        <p class="error">{settingsError}</p>
      {/if}
      <div class="setting-actions">
        <button type="submit">Save</button>
        <button type="button" class="danger" onclick={() => void quitApp()}>Quit</button>
      </div>
    </form>
  {/if}

  {#if graphOpen && math}
    <Graph series={math.series} markers={math.markers} frame={math.frame} ink={shownColors} />
  {/if}
</div>

<style>
  .panel {
    height: 100%;
    background: var(--background, #1c1c20);
    color: var(--text, #f3f4f6);
    border: 1px solid color-mix(in srgb, var(--text, #f3f4f6) 12%, transparent);
    border-radius: 12px;
    overflow: hidden;
    animation: in 70ms ease-out;
  }
  @keyframes in {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  .input-row {
    display: flex;
    align-items: center;
    padding: 4px 8px 4px 4px;
  }
  input {
    width: 100%;
    background: transparent;
    border: 0;
    outline: 0;
    padding: 12px 14px;
    font-size: 18px;
    caret-color: var(--accent, #6ea8fe);
    color: var(--text, #f3f4f6);
  }
  .gear {
    border: 0;
    background: transparent;
    color: var(--muted, #9aa3b2);
    width: 32px;
    height: 32px;
    border-radius: 8px;
    cursor: pointer;
  }
  .gear:hover,
  .gear.on {
    background: color-mix(in srgb, var(--text, white) 8%, transparent);
    color: var(--text, white);
  }
  .keys {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0 12px 12px;
  }
  .key-tabs {
    display: flex;
    gap: 2px;
    overflow-x: auto;
  }
  .key-tab {
    border: 0;
    background: transparent;
    color: var(--muted, #8b93a7);
    border-radius: 7px;
    height: 26px;
    padding: 0 8px;
    cursor: pointer;
    font-size: 12px;
    white-space: nowrap;
  }
  .key-tab.on,
  .key-tab:hover {
    background: color-mix(in srgb, var(--text, white) 10%, transparent);
    color: var(--text, white);
  }
  .key-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    min-height: 60px;
    align-content: flex-start;
  }
  .key {
    border: 0;
    background: color-mix(in srgb, var(--text, white) 8%, transparent);
    color: var(--text, #f4f6fb);
    border-radius: 7px;
    min-width: 36px;
    height: 28px;
    padding: 0 8px;
    cursor: pointer;
  }
  .key:hover {
    background: color-mix(in srgb, var(--accent, #6ea8fe) 28%, transparent);
  }
  .hint,
  .fine,
  .error {
    margin: 0;
    padding: 0 16px 10px;
    color: var(--muted, #8b93a7);
    font-size: 12px;
  }
  .error {
    color: #ff8b9a;
  }
  .list {
    max-height: 368px;
    overflow: auto;
    padding-bottom: 6px;
  }
  .row {
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    margin: 0 6px;
    padding: 7px 10px;
    border-radius: 8px;
    cursor: pointer;
  }
  .row.active {
    background: color-mix(in srgb, var(--accent, #6ea8fe) 16%, transparent);
  }
  .badge {
    width: 22px;
    text-align: center;
    font-size: 12px;
    font-weight: 650;
    color: var(--muted, #9aa3b2);
  }
  .badge[data-kind="App"] { color: #3dd68c; }
  .badge[data-kind="File"],
  .badge[data-kind="Folder"] { color: #ffb020; }
  .badge[data-kind="Calculate"] { color: var(--accent, #6ea8fe); }
  .badge[data-kind="Graph"] { color: #c084fc; }
  .badge[data-kind="Run"] { color: #ff6b81; }
  .badge[data-kind="Terminal"] { color: #22d3ee; }
  .badge[data-kind="Sleep"] { color: #93c5fd; }
  .main {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .title,
  .sub {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    color: var(--muted, #9aa3b2);
    font-size: 12px;
  }
  .sub.arm {
    color: #ffb020;
  }
  .kind {
    color: var(--muted, #8b93a7);
    font-size: 11px;
  }
  .out {
    margin: 0 10px 10px;
    padding: 8px 10px;
    max-height: 140px;
    overflow: auto;
    background: var(--surface, #0e1014);
    color: var(--text, #f3f4f6);
    border-radius: 8px;
    font-family: ui-monospace, Consolas, monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
  }
  .settings {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0 12px 12px;
  }
  .settings label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--muted, #b7becc);
  }
  .settings input,
  .settings select {
    font-size: 14px;
    padding: 8px 10px;
    border-radius: 8px;
    border: 1px solid color-mix(in srgb, var(--text, white) 12%, transparent);
    background: var(--surface, #12141a);
    color: var(--text, #f3f4f6);
  }
  .swatches,
  .percents {
    display: grid;
    gap: 8px;
  }
  .swatches {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
  .percents {
    grid-template-columns: 1fr 1fr;
  }
  .settings input[type="color"] {
    padding: 2px;
    height: 32px;
    width: 100%;
  }
  .setting-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--muted, #8b93a7);
    font-size: 12px;
  }
  .setting-actions button,
  .settings button {
    border: 0;
    background: color-mix(in srgb, var(--accent, #6ea8fe) 18%, transparent);
    color: var(--text, #f3f4f6);
    border-radius: 8px;
    padding: 6px 10px;
    cursor: pointer;
  }
  .danger {
    background: color-mix(in srgb, #ff6b81 22%, transparent) !important;
  }
</style>
