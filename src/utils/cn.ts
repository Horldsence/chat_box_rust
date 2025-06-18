import { type ClassValue, clsx } from "clsx";

/**
 * Utility function to merge class names conditionally
 * Combines clsx for conditional classes with string concatenation
 */
export function cn(...inputs: ClassValue[]) {
  return clsx(inputs);
}

/**
 * Helper function for conditional classes with better TypeScript support
 */
export function conditionalClass(
  condition: boolean,
  trueClass: string,
  falseClass: string = "",
) {
  return condition ? trueClass : falseClass;
}

/**
 * Helper function to merge theme-aware classes
 */
export function themeClass(lightClass: string, darkClass: string) {
  return `${lightClass} dark:${darkClass}`;
}

/**
 * Common responsive breakpoint utilities
 */
export const breakpoints = {
  sm: "640px",
  md: "768px",
  lg: "1024px",
  xl: "1280px",
  "2xl": "1536px",
};

/**
 * Helper function to create responsive classes
 */
export function responsive(classes: Record<string, string>) {
  return Object.entries(classes)
    .map(([breakpoint, className]) =>
      breakpoint === "base" ? className : `${breakpoint}:${className}`,
    )
    .join(" ");
}
