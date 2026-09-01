export default function Home() {
  return (
    <main className="fixed inset-0 bg-[#0a0d12]">
      <iframe
        src="/voxelcraft.html"
        title="VoxelCraft — Rust + wgpu voxel engine"
        className="h-full w-full border-0"
        allow="autoplay"
      />
    </main>
  );
}
