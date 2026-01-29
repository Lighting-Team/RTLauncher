import type { NextConfig } from "next";

const isProd = process.env.NODE_ENV === 'production';

const internalHost = process.env.TAURI_DEV_HOST || 'localhost';

const nextConfig: NextConfig = {
  // 确保 Next.js 使用 SSG 而不是 SSR
  output: 'export',
  // 在 SSG 模式下使用 Next.js 的 Image 组件需要此功能。
  images: {
      unoptimized: true,
  },
  // 配置 assetPrefix 正确解析资源
  assetPrefix: isProd ? undefined : `http://${internalHost}:3000`,
};

export default nextConfig;
