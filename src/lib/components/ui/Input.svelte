<script lang="ts">
  import { createEventDispatcher } from "svelte";

  // Props
  export let type: "text" | "email" | "password" | "number" | "tel" | "url" | "search" = "text";
  export let value = "";
  export let placeholder = "";
  export let disabled = false;
  export let readonly = false;
  export let required = false;
  export let autofocus = false;
  export let autocomplete = undefined;
  export let size: "sm" | "md" | "lg" = "md";
  export let variant: "default" | "filled" | "underline" = "default";
  export let error: string | boolean = false;
  export let success = false;
  export let fullWidth = false;
  export let min: number | undefined = undefined;
  export let max: number | undefined = undefined;
  export let step: number | undefined = undefined;
  export let maxlength: number | undefined = undefined;
  export let pattern: string | undefined = undefined;
  export let ariaLabel: string | undefined = undefined;
  export let ariaDescribedby: string | undefined = undefined;
  export let id: string | undefined = undefined;
  export let name: string | undefined = undefined;

  // Icon slots
  export let leftIcon = false;
  export let rightIcon = false;

  // Loading state
  export let loading = false;

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Internal state
  let focused = false;
  let inputElement: HTMLInputElement;

  // Generate unique ID if not provided
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`;
  const errorId = `${inputId}-error`;
  const helpId = `${inputId}-help`;

  // Handle input events
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    dispatch("input", { value, event });
  }

  function handleChange(event: Event) {
    dispatch("change", { value, event });
  }

  function handleFocus(event: FocusEvent) {
    focused = true;
    dispatch("focus", { value, event });
  }

  function handleBlur(event: FocusEvent) {
    focused = false;
    dispatch("blur", { value, event });
  }

  function handleKeydown(event: KeyboardEvent) {
    dispatch("keydown", { value, event });
  }

  function handleKeyup(event: KeyboardEvent) {
    dispatch("keyup", { value, event });
  }

  // Public methods
  export function focus() {
    inputElement?.focus();
  }

  export function blur() {
    inputElement?.blur();
  }

  export function select() {
    inputElement?.select();
  }

  // Computed classes
  $: classes = [
    "input-wrapper",
    `input-${size}`,
    `input-${variant}`,
    disabled && "input-disabled",
    readonly && "input-readonly",
    error && "input-error",
    success && "input-success",
    focused && "input-focused",
    loading && "input-loading",
    fullWidth && "input-full-width",
    leftIcon && "input-with-left-icon",
    rightIcon && "input-with-right-icon",
  ]
    .filter(Boolean)
    .join(" ");

  $: inputClasses = [
    "input",
    leftIcon && "input-with-left-padding",
    rightIcon && "input-with-right-padding",
  ]
    .filter(Boolean)
    .join(" ");

  $: ariaDescribedByValue =
    [ariaDescribedby, error && errorId, $$slots.help && helpId].filter(Boolean).join(" ") ||
    undefined;
</script>

<div class={classes}>
  <!-- Left Icon Slot -->
  {#if leftIcon}
    <div class="input-icon input-icon-left" aria-hidden="true">
      <slot name="leftIcon" />
    </div>
  {/if}

  <!-- Input Element -->
  <input
    bind:this={inputElement}
    {type}
    {id}
    {name}
    {value}
    {placeholder}
    {disabled}
    {readonly}
    {required}
    autofocus={autofocus || false}
    {autocomplete}
    {min}
    {max}
    {step}
    {maxlength}
    {pattern}
    class={inputClasses}
    aria-label={ariaLabel}
    aria-describedby={ariaDescribedByValue}
    aria-invalid={!!error}
    on:input={handleInput}
    on:change={handleChange}
    on:focus={handleFocus}
    on:blur={handleBlur}
    on:keydown={handleKeydown}
    on:keyup={handleKeyup}
  />

  <!-- Right Icon Slot -->
  {#if rightIcon}
    <div class="input-icon input-icon-right" aria-hidden="true">
      <slot name="rightIcon" />
    </div>
  {/if}

  <!-- Loading Spinner -->
  {#if loading}
    <div class="input-spinner" aria-hidden="true">
      <div class="spinner"></div>
    </div>
  {/if}
</div>

<!-- Help Text -->
{#if $$slots.help}
  <div id={helpId} class="input-help">
    <slot name="help" />
  </div>
{/if}

<!-- Error Message -->
{#if error && typeof error === "string"}
  <div id={errorId} class="input-error-message" role="alert">
    {error}
  </div>
{/if}

<style>
  .input-wrapper {
    position: relative;
    display: inline-flex;
    align-items: center;
    background: var(--color-bg-primary, white);
    border: 1px solid var(--color-border-medium, #cbd5e0);
    border-radius: var(--radius-md, 8px);
    transition: all 0.2s ease-in-out;
    box-sizing: border-box;
    font-family: inherit;
  }

  .input {
    flex: 1;
    width: 100%;
    border: none;
    outline: none;
    background: transparent;
    color: var(--color-text-primary, #2d3748);
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    box-sizing: border-box;
  }

  .input::placeholder {
    color: var(--color-text-muted, #718096);
    opacity: 1;
  }

  /* Sizes */
  .input-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
    line-height: 1.25rem;
    min-height: 2rem;
  }

  .input-sm .input {
    padding: 0;
  }

  .input-md {
    padding: 0.5rem 0.75rem;
    font-size: 1rem;
    line-height: 1.5rem;
    min-height: 2.5rem;
  }

  .input-md .input {
    padding: 0;
  }

  .input-lg {
    padding: 0.75rem 1rem;
    font-size: 1.125rem;
    line-height: 1.75rem;
    min-height: 3rem;
  }

  .input-lg .input {
    padding: 0;
  }

  /* Variants */
  .input-default {
    border-style: solid;
  }

  .input-filled {
    background: var(--color-bg-secondary, #f7fafc);
    border-color: transparent;
  }

  .input-underline {
    background: transparent;
    border: none;
    border-bottom: 2px solid var(--color-border-medium, #cbd5e0);
    border-radius: 0;
  }

  /* States */
  .input-focused {
    border-color: var(--color-primary, #667eea);
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .input-underline.input-focused {
    border-bottom-color: var(--color-primary, #667eea);
    box-shadow: 0 1px 0 0 var(--color-primary, #667eea);
  }

  .input-error {
    border-color: var(--color-error, #f56565);
  }

  .input-error.input-focused {
    border-color: var(--color-error, #f56565);
    box-shadow: 0 0 0 3px rgba(245, 101, 101, 0.1);
  }

  .input-success {
    border-color: var(--color-success, #48bb78);
  }

  .input-success.input-focused {
    border-color: var(--color-success, #48bb78);
    box-shadow: 0 0 0 3px rgba(72, 187, 120, 0.1);
  }

  .input-disabled {
    opacity: 0.6;
    cursor: not-allowed;
    background: var(--color-bg-tertiary, #edf2f7);
  }

  .input-disabled .input {
    cursor: not-allowed;
  }

  .input-readonly {
    background: var(--color-bg-tertiary, #edf2f7);
  }

  .input-full-width {
    width: 100%;
  }

  /* Icons */
  .input-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted, #718096);
    flex-shrink: 0;
  }

  .input-icon-left {
    margin-right: 0.5rem;
  }

  .input-icon-right {
    margin-left: 0.5rem;
  }

  .input-with-left-icon {
    padding-left: 0.75rem;
  }

  .input-with-right-icon {
    padding-right: 0.75rem;
  }

  .input-with-left-padding {
    padding-left: 0;
  }

  .input-with-right-padding {
    padding-right: 0;
  }

  /* Loading spinner */
  .input-spinner {
    position: absolute;
    right: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid var(--color-border-light, #e2e8f0);
    border-top: 2px solid var(--color-primary, #667eea);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Help text */
  .input-help {
    margin-top: 0.375rem;
    font-size: 0.875rem;
    color: var(--color-text-muted, #718096);
    line-height: 1.25rem;
  }

  /* Error message */
  .input-error-message {
    margin-top: 0.375rem;
    font-size: 0.875rem;
    color: var(--color-error, #f56565);
    line-height: 1.25rem;
  }

  /* Dark theme adjustments */
  :global([data-theme="dark"]) .input-wrapper {
    background: var(--color-bg-primary, #1a202c);
    border-color: var(--color-border-medium, #4a5568);
  }

  :global([data-theme="dark"]) .input {
    color: var(--color-text-primary, #f7fafc);
  }

  :global([data-theme="dark"]) .input::placeholder {
    color: var(--color-text-muted, #a0aec0);
  }

  :global([data-theme="dark"]) .input-filled {
    background: var(--color-bg-secondary, #2d3748);
  }

  :global([data-theme="dark"]) .input-disabled {
    background: var(--color-bg-tertiary, #4a5568);
  }

  :global([data-theme="dark"]) .input-readonly {
    background: var(--color-bg-tertiary, #4a5568);
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .input-wrapper {
      border-width: 2px;
    }

    .input-underline {
      border-bottom-width: 3px;
    }
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .input-wrapper {
      transition: none;
    }

    .spinner {
      animation: none;
    }
  }

  /* Focus within for accessibility */
  .input-wrapper:focus-within {
    border-color: var(--color-primary, #667eea);
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .input-underline:focus-within {
    border-bottom-color: var(--color-primary, #667eea);
    box-shadow: 0 1px 0 0 var(--color-primary, #667eea);
  }

  .input-error:focus-within {
    border-color: var(--color-error, #f56565);
    box-shadow: 0 0 0 3px rgba(245, 101, 101, 0.1);
  }

  .input-success:focus-within {
    border-color: var(--color-success, #48bb78);
    box-shadow: 0 0 0 3px rgba(72, 187, 120, 0.1);
  }

  /* Print styles */
  @media print {
    .input-wrapper {
      background: white !important;
      border: 1px solid black !important;
      box-shadow: none !important;
    }

    .input {
      color: black !important;
    }
  }
</style>
