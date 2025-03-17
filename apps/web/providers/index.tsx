"use client";

import { QueryProvider } from "./query";
import { ThemeProvider } from "./theme";

/**
 * Providers component that wraps the application with all necessary providers
 */
export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <QueryProvider>
      <ThemeProvider>{children}</ThemeProvider>
    </QueryProvider>
  );
}

export default Providers;
