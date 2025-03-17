import { fetchConfig } from "@/lib/api";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";

export default async function EditConfigPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const configId = (await params).id;
  const config = await fetchConfig(configId);

  if (!config) {
    notFound();
  }

  async function updateConfigAction(formData: FormData) {
    "use server";

    const name = formData.get("name") as string;
    const baseUrl = formData.get("baseUrl") as string;
    const maxDepth = Number.parseInt(formData.get("maxDepth") as string, 10);
    const maxPagesPerJob = Number.parseInt(
      formData.get("maxPagesPerJob") as string,
      10
    );
    const includePatterns = (formData.get("includePatterns") as string)
      .split("\n")
      .map((pattern) => pattern.trim())
      .filter(Boolean);
    const excludePatterns = (formData.get("excludePatterns") as string)
      .split("\n")
      .map((pattern) => pattern.trim())
      .filter(Boolean);
    const active = formData.get("active") === "on";

    const updatedConfig = {
      id: configId,
      name,
      base_url: baseUrl,
      max_depth: maxDepth,
      max_pages_per_job: maxPagesPerJob,
      include_patterns: includePatterns,
      exclude_patterns: excludePatterns,
      active,
      headers: config?.headers || {},
      created_at: config?.created_at || new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    // In a real implementation, you would call an API to update the config
    // await updateConfig(updatedConfig);

    redirect(`/scraper/configs/${configId}`);
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">
            Edit Configuration
          </h2>
          <p className="text-muted-foreground">
            Edit configuration: {config.name}
          </p>
        </div>
        <Link
          href={`/scraper/configs/${configId}`}
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
        >
          Cancel
        </Link>
      </div>

      <form action={updateConfigAction} className="space-y-6">
        <div className="space-y-4">
          <div className="grid gap-2">
            <label
              htmlFor="name"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              Name
            </label>
            <input
              id="name"
              name="name"
              className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              defaultValue={config.name}
              required
            />
          </div>

          <div className="grid gap-2">
            <label
              htmlFor="baseUrl"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              Base URL
            </label>
            <input
              id="baseUrl"
              name="baseUrl"
              type="url"
              className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              defaultValue={config.base_url}
              required
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <label
                htmlFor="maxDepth"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Max Depth
              </label>
              <input
                id="maxDepth"
                name="maxDepth"
                type="number"
                min="1"
                max="10"
                defaultValue={config.max_depth}
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                required
              />
            </div>

            <div className="grid gap-2">
              <label
                htmlFor="maxPagesPerJob"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Max Pages Per Job
              </label>
              <input
                id="maxPagesPerJob"
                name="maxPagesPerJob"
                type="number"
                min="1"
                max="1000"
                defaultValue={config.max_pages_per_job}
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                required
              />
            </div>
          </div>

          <div className="grid gap-2">
            <label
              htmlFor="includePatterns"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              Include Patterns (one per line)
            </label>
            <textarea
              id="includePatterns"
              name="includePatterns"
              rows={3}
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              defaultValue={config.include_patterns?.join("\n") || ""}
            />
            <p className="text-xs text-muted-foreground">
              Leave empty to include all URLs. These are regex patterns.
            </p>
          </div>

          <div className="grid gap-2">
            <label
              htmlFor="excludePatterns"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              Exclude Patterns (one per line)
            </label>
            <textarea
              id="excludePatterns"
              name="excludePatterns"
              rows={3}
              className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
              defaultValue={config.exclude_patterns?.join("\n") || ""}
            />
            <p className="text-xs text-muted-foreground">
              Leave empty to exclude no URLs. These are regex patterns.
            </p>
          </div>

          <div className="flex items-center space-x-2">
            <input
              id="active"
              name="active"
              type="checkbox"
              defaultChecked={config.active}
              className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
            />
            <label
              htmlFor="active"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              Active
            </label>
          </div>
        </div>

        <div className="flex justify-end">
          <button
            type="submit"
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
          >
            Update Configuration
          </button>
        </div>
      </form>
    </div>
  );
}
