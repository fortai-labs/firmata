import { createConfig } from "@/lib/api";
import Link from "next/link";
import { redirect } from "next/navigation";

export default function NewConfigPage() {
  async function createConfigAction(formData: FormData) {
    "use server";

    const name = formData.get("name") as string;
    const baseUrl = formData.get("baseUrl") as string;
    const maxDepth = Number.parseInt(formData.get("maxDepth") as string) || 3;
    const maxPagesPerJob =
      Number.parseInt(formData.get("maxPagesPerJob") as string) || 0;

    // Handle patterns as arrays
    const includePatterns = (formData.get("includePatterns") as string)
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0);

    const excludePatterns = (formData.get("excludePatterns") as string)
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0);

    // Create the config object
    const newConfig = {
      name,
      base_url: baseUrl,
      max_depth: maxDepth,
      max_pages_per_job: maxPagesPerJob || undefined,
      include_patterns: includePatterns,
      exclude_patterns: excludePatterns,
      headers: {},
      active: true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    const config = await createConfig(newConfig);
    if (config) {
      redirect(`/scraper/configs/${config.id}`);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">
            Create New Configuration
          </h2>
          <p className="text-muted-foreground">
            Set up a new scraper configuration for websites you want to crawl
          </p>
        </div>
        <Link
          href="/scraper/configs"
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
        >
          Cancel
        </Link>
      </div>

      <form action={createConfigAction} className="space-y-6">
        <div className="rounded-lg border p-6 space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <label htmlFor="name" className="text-sm font-medium">
                Name
              </label>
              <input
                id="name"
                name="name"
                type="text"
                required
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                placeholder="My Scraper Configuration"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="baseUrl" className="text-sm font-medium">
                Base URL
              </label>
              <input
                id="baseUrl"
                name="baseUrl"
                type="url"
                required
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                placeholder="https://example.com"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="maxDepth" className="text-sm font-medium">
                Max Depth
              </label>
              <input
                id="maxDepth"
                name="maxDepth"
                type="number"
                min="1"
                defaultValue="3"
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              />
              <p className="text-xs text-muted-foreground">
                Maximum link depth to crawl from the base URL
              </p>
            </div>

            <div className="space-y-2">
              <label htmlFor="maxPagesPerJob" className="text-sm font-medium">
                Max Pages Per Job
              </label>
              <input
                id="maxPagesPerJob"
                name="maxPagesPerJob"
                type="number"
                min="0"
                defaultValue="0"
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              />
              <p className="text-xs text-muted-foreground">
                Maximum pages to crawl per job (0 for unlimited)
              </p>
            </div>
          </div>

          <div className="space-y-2">
            <label htmlFor="includePatterns" className="text-sm font-medium">
              Include Patterns (one per line)
            </label>
            <textarea
              id="includePatterns"
              name="includePatterns"
              rows={3}
              className="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              placeholder="*/blog/*\n*/articles/*"
            />
            <p className="text-xs text-muted-foreground">
              Only URLs matching these patterns will be crawled (leave empty to
              include all)
            </p>
          </div>

          <div className="space-y-2">
            <label htmlFor="excludePatterns" className="text-sm font-medium">
              Exclude Patterns (one per line)
            </label>
            <textarea
              id="excludePatterns"
              name="excludePatterns"
              rows={3}
              className="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              placeholder="*/admin/*\n*/login/*"
            />
            <p className="text-xs text-muted-foreground">
              URLs matching these patterns will be excluded from crawling
            </p>
          </div>
        </div>

        <div className="flex justify-end">
          <button
            type="submit"
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
          >
            Create Configuration
          </button>
        </div>
      </form>

      <div className="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-900/20 dark:bg-blue-900/10">
        <h3 className="text-lg font-medium text-blue-800 dark:text-blue-400">
          Tips
        </h3>
        <ul className="mt-2 list-disc list-inside text-sm text-blue-700 dark:text-blue-300 space-y-1">
          <li>
            Use specific include patterns to limit the scope of your crawl
          </li>
          <li>
            Exclude patterns like login pages, admin areas, and user profiles
          </li>
          <li>Start with a small max depth (2-3) to test your configuration</li>
          <li>Set a reasonable page limit for initial crawls</li>
        </ul>
      </div>
    </div>
  );
}
