<!--
  A time field that opens two wheels, hours and minutes, and on a twelve hour clock a switch for AM and PM.
  The value is `HH:mm` on a 24 hour clock, whichever way it is shown.
-->
<script lang="ts">
  import { Button as ButtonPrimitive, Icon, Popover, Text, View } from "@neoworks/svelte-gpui/ui";
  import clock from "@phosphor-icons/core/regular/clock.svg";
  import { typeStyle } from "../theme";
  import { currentTheme } from "../theme.svelte";
  import { pickerPanelOffset, supportingTextLook, timeFieldLabelLook, timeFieldLook, timePanelLook, timeWheelWidth } from "./picker-look";
  import SegmentedButton from "./SegmentedButton.svelte";
  import {
    displayedHour,
    minuteOf,
    minuteValues,
    periodOf,
    timeText,
    timeWithHour,
    timeWithMinute,
    timeWithPeriod,
    twelveHours,
    twentyFourHours,
    type DayPeriod,
  } from "./time";
  import WheelColumn from "./WheelColumn.svelte";

  interface Props {
    /** Time as `HH:mm` on a 24 hour clock, whichever way it is shown. */
    value: string;
    onChange: (value: string) => void;
    /** Name of the field, shown inside it. */
    label?: string;
    /** Help under the field, in the danger colour when `error` is on. */
    supportingText?: string;
    error?: boolean;
    /** Shows hours from 1 to 12 with an AM and PM switch. */
    twelveHour?: boolean;
    /** Spacing of the minutes wheel, such as 5 or 15. */
    minuteStep?: number;
  }

  let { value, onChange, label = "Time", supportingText, error = false, twelveHour = false, minuteStep = 1 }: Props = $props();

  const periodOptions = [
    { value: "am", label: "AM" },
    { value: "pm", label: "PM" },
  ];

  const theme = $derived(currentTheme());

  let open = $state(false);

  const hourValues = $derived.by(() => {
    if (twelveHour) {
      return twelveHours;
    }
    return twentyFourHours;
  });
  const fieldLook = $derived(timeFieldLook(theme, { open, error }));
  const supportingLook = $derived(supportingTextLook(theme, error));

  function setPeriod(selection: string | string[]): void {
    onChange(timeWithPeriod(value, selection as DayPeriod));
  }
</script>

<View style={{ minWidth: 0 }}>
  <Popover
    bind:open
    side="bottom"
    align="start"
    offset={pickerPanelOffset}
    anchorStyle={{ alignSelf: "stretch" }}
    style={timePanelLook(theme)}
  >
    {#snippet trigger({ toggle })}
      <ButtonPrimitive style={fieldLook.style} hover={fieldLook.hover} focus={fieldLook.focus} onpress={toggle}>
        <Text style={{ flex: 1, minWidth: 0, truncate: true }}>{timeText(value, twelveHour)}</Text>
        <Icon svg={clock} size={20} color={theme.colors.muted} style={{ shrink: 0 }} />
        <Text style={timeFieldLabelLook(theme, { open, error })}>{label}</Text>
      </ButtonPrimitive>
    {/snippet}
    {#snippet children()}
      <View style={{ direction: "row", align: "center", gap: 8 }}>
        <WheelColumn
          values={hourValues}
          value={displayedHour(value, twelveHour)}
          onChange={(hour) => onChange(timeWithHour(value, hour, twelveHour))}
          label="Hours"
          style={{ width: timeWheelWidth }}
        />
        <Text style={{ ...typeStyle("title"), color: theme.colors.muted }}>:</Text>
        <WheelColumn
          values={minuteValues(minuteStep)}
          value={minuteOf(value)}
          onChange={(minute) => onChange(timeWithMinute(value, minute))}
          label="Minutes"
          style={{ width: timeWheelWidth }}
        />
        {#if twelveHour}
          <View style={{ paddingLeft: 8 }}>
            <SegmentedButton options={periodOptions} value={periodOf(value)} size="sm" onchange={setPeriod} />
          </View>
        {/if}
      </View>
    {/snippet}
  </Popover>

  {#if supportingText}
    <Text style={supportingLook}>{supportingText}</Text>
  {/if}
</View>
