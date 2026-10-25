// Time arithmetic for the time picker. A time is `HH:mm` on a 24 hour clock, whichever way it is shown.

export type DayPeriod = "am" | "pm";

export const twentyFourHours = Array.from({ length: 24 }, (_, index) => index);
export const twelveHours = Array.from({ length: 12 }, (_, index) => index + 1);

export function padTwoDigits(number: number): string {
  return number.toString().padStart(2, "0");
}

export function hourOf(time: string): number {
  return Number(time.slice(0, 2));
}

export function minuteOf(time: string): number {
  return Number(time.slice(3, 5));
}

export function periodOf(time: string): DayPeriod {
  if (hourOf(time) >= 12) {
    return "pm";
  }
  return "am";
}

/** The hour as the wheel shows it: 0 to 23, or 1 to 12 on a twelve hour clock where midnight is 12. */
export function displayedHour(time: string, twelveHour: boolean): number {
  const hour = hourOf(time);
  if (!twelveHour) {
    return hour;
  }
  if (hour % 12 === 0) {
    return 12;
  }
  return hour % 12;
}

/** The minutes the wheel offers, such as 0, 15, 30 and 45 for a step of 15. */
export function minuteValues(minuteStep: number): number[] {
  return Array.from({ length: Math.ceil(60 / minuteStep) }, (_, index) => index * minuteStep);
}

export function formatTime(hour: number, minute: number): string {
  return `${padTwoDigits(hour)}:${padTwoDigits(minute)}`;
}

/** The text on the field: `HH:mm`, or `h:mm AM` on a twelve hour clock. */
export function timeText(time: string, twelveHour: boolean): string {
  if (!twelveHour) {
    return time;
  }
  return `${displayedHour(time, true)}:${padTwoDigits(minuteOf(time))} ${periodOf(time).toUpperCase()}`;
}

/** The time after choosing `selectedHour` on the wheel, which keeps the current minute and, on a twelve hour clock, the period. */
export function timeWithHour(time: string, selectedHour: number, twelveHour: boolean): string {
  if (!twelveHour) {
    return formatTime(selectedHour, minuteOf(time));
  }
  const base = selectedHour % 12;
  if (periodOf(time) === "pm") {
    return formatTime(base + 12, minuteOf(time));
  }
  return formatTime(base, minuteOf(time));
}

export function timeWithMinute(time: string, selectedMinute: number): string {
  return formatTime(hourOf(time), selectedMinute);
}

export function timeWithPeriod(time: string, period: DayPeriod): string {
  const base = hourOf(time) % 12;
  if (period === "pm") {
    return formatTime(base + 12, minuteOf(time));
  }
  return formatTime(base, minuteOf(time));
}
