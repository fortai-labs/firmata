"use client";

import { ThemeProvider as NextThemesProvider } from "next-themes";
import type { ThemeProviderProps } from "next-themes/dist/types";
import { useEffect } from "react";

const LIGHT_THEME_COLOR = "hsl(0 0% 100%)";
const DARK_THEME_COLOR = "hsl(240deg 10% 3.92%)";

/**
 * Theme provider component that wraps the application with next-themes
 * Handles theme switching and persistence
 */
export function ThemeProvider({ children, ...props }: ThemeProviderProps) {
  // This effect handles updating the theme-color meta tag based on the current theme
  useEffect(() => {
    const updateThemeColor = () => {
      const isDark = document.documentElement.classList.contains("dark");
      let meta = document.querySelector('meta[name="theme-color"]');

      if (!meta) {
        meta = document.createElement("meta");
        meta.setAttribute("name", "theme-color");
        document.head.appendChild(meta);
      }

      meta.setAttribute(
        "content",
        isDark ? DARK_THEME_COLOR : LIGHT_THEME_COLOR
      );
    };

    // Set up observer to watch for class changes on the html element
    const observer = new MutationObserver(updateThemeColor);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    // Initial update
    updateThemeColor();

    // Clean up observer on unmount
    return () => observer.disconnect();
  }, []);

  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme="light"
      enableSystem
      disableTransitionOnChange
      {...props}
    >
      {children}
    </NextThemesProvider>
  );
}

export default ThemeProvider;
