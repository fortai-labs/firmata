-- Fix the http_headers column in the pages table

-- Check if http_headers column exists, if not add it
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'pages' AND column_name = 'http_headers'
    ) THEN
        ALTER TABLE pages ADD COLUMN http_headers JSONB NOT NULL DEFAULT '{}';
    END IF;
END $$;

-- If the column exists but is not of type JSONB, convert it
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'pages' AND column_name = 'http_headers'
        AND data_type != 'jsonb'
    ) THEN
        ALTER TABLE pages ALTER COLUMN http_headers TYPE JSONB USING http_headers::jsonb;
    END IF;
END $$; 