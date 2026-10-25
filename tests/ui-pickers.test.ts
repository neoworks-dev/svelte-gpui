import { describe, expect, test } from "bun:test";
import {
  addMonthsClamped,
  appendDigit,
  dateFromDigits,
  digitsAfterEdit,
  digitsFromDate,
  formatMask,
  formatMonthYear,
  isSelectable,
  monthGrid,
  sameDay,
  startOfWeek,
  weekdayHeaders,
} from "../packages/svelte-gpui-ui/src/components/calendar";
import {
  arcDots,
  indeterminateBarLeft,
  percentOf,
  ringRadius,
  ringStrokeWidth,
} from "../packages/svelte-gpui-ui/src/components/progressModel";
import {
  displayedHour,
  minuteValues,
  periodOf,
  timeText,
  timeWithHour,
  timeWithMinute,
  timeWithPeriod,
} from "../packages/svelte-gpui-ui/src/components/time";
import {
  clampIndex,
  indexAfterDrag,
  indexAfterTurn,
  rowTop,
  targetIndexFor,
  turnAfterWheel,
  valueAt,
  visibleIndexes,
  wheelItemHeight,
  wheelPixelsPerItem,
  wrapIndex,
} from "../packages/svelte-gpui-ui/src/components/wheelModel";

function dayNumbers(week: Date[]): number[] {
  return week.map((day) => day.getDate());
}

describe("month grid", () => {
  test("starts on the weekday before the first when weeks start on Monday", () => {
    const weeks = monthGrid(new Date(2026, 9, 14), 1);
    expect(weeks).toHaveLength(6);
    expect(dayNumbers(weeks[0])).toEqual([28, 29, 30, 1, 2, 3, 4]);
    expect(dayNumbers(weeks[5])).toEqual([2, 3, 4, 5, 6, 7, 8]);
  });

  test("starts on the Sunday before the first when weeks start on Sunday", () => {
    const weeks = monthGrid(new Date(2026, 9, 14), 0);
    expect(dayNumbers(weeks[0])).toEqual([27, 28, 29, 30, 1, 2, 3]);
  });

  test("a month that begins on the first weekday has no days of the month before", () => {
    const weeks = monthGrid(new Date(2026, 5, 1), 1);
    expect(dayNumbers(weeks[0])).toEqual([1, 2, 3, 4, 5, 6, 7]);
    expect(weeks[0][0].getMonth()).toBe(5);
  });

  test("counts 29 days in a leap February", () => {
    const days = monthGrid(new Date(2028, 1, 10), 1).flat();
    const inFebruary = days.filter((day) => day.getMonth() === 1);
    expect(inFebruary).toHaveLength(29);
  });

  test("crosses a year boundary", () => {
    const weeks = monthGrid(new Date(2026, 11, 3), 1);
    const lastDay = weeks[5][6];
    expect(lastDay.getFullYear()).toBe(2027);
    expect(lastDay.getMonth()).toBe(0);
  });

  test("weekday headers follow the first row", () => {
    expect(weekdayHeaders(monthGrid(new Date(2026, 9, 1), 1)[0])).toEqual(["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]);
    expect(weekdayHeaders(monthGrid(new Date(2026, 9, 1), 0)[0])).toEqual(["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]);
  });
});

describe("dates and limits", () => {
  const minimum = new Date(2026, 9, 10);
  const maximum = new Date(2026, 9, 20);

  test("the limits themselves can be chosen, the days outside them cannot", () => {
    expect(isSelectable(new Date(2026, 9, 10, 23, 59), minimum, maximum)).toBe(true);
    expect(isSelectable(new Date(2026, 9, 20, 0, 1), minimum, maximum)).toBe(true);
    expect(isSelectable(new Date(2026, 9, 9), minimum, maximum)).toBe(false);
    expect(isSelectable(new Date(2026, 9, 21), minimum, maximum)).toBe(false);
  });

  test("without limits every day can be chosen", () => {
    expect(isSelectable(new Date(1900, 0, 1), undefined, undefined)).toBe(true);
  });

  test("moving by months keeps the day or falls back to the last day of a short month", () => {
    expect(addMonthsClamped(new Date(2026, 0, 31), 1)).toEqual(new Date(2026, 1, 28));
    expect(addMonthsClamped(new Date(2028, 0, 31), 1)).toEqual(new Date(2028, 1, 29));
    expect(addMonthsClamped(new Date(2026, 2, 15), -3)).toEqual(new Date(2025, 11, 15));
  });

  test("the start of a week depends on the first weekday", () => {
    const wednesday = new Date(2026, 9, 14);
    expect(startOfWeek(wednesday, 1)).toEqual(new Date(2026, 9, 12));
    expect(startOfWeek(wednesday, 0)).toEqual(new Date(2026, 9, 11));
    expect(sameDay(startOfWeek(new Date(2026, 9, 12), 1), new Date(2026, 9, 12))).toBe(true);
  });

  test("the month and year are written out in English", () => {
    expect(formatMonthYear(new Date(2026, 9, 14))).toBe("October 2026");
  });
});

describe("typing a date", () => {
  test("a month digit from 2 and a day digit from 4 get a leading zero", () => {
    expect(appendDigit("", "1")).toBe("1");
    expect(appendDigit("", "9")).toBe("09");
    expect(appendDigit("12", "5")).toBe("1205");
    expect(appendDigit("12", "2")).toBe("122");
  });

  test("numbers outside a month or day are ignored", () => {
    expect(appendDigit("1", "3")).toBe("1");
    expect(appendDigit("0", "0")).toBe("0");
    expect(appendDigit("123", "2")).toBe("123");
    expect(appendDigit("120", "0")).toBe("120");
  });

  test("the mask adds a separator once the next part starts", () => {
    expect(formatMask("12")).toBe("12");
    expect(formatMask("122")).toBe("12/2");
    expect(formatMask("1225")).toBe("12/25");
    expect(formatMask("12252")).toBe("12/25/2");
    expect(formatMask("12252026")).toBe("12/25/2026");
  });

  test("eight digits make a date unless the day does not exist", () => {
    expect(dateFromDigits("12252026")).toEqual(new Date(2026, 11, 25));
    expect(dateFromDigits("02302026")).toBeNull();
    expect(dateFromDigits("02292028")).toEqual(new Date(2028, 1, 29));
    expect(dateFromDigits("0229202")).toBeNull();
  });

  test("a date reads back as the digits it came from", () => {
    expect(digitsFromDate(new Date(2026, 0, 5))).toBe("01052026");
  });

  test("one typed digit follows the segment rules while a deletion or paste takes the digits as they are", () => {
    expect(digitsAfterEdit("", "9")).toBe("09");
    expect(digitsAfterEdit("09", "095")).toBe("0905");
    expect(digitsAfterEdit("0905", "09/0")).toBe("090");
    expect(digitsAfterEdit("", "12/25/2026 ")).toBe("12252026");
    expect(digitsAfterEdit("12252026", "12/25/20269")).toBe("12252026");
  });
});

describe("time", () => {
  test("a twelve hour clock shows midnight and noon as 12", () => {
    expect(displayedHour("00:15", true)).toBe(12);
    expect(displayedHour("12:15", true)).toBe(12);
    expect(displayedHour("13:15", true)).toBe(1);
    expect(displayedHour("13:15", false)).toBe(13);
  });

  test("the text follows the clock", () => {
    expect(timeText("14:30", false)).toBe("14:30");
    expect(timeText("14:05", true)).toBe("2:05 PM");
    expect(timeText("00:00", true)).toBe("12:00 AM");
  });

  test("choosing an hour on a twelve hour clock keeps the period", () => {
    expect(timeWithHour("14:30", 12, true)).toBe("12:30");
    expect(timeWithHour("09:05", 12, true)).toBe("00:05");
    expect(timeWithHour("14:30", 3, true)).toBe("15:30");
    expect(timeWithHour("14:30", 7, false)).toBe("07:30");
  });

  test("switching the period moves the hour by twelve", () => {
    expect(timeWithPeriod("09:05", "pm")).toBe("21:05");
    expect(timeWithPeriod("21:05", "am")).toBe("09:05");
    expect(timeWithPeriod("12:00", "am")).toBe("00:00");
    expect(timeWithPeriod("00:10", "pm")).toBe("12:10");
    expect(periodOf("12:00")).toBe("pm");
    expect(periodOf("11:59")).toBe("am");
  });

  test("choosing a minute keeps the hour", () => {
    expect(timeWithMinute("14:30", 5)).toBe("14:05");
  });

  test("minutes are offered at the step, up to the hour", () => {
    expect(minuteValues(15)).toEqual([0, 15, 30, 45]);
    expect(minuteValues(5)).toHaveLength(12);
    expect(minuteValues(25)).toEqual([0, 25, 50]);
  });
});

describe("wheel column", () => {
  test("wrapping goes round both ways", () => {
    expect(wrapIndex(-1, 24)).toBe(23);
    expect(wrapIndex(25, 24)).toBe(1);
    expect(valueAt([5, 10, 15], -4)).toBe(15);
  });

  test("a wrapping column turns the short way to a value", () => {
    expect(targetIndexFor(23, 0, 24, true)).toBe(24);
    expect(targetIndexFor(0, 23, 24, true)).toBe(-1);
    expect(targetIndexFor(100, 100 % 24, 24, true)).toBe(100);
    expect(targetIndexFor(5, 9, 24, true)).toBe(9);
  });

  test("a bounded column goes straight to the value", () => {
    expect(targetIndexFor(0, 100, 201, false)).toBe(100);
  });

  test("turning stops at the ends of a bounded column and carries on in a wrapping one", () => {
    expect(indexAfterTurn(2, 5, 4, false)).toBe(3);
    expect(indexAfterTurn(1, -5, 4, false)).toBe(0);
    expect(indexAfterTurn(2, 5, 4, true)).toBe(7);
    expect(clampIndex(9, 4)).toBe(3);
  });

  test("dragging down pulls earlier rows into the middle", () => {
    expect(indexAfterDrag(10, wheelItemHeight * 2)).toBe(8);
    expect(indexAfterDrag(10, -wheelItemHeight / 2)).toBe(10.5);
  });

  test("the first wheel movement after a pause turns one row, the rest turn a row per step", () => {
    const first = turnAfterWheel(0, 3, true);
    expect(first.steps).toBe(1);
    expect(first.accumulated).toBe(0);
    const partial = turnAfterWheel(0, wheelPixelsPerItem / 2, false);
    expect(partial.steps).toBe(0);
    const completed = turnAfterWheel(partial.accumulated, wheelPixelsPerItem / 2 + wheelPixelsPerItem, false);
    expect(completed.steps).toBe(2);
    expect(turnAfterWheel(0, -wheelPixelsPerItem * 2, false).steps).toBe(-2);
    expect(turnAfterWheel(0, -3, true).steps).toBe(-1);
    expect(turnAfterWheel(7, 0, true).steps).toBe(0);
  });

  test("a wrapping column draws rows past its ends and a bounded one does not", () => {
    expect(visibleIndexes(0, 24, true)).toEqual([-3, -2, -1, 0, 1, 2, 3]);
    expect(visibleIndexes(0, 24, false)).toEqual([0, 1, 2, 3]);
    expect(visibleIndexes(23.4, 24, false)).toEqual([20, 21, 22, 23]);
  });

  test("the centred row sits two rows from the top", () => {
    expect(rowTop(5, 5)).toBe(wheelItemHeight * 2);
    expect(rowTop(6, 5.5)).toBe(wheelItemHeight * 2.5);
  });
});

describe("progress", () => {
  test("the fill is a percentage held inside 0 to 100", () => {
    expect(percentOf(25, 100)).toBe(25);
    expect(percentOf(3, 8)).toBe(37.5);
    expect(percentOf(150, 100)).toBe(100);
    expect(percentOf(-4, 100)).toBe(0);
    expect(percentOf(undefined, 100)).toBe(0);
    expect(percentOf(5, 0)).toBe(0);
  });

  test("the sliding bar enters from the left edge and leaves past the right one", () => {
    expect(indeterminateBarLeft(0, 500)).toBe(-200);
    expect(indeterminateBarLeft(1, 500)).toBe(500);
    expect(indeterminateBarLeft(0.5, 500)).toBe(150);
  });

  test("the ring is scaled from its 48 unit box", () => {
    const geometry = { size: 96, thickness: 4 };
    expect(ringStrokeWidth(geometry)).toBe(8);
    expect(ringRadius(geometry)).toBe(44);
  });

  test("an arc starts at twelve o'clock and runs clockwise", () => {
    const geometry = { size: 48, thickness: 4 };
    const dots = arcDots(geometry, 0, 90);
    const first = dots[0];
    const last = dots[dots.length - 1];
    expect(first.x).toBeCloseTo(24, 5);
    expect(first.y).toBeCloseTo(24 - 22, 5);
    expect(last.x).toBeCloseTo(24 + 22, 5);
    expect(last.y).toBeCloseTo(24, 5);
  });

  test("an empty arc has no dots and a full one closes on itself", () => {
    const geometry = { size: 48, thickness: 4 };
    expect(arcDots(geometry, 0, 0)).toEqual([]);
    const full = arcDots(geometry, 0, 360);
    expect(full[0].x).toBeCloseTo(full[full.length - 1].x, 5);
    expect(full[0].y).toBeCloseTo(full[full.length - 1].y, 5);
  });
});
