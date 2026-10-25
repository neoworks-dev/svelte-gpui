// Calendar arithmetic for the date picker. Dates are local calendar days; the time of day is ignored.

export type WeekStart = 0 | 1;

const monthNames = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

const weekdayNames = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

export const monthShortLabels = monthNames.map((name) => name.slice(0, 3));

export function startOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

export function addMonths(date: Date, months: number): Date {
  return new Date(date.getFullYear(), date.getMonth() + months, 1);
}

export function addDays(date: Date, days: number): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() + days);
}

export function sameDay(first: Date, second: Date): boolean {
  return (
    first.getFullYear() === second.getFullYear() &&
    first.getMonth() === second.getMonth() &&
    first.getDate() === second.getDate()
  );
}

export function sameMonth(first: Date, second: Date): boolean {
  return first.getFullYear() === second.getFullYear() && first.getMonth() === second.getMonth();
}

/** Six weeks of seven days that contain the month, starting on `weekStartsOn`. */
export function monthGrid(anchor: Date, weekStartsOn: WeekStart): Date[][] {
  const first = startOfMonth(anchor);
  const offset = (first.getDay() - weekStartsOn + 7) % 7;
  const gridStart = new Date(first.getFullYear(), first.getMonth(), 1 - offset);
  const weeks: Date[][] = [];
  for (let weekIndex = 0; weekIndex < 6; weekIndex++) {
    const week: Date[] = [];
    for (let dayIndex = 0; dayIndex < 7; dayIndex++) {
      week.push(addDays(gridStart, weekIndex * 7 + dayIndex));
    }
    weeks.push(week);
  }
  return weeks;
}

export function isBeforeDay(first: Date, second: Date): boolean {
  const firstDay = new Date(first.getFullYear(), first.getMonth(), first.getDate()).getTime();
  const secondDay = new Date(second.getFullYear(), second.getMonth(), second.getDate()).getTime();
  return firstDay < secondDay;
}

/** Whether `day` lies inside the optional `minimum` and `maximum` dates, both included. */
export function isSelectable(day: Date, minimum: Date | undefined, maximum: Date | undefined): boolean {
  if (minimum !== undefined && isBeforeDay(day, minimum)) {
    return false;
  }
  if (maximum !== undefined && isBeforeDay(maximum, day)) {
    return false;
  }
  return true;
}

export function startOfWeek(date: Date, weekStartsOn: WeekStart): Date {
  const offset = (date.getDay() - weekStartsOn + 7) % 7;
  return addDays(date, -offset);
}

/** Moves by whole months and keeps the day when the target month is long enough, otherwise uses its last day. */
export function addMonthsClamped(date: Date, months: number): Date {
  const target = new Date(date.getFullYear(), date.getMonth() + months, 1);
  const lastDay = new Date(target.getFullYear(), target.getMonth() + 1, 0).getDate();
  return new Date(target.getFullYear(), target.getMonth(), Math.min(date.getDate(), lastDay));
}

/** A date from its parts, or null when the day overflows the month (31 February). */
export function buildDate(year: number, monthIndex: number, day: number): Date | null {
  const date = new Date(year, monthIndex, day);
  const overflowed = date.getFullYear() !== year || date.getMonth() !== monthIndex || date.getDate() !== day;
  if (overflowed) {
    return null;
  }
  return date;
}

/** The mmddyyyy digits of a date, zero padded. */
export function digitsFromDate(date: Date): string {
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return month + day + String(date.getFullYear()).padStart(4, "0");
}

/** Digits shown as mm/dd/yyyy, adding separators only once the next segment starts. */
export function formatMask(digits: string): string {
  let masked = digits.slice(0, 2);
  if (digits.length > 2) {
    masked += "/" + digits.slice(2, 4);
  }
  if (digits.length > 4) {
    masked += "/" + digits.slice(4, 8);
  }
  return masked;
}

export function dateFromDigits(digits: string): Date | null {
  if (digits.length !== 8) {
    return null;
  }
  return buildDate(Number(digits.slice(4, 8)), Number(digits.slice(0, 2)) - 1, Number(digits.slice(2, 4)));
}

/** The digits in typed or pasted text, at most the eight a date has. */
export function digitsOf(text: string): string {
  return text.replace(/\D/g, "").slice(0, 8);
}

function withinRange(value: number, minimum: number, maximum: number): boolean {
  return value >= minimum && value <= maximum;
}

/**
 * Adds a typed digit to the buffer with per-segment limits, so segments advance by themselves:
 * a month digit from 2 or a day digit from 4 gets a leading zero, and out-of-range numbers are ignored.
 */
export function appendDigit(digits: string, key: string): string {
  const position = digits.length;
  const digit = Number(key);
  if (position === 0 && digit >= 2) {
    return "0" + key;
  }
  if (position === 2 && digit >= 4) {
    return digits + "0" + key;
  }
  if (position === 1 && !withinRange(Number(digits + key), 1, 12)) {
    return digits;
  }
  if (position === 3 && !withinRange(Number(digits.slice(2) + key), 1, 31)) {
    return digits;
  }
  return digits + key;
}

/** "October 2026". */
export function formatMonthYear(date: Date): string {
  return `${monthNames[date.getMonth()]} ${date.getFullYear()}`;
}

/** The first two letters of the weekdays of one grid row, as the header above the days. */
export function weekdayHeaders(firstWeek: Date[]): string[] {
  return firstWeek.map((day) => weekdayNames[day.getDay()].slice(0, 2));
}

/**
 * The digit buffer after the field's text changed. One typed digit goes through `appendDigit`'s segment rules;
 * anything else, such as a deletion or a paste, takes the digits as they are.
 */
export function digitsAfterEdit(previousDigits: string, text: string): string {
  const editedDigits = digitsOf(text);
  const typedOneDigit = editedDigits.length === previousDigits.length + 1 && editedDigits.startsWith(previousDigits);
  if (typedOneDigit && previousDigits.length < 8) {
    return appendDigit(previousDigits, editedDigits.slice(-1));
  }
  return editedDigits;
}
