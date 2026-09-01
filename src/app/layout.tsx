import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/toaster";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "VoxelCraft — Rust + wgpu Minecraft 1.16.5-style Voxel Engine",
  description: "VoxelCraft: a self-made Minecraft 1.16.5-style voxel engine in Rust + wgpu. Native (Vulkan/DX12/Metal) and browser (WebGPU/WebGL2 WASM) from one codebase. Procedural textures & audio, 1.16.5-style HUD, menus, shaders and post-processing.",
  keywords: ["VoxelCraft", "Rust", "wgpu", "WebGPU", "WASM", "voxel engine", "Minecraft clone", "game engine"],
  authors: [{ name: "CodeAbhi826" }],
  icons: {
    icon: "https://z-cdn.chatglm.cn/z-ai/static/logo.svg",
  },
  openGraph: {
    title: "VoxelCraft",
    description: "Rust + wgpu voxel engine (Minecraft 1.16.5-style)",
    url: "https://chat.z.ai",
    siteName: "Z.ai",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "VoxelCraft",
    description: "Rust + wgpu voxel engine (Minecraft 1.16.5-style)",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased bg-background text-foreground`}
      >
        {children}
        <Toaster />
      </body>
    </html>
  );
}
