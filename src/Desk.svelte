<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Graph from "./Graph.svelte";
  import MathField from "./MathField.svelte";
  import { evalGraph, inTauri, runQuery, takeDeskLines } from "./lib/api";
  import {
    applyFunctions,
    bindingsFrom,
    boundaryDashed,
    compileLine,
    fanJobs,
    linearFit,
    quadraticFit,
    sliderSuggestions,
    substitute,
    type FnDef,
    type Job,
    type SliderDef,
  } from "./lib/desmos";
  import { formatQuantity, type NumberMode } from "./lib/numbers";
  import { presetColors } from "./lib/theme";
  import type { QueryResponse, SeriesMeta } from "./lib/types";

  const PALETTE = ["#6ea8fe", "#3dd68c", "#ffb020", "#ff6b81", "#c084fc", "#22d3ee"];
  const QUEUE = "topcalc-desk-queue";
  const MODE_KEY = "topcalc-number-mode";
  const COORD_KEY = "topcalc-coord-mode";
  const ink = presetColors("dark");
  type CoordMode = "cartesian" | "polar" | "radial";

  type Fit = "none" | "line" | "quad";
  type ExprLine = {
    id: number;
    kind: "expr" | "table" | "note";
    text: string;
    hidden: boolean;
    color: string;
    style: "line" | "dashed" | "points";
    sliderMin: number;
    sliderMax: number;
    sliderStep: number;
    sliderResolved: number | null;
    tableX: string[];
    tableY: string[];
    fit: Fit;
    response: QueryResponse | null;
    error: string;
  };

  let nextId = 1;
  let lines = $state<ExprLine[]>([blankLine()]);
  let draftId = $state<number | null>(1);
  let degrees = $state(false);
  let numberMode = $state<NumberMode>(storedMode());
  let coordMode = $state<CoordMode>(storedCoord());
  let picking = $state(false);
  let menuFor = $state<number | null>(null);
  let toolsOpen = $state(false);
  const tokens = new Map<number, number>();

  function storedMode(): NumberMode {
    const saved = localStorage.getItem(MODE_KEY);
    if (saved === "integer" || saved === "irrational" || saved === "complex" || saved === "decimal") return saved;
    return "decimal";
  }

  function storedCoord(): CoordMode {
    const saved = localStorage.getItem(COORD_KEY);
    if (saved === "polar" || saved === "radial" || saved === "cartesian") return saved;
    return "cartesian";
  }

  function chooseMode(mode: NumberMode) {
    numberMode = mode;
    localStorage.setItem(MODE_KEY, mode);
  }

  function chooseCoord(mode: CoordMode) {
    coordMode = mode;
    localStorage.setItem(COORD_KEY, mode);
  }

  function blankLine(): ExprLine {
    const id = nextId++;
    return {
      id,
      kind: "expr",
      text: "",
      hidden: false,
      color: PALETTE[(id - 1) % PALETTE.length],
      style: "line",
      sliderMin: -10,
      sliderMax: 10,
      sliderStep: 0.1,
      sliderResolved: null,
      tableX: ["0", "1"],
      tableY: ["0", "1"],
      fit: "none",
      response: null,
      error: "",
    };
  }

  function functionsOf(source: ExprLine[]): FnDef[] {
    return source.flatMap((line) => {
      if (line.kind !== "expr") return [];
      const compiled = compileLine(line.text, []);
      return compiled.type === "functionDef" ? [compiled.fn] : [];
    });
  }

  function slidersOf(source: ExprLine[]): SliderDef[] {
    const fns = functionsOf(source);
    return source.flatMap((line) => {
      if (line.kind !== "expr") return [];
      const compiled = compileLine(line.text, fns);
      if (compiled.type !== "slider") return [];
      if (compiled.slider.pending) {
        return line.sliderResolved == null ? [] : [{ ...compiled.slider, value: line.sliderResolved, pending: null }];
      }
      return [compiled.slider];
    });
  }

  let series = $derived.by(() => {
    const fns = functionsOf(lines);
    const sliders = slidersOf(lines);
    const bindings = bindingsFrom(sliders);
    const drawn: SeriesMeta[] = [];
    for (const line of lines) {
      if (line.hidden) continue;
      if (line.kind === "table") {
        const xs: number[] = [];
        const ys: number[] = [];
        line.tableX.forEach((raw, index) => {
          const x = Number(raw);
          const y = Number(line.tableY[index]);
          if (Number.isFinite(x) && Number.isFinite(y)) {
            xs.push(x);
            ys.push(y);
            drawn.push({
              label: `(${x}, ${y})`,
              expression: `${x}||${y}`,
              color: line.color,
              plot: "point",
              style: "points",
            });
          }
        });
        if (line.fit === "line") {
          const fit = linearFit(xs, ys);
          if (fit) {
            drawn.push({
              label: "linear fit",
              expression: `${fit.slope}*x+(${fit.intercept})`,
              color: line.color,
              plot: "function",
              style: "line",
            });
          }
        }
        if (line.fit === "quad") {
          const fit = quadraticFit(xs, ys);
          if (fit) {
            drawn.push({
              label: "quadratic fit",
              expression: `${fit.a}*x^2+(${fit.b})*x+(${fit.c})`,
              color: line.color,
              plot: "function",
              style: "line",
            });
          }
        }
        continue;
      }
      if (line.kind !== "expr") continue;
      const compiled = compileLine(line.text, fns);
      if (compiled.type !== "plot" && compiled.type !== "functionDef") continue;
      const jobs = fanJobs(compiled.jobs, sliders).map((item) => ({
        ...item,
        expression: item.plot === "vertical" ? substitute(item.expression, bindings) : item.expression,
      }));
      for (const item of jobs) {
        drawn.push({
          label: item.label,
          expression: item.expression,
          color: line.color,
          plot: item.plot,
          domain: item.domain,
          style: boundaryDashed(line.text) ? "dashed" : "line",
        });
      }
    }
    return drawn;
  });

  let markers = $derived(
    lines.filter((line) => !line.hidden).flatMap((line) => line.response?.math?.markers ?? []),
  );

  let graphBindings = $derived(bindingsFrom(slidersOf(lines)));

  let textSig = $derived(
    lines
      .map((line) => `${line.id}\n${line.kind}\n${line.text}\n${line.tableX.join(",")}\n${line.tableY.join(",")}\n${line.fit}\n${degrees ? 1 : 0}\n${numberMode}`)
      .join("\n---\n"),
  );

  $effect(() => {
    const snapshot = textSig.split("\n---\n").filter(Boolean);
    const handle = setTimeout(() => {
      for (const chunk of snapshot) {
        const id = Number(chunk.slice(0, chunk.indexOf("\n")));
        const line = lines.find((item) => item.id === id);
        if (line) void evaluate(line);
      }
    }, 60);
    return () => clearTimeout(handle);
  });

  function setLine(id: number, patch: Partial<ExprLine>) {
    lines = lines.map((line) => (line.id === id ? { ...line, ...patch } : line));
  }

  async function evaluate(line: ExprLine) {
    const token = (tokens.get(line.id) ?? 0) + 1;
    tokens.set(line.id, token);
    if (line.kind !== "expr") {
      if (tokens.get(line.id) === token) setLine(line.id, { response: null, error: "" });
      return;
    }
    const fns = functionsOf(lines.filter((item) => item.id !== line.id));
    const compiled = compileLine(line.text, fns);
    if (compiled.type === "empty" || compiled.type === "note") {
      if (tokens.get(line.id) === token) setLine(line.id, { response: null, error: "", sliderResolved: null });
      return;
    }
    if (compiled.type === "error") {
      if (tokens.get(line.id) === token) setLine(line.id, { response: null, error: compiled.message });
      return;
    }
    if (compiled.type === "slider" && compiled.slider.pending) {
      try {
        const others = bindingsFrom(slidersOf(lines.filter((item) => item.id !== line.id)));
        const value = Number(await evalGraph(compiled.slider.pending, others, degrees));
        if (tokens.get(line.id) !== token) return;
        setLine(line.id, { sliderResolved: Number.isFinite(value) ? value : null, response: null, error: "" });
      } catch (err) {
        if (tokens.get(line.id) !== token) return;
        setLine(line.id, {
          sliderResolved: null,
          response: null,
          error: err instanceof Error ? err.message : String(err),
        });
      }
      return;
    }
    if (compiled.type === "slider") {
      const value = compiled.slider.value;
      const patch: Partial<ExprLine> = { sliderResolved: null, response: null, error: "" };
      if (value != null && (value < line.sliderMin || value > line.sliderMax)) {
        patch.sliderMin = Math.min(line.sliderMin, value - 10);
        patch.sliderMax = Math.max(line.sliderMax, value + 10);
      }
      if (tokens.get(line.id) === token) setLine(line.id, patch);
      return;
    }
    if (compiled.type === "value") {
      try {
        const bindings = bindingsFrom(slidersOf(lines));
        const shown = await evalGraph(applyFunctions(compiled.expression, fns), bindings, degrees, numberMode);
        if (tokens.get(line.id) !== token) return;
        setLine(line.id, {
          response: {
            mode: "math",
            rows: [{ id: "", kind: "Calculate", title: shown, subtitle: line.text, action: { type: "copy", text: shown } }],
            math: {
              largeValue: shown,
              autoGraph: false,
              interpretationLatex: "",
              resultLabel: "Result",
              resultLatex: null,
              note: null,
              series: [],
              markers: [],
              frame: null,
            },
          },
          error: "",
        });
      } catch (err) {
        if (tokens.get(line.id) !== token) return;
        setLine(line.id, {
          response: null,
          error: err instanceof Error ? err.message : String(err),
        });
      }
      return;
    }
    const query = compiled.type === "plot" ? compiled.query : null;
    const special = compiled.jobs.some((item) => item.plot !== "function" && item.plot !== "vertical");
    if (!query) {
      if (tokens.get(line.id) === token) setLine(line.id, { response: null, error: "" });
      return;
    }
    try {
      const bindings = bindingsFrom(slidersOf(lines));
      const response = await runQuery(substitute(applyFunctions(query, fns), bindings));
      if (tokens.get(line.id) !== token) return;
      setLine(line.id, { response, error: "" });
    } catch (err) {
      if (tokens.get(line.id) !== token) return;
      setLine(line.id, {
        response: null,
        error: special ? "" : err instanceof Error ? err.message : String(err),
      });
    }
  }

  async function focusLine(id: number) {
    await tick();
    document.querySelector<HTMLElement>(`[data-line="${id}"]`)?.focus();
  }

  function applyIncoming(expression: string, commit: boolean) {
    const text = expression.trim();
    if (!text) return;
    if (draftId == null || !lines.some((line) => line.id === draftId)) {
      const empty = lines.find((line) => line.kind === "expr" && line.text.trim() === "");
      if (empty) draftId = empty.id;
      else {
        const created = blankLine();
        lines = [...lines, created];
        draftId = created.id;
      }
    }
    const id = draftId;
    setLine(id, { text, kind: "expr" });
    if (commit) {
      draftId = null;
      const created = blankLine();
      lines = [...lines, created];
      focusLine(created.id);
    } else {
      focusLine(id);
    }
  }

  function insertAfter(index: number, created: ExprLine) {
    const next = lines.slice();
    next.splice(index + 1, 0, created);
    lines = next;
    draftId = null;
    if (created.kind === "expr") focusLine(created.id);
  }

  function onLineKey(event: KeyboardEvent, index: number) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      const current = lines[index];
      if (!current || (current.kind === "expr" && current.text.trim() === "")) return;
      insertAfter(index, blankLine());
      return;
    }
    if (event.key === "Backspace" && lines[index]?.kind === "expr" && lines[index]?.text === "" && lines.length > 1) {
      event.preventDefault();
      const removed = lines[index];
      const neighbor = lines[index - 1] ?? lines[index + 1];
      lines = lines.filter((line) => line.id !== removed.id);
      if (draftId === removed.id) draftId = null;
      if (neighbor) focusLine(neighbor.id);
    }
  }

  function sliderOf(line: ExprLine): SliderDef | null {
    if (line.kind !== "expr") return null;
    const compiled = compileLine(line.text, functionsOf(lines));
    if (compiled.type !== "slider" || compiled.slider.values) return null;
    const value = compiled.slider.value ?? line.sliderResolved;
    if (value == null) return null;
    return { ...compiled.slider, value };
  }

  function writeSlider(line: ExprLine, value: number) {
    const slider = sliderOf(line);
    if (!slider) return;
    const shown = Math.round(value * 1000) / 1000;
    setLine(line.id, { text: `${slider.name} = ${shown}` });
  }

  function resultText(line: ExprLine): string {
    const slider = sliderOf(line);
    if (slider?.values) return slider.values.join(", ");
    if (line.kind === "table" && line.fit === "line") {
      const xs = line.tableX.map(Number);
      const ys = line.tableY.map(Number);
      const pairs = xs.map((x, index) => [x, ys[index]] as const).filter((pair) => pair.every(Number.isFinite));
      const fit = linearFit(pairs.map((pair) => pair[0]), pairs.map((pair) => pair[1]));
      return fit ? `y = ${trim(fit.slope)}x + ${trim(fit.intercept)}` : "";
    }
    if (line.kind === "table" && line.fit === "quad") {
      const xs = line.tableX.map(Number);
      const ys = line.tableY.map(Number);
      const pairs = xs.map((x, index) => [x, ys[index]] as const).filter((pair) => pair.every(Number.isFinite));
      const fit = quadraticFit(pairs.map((pair) => pair[0]), pairs.map((pair) => pair[1]));
      return fit ? `y = ${trim(fit.a)}x^2 + ${trim(fit.b)}x + ${trim(fit.c)}` : "";
    }
    const math = line.response?.math;
    if (!math) return "";
    if (math.largeValue) return math.largeValue;
    const row = line.response?.rows.find((item) => item.kind === "Solution" || item.kind === "Calculate");
    if (row) return row.title;
    if (math.note) return math.note;
    return "";
  }

  function trim(value: number): string {
    const rounded = Math.round(value * 1e6) / 1e6;
    return String(rounded);
  }

  function addSlider(name: string, beforeId: number) {
    const created = blankLine();
    created.text = `${name} = 1`;
    const index = lines.findIndex((line) => line.id === beforeId);
    const next = lines.slice();
    next.splice(index < 0 ? next.length : index, 0, created);
    lines = next;
  }

  function suggestionsFor(line: ExprLine): string[] {
    if (line.kind !== "expr" || !line.text.trim()) return [];
    const compiled = compileLine(line.text, functionsOf(lines));
    if (compiled.type !== "plot" && compiled.type !== "functionDef" && compiled.type !== "value") return [];
    const defined = [
      ...slidersOf(lines).map((slider) => slider.name),
      ...functionsOf(lines).map((fn) => fn.name),
    ];
    return sliderSuggestions(line.text, defined);
  }

  function addTable() {
    const created = blankLine();
    created.kind = "table";
    lines = [...lines, created];
  }

  function removeLine(id: number) {
    if (lines.length === 1) {
      setLine(id, { text: "", kind: "expr", hidden: false, error: "", response: null });
      return;
    }
    lines = lines.filter((line) => line.id !== id);
    if (draftId === id) draftId = null;
    if (menuFor === id) menuFor = null;
  }

  function addNote() {
    const created = blankLine();
    created.kind = "note";
    created.text = "\"Note\"";
    lines = [...lines, created];
  }

  function pointText(x: number, y: number): string {
    return `(${formatQuantity(x, numberMode)}, ${formatQuantity(y, numberMode)})`;
  }

  function dropPoint(x: number, y: number) {
    const created = blankLine();
    created.text = pointText(x, y);
    lines = [...lines, created];
    picking = false;
  }

  function placePoint(x: number, y: number) {
    const created = blankLine();
    created.text = pointText(x, y);
    lines = [...lines, created];
  }

  function drainLocal() {
    const raw = localStorage.getItem(QUEUE);
    if (!raw) return;
    localStorage.removeItem(QUEUE);
    try {
      const queued = JSON.parse(raw) as { expression: string; commit: boolean }[];
      for (const item of queued) applyIncoming(item.expression, item.commit);
    } catch {
      /* Ignore a damaged queue. */
    }
  }

  async function pullRemote() {
    if (!inTauri()) {
      drainLocal();
      return;
    }
    const queued = await takeDeskLines();
    for (const item of queued) applyIncoming(item.expression, item.commit);
  }

  onMount(() => {
    document.title = "TopCalc Graph";
    void pullRemote();
    window.addEventListener("storage", drainLocal);
    let stop = () => {};
    if (inTauri()) {
      void listen("desk-refresh", () => {
        void pullRemote();
      }).then((unlisten) => {
        stop = unlisten;
        void pullRemote();
      });
    }
    return () => {
      stop();
      window.removeEventListener("storage", drainLocal);
    };
  });

  function jobsPreview(line: ExprLine): Job[] {
    const compiled = compileLine(line.text, functionsOf(lines));
    return compiled.type === "plot" || compiled.type === "functionDef" ? compiled.jobs : [];
  }
</script>

<div class="desk">
  <aside>
    <header>
      <strong>Algebra</strong>
      <button type="button" class="plus" aria-expanded={toolsOpen} aria-label="Add" onclick={() => (toolsOpen = !toolsOpen)}>+</button>
      {#if toolsOpen}
        <div class="plus-menu">
          <button type="button" onclick={() => { insertAfter(lines.length - 1, blankLine()); toolsOpen = false; }}>Expression</button>
          <button type="button" onclick={() => { addTable(); toolsOpen = false; }}>Table</button>
          <button type="button" onclick={() => { addNote(); toolsOpen = false; }}>Text</button>
          <button type="button" class:on={picking} onclick={() => { picking = !picking; toolsOpen = false; }}>Point</button>
        </div>
      {/if}
    </header>
    <div class="lines">
      {#each lines as line, index (line.id)}
        {@const slider = sliderOf(line)}
        {@const suggestions = suggestionsFor(line)}
        <div class="object" style:--swatch={line.color} class:hidden={line.hidden} class:selected={menuFor === line.id}>
          <button
            type="button"
            class="marble"
            style:--swatch={line.color}
            aria-pressed={!line.hidden}
            aria-label={line.hidden ? "Show" : "Hide"}
            onclick={() => setLine(line.id, { hidden: !line.hidden })}
          ></button>
          <div class="fields">
            {#if line.kind === "note"}
              <input
                data-line={line.id}
                value={line.text}
                aria-label={`Note ${index + 1}`}
                oninput={(event) => setLine(line.id, { text: event.currentTarget.value })}
                onfocus={() => (menuFor = line.id)}
                onkeydown={(event) => onLineKey(event, index)}
              />
            {:else if line.kind === "table"}
              <div class="table">
                <span>x</span>
                <span>y</span>
                {#each line.tableX as cell, row}
                  <input
                    value={cell}
                    aria-label={`x ${row + 1}`}
                    oninput={(event) => {
                      const tableX = line.tableX.slice();
                      tableX[row] = event.currentTarget.value;
                      setLine(line.id, { tableX });
                    }}
                    onfocus={() => (menuFor = line.id)}
                  />
                  <input
                    value={line.tableY[row]}
                    aria-label={`y ${row + 1}`}
                    oninput={(event) => {
                      const tableY = line.tableY.slice();
                      tableY[row] = event.currentTarget.value;
                      setLine(line.id, { tableY });
                    }}
                    onfocus={() => (menuFor = line.id)}
                  />
                {/each}
              </div>
              <div class="tools">
                <button type="button" onclick={() => setLine(line.id, { tableX: [...line.tableX, ""], tableY: [...line.tableY, ""] })}>Row</button>
                <button type="button" class:on={line.fit === "line"} onclick={() => setLine(line.id, { fit: line.fit === "line" ? "none" : "line" })}>Linear fit</button>
                <button type="button" class:on={line.fit === "quad"} onclick={() => setLine(line.id, { fit: line.fit === "quad" ? "none" : "quad" })}>Quadratic fit</button>
              </div>
            {:else}
              <MathField
                plain
                value={line.text}
                lineId={line.id}
                label={`Expression ${index + 1}`}
                placeholder=""
                onChange={(text) => setLine(line.id, { text })}
                onKeydown={(event) => onLineKey(event, index)}
                onFocus={() => (menuFor = line.id)}
              />
              {#if slider && slider.value != null}
                <label class="slider">
                  <input
                    type="range"
                    min={line.sliderMin}
                    max={line.sliderMax}
                    step={line.sliderStep}
                    value={slider.value}
                    aria-label={`${slider.name} slider`}
                    oninput={(event) => writeSlider(line, Number(event.currentTarget.value))}
                  />
                  <input
                    class="bound"
                    value={line.sliderMin}
                    aria-label="Slider minimum"
                    onchange={(event) => setLine(line.id, { sliderMin: Number(event.currentTarget.value) })}
                  />
                  <input
                    class="bound"
                    value={line.sliderMax}
                    aria-label="Slider maximum"
                    onchange={(event) => setLine(line.id, { sliderMax: Number(event.currentTarget.value) })}
                  />
                </label>
              {/if}
              {#if suggestions.length}
                <div class="suggests">
                  {#each suggestions as name (name)}
                    <button type="button" class="suggest" onclick={() => addSlider(name, line.id)}>
                      {name} = 1
                    </button>
                  {/each}
                </div>
              {/if}
            {/if}
            {#if resultText(line)}
              <p class="result">{resultText(line)}</p>
            {/if}
            {#if line.error}
              <p class="error">{line.error}</p>
            {/if}
            {#if jobsPreview(line).some((item) => item.plot === "implicit")}
              <p class="result">Implicit curve</p>
            {/if}
          </div>
          {#if menuFor === line.id}
            <div class="stylebar">
              {#each PALETTE as color}
                <button type="button" class="dot" style:background={color} aria-label={color} onclick={() => setLine(line.id, { color })}></button>
              {/each}
              <button type="button" onclick={() => removeLine(line.id)}>Delete</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </aside>
  <div class="stage">
    <Graph
      {series}
      {markers}
      {ink}
      bindings={graphBindings}
      {degrees}
      {numberMode}
      {coordMode}
      onDegrees={(value) => (degrees = value)}
      onNumberMode={chooseMode}
      onCoordMode={chooseCoord}
      pickMode={picking}
      onPick={dropPoint}
      onPlacePoint={placePoint}
      fill
      pinView
      showChips={false}
    />
  </div>
</div>

<style>
  .desk {
    display: flex;
    height: 100%;
    min-height: 0;
    background: #1c1c20;
    color: #f3f4f6;
  }
  aside {
    width: 340px;
    flex: none;
    display: flex;
    flex-direction: column;
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    min-height: 0;
    background: #1c1c20;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: relative;
    padding: 10px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }
  header strong {
    font-size: 15px;
    font-weight: 600;
  }
  .plus {
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: inherit;
    font-size: 22px;
    line-height: 1;
    cursor: pointer;
  }
  .plus:hover,
  .plus[aria-expanded="true"] {
    background: rgba(255, 255, 255, 0.06);
  }
  .plus-menu {
    position: absolute;
    z-index: 3;
    top: 42px;
    right: 8px;
    display: flex;
    flex-direction: column;
    min-width: 140px;
    padding: 6px;
    background: #12141a;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
  }
  .plus-menu button,
  .stylebar button:not(.dot),
  .tools button {
    border: 0;
    background: transparent;
    color: inherit;
    border-radius: 6px;
    padding: 6px 8px;
    text-align: left;
    cursor: pointer;
  }
  .plus-menu button:hover,
  .stylebar button:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .result,
  .error {
    color: #8b93a7;
    font-size: 12px;
  }
  .tools {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .tools button.on {
    background: #6ea8fe;
    color: #081018;
  }
  .lines {
    overflow: auto;
    padding: 8px 0 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .object {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr);
    gap: 6px 10px;
    align-items: start;
    position: relative;
    margin: 0 10px;
    padding: 8px 10px;
    border-radius: 12px;
    background: #12141a;
    border: 1px solid rgba(255, 255, 255, 0.06);
    box-shadow: inset 3px 0 0 var(--swatch, #6ea8fe);
  }
  .object.selected {
    border-color: rgba(110, 168, 254, 0.45);
  }
  .object.hidden {
    opacity: 0.45;
  }
  .marble {
    width: 14px;
    height: 14px;
    margin-top: 7px;
    border: 2px solid var(--swatch);
    border-radius: 99px;
    background: var(--swatch);
    padding: 0;
    cursor: pointer;
  }
  .object.hidden .marble {
    background: transparent;
  }
  .stylebar {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    padding: 2px 0 4px;
  }
  .dot {
    width: 14px;
    height: 14px;
    border: 0;
    border-radius: 99px;
    padding: 0;
    cursor: pointer;
  }
  .fields {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .suggests {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .suggest {
    border: 0;
    border-radius: 999px;
    background: rgba(110, 168, 254, 0.16);
    color: #d6e6ff;
    padding: 4px 10px;
    font: inherit;
    cursor: pointer;
  }
  .suggest:hover {
    background: rgba(110, 168, 254, 0.28);
  }
  input {
    width: 100%;
    border: 0;
    border-radius: 8px;
    background: #1c1c20;
    color: #f3f4f6;
    padding: 6px 8px;
    outline: 1px solid rgba(255, 255, 255, 0.06);
  }
  input:focus {
    outline-color: #6ea8fe;
  }
  .slider {
    display: grid;
    grid-template-columns: 1fr 52px 52px;
    gap: 6px;
    align-items: center;
  }
  .slider input[type="range"] {
    padding: 0;
    background: transparent;
  }
  .bound {
    padding: 4px 6px;
  }
  .table {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
  }
  .result,
  .error {
    margin: 0;
    padding: 0 4px;
  }
  .error {
    color: #ff8b9a;
  }
  .stage {
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
</style>
