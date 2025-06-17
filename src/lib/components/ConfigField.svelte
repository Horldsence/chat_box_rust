<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let label: string;
  export let value: any = "";
  export let type:
    | "text"
    | "number"
    | "password"
    | "select"
    | "toggle"
    | "range"
    | "textarea"
    | "file"
    | "color" = "text";
  export let placeholder: string = "";
  export let description: string = "";
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let readonly: boolean = false;
  export let error: string = "";
  export let warning: string = "";
  export let info: string = "";

  // Select specific props
  export let options: Array<{
    value: any;
    label: string;
    description?: string;
    disabled?: boolean;
  }> = [];

  // Range specific props
  export let min: number = 0;
  export let max: number = 100;
  export let step: number = 1;

  // File specific props
  export let accept: string = "";
  export let multiple: boolean = false;

  // Textarea specific props
  export let rows: number = 3;
  export let maxlength: number | undefined = undefined;

  // Additional validation
  export let pattern: string = "";
  export let minLength: number | undefined = undefined;
  export let maxLength: number | undefined = undefined;

  const dispatch = createEventDispatcher<{
    change: { value: any };
    input: { value: any };
    focus: void;
    blur: void;
    keydown: KeyboardEvent;
  }>();

  let inputElement: HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement;
  let isFocused = false;

  $: hasMessage = error || warning || info;
  $: messageType = error ? "error" : warning ? "warning" : "info";
  $: messageText = error || warning || info;
  $: isInvalid = !!error;

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement;

    if (type === "number") {
      value = (target as HTMLInputElement).valueAsNumber || 0;
    } else if (type === "toggle") {
      value = (target as HTMLInputElement).checked;
    } else if (type === "file") {
      value = (target as HTMLInputElement).files;
    } else {
      value = target.value;
    }

    dispatch("input", { value });
  }

  function handleChange(event: Event) {
    handleInput(event);
    dispatch("change", { value });
  }

  function handleFocus() {
    isFocused = true;
    dispatch("focus");
  }

  function handleBlur() {
    isFocused = false;
    dispatch("blur");
  }

  function handleKeydown(event: KeyboardEvent) {
    dispatch("keydown", event);
  }

  export function focus() {
    inputElement?.focus();
  }

  export function blur() {
    inputElement?.blur();
  }

  export function select() {
    if (inputElement && "select" in inputElement) {
      inputElement.select();
    }
  }
</script>

<div
  class="config-field"
  class:disabled
  class:readonly
  class:focused={isFocused}
  class:invalid={isInvalid}
>
  <div class="field-header">
    <label for={`field-${label}`} class="field-label">
      {label}
      {#if required}
        <span class="required-indicator" aria-label="必填" title="必填">*</span>
      {/if}
    </label>

    {#if description}
      <p class="field-description">{description}</p>
    {/if}
  </div>

  <div class="field-input">
    {#if type === "select"}
      <select
        bind:this={inputElement}
        id={`field-${label}`}
        bind:value
        {disabled}
        {required}
        aria-invalid={isInvalid}
        aria-describedby={hasMessage ? `${label}-message` : undefined}
        on:change={handleChange}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeydown}
        class="select-input"
      >
        {#each options as option}
          <option value={option.value} disabled={option.disabled} title={option.description}>
            {option.label}
          </option>
        {/each}
      </select>
    {:else if type === "toggle"}
      <label class="toggle-container">
        <input
          bind:this={inputElement}
          id={`field-${label}`}
          type="checkbox"
          bind:checked={value}
          {disabled}
          {required}
          aria-invalid={isInvalid}
          aria-describedby={hasMessage ? `${label}-message` : undefined}
          on:change={handleChange}
          on:focus={handleFocus}
          on:blur={handleBlur}
          on:keydown={handleKeydown}
          class="toggle-input"
        />
        <span class="toggle-slider" aria-hidden="true"></span>
        <span class="toggle-label">
          {value ? "开启" : "关闭"}
        </span>
      </label>
    {:else if type === "range"}
      <div class="range-container">
        <input
          bind:this={inputElement}
          id={`field-${label}`}
          type="range"
          bind:value
          {min}
          {max}
          {step}
          {disabled}
          {required}
          aria-invalid={isInvalid}
          aria-describedby={hasMessage ? `${label}-message` : undefined}
          on:input={handleInput}
          on:change={handleChange}
          on:focus={handleFocus}
          on:blur={handleBlur}
          on:keydown={handleKeydown}
          class="range-input"
        />
        <div class="range-values">
          <span class="range-min">{min}</span>
          <span class="range-current">{value}</span>
          <span class="range-max">{max}</span>
        </div>
      </div>
    {:else if type === "textarea"}
      <textarea
        bind:this={inputElement}
        id={`field-${label}`}
        bind:value
        {placeholder}
        {disabled}
        {readonly}
        {required}
        {rows}
        maxlength={maxlength || maxLength}
        minlength={minLength}
        aria-invalid={isInvalid}
        aria-describedby={hasMessage ? `${label}-message` : undefined}
        on:input={handleInput}
        on:change={handleChange}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeydown}
        class="textarea-input"
      ></textarea>
    {:else if type === "file"}
      <input
        bind:this={inputElement}
        id={`field-${label}`}
        type="file"
        {accept}
        {multiple}
        {disabled}
        {required}
        aria-invalid={isInvalid}
        aria-describedby={hasMessage ? `${label}-message` : undefined}
        on:change={handleChange}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeydown}
        class="file-input"
      />
    {:else}
      <input
        bind:this={inputElement}
        id={`field-${label}`}
        {type}
        bind:value
        {placeholder}
        {disabled}
        {readonly}
        {required}
        {pattern}
        minlength={minLength}
        maxlength={maxLength}
        {min}
        {max}
        {step}
        aria-invalid={isInvalid}
        aria-describedby={hasMessage ? `${label}-message` : undefined}
        on:input={handleInput}
        on:change={handleChange}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeydown}
        class="text-input"
      />
    {/if}
  </div>

  {#if hasMessage}
    <div id={`${label}-message`} class="field-message {messageType}" role="alert">
      <svg
        class="message-icon"
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        aria-hidden="true"
      >
        {#if messageType === "error"}
          <circle cx="8" cy="8" r="8" fill="currentColor" />
          <path d="M6 6l4 4M10 6l-4 4" stroke="white" stroke-width="2" stroke-linecap="round" />
        {:else if messageType === "warning"}
          <path d="M8 1l7 14H1L8 1z" fill="currentColor" />
          <path d="M8 6v4M8 12h.01" stroke="white" stroke-width="2" stroke-linecap="round" />
        {:else}
          <circle cx="8" cy="8" r="8" fill="currentColor" />
          <path
            d="M6 8l2 2 4-4"
            stroke="white"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        {/if}
      </svg>
      <span class="message-text">{messageText}</span>
    </div>
  {/if}
</div>

<style>
  .config-field {
    display: block;
    width: 100%;
  }

  .config-field.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .config-field.readonly {
    opacity: 0.75;
  }

  .field-header {
    margin-bottom: 0.5rem;
  }

  .field-label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
    margin-bottom: 0.25rem;
  }

  :global(.dark) .field-label {
    color: #e5e7eb;
  }

  .required-indicator {
    color: #ef4444;
    margin-left: 0.25rem;
  }

  .field-description {
    font-size: 0.75rem;
    color: #6b7280;
    line-height: 1.625;
  }

  :global(.dark) .field-description {
    color: #9ca3af;
  }

  .field-input {
    position: relative;
  }

  /* Base input styles */
  .text-input,
  .textarea-input,
  .select-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    background-color: white;
    color: #111827;
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }

  :global(.dark) .text-input,
  :global(.dark) .textarea-input,
  :global(.dark) .select-input {
    border-color: #4b5563;
    background-color: #1f2937;
    color: #f9fafb;
  }

  .text-input::placeholder,
  .textarea-input::placeholder {
    color: #9ca3af;
  }

  :global(.dark) .text-input::placeholder,
  :global(.dark) .textarea-input::placeholder {
    color: #6b7280;
  }

  .text-input:focus,
  .textarea-input:focus,
  .select-input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  :global(.dark) .text-input:focus,
  :global(.dark) .textarea-input:focus,
  :global(.dark) .select-input:focus {
    border-color: #60a5fa;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  .config-field.invalid .text-input,
  .config-field.invalid .textarea-input,
  .config-field.invalid .select-input {
    border-color: #fca5a5;
  }

  :global(.dark) .config-field.invalid .text-input,
  :global(.dark) .config-field.invalid .textarea-input,
  :global(.dark) .config-field.invalid .select-input {
    border-color: #dc2626;
  }

  .config-field.invalid .text-input:focus,
  .config-field.invalid .textarea-input:focus,
  .config-field.invalid .select-input:focus {
    border-color: #ef4444;
    box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
  }

  :global(.dark) .config-field.invalid .text-input:focus,
  :global(.dark) .config-field.invalid .textarea-input:focus,
  :global(.dark) .config-field.invalid .select-input:focus {
    border-color: #f87171;
    box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.1);
  }

  /* Select specific styles */
  .select-input {
    cursor: pointer;
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
    background-position: right 0.5rem center;
    background-repeat: no-repeat;
    background-size: 1.5em 1.5em;
    padding-right: 2.5rem;
  }

  /* Toggle specific styles */
  .toggle-container {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .toggle-input {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .toggle-slider {
    position: relative;
    display: inline-flex;
    height: 1.5rem;
    width: 2.75rem;
    flex-shrink: 0;
    border-radius: 9999px;
    border: 2px solid transparent;
    background-color: #e5e7eb;
    transition: background-color 0.2s ease-in-out;
  }

  :global(.dark) .toggle-slider {
    background-color: #4b5563;
  }

  .toggle-slider::before {
    content: "";
    pointer-events: none;
    display: inline-block;
    height: 1.25rem;
    width: 1.25rem;
    transform: translateX(0);
    border-radius: 9999px;
    background-color: white;
    box-shadow:
      0 1px 3px 0 rgba(0, 0, 0, 0.1),
      0 1px 2px 0 rgba(0, 0, 0, 0.06);
    transition: transform 0.2s ease-in-out;
  }

  .toggle-input:checked + .toggle-slider {
    background-color: #2563eb;
  }

  :global(.dark) .toggle-input:checked + .toggle-slider {
    background-color: #3b82f6;
  }

  .toggle-input:checked + .toggle-slider::before {
    transform: translateX(1.25rem);
  }

  .toggle-input:focus + .toggle-slider {
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  :global(.dark) .toggle-input:focus + .toggle-slider {
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  .toggle-label {
    margin-left: 0.75rem;
    font-size: 0.875rem;
    color: #374151;
  }

  :global(.dark) .toggle-label {
    color: #e5e7eb;
  }

  /* Range specific styles */
  .range-container {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .range-input {
    width: 100%;
    height: 0.5rem;
    background-color: #e5e7eb;
    border-radius: 0.5rem;
    appearance: none;
    cursor: pointer;
  }

  :global(.dark) .range-input {
    background-color: #4b5563;
  }

  .range-input::-webkit-slider-thumb {
    appearance: none;
    width: 1rem;
    height: 1rem;
    background-color: #2563eb;
    border-radius: 50%;
    cursor: pointer;
  }

  :global(.dark) .range-input::-webkit-slider-thumb {
    background-color: #3b82f6;
  }

  .range-input::-moz-range-thumb {
    width: 1rem;
    height: 1rem;
    background-color: #2563eb;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  :global(.dark) .range-input::-moz-range-thumb {
    background-color: #3b82f6;
  }

  .range-values {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: #6b7280;
  }

  :global(.dark) .range-values {
    color: #9ca3af;
  }

  .range-current {
    font-weight: 500;
    color: #374151;
  }

  :global(.dark) .range-current {
    color: #e5e7eb;
  }

  /* File input styles */
  .file-input {
    width: 100%;
    font-size: 0.875rem;
    color: #6b7280;
    cursor: pointer;
  }

  :global(.dark) .file-input {
    color: #9ca3af;
  }

  .file-input::file-selector-button {
    margin-right: 0.75rem;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    border: 0;
    font-size: 0.875rem;
    font-weight: 500;
    background-color: #dbeafe;
    color: #1d4ed8;
    cursor: pointer;
  }

  :global(.dark) .file-input::file-selector-button {
    background-color: rgba(30, 58, 138, 0.2);
    color: #93c5fd;
  }

  .file-input:hover::file-selector-button {
    background-color: #bfdbfe;
  }

  :global(.dark) .file-input:hover::file-selector-button {
    background-color: rgba(30, 58, 138, 0.3);
  }

  /* Message styles */
  .field-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding: 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  .field-message.error {
    background-color: #fef2f2;
    color: #b91c1c;
  }

  :global(.dark) .field-message.error {
    background-color: rgba(185, 28, 28, 0.2);
    color: #fca5a5;
  }

  .field-message.warning {
    background-color: #fffbeb;
    color: #d97706;
  }

  :global(.dark) .field-message.warning {
    background-color: rgba(217, 119, 6, 0.2);
    color: #fcd34d;
  }

  .field-message.info {
    background-color: #eff6ff;
    color: #1d4ed8;
  }

  :global(.dark) .field-message.info {
    background-color: rgba(29, 78, 216, 0.2);
    color: #93c5fd;
  }

  .message-icon {
    flex-shrink: 0;
  }

  .message-text {
    flex: 1;
  }

  /* Responsive adjustments */
  @media (max-width: 640px) {
    .toggle-container {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.5rem;
    }

    .toggle-label {
      margin-left: 0;
    }

    .range-values {
      font-size: 0.75rem;
    }
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .text-input,
    .textarea-input,
    .select-input {
      border-width: 2px;
    }

    .toggle-slider {
      border: 2px solid #9ca3af;
    }
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .text-input,
    .textarea-input,
    .select-input,
    .toggle-slider,
    .range-input {
      transition: none;
    }
  }
</style>
