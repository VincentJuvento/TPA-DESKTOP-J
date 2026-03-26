<script lang="ts">
  let {
    label = '',
    type = 'text',
    value = $bindable(''),
    placeholder = '',
    required = false,
    rows = 4,
    options = [] as { value: string; label: string }[],
    disabled = false,
    hint = ''
  } = $props();

  // Generate a stable id from the label for accessibility
  const fieldId = label
    ? 'field-' + label.toLowerCase().replace(/[^a-z0-9]+/g, '-')
    : undefined;
</script>

<div class="field">
  {#if label}
    <label class="field-label" for={fieldId}>
      {label}{#if required}<span class="required">*</span>{/if}
    </label>
  {/if}
  {#if type === 'textarea'}
    <textarea class="field-input" id={fieldId} bind:value {placeholder} {rows} {disabled}></textarea>
  {:else if type === 'select'}
    <select class="field-input" id={fieldId} bind:value {disabled}>
      <option value="">-- Select --</option>
      {#each options as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  {:else}
    <input class="field-input" id={fieldId} {type} bind:value {placeholder} {required} {disabled} />
  {/if}
  {#if hint}
    <span class="field-hint">{hint}</span>
  {/if}
</div>

<style>
  .field { display: flex; flex-direction: column; gap: 0.375rem; }
  .field-label {
    font-family: 'Space Mono', monospace;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #8fa3cc;
  }
  .required { color: #ff4466; margin-left: 2px; }
  .field-input {
    background: #05070f;
    border: 1px solid #1e2d4a;
    border-radius: 4px;
    color: #e8eeff;
    font-family: 'Outfit', sans-serif;
    font-size: 0.9rem;
    padding: 0.625rem 0.875rem;
    transition: border-color 0.15s;
    width: 100%;
    box-sizing: border-box;
  }
  .field-input:focus { outline: none; border-color: #3d7fff; box-shadow: 0 0 0 2px rgba(61,127,255,0.15); }
  .field-input:disabled { opacity: 0.5; cursor: not-allowed; }
  .field-input::placeholder { color: #4a5d82; }
  textarea.field-input { resize: vertical; min-height: 80px; }
  select.field-input { cursor: pointer; }
  .field-hint { font-size: 0.75rem; color: #4a5d82; }
</style>
