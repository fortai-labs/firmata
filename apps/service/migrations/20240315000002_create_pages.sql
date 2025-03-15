CREATE TABLE IF NOT EXISTS pages (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id),
    url VARCHAR(2048) NOT NULL,
    normalized_url VARCHAR(2048) NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    http_status INTEGER NOT NULL,
    http_headers JSONB NOT NULL DEFAULT '{}',
    crawled_at TIMESTAMP WITH TIME ZONE NOT NULL,
    html_storage_path VARCHAR(1024),
    markdown_storage_path VARCHAR(1024),
    title VARCHAR(1024),
    metadata JSONB NOT NULL DEFAULT '{}',
    error_message TEXT,
    depth INTEGER NOT NULL,
    parent_url VARCHAR(2048)
);

CREATE INDEX idx_pages_job_id ON pages(job_id);
CREATE INDEX idx_pages_normalized_url ON pages(normalized_url);
CREATE INDEX idx_pages_content_hash ON pages(content_hash);
CREATE INDEX idx_pages_crawled_at ON pages(crawled_at); 