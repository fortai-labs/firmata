import { fetchPageMarkdown } from "@/lib/api";
import Link from "next/link";
import { notFound } from "next/navigation";

export default async function PageMarkdownPage({
  params,
}: {
  params: { id: string };
}) {
  const pageId = params.id;
  const markdown = await fetchPageMarkdown(pageId);

  if (!markdown) {
    notFound();
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">
            Markdown Content
          </h2>
          <p className="text-muted-foreground">
            Viewing Markdown content for page {pageId}
          </p>
        </div>
        <div className="flex space-x-2">
          <Link
            href={`/scraper/pages/${pageId}`}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
          >
            Back to Page
          </Link>
        </div>
      </div>

      <div className="rounded-lg border">
        <div className="flex items-center justify-between border-b bg-muted px-4 py-2">
          <h3 className="text-sm font-medium">Markdown Source</h3>
          <button
            type="button"
            onClick={() => {
              const blob = new Blob([markdown], { type: "text/markdown" });
              const url = URL.createObjectURL(blob);
              const a = document.createElement("a");
              a.href = url;
              a.download = `page-${pageId}.md`;
              document.body.appendChild(a);
              a.click();
              document.body.removeChild(a);
              URL.revokeObjectURL(url);
            }}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 px-3"
          >
            Download
          </button>
        </div>
        <div className="max-h-[calc(100vh-200px)] overflow-auto p-4">
          <pre className="text-xs whitespace-pre-wrap break-all">
            <code>{markdown}</code>
          </pre>
        </div>
      </div>
    </div>
  );
}
