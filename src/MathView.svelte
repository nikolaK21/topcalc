<script lang="ts">
  import katex from "katex";
  import "katex/dist/katex.min.css";

  let {
    interpretation,
    resultLabel,
    resultLatex,
    note,
  }: {
    interpretation: string;
    resultLabel: string | null;
    resultLatex: string | null;
    note: string | null;
  } = $props();

  function typeset(node: HTMLElement, latex: string) {
    const paint = (value: string) => {
      katex.render(value || "", node, { throwOnError: false, displayMode: true });
    };
    paint(latex);
    return {
      update(value: string) {
        paint(value);
      },
    };
  }
</script>

<section class="pods">
  <div class="pod">
    <div class="kicker">Input</div>
    <div class="math" use:typeset={interpretation}></div>
  </div>
  {#if resultLatex}
    <div class="pod">
      <div class="kicker">{resultLabel ?? "Result"}</div>
      <div class="math" use:typeset={resultLatex}></div>
      {#if note}
        <p class="note">{note}</p>
      {/if}
    </div>
  {/if}
</section>

<style>
  .pods {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0 14px 8px;
  }
  .pod {
    background: color-mix(in srgb, var(--text, white) 5%, transparent);
    border-radius: 10px;
    padding: 6px 10px 4px;
  }
  .kicker {
    color: var(--muted, #8b93a7);
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .math {
    overflow-x: auto;
    padding: 2px 0 4px;
  }
  .math :global(.katex) {
    color: var(--text, #f4f6fb);
    font-size: 1.28rem;
  }
  .note {
    margin: 0 0 6px;
    color: var(--muted, #8b93a7);
    font-size: 12px;
  }
</style>
