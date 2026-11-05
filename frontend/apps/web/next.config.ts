import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["@accurit/core", "@accurit/shared"],
  // "standalone" only for the Docker image (apps/web/Dockerfile sets
  // NEXT_OUTPUT=standalone) so `next start` keeps working for local dev.
  output: process.env.NEXT_OUTPUT === "standalone" ? "standalone" : undefined,
};

export default nextConfig;
