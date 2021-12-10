export function countLines(s: string): number {
  if (!s) {
    return 0;
  }
  return s
    .trim()
    .split("\n")
    .filter((v) => v).length;
}
