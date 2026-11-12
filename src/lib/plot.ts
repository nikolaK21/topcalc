import { luminance } from "./theme";

export type View = {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
};

export type DrawnSeries = {
  color: string;
  lines: number[][][];
  paint?: "stroke" | "below" | "above" | "cells" | "points";
  dashed?: boolean;
  spokes?: boolean;
};

export function chooseY(values: number[]): [number, number] {
  const finite = values.filter((value) => Number.isFinite(value)).sort((a, b) => a - b);
  if (finite.length === 0) return [-6, 6];
  const inBand = finite.filter((value) => Math.abs(value) <= 20).length / finite.length;
  if (inBand > 0.55) {
    const band = finite.filter((value) => Math.abs(value) <= 20);
    return padRange(band[0] ?? -6, band[band.length - 1] ?? 6, 6);
  }
  const lo = finite[Math.floor((finite.length - 1) * 0.05)] ?? -6;
  const hi = finite[Math.floor((finite.length - 1) * 0.95)] ?? 6;
  return padRange(lo, hi, 1);
}

function padRange(lo: number, hi: number, minSpan: number): [number, number] {
  if (!Number.isFinite(lo) || !Number.isFinite(hi)) return [-6, 6];
  if (hi - lo < minSpan) {
    const mid = (hi + lo) / 2;
    lo = mid - minSpan / 2;
    hi = mid + minSpan / 2;
  }
  const pad = (hi - lo) * 0.08;
  return [lo - pad, hi + pad];
}

export function niceStep(span: number, targetTicks: number): number {
  if (!Number.isFinite(span) || span <= 0) return 1;
  const rough = span / Math.max(1, targetTicks);
  const pow = 10 ** Math.floor(Math.log10(rough));
  const err = rough / pow;
  const nice = err >= 7.5 ? 10 : err >= 3.5 ? 5 : err >= 1.5 ? 2 : 1;
  return nice * pow;
}

export function formatTick(value: number): string {
  if (!Number.isFinite(value)) return "";
  const abs = Math.abs(value);
  if (abs !== 0 && (abs < 0.001 || abs >= 10000)) return value.toExponential(0);
  const rounded = Math.round(value * 1e6) / 1e6;
  return String(rounded);
}

export type PlotInk = {
  surface: string;
  text: string;
  muted: string;
};

export function drawPlot(
  canvas: HTMLCanvasElement,
  view: View,
  series: DrawnSeries[],
  hover: { x: number; y: number } | null,
  markers: { x: number; y: number }[] = [],
  ink: PlotInk = { surface: "#12141a", text: "#f3f4f6", muted: "#8b93a7" },
  options: { plane?: boolean; grid?: boolean; minor?: boolean; coords?: "cartesian" | "polar" | "radial"; tracker?: { x: number; y: number } | null } = {},
): void {
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, rect.width);
  const height = Math.max(1, rect.height);
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = ink.surface;
  ctx.fillRect(0, 0, width, height);
  const light = luminance(ink.text) > 0.45;
  const grid = light ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.08)";
  const axis = light
    ? `rgba(255,255,255,${options.plane ? 0.82 : 0.35})`
    : `rgba(0,0,0,${options.plane ? 0.55 : 0.35})`;

  const pad = plotPadding(options.plane === true);
  const plotW = Math.max(1, width - pad.l - pad.r);
  const plotH = Math.max(1, height - pad.t - pad.b);
  const xSpan = view.xMax - view.xMin || 1;
  const ySpan = view.yMax - view.yMin || 1;
  const xToPx = (x: number) => pad.l + ((x - view.xMin) / xSpan) * plotW;
  const yToPx = (y: number) => pad.t + ((view.yMax - y) / ySpan) * plotH;

  const xStep = niceStep(xSpan, options.plane ? 10 : 8);
  const yStep = niceStep(ySpan, options.plane ? 8 : 6);
  ctx.save();
  ctx.beginPath();
  ctx.rect(pad.l, pad.t, plotW, plotH);
  ctx.clip();
  ctx.lineWidth = 1;
  const polar = options.coords === "polar" || options.coords === "radial";
  if (polar && options.grid !== false) {
    drawPolarGrid(ctx, view, xToPx, yToPx, plotW, plotH, grid);
  } else {
    if (options.grid !== false && options.plane && options.minor !== false) {
      drawMinorGrid(ctx, view, xStep, yStep, xToPx, yToPx, pad, plotW, plotH, grid);
    }
    if (options.grid !== false) {
      ctx.strokeStyle = grid;
      for (let x = Math.ceil(view.xMin / xStep) * xStep; x <= view.xMax; x += xStep) {
        const px = xToPx(x);
        ctx.beginPath();
        ctx.moveTo(px, pad.t);
        ctx.lineTo(px, pad.t + plotH);
        ctx.stroke();
      }
      for (let y = Math.ceil(view.yMin / yStep) * yStep; y <= view.yMax; y += yStep) {
        const py = yToPx(y);
        ctx.beginPath();
        ctx.moveTo(pad.l, py);
        ctx.lineTo(pad.l + plotW, py);
        ctx.stroke();
      }
    }
  }
  ctx.restore();

  if (options.plane) {
    drawPlaneAxes(ctx, view, ink, axis, xToPx, yToPx, pad, plotW, plotH, xStep, yStep);
  } else {
    ctx.fillStyle = ink.muted;
    ctx.font = "11px ui-monospace, Consolas, monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "alphabetic";
    for (let x = Math.ceil(view.xMin / xStep) * xStep; x <= view.xMax; x += xStep) {
      ctx.fillText(formatTick(x), xToPx(x), height - 8);
    }
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    for (let y = Math.ceil(view.yMin / yStep) * yStep; y <= view.yMax; y += yStep) {
      ctx.fillText(formatTick(y), pad.l - 6, yToPx(y));
    }

    if (view.xMin < 0 && view.xMax > 0) {
      ctx.strokeStyle = axis;
      ctx.beginPath();
      ctx.moveTo(xToPx(0), pad.t);
      ctx.lineTo(xToPx(0), pad.t + plotH);
      ctx.stroke();
    }
    if (view.yMin < 0 && view.yMax > 0) {
      ctx.strokeStyle = axis;
      ctx.beginPath();
      ctx.moveTo(pad.l, yToPx(0));
      ctx.lineTo(pad.l + plotW, yToPx(0));
      ctx.stroke();
    }
  }

  ctx.save();
  ctx.beginPath();
  ctx.rect(pad.l, pad.t, plotW, plotH);
  ctx.clip();
  ctx.lineWidth = 2;
  ctx.lineJoin = "round";
  ctx.lineCap = "round";
  for (const item of series) {
    const paint = item.paint ?? "stroke";
    ctx.fillStyle = item.color;
    ctx.strokeStyle = item.color;
    ctx.globalAlpha = paint === "cells" || paint === "below" || paint === "above" ? 0.28 : 1;
    if (paint === "below" || paint === "above" || paint === "cells") {
      for (const line of item.lines) {
        if (line.length < 2) continue;
        ctx.beginPath();
        line.forEach((point, index) => {
          const px = xToPx(point[0] ?? 0);
          const py = yToPx(point[1] ?? 0);
          if (index === 0) ctx.moveTo(px, py);
          else ctx.lineTo(px, py);
        });
        if (paint !== "cells") {
          const edge = paint === "below" ? yToPx(view.yMin) : yToPx(view.yMax);
          const last = line[line.length - 1] ?? [0, 0];
          const first = line[0] ?? [0, 0];
          ctx.lineTo(xToPx(last[0] ?? 0), edge);
          ctx.lineTo(xToPx(first[0] ?? 0), edge);
        }
        ctx.closePath();
        ctx.fill();
      }
    }
    ctx.globalAlpha = 1;
    if (item.spokes) {
      ctx.globalAlpha = 0.35;
      ctx.setLineDash([]);
      for (const line of item.lines) {
        const step = Math.max(1, Math.floor(line.length / 36));
        for (let index = 0; index < line.length; index += step) {
          const point = line[index];
          if (!point) continue;
          ctx.beginPath();
          ctx.moveTo(xToPx(0), yToPx(0));
          ctx.lineTo(xToPx(point[0] ?? 0), yToPx(point[1] ?? 0));
          ctx.stroke();
        }
      }
      ctx.globalAlpha = 1;
    }
    if (paint === "points") {
      for (const line of item.lines) {
        for (const point of line) {
          ctx.beginPath();
          ctx.arc(xToPx(point[0] ?? 0), yToPx(point[1] ?? 0), 4.5, 0, Math.PI * 2);
          ctx.fill();
        }
      }
      continue;
    }
    if (paint === "cells") continue;
    ctx.setLineDash(item.dashed ? [7, 5] : []);
    for (const line of item.lines) {
      if (line.length < 2) continue;
      ctx.beginPath();
      line.forEach((point, index) => {
        const px = xToPx(point[0] ?? 0);
        const py = yToPx(point[1] ?? 0);
        if (index === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      });
      ctx.stroke();
    }
    ctx.setLineDash([]);
  }
  for (const marker of markers) {
    if (!Number.isFinite(marker.x) || !Number.isFinite(marker.y)) continue;
    const px = xToPx(marker.x);
    const py = yToPx(marker.y);
    ctx.beginPath();
    ctx.fillStyle = ink.text;
    ctx.arc(px, py, 4.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.lineWidth = 2;
    ctx.strokeStyle = ink.surface;
    ctx.stroke();
  }
  if (options.tracker) drawCrosshair(ctx, options.tracker.x, options.tracker.y, xToPx, yToPx, pad, plotW, plotH, axis);
  if (hover) {
    if (!options.tracker) drawCrosshair(ctx, hover.x, hover.y, xToPx, yToPx, pad, plotW, plotH, axis);
    ctx.fillStyle = ink.text;
    ctx.beginPath();
    ctx.arc(xToPx(hover.x), yToPx(hover.y), 3.5, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.restore();
}

export function plotPadding(plane = false) {
  return plane ? { l: 18, r: 36, t: 28, b: 18 } : { l: 52, r: 14, t: 12, b: 28 };
}

function drawCrosshair(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  xToPx: (x: number) => number,
  yToPx: (y: number) => number,
  pad: { l: number; t: number },
  plotW: number,
  plotH: number,
  axis: string,
) {
  ctx.save();
  ctx.strokeStyle = axis;
  ctx.lineWidth = 1;
  ctx.setLineDash([4, 4]);
  ctx.beginPath();
  ctx.moveTo(xToPx(x), pad.t);
  ctx.lineTo(xToPx(x), pad.t + plotH);
  ctx.moveTo(pad.l, yToPx(y));
  ctx.lineTo(pad.l + plotW, yToPx(y));
  ctx.stroke();
  ctx.restore();
}

function drawPolarGrid(
  ctx: CanvasRenderingContext2D,
  view: View,
  xToPx: (x: number) => number,
  yToPx: (y: number) => number,
  plotW: number,
  plotH: number,
  grid: string,
) {
  const originX = xToPx(0);
  const originY = yToPx(0);
  const reach = Math.hypot(
    Math.max(Math.abs(view.xMin), Math.abs(view.xMax)),
    Math.max(Math.abs(view.yMin), Math.abs(view.yMax)),
  );
  const step = niceStep(reach, 5);
  const unit = Math.abs(xToPx(step) - originX);
  ctx.strokeStyle = grid;
  ctx.lineWidth = 1;
  for (let radius = step; radius <= reach + step; radius += step) {
    ctx.beginPath();
    ctx.arc(originX, originY, (radius / step) * unit, 0, Math.PI * 2);
    ctx.stroke();
  }
  const arm = Math.hypot(plotW, plotH);
  for (let spoke = 0; spoke < 12; spoke += 1) {
    const angle = (spoke * Math.PI) / 6;
    ctx.beginPath();
    ctx.moveTo(originX, originY);
    ctx.lineTo(originX + Math.cos(angle) * arm, originY - Math.sin(angle) * arm);
    ctx.stroke();
  }
}

function drawMinorGrid(
  ctx: CanvasRenderingContext2D,
  view: View,
  xStep: number,
  yStep: number,
  xToPx: (x: number) => number,
  yToPx: (y: number) => number,
  pad: { l: number; r: number; t: number; b: number },
  plotW: number,
  plotH: number,
  grid: string,
) {
  const xMinor = xStep / 5;
  const yMinor = yStep / 5;
  ctx.strokeStyle = grid;
  ctx.globalAlpha = 0.45;
  if (plotW / ((view.xMax - view.xMin) / xMinor) >= 8) {
    for (let x = Math.ceil(view.xMin / xMinor) * xMinor; x <= view.xMax; x += xMinor) {
      const px = xToPx(x);
      ctx.beginPath();
      ctx.moveTo(px, pad.t);
      ctx.lineTo(px, pad.t + plotH);
      ctx.stroke();
    }
  }
  if (plotH / ((view.yMax - view.yMin) / yMinor) >= 8) {
    for (let y = Math.ceil(view.yMin / yMinor) * yMinor; y <= view.yMax; y += yMinor) {
      const py = yToPx(y);
      ctx.beginPath();
      ctx.moveTo(pad.l, py);
      ctx.lineTo(pad.l + plotW, py);
      ctx.stroke();
    }
  }
  ctx.globalAlpha = 1;
}

function drawArrow(ctx: CanvasRenderingContext2D, x: number, y: number, angle: number) {
  const size = 10;
  ctx.beginPath();
  ctx.moveTo(x, y);
  ctx.lineTo(x - size * Math.cos(angle - 0.38), y - size * Math.sin(angle - 0.38));
  ctx.moveTo(x, y);
  ctx.lineTo(x - size * Math.cos(angle + 0.38), y - size * Math.sin(angle + 0.38));
  ctx.stroke();
}

function drawPlaneAxes(
  ctx: CanvasRenderingContext2D,
  view: View,
  ink: PlotInk,
  axis: string,
  xToPx: (x: number) => number,
  yToPx: (y: number) => number,
  pad: { l: number; t: number },
  plotW: number,
  plotH: number,
  xStep: number,
  yStep: number,
) {
  const left = pad.l;
  const top = pad.t;
  const right = left + plotW;
  const bottom = top + plotH;
  const axisX = xToPx(0);
  const axisY = yToPx(0);
  const showY = axisX >= left && axisX <= right;
  const showX = axisY >= top && axisY <= bottom;
  ctx.strokeStyle = axis;
  ctx.fillStyle = ink.text;
  ctx.lineWidth = 1.4;
  ctx.font = "13px Segoe UI, sans-serif";
  if (showX) {
    ctx.beginPath();
    ctx.moveTo(left, axisY);
    ctx.lineTo(right, axisY);
    ctx.stroke();
    drawArrow(ctx, right, axisY, 0);
    ctx.textAlign = "right";
    ctx.textBaseline = "bottom";
    ctx.fillText("x", right - 2, axisY - 4);
  }
  if (showY) {
    ctx.beginPath();
    ctx.moveTo(axisX, bottom);
    ctx.lineTo(axisX, top);
    ctx.stroke();
    drawArrow(ctx, axisX, top, -Math.PI / 2);
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText("y", axisX + 6, top + 2);
  }
  ctx.fillStyle = ink.muted;
  ctx.font = "12px Segoe UI, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (let x = Math.ceil(view.xMin / xStep) * xStep; x <= view.xMax; x += xStep) {
    if (Math.abs(x) < xStep * 0.2) continue;
    const px = xToPx(x);
    let labelY = showX ? axisY + 4 : bottom - 16;
    if (showX && labelY > bottom - 16) labelY = axisY - 16;
    ctx.fillText(formatTick(x), px, labelY);
  }
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  for (let y = Math.ceil(view.yMin / yStep) * yStep; y <= view.yMax; y += yStep) {
    if (Math.abs(y) < yStep * 0.2) continue;
    const py = yToPx(y);
    let labelX = showY ? axisX + 6 : left + 4;
    if (showY && labelX > right - 28) labelX = axisX - 28;
    ctx.fillText(formatTick(y), labelX, py);
  }
}
