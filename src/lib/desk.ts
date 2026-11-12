export function deskExpression(text: string): string {
  const trimmed = text.trim();
  if (trimmed.startsWith("=")) return trimmed.slice(1).trim();
  return trimmed;
}

export function hasTopLevelEquals(text: string): boolean {
  let depth = 0;
  for (const ch of text) {
    if (ch === "(" || ch === "{" || ch === "[") depth += 1;
    else if (ch === ")" || ch === "}" || ch === "]") depth = Math.max(0, depth - 1);
    else if (ch === "=" && depth === 0) return true;
  }
  return false;
}

/** An equation, or math that starts with `=`, belongs in the graph window. */
export function wantsDesk(text: string): boolean {
  const trimmed = text.trim();
  if (!trimmed || trimmed === "=") return false;
  if (trimmed.startsWith("=")) return deskExpression(trimmed).length > 0;
  return hasTopLevelEquals(trimmed);
}
