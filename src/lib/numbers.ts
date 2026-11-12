export type NumberMode = "decimal" | "integer" | "irrational" | "complex";

const PI = Math.PI;
const E = Math.E;

export function formatQuantity(value: number, mode: NumberMode = "decimal"): string {
  if (!Number.isFinite(value)) return "undefined";
  if (mode === "integer") return String(Math.round(value));
  if (mode === "irrational" || mode === "complex") return formatExact(value, 2e-3) ?? decimal(value);
  return decimal(value);
}

function decimal(value: number): string {
  if (Math.abs(value) < 1e-12) return "0";
  const rounded = Math.round(value * 1e10) / 1e10;
  let text = String(rounded);
  if (text.includes(".")) text = text.replace(/0+$/, "").replace(/\.$/, "");
  return text || "0";
}

function formatExact(value: number, tol: number): string | null {
  if (Math.abs(value) <= tol) return "0";
  const sign = value < 0 ? "-" : "";
  const magnitude = Math.abs(value);
  const whole = integerText(magnitude, tol);
  if (whole) return sign + whole;
  const pi = unitMultiple(magnitude / PI, "pi", tol);
  if (pi) return sign + pi;
  if (Math.abs(magnitude - E) <= tol * (1 + magnitude)) return sign + "e";
  for (let n = 2; n <= 48; n += 1) {
    if (!squareFree(n)) continue;
    const form = unitMultiple(magnitude / Math.sqrt(n), `sqrt(${n})`, tol);
    if (form) return sign + form;
  }
  const pair = rationalPair(magnitude, 12, tol);
  if (!pair) return null;
  const [numer, denom] = pair;
  if (denom === 1) return sign + String(numer);
  return `${sign}${numer}/${denom}`;
}

function integerText(value: number, tol: number): string | null {
  const rounded = Math.round(value);
  if (Math.abs(rounded) >= 1e15) return null;
  if (Math.abs(value - rounded) <= tol * (1 + Math.abs(value))) return String(rounded);
  return null;
}

function unitMultiple(value: number, unit: string, tol: number): string | null {
  const pair = rationalPair(value, 12, tol);
  if (!pair) return null;
  const numer = Math.abs(pair[0]);
  const denom = pair[1];
  if (numer === 0) return null;
  if (numer === 1 && denom === 1) return unit;
  if (denom === 1) return `${numer}*${unit}`;
  if (numer === 1) return `${unit}/${denom}`;
  return `(${numer}/${denom})*${unit}`;
}

function rationalPair(value: number, maxDen: number, tol: number): [number, number] | null {
  if (!Number.isFinite(value)) return null;
  for (let denom = 1; denom <= maxDen; denom += 1) {
    const numer = Math.round(value * denom);
    if (Math.abs(numer) > 240) continue;
    const approx = numer / denom;
    if (Math.abs(value - approx) <= tol * (1 + Math.abs(value))) return [numer, denom];
  }
  return null;
}

function squareFree(n: number): boolean {
  let x = n;
  let p = 2;
  while (p * p <= x) {
    if (x % (p * p) === 0) return false;
    if (x % p === 0) x = Math.floor(x / p);
    p += 1;
  }
  return true;
}
