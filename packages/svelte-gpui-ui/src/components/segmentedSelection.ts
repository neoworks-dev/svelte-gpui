/** The values after switching `value` on or off, keeping the rest in order. */
export function toggleValue(values: string[], value: string): string[] {
  if (values.includes(value)) {
    return values.filter((entry) => entry !== value);
  }
  return [...values, value];
}
