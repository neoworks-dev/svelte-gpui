<!-- Icons are SVG markup tinted by the text colour; images load near the window. -->
<script lang="ts">
  import { Button, Icon, Image, Text, View } from "@neoworks/svelte-gpui/ui";
  import Card from "../components/Card.svelte";
  import Segmented from "../components/Segmented.svelte";
  import { caption, colors, mono } from "../theme";

  import airplane from "@phosphor-icons/core/regular/airplane-tilt.svg";
  import bell from "@phosphor-icons/core/regular/bell.svg";
  import bug from "@phosphor-icons/core/regular/bug.svg";
  import camera from "@phosphor-icons/core/regular/camera.svg";
  import cloud from "@phosphor-icons/core/regular/cloud.svg";
  import code from "@phosphor-icons/core/regular/code.svg";
  import cube from "@phosphor-icons/core/regular/cube.svg";
  import flask from "@phosphor-icons/core/regular/flask.svg";
  import gear from "@phosphor-icons/core/regular/gear.svg";
  import git from "@phosphor-icons/core/regular/git-branch.svg";
  import heart from "@phosphor-icons/core/regular/heart.svg";
  import lightning from "@phosphor-icons/core/regular/lightning.svg";
  import moon from "@phosphor-icons/core/regular/moon.svg";
  import music from "@phosphor-icons/core/regular/music-note.svg";
  import palette from "@phosphor-icons/core/regular/palette.svg";
  import rocket from "@phosphor-icons/core/regular/rocket.svg";
  import star from "@phosphor-icons/core/regular/star.svg";
  import terminal from "@phosphor-icons/core/regular/terminal.svg";
  import heartFill from "@phosphor-icons/core/fill/heart-fill.svg";
  import starFill from "@phosphor-icons/core/fill/star-fill.svg";

  const icons = [
    ["airplane", airplane], ["bell", bell], ["bug", bug], ["camera", camera], ["cloud", cloud],
    ["code", code], ["cube", cube], ["flask", flask], ["gear", gear], ["git", git],
    ["lightning", lightning], ["moon", moon], ["music", music], ["palette", palette],
    ["rocket", rocket], ["terminal", terminal],
  ] as const;

  let liked = $state(false);
  let starred = $state(true);
  let fit = $state<"cover" | "contain" | "fill">("cover");
  let remote = $state<"loading" | "loaded" | "failed">("loading");
</script>

<View style={{ gap: 16 }}>
  <Card title="Icons" note="Phosphor's SVG files, imported as text. Hover one: its colour comes from the button's text colour.">
    <View style={{ direction: "row", wrap: true, gap: 6 }}>
      {#each icons as [name, svg] (name)}
        <Button
          style={{ width: 72, paddingY: 10, gap: 6, direction: "column", align: "center", radius: 8, color: colors.dim, transition: 120 }}
          hover={{ background: colors.raised, color: colors.accentText }}
        >
          <Icon {svg} size={22} />
          <Text style={{ fontSize: 11 }}>{name}</Text>
        </Button>
      {/each}
    </View>
    <View style={{ direction: "row", gap: 12, align: "center" }}>
      <Button style={{ gap: 6, color: liked ? colors.pink : colors.dim }} onpress={() => (liked = !liked)}>
        <Icon svg={liked ? heartFill : heart} size={20} />
        <Text>{liked ? "Liked" : "Like"}</Text>
      </Button>
      <Button style={{ gap: 6, color: starred ? colors.amber : colors.dim }} onpress={() => (starred = !starred)}>
        <Icon svg={starred ? starFill : star} size={20} />
        <Text>{starred ? "Starred" : "Star"}</Text>
      </Button>
    </View>
  </Card>

  <View style={{ direction: "row", gap: 16, align: "start" }}>
    <Card title="Images" note="A file next to the bundle, drawn with object fit and rounded corners." style={{ flex: 1 }}>
      <Segmented options={["cover", "contain", "fill"]} bind:value={fit} />
      <View style={{ direction: "row", gap: 10 }}>
        <Image src="assets/demo.png" {fit} style={{ width: 200, height: 120, radius: 8, background: colors.window }} />
        <Image src="assets/demo.png" {fit} style={{ width: 120, height: 120, radius: 60, background: colors.window }} />
      </View>
    </Card>
    <Card title="Remote" note="Fetched on a thread and cached on disk; onload and onerror report how it went." style={{ flex: 1 }}>
      <View style={{ direction: "row", gap: 12, align: "center" }}>
        <Image
          src="https://avatars.githubusercontent.com/u/9919?s=160"
          fit="cover"
          style={{ size: 64, radius: 32, background: colors.raised }}
          onload={() => (remote = "loaded")}
          onerror={() => (remote = "failed")}
        />
        <View style={{ gap: 2 }}>
          <Text>github.com/github</Text>
          <Text style={{ ...mono, color: remote === "loaded" ? colors.green : remote === "failed" ? colors.red : colors.faint }}>{remote}</Text>
        </View>
      </View>
      <Text style={caption}>Images load once they are within a screen of the window, decoded no larger than they are shown.</Text>
    </Card>
  </View>
</View>
