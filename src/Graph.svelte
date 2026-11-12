<script lang="ts">
  import { onMount } from "svelte";
  import { sampleGraph } from "./lib/api";
  import { formatQuantity, type NumberMode } from "./lib/numbers";
  import { chooseY, drawPlot, plotPadding, type View } from "./lib/plot";
  import { specialPoints, type SnapPoint } from "./lib/snap";
  import type { PlotFrame, PlotMarker, SeriesMeta, ThemeColors } from "./lib/types";

  type CoordMode = "cartesian" | "polar" | "radial";

  let {
    series,
    markers = [],
    frame = null,
    ink,
    fill = false,
    pinView = false,
    showChips = true,
    bindings = [],
    degrees = false,
    pickMode = false,
    numberMode = "decimal",
    coordMode = "cartesian",
    onDegrees,
    onNumberMode,
    onCoordMode,
    onPick = undefined,
    onPlacePoint = undefined,
  }: {
    series: SeriesMeta[];
    markers?: PlotMarker[];
    frame?: PlotFrame | null;
    ink: ThemeColors;
    fill?: boolean;
    pinView?: boolean;
    showChips?: boolean;
    bindings?: { name: string; value: number }[];
    degrees?: boolean;
    pickMode?: boolean;
    numberMode?: NumberMode;
    coordMode?: CoordMode;
    onDegrees?: (degrees: boolean) => void;
    onNumberMode?: (mode: NumberMode) => void;
    onCoordMode?: (mode: CoordMode) => void;
    onPick?: (x: number, y: number) => void;
    onPlacePoint?: (x: number, y: number) => void;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let xMin = $state(-10);
  let xMax = $state(10);
  let yMin = $state(-6);
  let yMax = $state(6);
  let autoY = $state(true);
  let lines = $state<number[][][][]>([]);
  let hidden = $state<Record<number, boolean>>({});
  let hover = $state<{ x: number; y: number; label: string } | null>(null);
  let cursor = $state<{ x: number; y: number } | null>(null);
  let snap = $state<SnapPoint | null>(null);
  let places = $state<SnapPoint[]>([]);
  let showGrid = $state(true);
  let showMinor = $state(true);
  let settingsOpen = $state(false);
  let paint = $state(0);
  let requestId = 0;
  let lastSig = "";
  let dragging = false;
  let lastPointer = { x: 0, y: 0 };
  let press: { x: number; y: number; hit: SnapPoint | null } | null = null;

  let signature = $derived(series.map((item) => `${item.plot}:${item.expression}`).join("\n"));
  let home = { xMin: -10, xMax: 10, yMin: -6, yMax: 6, autoY: true };
  let pxPerUnit = 0;
  let moved = false;

  function applyHome() {
    if (
      frame &&
      [frame.xMin, frame.xMax, frame.yMin, frame.yMax].every((value) => Number.isFinite(Number(value)))
    ) {
      xMin = Number(frame.xMin);
      xMax = Number(frame.xMax);
      yMin = Number(frame.yMin);
      yMax = Number(frame.yMax);
      autoY = false;
    } else {
      xMin = -10;
      xMax = 10;
      autoY = true;
    }
    home = { xMin, xMax, yMin, yMax, autoY };
  }

  $effect(() => {
    const sig = signature;
    const _frame = frame;
    if (sig === lastSig) return;
    const first = lastSig === "";
    lastSig = sig;
    void _frame;
    if (pinView) {
      if (first) hidden = {};
    } else {
      applyHome();
      hidden = {};
    }
    hover = null;
  });

  $effect(() => {
    const x0 = xMin;
    const x1 = xMax;
    const y0 = yMin;
    const y1 = yMax;
    const expressions = series.map((item) => `${item.plot}:${item.expression}:${item.domain ?? ""}`);
    const bound = bindings.map((item) => `${item.name}=${item.value}`).join(",");
    const angle = degrees;
    const coords = coordMode;
    const handle = setTimeout(() => {
      void pull(expressions, bound, angle, coords, x0, x1, y0, y1);
    }, 30);
    return () => clearTimeout(handle);
  });

  $effect(() => {
    const node = canvas;
    const view: View = { xMin, xMax, yMin, yMax };
    const drawn = series
      .map((item, index) => ({
        color: item.color,
        lines: hidden[index] ? [] : linesFor(item, index),
        paint: paintOf(item),
        dashed: item.style === "dashed",
        spokes: coordMode === "radial" && (item.plot === "function" || item.plot === "polar"),
      }))
      .filter((item) => item.lines.length > 0);
    const pointer = hover ? { x: hover.x, y: hover.y } : null;
    const tracker = pinView && cursor ? cursor : null;
    const dots = markers
      .map((marker) => ({ x: Number(marker.x), y: Number(marker.y) }))
      .filter((marker) => Number.isFinite(marker.x) && Number.isFinite(marker.y));
    const surface = ink.surface;
    const text = ink.text;
    const muted = ink.muted;
    if (!node || paint < 0) return;
    drawPlot(node, view, drawn, pointer, dots, { surface, text, muted }, { plane: pinView, grid: showGrid, minor: showMinor, coords: coordMode, tracker });
  });

  function paintOf(item: SeriesMeta): "stroke" | "below" | "above" | "cells" | "points" {
    if (item.style === "points" || item.plot === "point") return "points";
    if (item.plot === "below") return "below";
    if (item.plot === "above") return "above";
    if (item.plot === "region") return "cells";
    return "stroke";
  }

  function linesFor(item: SeriesMeta, index: number): number[][][] {
    if (item.plot === "vertical") {
      const x = Number(item.expression);
      if (!Number.isFinite(x)) return [];
      return [[[x, yMin], [x, yMax]]];
    }
    return lines[index] ?? [];
  }

  async function pull(_expressions: string[], _bound: string, angle: boolean, coords: CoordMode, x0: number, x1: number, y0: number, y1: number) {
    const id = ++requestId;
    const jobs = series
      .map((item, index) => ({ index, expression: item.expression, plot: item.plot, domain: item.domain ?? "" }))
      .filter((item) => item.plot !== "vertical");
    if (jobs.length === 0) {
      lines = series.map(() => []);
      places = specialPoints(verticalTraces(y0, y1));
      paint += 1;
      return;
    }
    try {
      const response = await sampleGraph({
        expressions: jobs.map((item) => item.expression),
        jobs: jobs.map((item) => ({
          expression: item.expression,
          plot: coords !== "cartesian" && item.plot === "function" ? "polar" : item.plot || "function",
          domain: item.domain,
        })),
        bindings,
        xMin: x0,
        xMax: x1,
        yMin: y0,
        yMax: y1,
        samples: 520,
        degrees: angle,
      });
      if (id !== requestId) return;
      const next = series.map(() => [] as number[][][]);
      response.series.forEach((item, index) => {
        const slot = jobs[index];
        if (!slot) return;
        next[slot.index] = item.lines;
      });
      lines = next;
      places = specialPoints(next.map((item, index) => verticalOr(item, index, y0, y1)));
      if (autoY && !pinView) {
        const values: number[] = [];
        next.forEach((item, index) => {
          if (hidden[index]) return;
          for (const line of item) {
            for (const point of line) values.push(point[1] ?? 0);
          }
        });
        for (const marker of markers) {
          const y = Number(marker.y);
          if (Number.isFinite(y)) values.push(y);
        }
        const [lo, hi] = chooseY(values);
        yMin = lo;
        yMax = hi;
      }
      paint += 1;
    } catch {
      paint += 1;
    }
  }

  function resetView() {
    hover = null;
    if (pinView) {
      moved = false;
      pxPerUnit = 0;
      autoY = false;
      applySquare(false);
      return;
    }
    xMin = home.xMin;
    xMax = home.xMax;
    yMin = home.yMin;
    yMax = home.yMax;
    autoY = home.autoY;
  }

  function applySquare(keepCenter: boolean) {
    if (!pinView) return;
    const { width, height } = spans();
    if (width < 8 || height < 8) return;
    const fresh = !moved || pxPerUnit <= 0;
    if (fresh) pxPerUnit = width / 20;
    const xMid = !fresh && keepCenter ? (xMin + xMax) / 2 : 0;
    const yMid = !fresh && keepCenter ? (yMin + yMax) / 2 : 0;
    const xSpan = width / pxPerUnit;
    const ySpan = height / pxPerUnit;
    const nextXMin = xMid - xSpan / 2;
    const nextYMin = yMid - ySpan / 2;
    if (
      Math.abs(nextXMin - xMin) < 1e-6 &&
      Math.abs(xMid + xSpan / 2 - xMax) < 1e-6 &&
      Math.abs(nextYMin - yMin) < 1e-6 &&
      Math.abs(yMid + ySpan / 2 - yMax) < 1e-6
    ) {
      return;
    }
    xMin = nextXMin;
    xMax = xMid + xSpan / 2;
    yMin = nextYMin;
    yMax = yMid + ySpan / 2;
    autoY = false;
  }

  function rememberScale() {
    const { width } = spans();
    if (pinView && width > 8) pxPerUnit = width / (xMax - xMin);
  }

  function zoomBy(factor: number) {
    autoY = false;
    moved = true;
    const xMid = (xMin + xMax) / 2;
    const yMid = (yMin + yMax) / 2;
    const nextX = (xMax - xMin) * factor;
    const nextY = (yMax - yMin) * factor;
    xMin = xMid - nextX / 2;
    xMax = xMid + nextX / 2;
    yMin = yMid - nextY / 2;
    yMax = yMid + nextY / 2;
    rememberScale();
  }

  function spans() {
    const node = canvas?.getBoundingClientRect();
    const pad = plotPadding(pinView);
    const width = Math.max(1, (node?.width ?? 1) - pad.l - pad.r);
    const height = Math.max(1, (node?.height ?? 1) - pad.t - pad.b);
    return { width, height, pad };
  }

  function verticalOr(item: number[][][], index: number, y0: number, y1: number): number[][][] {
    const source = series[index];
    if (source?.plot !== "vertical") return item;
    const x = Number(source.expression);
    if (!Number.isFinite(x)) return item;
    return [[[x, y0], [x, y1]]];
  }

  function verticalTraces(y0: number, y1: number): number[][][][] {
    return series.map((item) => {
      if (item.plot !== "vertical") return [];
      const x = Number(item.expression);
      if (!Number.isFinite(x)) return [];
      return [[[x, y0], [x, y1]]];
    });
  }

  function nearestSnap(point: { x: number; y: number }): SnapPoint | null {
    const { width, height } = spans();
    const xSpan = xMax - xMin || 1;
    const ySpan = yMax - yMin || 1;
    let best: SnapPoint | null = null;
    let bestDist = 16;
    for (const place of places) {
      const sx = ((place.x - xMin) / xSpan) * width;
      const sy = ((yMax - place.y) / ySpan) * height;
      const px = ((point.x - xMin) / xSpan) * width;
      const py = ((yMax - point.y) / ySpan) * height;
      const dist = Math.hypot(sx - px, sy - py);
      if (dist < bestDist) {
        best = place;
        bestDist = dist;
      }
    }
    return best;
  }

  function dataPoint(event: { clientX: number; clientY: number }): { x: number; y: number } | null {
    const rect = canvas?.getBoundingClientRect();
    if (!rect) return null;
    const { width, height, pad } = spans();
    const px = event.clientX - rect.left - pad.l;
    const py = event.clientY - rect.top - pad.t;
    return {
      x: xMin + (px / width) * (xMax - xMin),
      y: yMax - (py / height) * (yMax - yMin),
    };
  }

  function onPointerDown(event: PointerEvent) {
    if (pickMode) {
      const point = dataPoint(event);
      if (point && Number.isFinite(point.x) && Number.isFinite(point.y)) onPick?.(point.x, point.y);
      return;
    }
    press = { x: event.clientX, y: event.clientY, hit: snap };
    dragging = true;
    autoY = false;
    moved = true;
    lastPointer = { x: event.clientX, y: event.clientY };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onPointerMove(event: PointerEvent) {
    if (pinView) {
      const point = dataPoint(event);
      if (point && Number.isFinite(point.x) && Number.isFinite(point.y)) {
        const hit = nearestSnap(point);
        snap = hit;
        cursor = hit ? { x: hit.x, y: hit.y } : point;
      } else {
        snap = null;
        cursor = null;
      }
    }
    if (dragging) {
      const { width, height } = spans();
      const dx = event.clientX - lastPointer.x;
      const dy = event.clientY - lastPointer.y;
      lastPointer = { x: event.clientX, y: event.clientY };
      const xSpan = xMax - xMin;
      const ySpan = yMax - yMin;
      xMin -= (dx / width) * xSpan;
      xMax -= (dx / width) * xSpan;
      yMin += (dy / height) * ySpan;
      yMax += (dy / height) * ySpan;
      return;
    }
    trace(event);
  }

  function onPointerUp(event: PointerEvent) {
    if (press?.hit && Math.hypot(event.clientX - press.x, event.clientY - press.y) < 5) {
      onPlacePoint?.(press.hit.x, press.hit.y);
    }
    press = null;
    dragging = false;
  }

  function trace(event: PointerEvent) {
    const rect = canvas?.getBoundingClientRect();
    if (!rect) return;
    const { width, height, pad } = spans();
    const px = event.clientX - rect.left - pad.l;
    const py = event.clientY - rect.top - pad.t;
    if (px < 0 || py < 0 || px > width || py > height) {
      hover = null;
      return;
    }
    const xSpan = xMax - xMin;
    const dataX = xMin + (px / width) * xSpan;
    let best: { x: number; y: number; dist: number; label: string } | null = null;
    for (let index = 0; index < series.length; index += 1) {
      if (hidden[index]) continue;
      const item = series[index];
      if (!item) continue;
      if (item.plot === "vertical") {
        const x = Number(item.expression);
        if (Number.isFinite(x)) {
          const sx = pad.l + ((x - xMin) / xSpan) * width;
          const dist = Math.abs(sx - (px + pad.l));
          const y = yMax - (py / height) * (yMax - yMin);
          if (!best || dist < best.dist) best = { x, y, dist, label: item.label };
        }
        continue;
      }
      for (const line of linesFor(item, index)) {
        for (const point of line) {
          const pointX = point[0] ?? 0;
          const pointY = point[1] ?? 0;
          const sx = pad.l + ((pointX - xMin) / xSpan) * width;
          const sy = pad.t + ((yMax - pointY) / (yMax - yMin || 1)) * height;
          const dist = Math.hypot(sx - (px + pad.l), sy - (py + pad.t));
          if (!best || dist < best.dist) {
            best = { x: pointX, y: pointY, dist, label: item.label };
          }
        }
      }
    }
    hover = best && best.dist < 18 ? { x: best.x, y: best.y, label: best.label } : null;
  }

  function onWheel(event: WheelEvent) {
    event.preventDefault();
    autoY = false;
    moved = true;
    const rect = canvas?.getBoundingClientRect();
    if (!rect) return;
    const { width, height, pad } = spans();
    const factor = event.deltaY > 0 ? 1.12 : 1 / 1.12;
    const fx = (event.clientX - rect.left - pad.l) / width;
    const fy = (event.clientY - rect.top - pad.t) / height;
    const x = xMin + Math.min(1, Math.max(0, fx)) * (xMax - xMin);
    const y = yMax - Math.min(1, Math.max(0, fy)) * (yMax - yMin);
    const nextX = (xMax - xMin) * factor;
    const nextY = (yMax - yMin) * factor;
    const clampedFx = Math.min(1, Math.max(0, fx));
    const clampedFy = Math.min(1, Math.max(0, fy));
    xMin = x - clampedFx * nextX;
    xMax = xMin + nextX;
    yMin = y - (1 - clampedFy) * nextY;
    yMax = yMin + nextY;
    rememberScale();
  }

  onMount(() => {
    const node = canvas;
    if (!node) return;
    node.addEventListener("wheel", onWheel, { passive: false });
    const observer = new ResizeObserver(() => {
      if (pinView) applySquare(moved);
      paint += 1;
    });
    observer.observe(node);
    paint += 1;
    return () => {
      node.removeEventListener("wheel", onWheel);
      observer.disconnect();
    };
  });

  function toggle(index: number) {
    hidden = { ...hidden, [index]: !hidden[index] };
    if (autoY && !pinView) {
      const values: number[] = [];
      series.forEach((item, seriesIndex) => {
        if (hidden[seriesIndex]) return;
        for (const line of lines[seriesIndex] ?? []) {
          for (const point of line) values.push(point[1] ?? 0);
        }
        void item;
      });
      const [lo, hi] = chooseY(values);
      yMin = lo;
      yMax = hi;
    }
    paint += 1;
  }
</script>

<section class="graph" class:fill style:--surface={ink.surface} style:--text={ink.text} style:--muted={ink.muted}>
  {#if showChips}
  <div class="chips">
    {#each series as item, index (item.expression + index)}
      <button
        type="button"
        class="chip"
        class:off={hidden[index]}
        style:--swatch={item.color}
        onclick={() => toggle(index)}
      >
        <i></i>
        {item.label}
      </button>
    {/each}
    <button type="button" class="reset" onclick={resetView}>Reset</button>
  </div>
  {/if}
  <div class="viewport">
    {#if pinView}
      <div class="settings">
        <button type="button" class="gear" aria-label="Settings" aria-expanded={settingsOpen} onclick={() => (settingsOpen = !settingsOpen)}>
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <circle cx="8" cy="8" r="2.2" fill="none" stroke="currentColor" stroke-width="1.4" />
            <path d="M8 1.4v1.8M8 12.8v1.8M1.4 8h1.8M12.8 8h1.8M3.2 3.2l1.3 1.3M11.5 11.5l1.3 1.3M12.8 3.2l-1.3 1.3M4.5 11.5l-1.3 1.3" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
          </svg>
        </button>
        {#if settingsOpen}
          <div class="panel">
            <span>Coordinates</span>
            <div class="choice">
              <button type="button" class:on={coordMode === "cartesian"} onclick={() => onCoordMode?.("cartesian")}>Cartesian</button>
              <button type="button" class:on={coordMode === "polar"} onclick={() => onCoordMode?.("polar")}>Polar</button>
              <button type="button" class:on={coordMode === "radial"} onclick={() => onCoordMode?.("radial")}>Radial</button>
            </div>
            <span>Numbers</span>
            <div class="choice">
              <button type="button" class:on={numberMode === "decimal"} onclick={() => onNumberMode?.("decimal")}>Decimal</button>
              <button type="button" class:on={numberMode === "integer"} onclick={() => onNumberMode?.("integer")}>Integer</button>
              <button type="button" class:on={numberMode === "irrational"} onclick={() => onNumberMode?.("irrational")}>Irrational</button>
              <button type="button" class:on={numberMode === "complex"} onclick={() => onNumberMode?.("complex")}>Complex</button>
            </div>
            <span>Angle</span>
            <div class="choice">
              <button type="button" class:on={!degrees} onclick={() => onDegrees?.(false)}>Radians</button>
              <button type="button" class:on={degrees} onclick={() => onDegrees?.(true)}>Degrees</button>
            </div>
            <label><input type="checkbox" bind:checked={showGrid} /> Grid</label>
            <label><input type="checkbox" bind:checked={showMinor} /> Minor grid</label>
          </div>
        {/if}
      </div>
      <div class="zoom">
        <button type="button" aria-label="Zoom in" onclick={() => zoomBy(1 / 1.25)}>+</button>
        <button type="button" aria-label="Zoom out" onclick={() => zoomBy(1.25)}>−</button>
        <button type="button" aria-label="Standard view" onclick={resetView}>
          <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
            <path d="M2.2 7.2 8 2.4l5.8 4.8V13.6H10V9.2H6v4.4H2.2Z" fill="none" stroke="currentColor" stroke-width="1.4" />
          </svg>
        </button>
      </div>
    {/if}
  <canvas
    class:pick={pickMode}
    bind:this={canvas}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointerleave={() => {
      dragging = false;
      hover = null;
      cursor = null;
      snap = null;
    }}
    ondblclick={() => {
      if (!pinView) resetView();
    }}
  ></canvas>
    {#if pinView && cursor}
      <div class="coords">({formatQuantity(cursor.x, numberMode)}, {formatQuantity(cursor.y, numberMode)}){snap ? ` · ${snap.kind}` : ""}</div>
    {/if}
  </div>
  <div class="readout">
    {#if hover}
      {hover.label} · x = {hover.x.toPrecision(4)}, y = {hover.y.toPrecision(4)}
    {:else if !pinView}
      {pickMode ? "Click the graph to drop a point" : "Drag to pan · wheel to zoom · double-click resets"}
    {/if}
  </div>
</section>

<style>
  .graph {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0 10px 10px;
  }
  .graph.fill {
    height: 100%;
    min-height: 0;
    padding: 0;
    gap: 0;
    background: var(--surface, #12141a);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .chip,
  .reset {
    border: 0;
    background: color-mix(in srgb, var(--text, white) 8%, transparent);
    color: var(--text, #d5d8e0);
    border-radius: 999px;
    padding: 3px 8px;
    font: inherit;
    font-size: 12px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }
  .chip i {
    width: 8px;
    height: 8px;
    border-radius: 99px;
    background: var(--swatch);
  }
  .chip.off {
    opacity: 0.4;
  }
  .viewport {
    position: relative;
    display: flex;
    width: 100%;
    min-height: 0;
  }
  .settings {
    position: absolute;
    top: 12px;
    right: 12px;
    z-index: 2;
  }
  .gear {
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 8px;
    background: color-mix(in srgb, var(--surface, #12141a) 88%, var(--text, white));
    color: var(--text, #f3f4f6);
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .panel {
    position: absolute;
    top: 38px;
    right: 0;
    width: 220px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: 10px;
    background: #1c1c20;
    color: var(--text, #f3f4f6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 13px;
  }
  .panel span {
    color: var(--muted, #9aa3b2);
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .choice {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
  }
  .choice button:last-child:nth-child(odd) {
    grid-column: 1 / -1;
  }
  .choice button {
    border: 0;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: inherit;
    padding: 6px 0;
    cursor: pointer;
  }
  .choice button.on {
    background: #6ea8fe;
    color: #081018;
  }
  .panel label {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .zoom {
    position: absolute;
    right: 12px;
    bottom: 16px;
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .zoom button {
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 8px;
    background: color-mix(in srgb, var(--surface, #12141a) 88%, var(--text, white));
    color: var(--text, #f3f4f6);
    font: inherit;
    font-size: 16px;
    cursor: pointer;
    display: grid;
    place-items: center;
  }
  canvas {
    width: 100%;
    height: 280px;
    border-radius: 10px;
    cursor: grab;
    touch-action: none;
    background: var(--surface, #12141a);
  }
  canvas.pick {
    cursor: crosshair;
  }
  .fill .viewport {
    flex: 1;
  }
  .fill canvas {
    flex: 1;
    height: auto;
    min-height: 0;
    border-radius: 0;
  }
  .coords {
    position: absolute;
    left: 12px;
    bottom: 10px;
    color: var(--muted, #9aa3b2);
    font: 13px "Segoe UI", sans-serif;
    pointer-events: none;
  }
  .readout {
    min-height: 16px;
    color: var(--muted, #9aa3b2);
    font-size: 12px;
    font-family: ui-monospace, Consolas, monospace;
  }
</style>
