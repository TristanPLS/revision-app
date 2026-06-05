import type { NextConfig } from "next";

// Same-origin API proxy: the browser only ever calls /api/* on the current
// (Tailscale) origin, and Next forwards server-side to the Axum backend — so
// there is never any CORS, on any device.
const backend = process.env.BACKEND_INTERNAL_URL ?? "http://localhost:8080";

const nextConfig: NextConfig = {
  output: "standalone",
  // Pin the workspace root so standalone file-tracing is correct (a stray
  // lockfile in the home dir otherwise confuses inference).
  outputFileTracingRoot: __dirname,
  turbopack: { root: __dirname },
  // Hide the Next.js dev-tools indicator (the little logo bottom-left in dev).
  devIndicators: false,
  async rewrites() {
    return [{ source: "/api/:path*", destination: `${backend}/api/:path*` }];
  },
};

export default nextConfig;
