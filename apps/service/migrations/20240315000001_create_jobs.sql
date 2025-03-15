CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    config_id UUID NOT NULL REFERENCES scraper_configs(id),
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    pages_crawled INTEGER NOT NULL DEFAULT 0,
    pages_failed INTEGER NOT NULL DEFAULT 0,
    pages_skipped INTEGER NOT NULL DEFAULT 0,
    next_run_at TIMESTAMP WITH TIME ZONE,
    worker_id VARCHAR(255),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_jobs_config_id ON jobs(config_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_next_run_at ON jobs(next_run_at);
CREATE INDEX idx_jobs_worker_id ON jobs(worker_id); 