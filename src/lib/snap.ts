export type SnapKind = "intersection" | "vertex" | "intercept" | "inflection";

export type SnapPoint = {
  x: number;
  y: number;
  kind: SnapKind;
};

const RANK: Record<SnapKind, number> = {
  intersection: 0,
  intercept: 1,
  vertex: 2,
  inflection: 3,
};

export function specialPoints(series: number[][][][]): SnapPoint[] {
  const hits: SnapPoint[] = [];
  const curves: number[][][] = [];
  for (const item of series) {
    const pieces = item.filter((line) => line.length >= 2);
    if (pieces.length === 0) continue;
    curves.push(pieces.flatMap((line) => line));
    for (const line of pieces) {
      if (mostlyIncreasing(line)) {
        hits.push(...vertices(line));
        hits.push(...inflections(line));
      } else {
        hits.push(...radiusVertices(line));
      }
      hits.push(...intercepts(line));
    }
  }
  for (let left = 0; left < curves.length; left += 1) {
    for (let right = left + 1; right < curves.length; right += 1) {
      hits.push(...intersections(thin(curves[left] ?? [], 160), thin(curves[right] ?? [], 160)));
    }
  }
  return dedupe(hits).slice(0, 80);
}

function mostlyIncreasing(line: number[][]): boolean {
  let forward = 0;
  for (let index = 1; index < line.length; index += 1) {
    if ((line[index]?.[0] ?? 0) >= (line[index - 1]?.[0] ?? 0)) forward += 1;
  }
  return forward > (line.length - 1) * 0.8;
}

function vertices(line: number[][]): SnapPoint[] {
  const hits: SnapPoint[] = [];
  for (let index = 1; index < line.length - 1; index += 1) {
    const prev = line[index - 1];
    const mid = line[index];
    const next = line[index + 1];
    if (!prev || !mid || !next) continue;
    const rise = (mid[1] ?? 0) - (prev[1] ?? 0);
    const fall = (next[1] ?? 0) - (mid[1] ?? 0);
    if (rise * fall >= 0) continue;
    const vertex = quadVertex(prev, mid, next);
    hits.push({ x: vertex?.x ?? mid[0] ?? 0, y: vertex?.y ?? mid[1] ?? 0, kind: "vertex" });
  }
  return hits;
}

function radiusVertices(line: number[][]): SnapPoint[] {
  const hits: SnapPoint[] = [];
  const radius = (point: number[]) => Math.hypot(point[0] ?? 0, point[1] ?? 0);
  for (let index = 1; index < line.length - 1; index += 1) {
    const prev = line[index - 1];
    const mid = line[index];
    const next = line[index + 1];
    if (!prev || !mid || !next) continue;
    const rise = radius(mid) - radius(prev);
    const fall = radius(next) - radius(mid);
    if (rise * fall < 0 && Math.abs(rise) + Math.abs(fall) > 1e-4) {
      hits.push({ x: mid[0] ?? 0, y: mid[1] ?? 0, kind: "vertex" });
    }
  }
  return hits;
}

function inflections(line: number[][]): SnapPoint[] {
  const hits: SnapPoint[] = [];
  for (let index = 1; index < line.length - 2; index += 1) {
    const a = line[index - 1];
    const b = line[index];
    const c = line[index + 1];
    const d = line[index + 2];
    if (!a || !b || !c || !d) continue;
    const s1 = second(a, b, c);
    const s2 = second(b, c, d);
    const scale = Math.max(1e-3, Math.abs(b[1] ?? 0), Math.abs(c[1] ?? 0));
    if (Math.abs(s1) < 1e-4 * scale && Math.abs(s2) < 1e-4 * scale) continue;
    if (s1 * s2 >= 0) continue;
    const t = s1 / (s1 - s2);
    hits.push({
      x: (b[0] ?? 0) + ((c[0] ?? 0) - (b[0] ?? 0)) * t,
      y: (b[1] ?? 0) + ((c[1] ?? 0) - (b[1] ?? 0)) * t,
      kind: "inflection",
    });
  }
  return hits;
}

function second(a: number[], b: number[], c: number[]): number {
  return (c[1] ?? 0) - 2 * (b[1] ?? 0) + (a[1] ?? 0);
}

function intercepts(line: number[][]): SnapPoint[] {
  const hits: SnapPoint[] = [];
  for (let index = 1; index < line.length; index += 1) {
    const a = line[index - 1];
    const b = line[index];
    if (!a || !b) continue;
    const ay = a[1] ?? 0;
    const by = b[1] ?? 0;
    if (ay === 0 || ay * by < 0) {
      const t = ay === by ? 0 : ay / (ay - by);
      hits.push({ x: (a[0] ?? 0) + ((b[0] ?? 0) - (a[0] ?? 0)) * t, y: 0, kind: "intercept" });
    }
    const ax = a[0] ?? 0;
    const bx = b[0] ?? 0;
    if (ax === 0 || ax * bx < 0) {
      const t = ax === bx ? 0 : ax / (ax - bx);
      hits.push({ x: 0, y: ay + (by - ay) * t, kind: "intercept" });
    }
  }
  return hits;
}

function intersections(left: number[][], right: number[][]): SnapPoint[] {
  const hits: SnapPoint[] = [];
  for (let i = 1; i < left.length; i += 1) {
    const a = left[i - 1];
    const b = left[i];
    if (!a || !b) continue;
    for (let j = 1; j < right.length; j += 1) {
      const c = right[j - 1];
      const d = right[j];
      if (!c || !d) continue;
      const hit = segmentHit(a, b, c, d);
      if (hit) hits.push({ ...hit, kind: "intersection" });
      if (hits.length >= 24) return hits;
    }
  }
  return hits;
}

function segmentHit(a: number[], b: number[], c: number[], d: number[]): { x: number; y: number } | null {
  const rx = (b[0] ?? 0) - (a[0] ?? 0);
  const ry = (b[1] ?? 0) - (a[1] ?? 0);
  const sx = (d[0] ?? 0) - (c[0] ?? 0);
  const sy = (d[1] ?? 0) - (c[1] ?? 0);
  const denom = rx * sy - ry * sx;
  if (Math.abs(denom) < 1e-12) return null;
  const qx = (c[0] ?? 0) - (a[0] ?? 0);
  const qy = (c[1] ?? 0) - (a[1] ?? 0);
  const t = (qx * sy - qy * sx) / denom;
  const u = (qx * ry - qy * rx) / denom;
  if (t < 0 || t > 1 || u < 0 || u > 1) return null;
  return { x: (a[0] ?? 0) + t * rx, y: (a[1] ?? 0) + t * ry };
}

function quadVertex(a: number[], b: number[], c: number[]): { x: number; y: number } | null {
  const x0 = a[0] ?? 0;
  const y0 = a[1] ?? 0;
  const x1 = b[0] ?? 0;
  const y1 = b[1] ?? 0;
  const x2 = c[0] ?? 0;
  const y2 = c[1] ?? 0;
  const span = x2 - x0;
  if (Math.abs(span) < 1e-9 || Math.abs(x1 - x0) < 1e-9 || Math.abs(x2 - x1) < 1e-9) return null;
  const d01 = (y1 - y0) / (x1 - x0);
  const d12 = (y2 - y1) / (x2 - x1);
  const quad = (d12 - d01) / span;
  if (Math.abs(quad) < 1e-9) return null;
  const linear = d01 - quad * (x0 + x1);
  const constant = y0 - quad * x0 * x0 - linear * x0;
  const x = -linear / (2 * quad);
  if (x < Math.min(x0, x2) || x > Math.max(x0, x2)) return null;
  return { x, y: quad * x * x + linear * x + constant };
}

function thin(line: number[][], max: number): number[][] {
  if (line.length <= max) return line;
  const step = Math.ceil(line.length / max);
  const out: number[][] = [];
  for (let index = 0; index < line.length; index += step) {
    const point = line[index];
    if (point) out.push(point);
  }
  const last = line[line.length - 1];
  if (last && out[out.length - 1] !== last) out.push(last);
  return out;
}

function dedupe(hits: SnapPoint[]): SnapPoint[] {
  const kept: SnapPoint[] = [];
  for (const hit of hits) {
    if (!Number.isFinite(hit.x) || !Number.isFinite(hit.y)) continue;
    const same = kept.find((item) => Math.hypot(item.x - hit.x, item.y - hit.y) < 0.08);
    if (!same) {
      kept.push(hit);
      continue;
    }
    if (RANK[hit.kind] < RANK[same.kind]) {
      same.x = hit.x;
      same.y = hit.y;
      same.kind = hit.kind;
    }
  }
  return kept;
}
