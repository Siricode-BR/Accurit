import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["@accurit/core", "@accurit/shared"],
};

export default nextConfig;
