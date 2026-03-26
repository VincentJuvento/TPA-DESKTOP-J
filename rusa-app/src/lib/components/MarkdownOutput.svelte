<script lang="ts">
  import Table from './Table.svelte';

  let { markdown = '' } = $props<{ markdown: string }>();

  type Block =
    | { type: 'text'; text: string }
    | { type: 'table'; columns: { key: string; label: string }[]; rows: any[] };

  function splitPipes(line: string): string[] {
    const raw = line.split('|').map(s => s.trim());
    const start = raw[0] === '' ? 1 : 0;
    const end = raw[raw.length - 1] === '' ? raw.length - 1 : raw.length;
    return raw.slice(start, end);
  }

  function isSeparator(line: string): boolean {
    const t = line.trim();
    if (!t.includes('-') || !t.includes('|')) return false;
    const parts = splitPipes(t);
    if (!parts.length) return false;
    return parts.every(p => /^:?-{3,}:?$/.test(p.replace(/\s+/g, '')));
  }

  function parseBlocks(input: string): Block[] {
    const lines = input.replace(/\r\n/g, '\n').split('\n');
    const out: Block[] = [];
    let buf: string[] = [];

    const flushText = () => {
      const text = buf.join('\n').trimEnd();
      if (text.trim()) out.push({ type: 'text', text });
      buf = [];
    };

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const next = lines[i + 1] ?? '';
      const looksLikeHeader = line.includes('|') && isSeparator(next);
      if (!looksLikeHeader) {
        buf.push(line);
        continue;
      }

      flushText();

      const headers = splitPipes(line).map(h => h || '—');
      const columns = headers.map((h, idx) => ({ key: `c${idx}`, label: h }));
      i += 1;
      const rows: any[] = [];
      for (let j = i + 1; j < lines.length; j++) {
        const rline = lines[j];
        if (!rline.trim()) {
          i = j;
          break;
        }
        if (!rline.includes('|')) {
          i = j - 1;
          break;
        }
        const cells = splitPipes(rline);
        const row: any = {};
        columns.forEach((c, idx) => {
          row[c.key] = (cells[idx] ?? '').trim() || '—';
        });
        rows.push(row);
        if (j === lines.length - 1) i = j;
      }

      out.push({ type: 'table', columns, rows });
    }

    flushText();
    return out;
  }

  let blocks = $derived(parseBlocks(markdown || ''));
</script>

<div class="md">
  {#each blocks as b}
    {#if b.type === 'table'}
      <div class="md-block">
        <Table columns={b.columns} rows={b.rows} />
      </div>
    {:else}
      <pre class="md-text">{b.text}</pre>
    {/if}
  {/each}
</div>

<style>
  .md { display: flex; flex-direction: column; gap: 0.75rem; }
  .md-block { border-radius: 6px; overflow: hidden; }
  .md-text { background: #0d1528; border: 1px solid #1e2d4a; border-radius: 6px; padding: 0.75rem; color: #e8eeff; white-space: pre-wrap; font-family: 'Space Mono', monospace; font-size: 0.75rem; }
  .md-text:empty { display: none; }
</style>

