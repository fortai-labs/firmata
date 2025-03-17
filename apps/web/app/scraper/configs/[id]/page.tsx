import { fetchConfig, startJob } from "@/lib/api";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";

export default async function ConfigDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const configId = (await params).id;
  const config = await fetchConfig(configId);

  if (!config) {
    notFound();
  }

  async function startJobAction() {
    "use server";
    const job = await startJob(configId);
    console.log(`Job started: ${job}`);
    if (job) {
      redirect(`/scraper/jobs/${job.id}`);
    } else {
      // Handle error case
      console.error("Failed to start job", job);
      return;
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">
            Configuration Details
          </h2>
          <p className="text-muted-foreground">
            Viewing details for configuration: {config.name}
          </p>
        </div>
        <div className="flex space-x-2">
          <Link
            href="/scraper/configs"
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
          >
            Back to Configurations
          </Link>
          <Link
            href={`/scraper/configs/${configId}/edit`}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
          >
            Edit
          </Link>
          <form action={startJobAction}>
            <button
              type="submit"
              className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
            >
              Start Job
            </button>
          </form>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <div className="space-y-4">
          <div className="rounded-lg border p-4">
            <h3 className="text-lg font-medium">Basic Information</h3>
            <div className="mt-4 space-y-3">
              <div>
                <div className="text-sm font-medium text-muted-foreground">
                  Name
                </div>
                <div className="mt-1">{config.name}</div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">
                  Base URL
                </div>
                <div className="mt-1">
                  <a
                    href={config.base_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline dark:text-blue-400 break-all"
                  >
                    {config.base_url}
                  </a>
                </div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">
                  Max Depth
                </div>
                <div className="mt-1">{config.max_depth}</div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">
                  Max Pages Per Job
                </div>
                <div className="mt-1">
                  {config.max_pages_per_job || "Unlimited"}
                </div>
              </div>
              <div>
                <div className="text-sm font-medium text-muted-foreground">
                  Status
                </div>
                <div className="mt-1">
                  {config.active ? "Active" : "Inactive"}
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="space-y-4">
          {config.include_patterns && config.include_patterns.length > 0 && (
            <div className="rounded-lg border p-4">
              <h3 className="text-lg font-medium">Include Patterns</h3>
              <div className="mt-4">
                <ul className="list-disc list-inside space-y-1">
                  {config.include_patterns.map((pattern: string) => (
                    <li key={pattern}>{pattern}</li>
                  ))}
                </ul>
              </div>
            </div>
          )}

          {config.exclude_patterns && config.exclude_patterns.length > 0 && (
            <div className="rounded-lg border p-4">
              <h3 className="text-lg font-medium">Exclude Patterns</h3>
              <div className="mt-4">
                <ul className="list-disc list-inside space-y-1">
                  {config.exclude_patterns.map((pattern: string) => (
                    <li key={pattern}>{pattern}</li>
                  ))}
                </ul>
              </div>
            </div>
          )}

          {config.headers && Object.keys(config.headers).length > 0 && (
            <div className="rounded-lg border p-4">
              <h3 className="text-lg font-medium">Custom Headers</h3>
              <div className="mt-4 space-y-3">
                {Object.entries(config.headers).map(([key, value]) => (
                  <div key={key}>
                    <div className="text-sm font-medium text-muted-foreground">
                      {key}
                    </div>
                    <div className="mt-1 break-all">
                      {typeof value === "string"
                        ? value
                        : JSON.stringify(value)}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
