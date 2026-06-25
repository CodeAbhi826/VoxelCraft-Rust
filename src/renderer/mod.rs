// wgpu renderer: device setup, chunk mesh upload, world + UI rendering.
// This is the "shell" that displays the voxel engine.
// The engine itself (World, Chunk, MeshBuilder, Player) is pure logic and
// fully tested in /tests — the renderer just visualizes it.

pub mod texture;
pub mod pipeline;

use std::sync::Arc;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};

use crate::blocks::ChunkVertex;
use crate::world::mesher::ChunkMesh;
use crate::renderer::texture::TextureAtlas;
use crate::renderer::pipeline::WorldPipeline;

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub world_pipeline: WorldPipeline,
    pub texture_atlas: TextureAtlas,
    pub chunk_meshes: HashMap<(i32, i32), ChunkGlMesh>,
    pub depth_texture: wgpu::TextureView,
}

pub struct ChunkGlMesh {
    pub solid: Option<MeshBuffers>,
    pub transparent: Option<MeshBuffers>,
}

pub struct MeshBuffers {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
}

impl Renderer {
    pub async fn new(
        window: Arc<winit::window::Window>,
        instance: &wgpu::Instance,
    ) -> Self {
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapter found");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("VoxelCraft GPU"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            }, None)
            .await
            .expect("Failed to request GPU device");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps.formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let texture_atlas = TextureAtlas::new(&device, &queue);
        let world_pipeline = WorldPipeline::new(&device, config.format, &texture_atlas.bind_group_layout);
        let depth_texture = create_depth_texture(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            world_pipeline,
            texture_atlas,
            chunk_meshes: HashMap::new(),
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = create_depth_texture(&self.device, &self.config);
        }
    }

    /// Upload a chunk mesh to GPU buffers.
    pub fn upload_chunk_mesh(&mut self, cx: i32, cz: i32, mesh: ChunkMesh) {
        let entry = self.chunk_meshes.entry((cx, cz)).or_insert(ChunkGlMesh {
            solid: None,
            transparent: None,
        });

        entry.solid = if !mesh.solid_vertices.is_empty() {
            Some(upload_mesh(&self.device, &mesh.solid_vertices, &mesh.solid_indices))
        } else {
            None
        };
        entry.transparent = if !mesh.transparent_vertices.is_empty() {
            Some(upload_mesh(&self.device, &mesh.transparent_vertices, &mesh.transparent_indices))
        } else {
            None
        };
    }

    pub fn remove_chunk_mesh(&mut self, cx: i32, cz: i32) {
        self.chunk_meshes.remove(&(cx, cz));
    }

    /// Render the world + UI.
    pub fn render(
        &mut self,
        view: Mat4,
        proj: Mat4,
        cam_pos: Vec3,
        fog_color: [f32; 4],
        time_of_day: f32,
        egui_renderer: &mut Option<egui_wgpu::Renderer>,
        egui_primitives: Vec<egui::ClippedPrimitive>,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view_tex = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // === Compute frustum planes for culling ===
        let frustum = compute_frustum_planes(proj * view);

        // === UI prep (egui) ===
        let mut egui_cmds: Vec<wgpu::CommandBuffer> = Vec::new();
        if let Some(renderer) = egui_renderer.as_mut() {
            egui_cmds = renderer.update_buffers(
                &self.device, &self.queue, &mut encoder,
                &egui_primitives, screen_descriptor,
            );
        }

        // === World pass ===
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("World Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view_tex,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: fog_color[0] as f64,
                            g: fog_color[1] as f64,
                            b: fog_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mvp = proj * view;
            let mvp_bytes: [[f32; 4]; 4] = mvp.to_cols_array_2d();
            render_pass.set_pipeline(&self.world_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.world_pipeline.bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_atlas.bind_group, &[]);

            // Update uniforms
            self.queue.write_buffer(&self.world_pipeline.uniform_buf, 0, bytemuck::cast_slice(&[
                WorldUniforms {
                    mvp: mvp_bytes,
                    cam_pos: [cam_pos.x, cam_pos.y, cam_pos.z],
                    fog_color: [fog_color[0], fog_color[1], fog_color[2]],
                    fog_start: 16.0 * 6.0,
                    fog_end: 16.0 * 12.0,
                    _pad: [0.0; 2],
                },
            ]));

            // Draw all chunk meshes (with frustum + distance culling)
            let cull_radius_sq = (16.0_f32 * 12.0).powi(2);
            for ((cx, cz), mesh) in &self.chunk_meshes {
                let chunk_center_x = (cx * 16 + 8) as f32;
                let chunk_center_z = (cz * 16 + 8) as f32;

                // Distance culling
                let dx = chunk_center_x - cam_pos.x;
                let dz = chunk_center_z - cam_pos.z;
                if dx * dx + dz * dz > cull_radius_sq {
                    continue;
                }

                // Frustum culling — skip chunks outside the camera view
                if !chunk_in_frustum(chunk_center_x, 128.0, chunk_center_z, &frustum) {
                    continue;
                }

                if let Some(solid) = &mesh.solid {
                    render_pass.set_vertex_buffer(0, solid.vertex_buf.slice(..));
                    render_pass.set_index_buffer(solid.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..solid.index_count, 0, 0..1);
                }
            }
            // Transparent pass (after opaque, with depth write disabled)
            for ((cx, cz), mesh) in &self.chunk_meshes {
                let chunk_center_x = (cx * 16 + 8) as f32;
                let chunk_center_z = (cz * 16 + 8) as f32;
                let dx = chunk_center_x - cam_pos.x;
                let dz = chunk_center_z - cam_pos.z;
                if dx * dx + dz * dz > cull_radius_sq {
                    continue;
                }
                if !chunk_in_frustum(chunk_center_x, 128.0, chunk_center_z, &frustum) {
                    continue;
                }
                if let Some(transp) = &mesh.transparent {
                    render_pass.set_vertex_buffer(0, transp.vertex_buf.slice(..));
                    render_pass.set_index_buffer(transp.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..transp.index_count, 0, 0..1);
                }
            }
        }

        // === UI pass (egui) ===
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view_tex,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some(renderer) = egui_renderer.as_ref() {
                renderer.render(&mut render_pass, &egui_primitives, screen_descriptor);
            }
        }

        let main_cmd = encoder.finish();
        egui_cmds.push(main_cmd);
        self.queue.submit(egui_cmds);
        output.present();

        Ok(())
    }

    /// Render ONLY the egui UI (no 3D world). Used for Loading/Settings/Logger states.
    pub fn render_ui_only(
        &mut self,
        egui_renderer: &mut Option<egui_wgpu::Renderer>,
        egui_primitives: Vec<egui::ClippedPrimitive>,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        clear_color: [f32; 4],
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view_tex = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("UI Only Encoder"),
        });

        // egui buffer upload
        let mut egui_cmds: Vec<wgpu::CommandBuffer> = Vec::new();
        if let Some(renderer) = egui_renderer.as_mut() {
            egui_cmds = renderer.update_buffers(
                &self.device, &self.queue, &mut encoder,
                &egui_primitives, screen_descriptor,
            );
        }

        // Clear screen + draw egui (no world pass)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view_tex,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0] as f64,
                            g: clear_color[1] as f64,
                            b: clear_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some(renderer) = egui_renderer.as_ref() {
                renderer.render(&mut render_pass, &egui_primitives, screen_descriptor);
            }
        }

        let main_cmd = encoder.finish();
        egui_cmds.push(main_cmd);
        self.queue.submit(egui_cmds);
        output.present();
        Ok(())
    }
}

/// Compute the 6 frustum planes from a view-projection matrix.
/// Each plane is [a, b, c, d] where ax + by + cz + d = 0.
fn compute_frustum_planes(vp: Mat4) -> [[f32; 4]; 6] {
    let m = vp.to_cols_array_2d();
    // m[col][row] — glm uses column-major
    let planes = [
        // Left:   m[3] + m[0]
        [m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]],
        // Right:  m[3] - m[0]
        [m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]],
        // Bottom: m[3] + m[1]
        [m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]],
        // Top:    m[3] - m[1]
        [m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]],
        // Near:   m[3] + m[2]
        [m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]],
        // Far:    m[3] - m[2]
        [m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]],
    ];

    // Normalize each plane
    let mut normalized = [[0.0f32; 4]; 6];
    for (i, plane) in planes.iter().enumerate() {
        let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
        if len > 0.0 {
            for j in 0..4 {
                normalized[i][j] = plane[j] / len;
            }
        }
    }
    normalized
}

/// Test if a point (with bounding radius) is inside the frustum.
fn chunk_in_frustum(cx: f32, cy: f32, cz: f32, frustum: &[[f32; 4]; 6]) -> bool {
    let radius = 200.0; // generous — covers a full 16x384x16 chunk column
    for plane in frustum {
        let d = plane[0] * cx + plane[1] * cy + plane[2] * cz + plane[3];
        if d < -radius {
            return false;
        }
    }
    true
}

fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    tex.create_view(&wgpu::TextureViewDescriptor::default())
}

fn upload_mesh(device: &wgpu::Device, vertices: &[ChunkVertex], indices: &[u32]) -> MeshBuffers {
    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Chunk Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Chunk Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    MeshBuffers {
        vertex_buf,
        index_buf,
        index_count: indices.len() as u32,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WorldUniforms {
    mvp: [[f32; 4]; 4],
    cam_pos: [f32; 3],
    fog_color: [f32; 3],
    fog_start: f32,
    fog_end: f32,
    _pad: [f32; 2],
}
