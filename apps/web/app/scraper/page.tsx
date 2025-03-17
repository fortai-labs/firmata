import { fetchConfigs, fetchJobs } from "@/lib/api";
import Link from "next/link";

export default async function ScraperHomePage() {
  const configs = await fetchConfigs();
  const jobs = await fetchJobs();

  // Get recent jobs (last 5)
  const recentJobs = jobs.slice(0, 5);

  // Count jobs by status
  const jobStats = jobs.reduce(
    (acc, job) => {
      acc[job.status] = (acc[job.status] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  // Count active configs
  const activeConfigs = configs.filter((config) => config.active).length;

  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <h2 className="text-2xl font-bold tracking-tight">Scraper Dashboard</h2>
        <p className="text-muted-foreground">
          Manage your web scraper configurations and monitor jobs
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-lg border p-4 flex flex-col">
          <div className="text-sm font-medium text-muted-foreground">
            Total Configurations
          </div>
          <div className="text-2xl font-bold mt-2">{configs.length}</div>
          <div className="text-xs text-muted-foreground mt-1">
            {activeConfigs} active
          </div>
          <div className="mt-auto pt-4">
            <Link
              href="/scraper/configs"
              className="text-sm text-blue-600 hover:underline dark:text-blue-400"
            >
              View all configurations
            </Link>
          </div>
        </div>

        <div className="rounded-lg border p-4 flex flex-col">
          <div className="text-sm font-medium text-muted-foreground">
            Total Jobs
          </div>
          <div className="text-2xl font-bold mt-2">{jobs.length}</div>
          <div className="text-xs text-muted-foreground mt-1">
            {jobStats.in_progress || 0} in progress
          </div>
          <div className="mt-auto pt-4">
            <Link
              href="/scraper/jobs"
              className="text-sm text-blue-600 hover:underline dark:text-blue-400"
            >
              View all jobs
            </Link>
          </div>
        </div>

        <div className="rounded-lg border p-4 flex flex-col">
          <div className="text-sm font-medium text-muted-foreground">
            Completed Jobs
          </div>
          <div className="text-2xl font-bold mt-2">
            {jobStats.completed || 0}
          </div>
          <div className="text-xs text-muted-foreground mt-1">
            {jobStats.failed || 0} failed
          </div>
          <div className="mt-auto pt-4">
            <Link
              href="/scraper/jobs?status=completed"
              className="text-sm text-blue-600 hover:underline dark:text-blue-400"
            >
              View completed jobs
            </Link>
          </div>
        </div>

        <div className="rounded-lg border p-4 flex flex-col">
          <div className="text-sm font-medium text-muted-foreground">
            Quick Actions
          </div>
          <div className="mt-2 space-y-2">
            <Link
              href="/scraper/configs/new"
              className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 w-full"
            >
              New Configuration
            </Link>
          </div>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="text-lg font-medium">Recent Jobs</h3>
            <Link
              href="/scraper/jobs"
              className="text-sm text-blue-600 hover:underline dark:text-blue-400"
            >
              View all
            </Link>
          </div>

          <div className="rounded-lg border">
            {recentJobs.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground">
                No jobs found. Create a configuration and start a job.
              </div>
            ) : (
              <div className="divide-y">
                {recentJobs.map((job) => (
                  <div key={job.id} className="p-4 hover:bg-muted/50">
                    <div className="flex justify-between items-start">
                      <div>
                        <Link
                          href={`/scraper/jobs/${job.id}`}
                          className="font-medium hover:underline"
                        >
                          Job {job.id.substring(0, 8)}...
                        </Link>
                        <div className="text-sm text-muted-foreground mt-1">
                          Started {new Date(job.created_at).toLocaleString()}
                        </div>
                      </div>
                      <span
                        className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                          job.status === "completed"
                            ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                            : job.status === "in_progress"
                              ? "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400"
                              : job.status === "failed"
                                ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                                : job.status === "cancelled"
                                  ? "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400"
                                  : "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400"
                        }`}
                      >
                        {job.status.charAt(0).toUpperCase() +
                          job.status.slice(1).replace("_", " ")}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="text-lg font-medium">Active Configurations</h3>
            <Link
              href="/scraper/configs"
              className="text-sm text-blue-600 hover:underline dark:text-blue-400"
            >
              View all
            </Link>
          </div>

          <div className="rounded-lg border">
            {configs.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground">
                No configurations found. Create a new configuration to get
                started.
              </div>
            ) : (
              <div className="divide-y">
                {configs
                  .filter((config) => config.active)
                  .slice(0, 5)
                  .map((config) => (
                    <div key={config.id} className="p-4 hover:bg-muted/50">
                      <div className="flex justify-between items-start">
                        <div>
                          <Link
                            href={`/scraper/configs/${config.id}`}
                            className="font-medium hover:underline"
                          >
                            {config.name}
                          </Link>
                          <div className="text-sm text-muted-foreground mt-1 break-all">
                            {config.base_url}
                          </div>
                        </div>
                        <Link
                          href={`/scraper/configs/${config.id}/start`}
                          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-8 px-3"
                        >
                          Start Job
                        </Link>
                      </div>
                    </div>
                  ))}
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-900/20 dark:bg-blue-900/10">
        <h3 className="text-lg font-medium text-blue-800 dark:text-blue-400">
          Getting Started
        </h3>
        <p className="mt-2 text-sm text-blue-700 dark:text-blue-300">
          To start scraping a website, create a new configuration with the
          target URL and settings. Then start a job using that configuration.
          You can monitor the progress of your jobs and view the scraped content
          once the job is complete.
        </p>
        <div className="mt-4 flex space-x-4">
          <Link
            href="/scraper/configs/new"
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-blue-600 text-white hover:bg-blue-700 h-9 px-4 py-2"
          >
            Create Configuration
          </Link>
          <Link
            href="/scraper/jobs"
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-blue-600 text-blue-600 hover:bg-blue-50 h-9 px-4 py-2"
          >
            View Jobs
          </Link>
        </div>
      </div>
    </div>
  );
}
