import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

const serverSchema = {
  NEXT_RUNTIME: z.enum(["nodejs", "edge"]).optional(),

  NODE_ENV: z.enum(["test", "development", "production"]),

  // Added by Sentry Integration, Vercel Marketplace
  SENTRY_ORG: z.string().min(1).optional(),
  SENTRY_PROJECT: z.string().min(1).optional(),

  // Added by Vercel
  VERCEL: z.string().optional(),
  ANALYZE: z.string().optional(),
};

const clientSchema = {
  NEXT_PUBLIC_SENTRY_DSN: z.string().optional(),
};

const env = createEnv({
  server: serverSchema,
  client: clientSchema,
  emptyStringAsUndefined: true,
  runtimeEnv: {
    // DATABASE_URL: process.env.DATABASE_URL,

    NODE_ENV: process.env.NODE_ENV || "development",

    // Added by Sentry Integration, Vercel Marketplace
    SENTRY_ORG: process.env.SENTRY_ORG,
    SENTRY_PROJECT: process.env.SENTRY_PROJECT,

    VERCEL: process.env.VERCEL,
    ANALYZE: process.env.ANALYZE,
    NEXT_RUNTIME: process.env.NEXT_RUNTIME,
    NEXT_PUBLIC_SENTRY_DSN: process.env.NEXT_PUBLIC_SENTRY_DSN,
  },
  onValidationError: (err) => {
    throw err;
  },
  extends: [
    // add this if you are using cloudflare , plugins like  aws-s3 etc
    // cloudflare(),
    // posthogPreset(),
  ],
});

export default env;
export { env };
export type AppEnv = typeof env;
