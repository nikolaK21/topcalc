export type Binding = { name: string; value: number };

export type Job = {
  expression: string;
  plot: string;
  domain: string;
  label: string;
};

export type SliderDef = {
  name: string;
  value: number | null;
  values: number[] | null;
  pending: string | null;
};

export type FnDef = { name: string; body: string };

export type Compiled =
  | { type: "empty" }
  | { type: "note" }
  | { type: "slider"; slider: SliderDef }
  | { type: "functionDef"; fn: FnDef; jobs: Job[] }
  | { type: "plot"; jobs: Job[]; query: string | null }
  | { type: "value"; expression: string }
  | { type: "error"; message: string };

const RESERVED = new Set([
  "y", "x", "r", "t", "theta", "pi", "e", "inf", "infinity",
  "sin", "cos", "tan", "sec", "csc", "cot", "asin", "acos", "atan", "asec", "acsc", "acot",
  "arcsin", "arccos", "arctan", "arcsec", "arccsc", "arccot",
  "sinh", "cosh", "tanh", "sech", "csch", "coth", "asinh", "acosh", "atanh",
  "sqrt", "abs", "ln", "log", "log10", "log2", "floor", "ceil", "ceiling", "round", "sign",
  "fact", "factorial", "gamma", "sum", "prod", "product", "lim", "limit", "exp", "i", "int", "integral",
]);

function stripLead(text: string): string {
  const trimmed = text.trim();
  return trimmed.startsWith("=") ? trimmed.slice(1).trim() : trimmed;
}

function splitTop(input: string, op: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < input.length; index += 1) {
    const ch = input[index];
    if (ch === "(" || ch === "{" || ch === "[") depth += 1;
    else if (ch === ")" || ch === "}" || ch === "]") depth -= 1;
    else if (depth === 0 && input.startsWith(op, index)) {
      parts.push(input.slice(start, index).trim());
      index += op.length - 1;
      start = index + 1;
    }
  }
  parts.push(input.slice(start).trim());
  return parts;
}

function relation(input: string): { left: string; op: string; right: string } | null {
  let depth = 0;
  for (let index = 0; index < input.length; index += 1) {
    const ch = input[index];
    if (ch === "(" || ch === "{" || ch === "[") depth += 1;
    else if (ch === ")" || ch === "}" || ch === "]") depth -= 1;
    else if (depth === 0 && (ch === "<" || ch === ">" || ch === "=")) {
      const two = input.slice(index, index + 2);
      const op = two === "<=" || two === ">=" ? two : ch;
      const left = input.slice(0, index).trim();
      const right = input.slice(index + op.length).trim();
      if (!left || !right) return null;
      return { left, op, right };
    }
  }
  return null;
}

function domainSuffix(input: string): { body: string; domain: string } {
  const match = input.match(/^(.*?)\{([^{}]*)\}\s*$/);
  if (!match) return { body: input.trim(), domain: "" };
  return { body: match[1].trim(), domain: match[2].trim() };
}

function mentions(text: string, name: string): boolean {
  return new RegExp(`\\b${name}\\b`, "i").test(text);
}

function numericList(source: string): number[] | null {
  const match = source.match(/^\[(.*)\]$/);
  if (!match) return null;
  const parts = match[1].split(",").map((part) => part.trim()).filter(Boolean);
  if (parts.length === 0) return null;
  const values = parts.map(Number);
  return values.every((value) => Number.isFinite(value)) ? values : null;
}

function numeric(source: string): number | null {
  if (!/^-?\d+(\.\d+)?([eE][+-]?\d+)?$/.test(source.trim())) return null;
  const value = Number(source);
  return Number.isFinite(value) ? value : null;
}

function replaceCalls(text: string, name: string, body: string): string {
  const pattern = new RegExp(`\\b${name}\\s*\\(`, "gi");
  let output = "";
  let cursor = 0;
  for (const match of text.matchAll(pattern)) {
    const start = match.index ?? 0;
    const open = start + match[0].length - 1;
    let depth = 0;
    let end = -1;
    for (let index = open; index < text.length; index += 1) {
      if (text[index] === "(") depth += 1;
      else if (text[index] === ")") {
        depth -= 1;
        if (depth === 0) {
          end = index;
          break;
        }
      }
    }
    if (end < 0) break;
    const arg = text.slice(open + 1, end);
    const replaced = body.replace(/\bx\b/gi, `(${arg})`);
    output += text.slice(cursor, start) + `(${replaced})`;
    cursor = end + 1;
  }
  output += text.slice(cursor);
  return output;
}

export function applyFunctions(text: string, functions: FnDef[]): string {
  let current = text;
  for (let pass = 0; pass < 4; pass += 1) {
    let next = current;
    for (const fn of functions) next = replaceCalls(next, fn.name, fn.body);
    if (next === current) break;
    current = next;
  }
  return current;
}

function foldIndexes(text: string): string[] {
  const names: string[] = [];
  const pattern = /\b(?:sum|prod|product)_\{([^{}]*)\}/gi;
  for (const match of text.matchAll(pattern)) {
    const lower = match[1] ?? "";
    const named = lower.match(/^\s*([A-Za-z][A-Za-z0-9]*)\s*=/);
    names.push(named ? named[1].toLowerCase() : "n");
  }
  return names;
}

function job(expression: string, plot: string, domain: string, label: string): Job {
  return { expression, plot, domain, label };
}

export function sliderSuggestions(text: string, defined: Iterable<string>): string[] {
  const known = new Set([...defined].map((name) => name.toLowerCase()));
  for (const name of foldIndexes(text)) known.add(name);
  const found: string[] = [];
  const seen = new Set<string>();
  let index = 0;
  while (index < text.length) {
    const ch = text[index];
    if (ch === '"' || ch === "“") break;
    if (!/[A-Za-z]/.test(ch)) {
      index += 1;
      continue;
    }
    let end = index + 1;
    while (end < text.length && /[A-Za-z]/.test(text[end])) end += 1;
    if (/^\s*\(\s*x\s*\)\s*=/.test(text.slice(end))) {
      index = end;
      continue;
    }
    let cursor = index;
    while (cursor < end) {
      const cut = nameCut(text.slice(cursor, end), known);
      const name = text.slice(cursor, cursor + cut).toLowerCase();
      const rest = text.slice(cursor + cut, end).toLowerCase();
      const differential = name === "d" && rest.startsWith("x");
      if (!differential && !RESERVED.has(name) && !known.has(name) && !seen.has(name)) {
        seen.add(name);
        found.push(name);
      }
      cursor += Math.max(1, cut);
    }
    index = end;
  }
  return found;
}

function nameCut(word: string, known: Set<string>): number {
  const lower = word.toLowerCase();
  if (isKnownName(lower, known)) return lower.length;
  for (let len = lower.length - 1; len >= 1; len -= 1) {
    if (isKnownName(lower.slice(0, len), known)) return len;
  }
  return 1;
}

function isKnownName(name: string, known: Set<string>): boolean {
  return RESERVED.has(name) || known.has(name);
}

export function boundaryDashed(text: string): boolean {
  const compared = relation(stripLead(text));
  return compared?.op === "<" || compared?.op === ">";
}

export function dependsOnPlotVariable(text: string): boolean {
  const plotVars = new Set(["x", "y", "t", "theta"]);
  let index = 0;
  while (index < text.length) {
    const ch = text[index];
    if (ch === '"' || ch === "“") break;
    if (!/[A-Za-z]/.test(ch)) {
      index += 1;
      continue;
    }
    let end = index + 1;
    while (end < text.length && /[A-Za-z]/.test(text[end])) end += 1;
    let cursor = index;
    while (cursor < end) {
      const cut = nameCut(text.slice(cursor, end), new Set());
      const name = text.slice(cursor, cursor + cut).toLowerCase();
      if (plotVars.has(name)) return true;
      cursor += Math.max(1, cut);
    }
    index = end;
  }
  return false;
}

export function compileLine(raw: string, functions: FnDef[]): Compiled {
  const text = applyFunctions(stripLead(raw), functions);
  if (!text) return { type: "empty" };
  if (text.startsWith('"') || text.startsWith("“")) return { type: "note" };

  const defined = text.match(/^([A-Za-z][A-Za-z0-9]*)\(x\)\s*=\s*(.+)$/);
  if (defined) {
    const name = defined[1].toLowerCase();
    const body = defined[2].trim();
    return {
      type: "functionDef",
      fn: { name, body },
      jobs: [job(body, "function", "", `${name}(x) = ${body}`)],
    };
  }

  const pair = text.match(/^\(\s*([^,]+)\s*,\s*([^)]+)\)$/);
  if (pair) {
    const left = pair[1].trim();
    const right = pair[2].trim();
    const parametric = mentions(left, "t") || mentions(right, "t") || mentions(left, "theta") || mentions(right, "theta");
    if (parametric) return { type: "plot", jobs: [job(`${left}||${right}`, "parametric", "", text)], query: null };
    return { type: "plot", jobs: [job(`${left}||${right}`, "point", "", text)], query: null };
  }

  if (text.startsWith("{") && text.includes(":")) {
    return { type: "plot", jobs: [job(text, "piecewise", "", text)], query: null };
  }

  const compared = relation(text);
  if (compared && compared.op !== "=") {
    const { body, domain } = domainSuffix(text);
    const again = relation(body) ?? compared;
    if (again.left.toLowerCase() === "y" || again.right.toLowerCase() === "y") {
      const flipped = again.right.toLowerCase() === "y";
      const boundary = flipped ? again.left : again.right;
      let plot = again.op === "<" || again.op === "<=" ? "below" : "above";
      if (flipped) plot = plot === "below" ? "above" : "below";
      return { type: "plot", jobs: [job(boundary, plot, domain, text)], query: null };
    }
    return { type: "plot", jobs: [job(body, "region", domain, text)], query: null };
  }

  const equation = splitTop(text, "=");
  if (equation.length === 2 && equation[0] && equation[1]) {
    const [left, right] = equation;
    const { body: rightBody, domain } = domainSuffix(right);
    if (left.toLowerCase() === "y") {
      if (rightBody.startsWith("{") && rightBody.includes(":")) {
        return { type: "plot", jobs: [job(rightBody, "piecewise", domain, text)], query: text };
      }
      return { type: "plot", jobs: [job(rightBody, "function", domain, text)], query: text };
    }
    if (right.toLowerCase() === "y") {
      return { type: "plot", jobs: [job(left, "function", "", text)], query: text };
    }
    if (left.toLowerCase() === "r") {
      return { type: "plot", jobs: [job(right, "polar", "", text)], query: null };
    }
    if (right.toLowerCase() === "r") {
      return { type: "plot", jobs: [job(left, "polar", "", text)], query: null };
    }
    if (left.toLowerCase() === "x" && !mentions(right, "x") && !mentions(right, "y")) {
      return { type: "plot", jobs: [job(right, "vertical", "", text)], query: text };
    }
    const name = left.match(/^([A-Za-z][A-Za-z0-9]*)$/);
    if (name && !RESERVED.has(name[1].toLowerCase()) && !mentions(right, "x") && !mentions(right, "y") && !mentions(right, "t")) {
      const list = numericList(right);
      const value = numeric(right);
      return {
        type: "slider",
        slider: {
          name: name[1].toLowerCase(),
          value,
          values: list,
          pending: value == null && list == null ? right : null,
        },
      };
    }
    if (mentions(`${left}${right}`, "x") && mentions(`${left}${right}`, "y")) {
      return { type: "plot", jobs: [job(`(${left})-(${right})`, "implicit", "", text)], query: text };
    }
    return { type: "plot", jobs: [job(text, "function", "", text)], query: text };
  }

  const { body, domain } = domainSuffix(text);
  if (body.startsWith("{") && body.includes(":")) {
    return { type: "plot", jobs: [job(body, "piecewise", domain, text)], query: null };
  }
  if (!dependsOnPlotVariable(body)) return { type: "value", expression: body };
  return { type: "plot", jobs: [job(body, "function", domain, text)], query: text };
}

export function fanJobs(jobs: Job[], sliders: SliderDef[]): Job[] {
  const lists = sliders.filter((slider) => slider.values && slider.values.length > 0);
  const scalars = sliders.filter((slider) => slider.value != null && !slider.values);
  if (lists.length === 0) return jobs;
  let combos: Binding[][] = [scalars.map((slider) => ({ name: slider.name, value: slider.value as number }))];
  for (const list of lists) {
    const next: Binding[][] = [];
    for (const combo of combos) {
      for (const value of list.values ?? []) {
        next.push([...combo, { name: list.name, value }]);
        if (next.length >= 24) break;
      }
      if (next.length >= 24) break;
    }
    combos = next;
  }
  return combos.flatMap((combo, index) =>
    jobs.map((item) => ({
      ...item,
      label: combos.length > 1 ? `${item.label} (${index + 1})` : item.label,
      expression: substitute(item.expression, combo),
      domain: substitute(item.domain, combo),
    })),
  );
}

export function substitute(text: string, bindings: Binding[]): string {
  return text.replace(/\b[A-Za-z][A-Za-z0-9]*\b/g, (name) => {
    const found = bindings.find((binding) => binding.name === name.toLowerCase());
    if (!found) return name;
    return found.value < 0 ? `(${found.value})` : String(found.value);
  });
}

export function bindingsFrom(sliders: SliderDef[]): Binding[] {
  return sliders
    .filter((slider) => slider.value != null && !slider.values)
    .map((slider) => ({ name: slider.name, value: slider.value as number }));
}

export function linearFit(xs: number[], ys: number[]): { slope: number; intercept: number } | null {
  if (xs.length < 2 || xs.length !== ys.length) return null;
  const n = xs.length;
  const sx = xs.reduce((sum, value) => sum + value, 0);
  const sy = ys.reduce((sum, value) => sum + value, 0);
  const sxx = xs.reduce((sum, value) => sum + value * value, 0);
  const sxy = xs.reduce((sum, value, index) => sum + value * ys[index], 0);
  const denom = n * sxx - sx * sx;
  if (Math.abs(denom) < 1e-12) return null;
  const slope = (n * sxy - sx * sy) / denom;
  const intercept = (sy - slope * sx) / n;
  return { slope, intercept };
}

export function quadraticFit(xs: number[], ys: number[]): { a: number; b: number; c: number } | null {
  if (xs.length < 3 || xs.length !== ys.length) return null;
  const sums = [0, 0, 0, 0, 0];
  const rhs = [0, 0, 0];
  for (let index = 0; index < xs.length; index += 1) {
    const x = xs[index];
    const y = ys[index];
    const powers = [1, x, x * x, x ** 3, x ** 4];
    for (let power = 0; power < 5; power += 1) sums[power] += powers[power];
    rhs[0] += y;
    rhs[1] += x * y;
    rhs[2] += x * x * y;
  }
  const matrix = [
    [sums[0], sums[1], sums[2]],
    [sums[1], sums[2], sums[3]],
    [sums[2], sums[3], sums[4]],
  ];
  const solved = solve3(matrix, rhs);
  if (!solved) return null;
  return { c: solved[0], b: solved[1], a: solved[2] };
}

function solve3(matrix: number[][], rhs: number[]): number[] | null {
  const a = matrix.map((row, index) => [...row, rhs[index]]);
  for (let col = 0; col < 3; col += 1) {
    let pivot = col;
    for (let row = col + 1; row < 3; row += 1) {
      if (Math.abs(a[row][col]) > Math.abs(a[pivot][col])) pivot = row;
    }
    if (Math.abs(a[pivot][col]) < 1e-12) return null;
    [a[col], a[pivot]] = [a[pivot], a[col]];
    const scale = a[col][col];
    for (let index = col; index < 4; index += 1) a[col][index] /= scale;
    for (let row = 0; row < 3; row += 1) {
      if (row === col) continue;
      const factor = a[row][col];
      for (let index = col; index < 4; index += 1) a[row][index] -= factor * a[col][index];
    }
  }
  return [a[0][3], a[1][3], a[2][3]];
}
