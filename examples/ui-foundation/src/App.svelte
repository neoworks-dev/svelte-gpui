<!--
  Every foundation component in each of its variants, sizes and states, in light and dark.
-->
<script lang="ts">
  import { Icon, Scroll, Text, View } from "@neoworks/svelte-gpui/ui";
  import {
    Badge,
    Button,
    Card,
    Chip,
    Divider,
    IconButton,
    ListRow,
    buttonLook,
    cardLook,
    chipLook,
    iconButtonLook,
    listRowLook,
    currentTheme,
    themeMode,
    toggleThemeMode,
    typeStyle,
    type ButtonVariant,
    type IconButtonVariant,
  } from "@neoworks/svelte-gpui-ui";
  import { arrowRight, bell, caretDown, envelope, folder, gear, heart, heartFilled, plus, star, starFilled, trash } from "./icons";
  import Row from "./Row.svelte";
  import Section from "./Section.svelte";
  import StatePreview from "./StatePreview.svelte";

  const theme = $derived(currentTheme());
  const buttonVariants: ButtonVariant[] = ["primary", "secondary", "ghost", "accent", "danger"];
  const iconButtonVariants: IconButtonVariant[] = ["standard", "filled", "tonal", "outlined"];

  let favourite = $state(false);
  let pinned = $state(true);
  let filters = $state({ unread: true, starred: false, mine: false });
  let tags = $state(["design", "native"]);
  let pressedCount = $state(0);
  let cardPresses = $state(0);

  const chipOptions = { applied: false, elevated: false, hasLeading: false, hasTrailingIcon: false, removable: false };

  function toggleLabel(): string {
    if (themeMode() === "light") {
      return "Switch to dark";
    }
    return "Switch to light";
  }

  function removeTag(tag: string): void {
    tags = tags.filter((other) => other !== tag);
  }
</script>

<View style={{ size: "full", background: theme.colors.canvas }}>
  <View style={{ direction: "row", align: "center", justify: "between", padding: 20, paddingBottom: 8 }}>
    <Text style={{ ...typeStyle("title"), color: theme.colors.text }}>Foundation components, {themeMode()}</Text>
    <Button variant="primary" onpress={toggleThemeMode}>
      <Text>{toggleLabel()}</Text>
    </Button>
  </View>

  <Scroll style={{ flex: 1 }}>
    <View style={{ direction: "row", gap: 40, padding: 20, align: "start" }}>
      <View style={{ flex: 1, gap: 24 }}>
        <Section title="Button">
          {#each buttonVariants as variant (variant)}
            <Row caption={variant}>
              <Button {variant} size="sm"><Text>Small</Text></Button>
              <Button {variant} onpress={() => pressedCount++}><Text>Medium {pressedCount}</Text></Button>
              <Button {variant} size="lg"><Text>Large</Text></Button>
              <Button {variant} disabled><Text>Disabled</Text></Button>
            </Row>
          {/each}
          <Row caption="icons">
            <Button variant="primary" icon={plus}><Text>New</Text></Button>
            <Button trailingIcon={arrowRight}><Text>Continue</Text></Button>
            <Button variant="accent" round icon={plus} />
            <Button variant="secondary" round size="sm" icon={gear} />
            <Button variant="ghost" round size="lg" icon={bell} />
          </Row>
          <Button variant="primary" full><Text>Full width</Text></Button>
        </Section>

        <Section title="IconButton">
          {#each iconButtonVariants as variant (variant)}
            <Row caption={variant}>
              <IconButton {variant} size="sm" icon={star} label="Small" />
              <IconButton {variant} icon={star} label="Medium" />
              <IconButton {variant} size="lg" icon={star} label="Large" />
              <IconButton {variant} selected={favourite} icon={heart} selectedIcon={heartFilled} label="Favourite" onpress={() => (favourite = !favourite)} />
              <IconButton {variant} selected={pinned} icon={star} selectedIcon={starFilled} label="Pin" onpress={() => (pinned = !pinned)} />
              <IconButton {variant} icon={trash} label="Delete" disabled />
            </Row>
          {/each}
        </Section>

        <Section title="Chip">
          <Row caption="assist">
            <Chip label="Add to calendar" icon={plus} />
            <Chip label="Elevated" icon={folder} elevated />
            <Chip label="Disabled" disabled />
          </Row>
          <Row caption="filter">
            <Chip variant="filter" label="Unread" bind:selected={filters.unread} />
            <Chip variant="filter" label="Starred" bind:selected={filters.starred} />
            <Chip variant="filter" label="Mine" trailingIcon={caretDown} bind:selected={filters.mine} />
            <Chip variant="filter" label="Disabled" selected disabled />
          </Row>
          <Row caption="input">
            {#each tags as tag (tag)}
              <Chip variant="input" label={tag} onremove={() => removeTag(tag)} />
            {/each}
            <Chip variant="input" label="Fixed" />
          </Row>
          <Row caption="suggestion">
            <Chip variant="suggestion" label="Reply with thanks" />
            <Chip variant="suggestion" label="Sounds good" icon={heart} />
          </Row>
        </Section>
      </View>

      <View style={{ flex: 1, gap: 24 }}>
        <Section title="Badge">
          <Row caption="counts">
            <Badge count={3} />
            <Badge count={42} tone="neutral" />
            <Badge count={120} tone="accent" />
            <Badge dot />
            <Badge count={0} />
          </Row>
          <Row caption="attached">
            <Badge count={3}><IconButton icon={bell} label="Notifications" /></Badge>
            <Badge count={120} tone="accent"><IconButton icon={envelope} label="Mail" variant="tonal" /></Badge>
            <Badge dot><IconButton icon={gear} label="Settings" variant="outlined" /></Badge>
          </Row>
        </Section>

        <Section title="Divider">
          <Divider />
          <Divider strong />
          <Divider inset="start" />
          <Divider inset="both" />
          <View style={{ direction: "row", align: "center", gap: 12, height: 32 }}>
            <Text style={{ ...typeStyle("body"), color: theme.colors.text }}>Left</Text>
            <Divider orientation="vertical" />
            <Text style={{ ...typeStyle("body"), color: theme.colors.text }}>Middle</Text>
            <Divider orientation="vertical" strong />
            <Text style={{ ...typeStyle("body"), color: theme.colors.text }}>Right</Text>
          </View>
        </Section>

        <Section title="Card">
          <View style={{ direction: "row", gap: 14 }}>
            <Card headline="Outlined" supportingText="Groups content on the page." padding="sm" style={{ flex: 1 }} />
            <Card variant="filled" headline="Filled" supportingText="Sets a quiet area apart." padding="sm" style={{ flex: 1 }} />
          </View>
          <View style={{ direction: "row", gap: 14 }}>
            <Card
              variant="elevated"
              subhead="Press me"
              supportingText="An elevated card that is a button. Pressed {cardPresses} times."
              padding="sm"
              style={{ flex: 1 }}
              onpress={() => cardPresses++}
            />
            <Card variant="outlined" subhead="With actions" supportingText="Buttons sit bottom right." padding="sm" style={{ flex: 1 }}>
              {#snippet actions()}
                <Button variant="ghost" size="sm"><Text>Later</Text></Button>
                <Button variant="primary" size="sm"><Text>Open</Text></Button>
              {/snippet}
            </Card>
          </View>
        </Section>

        <Section title="ListRow">
          <ListRow title="Plain row" />
          <ListRow title="With a subtitle" subtitle="Detail on a second line" />
          <ListRow title="Opens something" subtitle="Press me" chevron onpress={() => pressedCount++}>
            {#snippet leading()}
              <IconButton icon={folder} label="Folder" variant="tonal" />
            {/snippet}
          </ListRow>
          <ListRow title="With a trailing badge" chevron onpress={() => pressedCount++}>
            {#snippet trailing()}
              <Badge count={7} tone="neutral" />
            {/snippet}
          </ListRow>
        </Section>

        <Section title="States: rest, hover, pressed, focus">
          <StatePreview caption="primary" look={buttonLook(theme, { variant: "primary", size: "md", full: false, round: false })}>
            <Text>Save</Text>
          </StatePreview>
          <StatePreview caption="secondary" look={buttonLook(theme, { variant: "secondary", size: "md", full: false, round: false })}>
            <Text>Cancel</Text>
          </StatePreview>
          <StatePreview caption="ghost" look={buttonLook(theme, { variant: "ghost", size: "md", full: false, round: false })}>
            <Text>Skip</Text>
          </StatePreview>
          <StatePreview caption="danger" look={buttonLook(theme, { variant: "danger", size: "md", full: false, round: false })}>
            <Text>Delete</Text>
          </StatePreview>
          <StatePreview caption="icon" look={iconButtonLook(theme, { variant: "tonal", size: "md", selected: undefined })}>
            <Icon svg={star} size={20} />
          </StatePreview>
          <StatePreview caption="chip" look={chipLook(theme, chipOptions)}>
            <Text>Starred</Text>
          </StatePreview>
          <StatePreview caption="applied" look={chipLook(theme, { ...chipOptions, applied: true })}>
            <Text>Unread</Text>
          </StatePreview>
          <StatePreview caption="card" look={cardLook(theme, { variant: "elevated", interactive: true })}>
            <View style={{ padding: 12 }}><Text>Card</Text></View>
          </StatePreview>
          <StatePreview caption="row" look={listRowLook(theme, true)}>
            <Text>Row</Text>
          </StatePreview>
        </Section>
      </View>
    </View>
  </Scroll>
</View>
