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
  title: "VoxelCraft — Rust + wgpu 1.16.5-style Voxel Engine",
  description: "VoxelCraft: a self-made 1.16.5-era reference-style voxel engine in Rust + wgpu. Native (Vulkan/DX12/Metal) and browser (WebGPU/WebGL2 WASM) from one codebase. Procedural textures & audio, 1.16.5-style HUD, menus, shaders and post-processing.",
  keywords: ["VoxelCraft", "Rust", "wgpu", "WebGPU", "WASM", "voxel engine", "voxel sandbox", "game engine"],
  authors: [{ name: "CodeAbhi826" }],
  openGraph: {
    title: "VoxelCraft",
    description: "Rust + wgpu voxel engine (1.16.5-era reference-style)",
    siteName: "VoxelCraft",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "VoxelCraft",
    description: "Rust + wgpu voxel engine (1.16.5-era reference-style)",
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
