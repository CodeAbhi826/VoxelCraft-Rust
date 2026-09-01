import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  typescript: {
    ignoreBuildErrors: true,
  },
  reactStrictMode: false,
  // The dev server's webpack watcher watches the ENTIRE project dir by
  // default. Unrelated file churn (screenshots written by E2E tests, user
  // uploads, tool outputs, logs, wasm rebuilds in public/) then triggers
  // "Fast Refresh had to perform a full reload" → the iframe hosting the
  // game remounts → the game visibly restarts (reported as "flickering").
  // Watch ONLY what can actually change the app: src/ + root configs.
  // public/ is served statically without needing a reload.
  webpack: (config, { dev }) => {
    if (dev) {
      const { join } = require("path");
      config.watchOptions = {
        ...config.watchOptions,
        ignored: [
          "**/node_modules/**",
          "**/.git/**",
          "**/.next/**",
          "**/upload/**",
          "**/tool-results/**",
          "**/docs/**",
          "**/scripts/**",
          "**/tests/**",
          "**/examples/**",
          "**/skills/**",
          "**/mini-services/**",
          "**/db/**",
          "**/prisma/**",
          "**/voxelcraft/target/**",
          "**/voxelcraft/wasm-out/**",
          "**/public/**",
          "**/*.log",
          "dev.log",
          "worklog.md",
          join(process.cwd(), "upload"),
        ],
      };
    }
    return config;
  },
};

export default nextConfig;
