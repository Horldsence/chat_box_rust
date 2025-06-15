<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	// Props
	export let variant: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger' | 'success' = 'primary';
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let disabled = false;
	export let loading = false;
	export let fullWidth = false;
	export let href: string | undefined = undefined;
	export let target: string | undefined = undefined;
	export let type: 'button' | 'submit' | 'reset' = 'button';
	export let ariaLabel: string | undefined = undefined;
	export let title: string | undefined = undefined;

	// Event dispatcher
	const dispatch = createEventDispatcher();

	// Handle click
	function handleClick(event: MouseEvent) {
		if (disabled || loading) {
			event.preventDefault();
			return;
		}
		dispatch('click', event);
	}

	// Handle keydown
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			if (!disabled && !loading) {
				dispatch('click', event);
			}
		}
	}

	// Computed classes
	$: classes = [
		'btn',
		`btn-${variant}`,
		`btn-${size}`,
		disabled && 'btn-disabled',
		loading && 'btn-loading',
		fullWidth && 'btn-full-width'
	].filter(Boolean).join(' ');
</script>

{#if href}
	<a
		{href}
		{target}
		class={classes}
		aria-label={ariaLabel}
		{title}
		role="button"
		tabindex={disabled ? -1 : 0}
		on:click={handleClick}
		on:keydown={handleKeydown}
	>
		{#if loading}
			<div class="btn-spinner" aria-hidden="true"></div>
		{/if}
		<span class="btn-content" class:loading>
			<slot />
		</span>
	</a>
{:else}
	<button
		{type}
		{disabled}
		class={classes}
		aria-label={ariaLabel}
		{title}
		on:click={handleClick}
	>
		{#if loading}
			<div class="btn-spinner" aria-hidden="true"></div>
		{/if}
		<span class="btn-content" class:loading>
			<slot />
		</span>
	</button>
{/if}

<style>
	.btn {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-sm, 0.5rem);
		font-family: inherit;
		font-weight: 500;
		text-decoration: none;
		text-align: center;
		border: 1px solid transparent;
		border-radius: var(--radius-md, 8px);
		cursor: pointer;
		transition: all 0.2s ease-in-out;
		user-select: none;
		vertical-align: middle;
		white-space: nowrap;
		outline: none;
		box-sizing: border-box;
	}

	/* Sizes */
	.btn-sm {
		padding: 0.375rem 0.75rem;
		font-size: 0.875rem;
		line-height: 1.25rem;
		min-height: 2rem;
	}

	.btn-md {
		padding: 0.5rem 1rem;
		font-size: 1rem;
		line-height: 1.5rem;
		min-height: 2.5rem;
	}

	.btn-lg {
		padding: 0.75rem 1.5rem;
		font-size: 1.125rem;
		line-height: 1.75rem;
		min-height: 3rem;
	}

	/* Variants */
	.btn-primary {
		background: var(--color-primary, #667eea);
		color: var(--color-text-inverse, white);
		border-color: var(--color-primary, #667eea);
	}

	.btn-primary:hover:not(.btn-disabled):not(.btn-loading) {
		background: var(--color-primary-hover, #5a67d8);
		border-color: var(--color-primary-hover, #5a67d8);
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
	}

	.btn-secondary {
		background: var(--color-secondary, #4fd1c7);
		color: var(--color-text-inverse, white);
		border-color: var(--color-secondary, #4fd1c7);
	}

	.btn-secondary:hover:not(.btn-disabled):not(.btn-loading) {
		background: var(--color-secondary-hover, #38b2ac);
		border-color: var(--color-secondary-hover, #38b2ac);
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(79, 209, 199, 0.3);
	}

	.btn-outline {
		background: transparent;
		color: var(--color-primary, #667eea);
		border-color: var(--color-primary, #667eea);
	}

	.btn-outline:hover:not(.btn-disabled):not(.btn-loading) {
		background: var(--color-primary, #667eea);
		color: var(--color-text-inverse, white);
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
	}

	.btn-ghost {
		background: transparent;
		color: var(--color-text-secondary, #4a5568);
		border-color: transparent;
	}

	.btn-ghost:hover:not(.btn-disabled):not(.btn-loading) {
		background: var(--color-bg-tertiary, rgba(0, 0, 0, 0.05));
		color: var(--color-text-primary, #2d3748);
	}

	.btn-danger {
		background: var(--color-error, #f56565);
		color: var(--color-text-inverse, white);
		border-color: var(--color-error, #f56565);
	}

	.btn-danger:hover:not(.btn-disabled):not(.btn-loading) {
		background: #e53e3e;
		border-color: #e53e3e;
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(245, 101, 101, 0.3);
	}

	.btn-success {
		background: var(--color-success, #48bb78);
		color: var(--color-text-inverse, white);
		border-color: var(--color-success, #48bb78);
	}

	.btn-success:hover:not(.btn-disabled):not(.btn-loading) {
		background: #38a169;
		border-color: #38a169;
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(72, 187, 120, 0.3);
	}

	/* States */
	.btn-disabled {
		opacity: 0.6;
		cursor: not-allowed;
		pointer-events: none;
	}

	.btn-loading {
		cursor: wait;
		pointer-events: none;
	}

	.btn-full-width {
		width: 100%;
	}

	/* Loading spinner */
	.btn-spinner {
		position: absolute;
		width: 1rem;
		height: 1rem;
		border: 2px solid transparent;
		border-top: 2px solid currentColor;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	.btn-content.loading {
		opacity: 0;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Focus styles */
	.btn:focus-visible {
		outline: 2px solid var(--color-primary, #667eea);
		outline-offset: 2px;
	}

	/* Active state */
	.btn:active:not(.btn-disabled):not(.btn-loading) {
		transform: translateY(0);
	}

	/* Dark theme adjustments */
	:global([data-theme="dark"]) .btn-ghost {
		color: var(--color-text-secondary, #a0aec0);
	}

	:global([data-theme="dark"]) .btn-ghost:hover:not(.btn-disabled):not(.btn-loading) {
		background: var(--color-bg-tertiary, rgba(255, 255, 255, 0.1));
		color: var(--color-text-primary, #f7fafc);
	}

	/* High contrast mode */
	@media (prefers-contrast: high) {
		.btn {
			border-width: 2px;
		}

		.btn-outline {
			border-width: 2px;
		}
	}

	/* Reduced motion */
	@media (prefers-reduced-motion: reduce) {
		.btn {
			transition: none;
		}

		.btn:hover {
			transform: none;
		}

		.btn-spinner {
			animation: none;
		}
	}

	/* Print styles */
	@media print {
		.btn {
			background: transparent !important;
			color: black !important;
			border: 1px solid black !important;
			box-shadow: none !important;
		}
	}
</style>
