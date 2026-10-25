<!--
  Chooses a day from a calendar. `title` is a heading-sized button for page headers; `field` is a form field you can
  also type in as mm/dd/yyyy. In the calendar, the arrows and Page keys move the day that has focus, the title steps from
  days to months to years, and Escape closes it. Month and weekday names are English.
-->
<script lang="ts">
  import { tick } from "svelte";
  import { Button as ButtonPrimitive, Icon, Popover, Text, View, type KeyEvent, type Style } from "@neoworks/svelte-gpui/ui";
  import calendarBlank from "@phosphor-icons/core/regular/calendar-blank.svg";
  import caretDown from "@phosphor-icons/core/regular/caret-down.svg";
  import caretLeft from "@phosphor-icons/core/regular/caret-left.svg";
  import caretRight from "@phosphor-icons/core/regular/caret-right.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import {
    addDays,
    addMonths,
    addMonthsClamped,
    dateFromDigits,
    digitsAfterEdit,
    digitsFromDate,
    formatMask,
    formatMonthYear,
    isSelectable,
    monthGrid,
    monthShortLabels,
    sameDay,
    sameMonth,
    startOfMonth,
    startOfWeek,
    weekdayHeaders,
    type WeekStart,
  } from "./calendar";
  import type { StateStyles } from "./look";
  import IconButton from "./IconButton.svelte";
  import {
    dateHeaderLook,
    datePanelLook,
    dateRowLook,
    dateTitleTriggerLook,
    dayLook,
    headerTitleLook,
    monthCellGap,
    monthLook,
    pickerPanelOffset,
    supportingTextLook,
    weekdayHeaderLook,
  } from "./picker-look";
  import TextField from "./TextField.svelte";
  import WheelColumn from "./WheelColumn.svelte";

  type Mode = "days" | "months" | "years";
  type Variant = "title" | "field";

  interface Props {
    /** The chosen day, or `null` for none yet. */
    value: Date | null;
    onChange: (date: Date) => void;
    /** Text of the `title` button instead of the month and year of `value` or the placeholder. */
    label?: string;
    /** Label inside the `field` variant. */
    fieldLabel?: string;
    placeholder?: string;
    /** Help under the `field` variant, in the danger colour when `error` is on. */
    supportingText?: string;
    error?: boolean;
    /** `title` is a heading-sized button for page headers, `field` is a form field you can also type in as mm/dd/yyyy. */
    variant?: Variant;
    /** 0 is Sunday, 1 is Monday. */
    weekStartsOn?: WeekStart;
    /** Earliest day that can be chosen. */
    min?: Date;
    /** Latest day that can be chosen. */
    max?: Date;
  }

  let {
    value,
    onChange,
    label,
    fieldLabel = "Date",
    placeholder = "Select date",
    supportingText,
    error = false,
    variant = "title",
    weekStartsOn = 1,
    min,
    max,
  }: Props = $props();

  const years = Array.from({ length: 2100 - 1900 + 1 }, (_, index) => 1900 + index);
  const dayOffsets: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -7, ArrowDown: 7 };
  const navigationLabels: Record<Mode, string> = { days: "month", months: "year", years: "decade" };
  const navigationSteps: Record<Mode, number> = { days: 1, months: 12, years: 120 };

  const theme = $derived(currentTheme());

  let open = $state(false);
  let mode: Mode = $state("days");
  let viewMonth = $state(startOfMonth(new Date()));
  let focusedDate = $state(new Date());
  let digits = $state("");
  let fieldText = $state("");
  let dayButtons: ReturnType<typeof ButtonPrimitive>[] = $state([]);
  let keyboardNavigating = $state(false);

  const weeks = $derived(monthGrid(viewMonth, weekStartsOn));
  const days = $derived(weeks.flat());
  const weekdayLabels = $derived(weekdayHeaders(weeks[0]));
  const fieldInvalid = $derived(error || (digits.length === 8 && dateFromDigits(digits) === null));
  const triggerText = $derived.by(() => {
    if (label !== undefined) {
      return label;
    }
    if (value === null) {
      return placeholder;
    }
    return formatMonthYear(value);
  });
  const triggerTextColor = $derived.by(() => {
    if (value === null && label === undefined) {
      return theme.colors.faint;
    }
    return theme.colors.text;
  });
  const triggerAnchorStyle = $derived.by(() => {
    if (variant === "field") {
      return { alignSelf: "stretch" as const };
    }
    return { alignSelf: "start" as const };
  });
  const headerText = $derived.by(() => {
    if (mode === "days") {
      return formatMonthYear(viewMonth);
    }
    return String(viewMonth.getFullYear());
  });

  $effect(() => {
    let chosenDigits = "";
    if (value !== null) {
      chosenDigits = digitsFromDate(value);
    }
    digits = chosenDigits;
    fieldText = formatMask(chosenDigits);
  });

  function isChosen(day: Date): boolean {
    return value !== null && sameDay(day, value);
  }

  function isChosenMonth(monthIndex: number): boolean {
    return value !== null && value.getFullYear() === viewMonth.getFullYear() && value.getMonth() === monthIndex;
  }

  function dayFocus(look: StateStyles): Style | undefined {
    if (keyboardNavigating) {
      return look.focus;
    }
    return undefined;
  }

  async function focusDay(day: Date): Promise<void> {
    await tick();
    const index = days.findIndex((candidate) => sameDay(candidate, day));
    dayButtons[index]?.focus();
  }

  function openCalendar(): void {
    let start = new Date();
    if (value !== null) {
      start = value;
    }
    focusedDate = start;
    viewMonth = startOfMonth(start);
    mode = "days";
    keyboardNavigating = false;
    open = true;
    focusDay(start);
  }

  function toggleCalendar(): void {
    if (open) {
      open = false;
      return;
    }
    openCalendar();
  }

  function pickDay(day: Date): void {
    if (!isSelectable(day, min, max)) {
      return;
    }
    digits = digitsFromDate(day);
    fieldText = formatMask(digits);
    onChange(day);
    open = false;
  }

  function moveFocus(target: Date): void {
    focusedDate = target;
    if (!sameMonth(target, viewMonth)) {
      viewMonth = startOfMonth(target);
    }
    focusDay(target);
  }

  function keyTarget(key: string): Date | undefined {
    const offset = dayOffsets[key];
    if (offset !== undefined) {
      return addDays(focusedDate, offset);
    }
    if (key === "PageUp") {
      return addMonthsClamped(focusedDate, -1);
    }
    if (key === "PageDown") {
      return addMonthsClamped(focusedDate, 1);
    }
    if (key === "Home") {
      return startOfWeek(focusedDate, weekStartsOn);
    }
    if (key === "End") {
      return addDays(startOfWeek(focusedDate, weekStartsOn), 6);
    }
    return undefined;
  }

  function handleGridKey(event: KeyEvent): void {
    const target = keyTarget(event.key);
    if (target === undefined) {
      return;
    }
    event.preventDefault();
    keyboardNavigating = true;
    moveFocus(target);
  }

  function pickMonth(monthIndex: number): void {
    viewMonth = new Date(viewMonth.getFullYear(), monthIndex, 1);
    mode = "days";
  }

  function pickYear(year: number): void {
    viewMonth = new Date(year, viewMonth.getMonth(), 1);
  }

  function navigate(direction: 1 | -1): void {
    viewMonth = addMonths(viewMonth, direction * navigationSteps[mode]);
  }

  function stepHeaderMode(): void {
    if (mode === "days") {
      mode = "months";
      return;
    }
    if (mode === "months") {
      mode = "years";
      return;
    }
    mode = "days";
  }

  function handleFieldInput(text: string): void {
    digits = digitsAfterEdit(digits, text);
    fieldText = formatMask(digits);
    const typed = dateFromDigits(digits);
    if (typed !== null) {
      onChange(typed);
    }
  }

  function commitField(): void {
    const typed = dateFromDigits(digits);
    if (typed !== null) {
      onChange(typed);
      return;
    }
    digits = "";
    if (value !== null) {
      digits = digitsFromDate(value);
    }
    fieldText = formatMask(digits);
  }
</script>

<View style={{ minWidth: 0 }}>
  <Popover
    bind:open
    side="bottom"
    align="start"
    offset={pickerPanelOffset}
    focusPanel={false}
    anchorStyle={triggerAnchorStyle}
    style={datePanelLook(theme)}
  >
    {#snippet trigger()}
      {#if variant === "field"}
        <TextField
          bind:value={fieldText}
          label={fieldLabel}
          placeholder="mm/dd/yyyy"
          error={fieldInvalid}
          oninput={handleFieldInput}
          onblur={commitField}
        >
          {#snippet trailing()}
            <IconButton icon={calendarBlank} label="Choose date" size="sm" onpress={toggleCalendar} />
          {/snippet}
        </TextField>
      {:else}
        {@const look = dateTitleTriggerLook(theme)}
        <ButtonPrimitive style={look.style} hover={look.hover} focus={look.focus} onpress={toggleCalendar}>
          <Text style={{ color: triggerTextColor, nowrap: true }}>{triggerText}</Text>
          <Icon svg={caretDown} size={16} color={theme.colors.muted} style={{ shrink: 0 }} />
        </ButtonPrimitive>
      {/if}
    {/snippet}
    {#snippet children()}
      {@const titleLook = headerTitleLook(theme)}
      <View style={dateHeaderLook}>
        <IconButton icon={caretLeft} label="Previous {navigationLabels[mode]}" size="sm" onpress={() => navigate(-1)} />
        <ButtonPrimitive style={titleLook.style} hover={titleLook.hover} focus={titleLook.focus} onpress={stepHeaderMode}>
          <Text>{headerText}</Text>
        </ButtonPrimitive>
        <IconButton icon={caretRight} label="Next {navigationLabels[mode]}" size="sm" onpress={() => navigate(1)} />
      </View>

      {#if mode === "days"}
        <View style={{ width: "full" }} onkey={handleGridKey}>
          <View style={dateRowLook}>
            {#each weekdayLabels as weekday, index (index)}
              <Text style={weekdayHeaderLook(theme)}>{weekday}</Text>
            {/each}
          </View>
          {#each weeks as week, weekIndex (weekIndex)}
            <View style={dateRowLook}>
              {#each week as day, dayIndex (dayIndex)}
                {@const index = weekIndex * 7 + dayIndex}
                {@const selectable = isSelectable(day, min, max)}
                {@const look = dayLook(theme, {
                  selected: isChosen(day),
                  today: sameDay(day, new Date()),
                  inViewedMonth: sameMonth(day, viewMonth),
                  selectable,
                })}
                <View style={{ flex: 1, align: "center" }}>
                  <ButtonPrimitive
                    bind:this={dayButtons[index]}
                    style={look.style}
                    hover={look.hover}
                    focus={dayFocus(look)}
                    disabledStyle={look.disabledStyle}
                    disabled={!selectable}
                    onpress={() => pickDay(day)}
                  >
                    <Text>{day.getDate()}</Text>
                  </ButtonPrimitive>
                </View>
              {/each}
            </View>
          {/each}
        </View>
      {:else if mode === "months"}
        <View style={{ gap: monthCellGap }}>
          {#each [0, 1, 2, 3] as rowIndex (rowIndex)}
            <View style={{ direction: "row", gap: monthCellGap }}>
              {#each monthShortLabels.slice(rowIndex * 3, rowIndex * 3 + 3) as monthLabel, columnIndex (columnIndex)}
                {@const monthIndex = rowIndex * 3 + columnIndex}
                {@const look = monthLook(theme, isChosenMonth(monthIndex))}
                <ButtonPrimitive style={look.style} hover={look.hover} focus={look.focus} onpress={() => pickMonth(monthIndex)}>
                  <Text>{monthLabel}</Text>
                </ButtonPrimitive>
              {/each}
            </View>
          {/each}
        </View>
      {:else}
        <WheelColumn values={years} value={viewMonth.getFullYear()} wrap={false} onChange={pickYear} label="Year" />
      {/if}
    {/snippet}
  </Popover>

  {#if variant === "field" && supportingText}
    <Text style={supportingTextLook(theme, fieldInvalid)}>{supportingText}</Text>
  {/if}
</View>
