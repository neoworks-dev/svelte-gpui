<script lang="ts">
  import { renderMarkdown } from '@grove/src/renderer/src/lib/markdown';
  const html = renderMarkdown('Hello **world**\n\n- one\n- two\n\n`code`');
  console.log('HTML:', JSON.stringify(html));
</script>
<div class="p-3 text-xs">{@html html}</div>
