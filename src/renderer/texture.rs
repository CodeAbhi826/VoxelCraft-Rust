// Procedural texture atlas: 19 tiles, 16x16 px each, drawn on CPU.

use wgpu::util::DeviceExt;

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub sampler: wgpu::Sampler,
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const TILE_SIZE: u32 = 16;
        const TILES: u32 = 19;
        let mut pixels = Vec::with_capacity((TILE_SIZE * TILE_SIZE * TILES * 4) as usize);

        // Generate each tile procedurally (same colors as the web version)
        for tile in 0..TILES {
            let (r, g, b, a) = tile_color(tile as u8);
            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    // Add per-pixel noise
                    let n = noise_xy(x as i32, y as i32, tile as i32) as i32;
                    let nr = (r as i32 + n - 10).clamp(0, 255) as u8;
                    let ng = (g as i32 + n - 10).clamp(0, 255) as u8;
                    let nb = (b as i32 + n - 10).clamp(0, 255) as u8;
                    pixels.extend_from_slice(&[nr, ng, nb, a]);
                }
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: TILE_SIZE,
                height: TILE_SIZE,
                depth_or_array_layers: TILES,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(TILE_SIZE * 4),
                rows_per_image: Some(TILE_SIZE),
            },
            wgpu::Extent3d { width: TILE_SIZE, height: TILE_SIZE, depth_or_array_layers: TILES },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        Self { texture, view, bind_group_layout, bind_group, sampler }
    }
}

fn tile_color(tile: u8) -> (u8, u8, u8, u8) {
    match tile {
        0  => (86, 145, 53, 255),    // grass top
        1  => (124, 86, 56, 255),    // dirt
        2  => (124, 86, 56, 255),    // dirt (dup for grass side)
        3  => (128, 128, 128, 255),  // stone
        4  => (110, 110, 110, 255),  // cobble
        5  => (165, 130, 80, 255),   // log top
        6  => (110, 80, 50, 255),    // log side
        7  => (54, 110, 36, 180),    // leaves (translucent)
        8  => (218, 205, 138, 255),  // sand
        9  => (54, 102, 200, 180),   // water (translucent)
        10 => (165, 130, 80, 255),   // planks
        11 => (60, 60, 60, 255),     // bedrock
        12 => (240, 245, 250, 255),  // snow
        13 => (200, 220, 240, 200),  // glass
        14 => (150, 60, 45, 255),    // brick
        15 => (40, 40, 40, 255),     // coal ore
        16 => (200, 165, 130, 255),  // iron ore
        17 => (240, 215, 80, 255),   // gold ore
        18 => (110, 230, 230, 255),  // diamond ore
        _ => (255, 0, 255, 255),     // missing
    }
}

fn noise_xy(x: i32, y: i32, seed: i32) -> u8 {
    let mut n = (x.wrapping_add(y.wrapping_mul(57)).wrapping_add(seed.wrapping_mul(137))) as u32;
    n = n.wrapping_mul(n.wrapping_mul(n.wrapping_mul(60493).wrapping_add(19990303)).wrapping_add(1376312589));
    (n & 0xff) as u8
}
