import { fetchConfig, startJob } from "@/lib/api";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";

export default async function StartJobPage({
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
    if (job) {
      redirect(`/scraper/jobs/${job.id}`);
    } else {
      // Handle error case
      console.error("Failed to start job");
      return;
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">Start Job</h2>
          <p className="text-muted-foreground">
            Start a new scraper job using configuration: {config.name}
          </p>
        </div>
        <Link
          href={`/scraper/configs/${configId}`}
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
        >
          Cancel
        </Link>
      </div>

      <div className="rounded-lg border p-6">
        <div className="space-y-4">
          <h3 className="text-lg font-medium">Configuration Summary</h3>

          <div className="grid gap-4 md:grid-cols-2">
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
              <div className="mt-1 break-all">{config.base_url}</div>
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
          </div>

          {config.include_patterns && config.include_patterns.length > 0 && (
            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Include Patterns
              </div>
              <div className="mt-1">
                <ul className="list-disc list-inside">
                  {config.include_patterns.map((pattern: string) => (
                    <li key={pattern}>{pattern}</li>
                  ))}
                </ul>
              </div>
            </div>
          )}

          {config.exclude_patterns && config.exclude_patterns.length > 0 && (
            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Exclude Patterns
              </div>
              <div className="mt-1">
                <ul className="list-disc list-inside">
                  {config.exclude_patterns.map((pattern: string) => (
                    <li key={pattern}>{pattern}</li>
                  ))}
                </ul>
              </div>
            </div>
          )}
        </div>

        <div className="mt-6 flex justify-end">
          <form action={startJobAction}>
            <button
              type="submit"
              className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
            >
              Start Job Now
            </button>
          </form>
        </div>
      </div>

      <div className="rounded-lg border border-yellow-200 bg-yellow-50 p-4 dark:border-yellow-900/20 dark:bg-yellow-900/10">
        <h3 className="text-lg font-medium text-yellow-800 dark:text-yellow-400">
          Important Notes
        </h3>
        <ul className="mt-2 list-disc list-inside text-sm text-yellow-700 dark:text-yellow-300 space-y-1">
          <li>Starting a job will begin crawling the website immediately</li>
          <li>The job will run until it completes or is manually cancelled</li>
          <li>
            Be respectful of the target website's resources and robots.txt rules
          </li>
          <li>Large crawls may take significant time to complete</li>
        </ul>
      </div>
    </div>
  );
}
