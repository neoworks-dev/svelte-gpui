/** A `#rrggbb` colour with an opacity from 0 to 1, as `#rrggbbaa`. */
export function withAlpha(color: string, opacity: number): string {
  const alpha = Math.round(Math.min(1, Math.max(0, opacity)) * 255);
  return `${color.slice(0, 7)}${alpha.toString(16).padStart(2, "0")}`;
}
