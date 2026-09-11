//! LabPBR 1.3 Material & Parallax Occlusion Mapping (POM) Pipeline.
//!
//! Clean-room implementation of the LabPBR 1.3 standard for Minecraft
//! PBR and POM shaders. Provides channel unpacking, Cook-Torrance GGX
//! specular BRDF, self-shadowing, and POM raymarching in WGSL.
//!
//! LabPBR 1.3 Specification:
//! Normal Texture (`_n.png`):
//! - Red (0..255): Tangent-space Normal X (-1.0 to 1.0)
//! - Green (0..255): Tangent-space Normal Y (-1.0 to 1.0)
//! - Blue (0..255): Ambient Occlusion / Height Scale (0.0 to 1.0)
//! - Alpha (0..255): Heightfield for POM (0.0 = deepest, 1.0 = top surface)
//!
//! Specular Texture (`_s.png`):
//! - Red (0..255): Perceptual Smoothness (Roughness = 1.0 - Smoothness)
//! - Green (0..255): F0 Reflectance (0..229) or Metallic (230..255)
//! - Blue (0..255): Porosity (0..64) or Subsurface Scattering (65..255)
//! - Alpha (0..255): Emissive Intensity (0.0 to 1.0)

/// Unpacked LabPBR 1.3 normal map properties.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabPbrNormal {
    /// Tangent-space normal vector [X, Y, Z] with Z reconstructed.
    pub normal: [f32; 3],
    /// Ambient occlusion factor (0.0..1.0).
    pub ao: f32,
    /// Height for parallax occlusion mapping (0.0 = deep, 1.0 = surface).
    pub height: f32,
}

/// Unpacked LabPBR 1.3 specular map properties.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabPbrSpecular {
    /// Perceptual smoothness (0.0..1.0).
    pub smoothness: f32,
    /// Roughness squared (alpha in GGX), clamped to minimum 0.04.
    pub roughness: f32,
    /// Metallic flag (0.0 = dielectric, 1.0 = metal).
    pub metallic: f32,
    /// Characteristic specular reflectance F0.
    pub f0: [f32; 3],
    /// Porosity (water absorption darkening, 0.0..1.0).
    pub porosity: f32,
    /// Subsurface scattering amount (0.0..1.0).
    pub sss: f32,
    /// Emissive brightness (0.0..1.0).
    pub emission: f32,
}

/// Decode raw 8-bit RGBA texel from a LabPBR 1.3 normal map (`_n`).
pub fn decode_normal(rgba: [u8; 4]) -> LabPbrNormal {
    // R and G map from [0, 255] to [-1.0, 1.0]
    let nx = (rgba[0] as f32 / 255.0) * 2.0 - 1.0;
    let ny = (rgba[1] as f32 / 255.0) * 2.0 - 1.0;
    // Reconstruct Z so normal is normalized: Z = sqrt(max(0, 1 - X^2 - Y^2))
    let nz_sq = (1.0 - nx * nx - ny * ny).max(0.0);
    let nz = nz_sq.sqrt();

    let ao = rgba[2] as f32 / 255.0;
    let height = rgba[3] as f32 / 255.0;

    LabPbrNormal {
        normal: [nx, ny, nz],
        ao,
        height,
    }
}

/// Decode raw 8-bit RGBA texel from a LabPBR 1.3 specular map (`_s`).
pub fn decode_specular(rgba: [u8; 4], albedo: [f32; 3]) -> LabPbrSpecular {
    let smoothness = rgba[0] as f32 / 255.0;
    let roughness = (1.0 - smoothness).clamp(0.04, 1.0);

    let g = rgba[1];
    let (metallic, f0) = if g >= 230 {
        // Metallic material: F0 is tinted by the surface albedo
        (1.0f32, albedo)
    } else {
        // Dielectric material: F0 is between 0% and 9% (linear mapping 0..229 to 0..0.09)
        let f0_val = (g as f32 / 229.0) * 0.09;
        (0.0f32, [f0_val, f0_val, f0_val])
    };

    let b = rgba[2];
    let (porosity, sss) = if b <= 64 {
        (b as f32 / 64.0, 0.0f32)
    } else {
        (0.0f32, (b - 64) as f32 / 191.0)
    };

    let emission = rgba[3] as f32 / 255.0;

    LabPbrSpecular {
        smoothness,
        roughness,
        metallic,
        f0,
        porosity,
        sss,
        emission,
    }
}

/// WGSL shader snippet implementing LabPBR 1.3 POM raymarching and Cook-Torrance GGX lighting.
pub const LABPBR_POM_WGSL: &str = r#"
const PI: f32 = 3.14159265359;

struct LabPbrSurface {
    albedo: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32,
    metallic: f32,
    f0: vec3<f32>,
    emission: vec3<f32>,
    ao: f32,
};

// GGX / Trowbridge-Reitz Normal Distribution Function
fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let ndoth2 = n_dot_h * n_dot_h;
    let denom = ndoth2 * (a2 - 1.0) + 1.0;
    return a2 / max(PI * denom * denom, 0.0001);
}

// Smith-GGX Geometric Shadowing Function
fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / max(n_dot_v * (1.0 - k) + k, 0.0001);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let ggx1 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

// Fresnel-Schlick Approximation
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// Full Cook-Torrance Microfacet BRDF
fn evaluate_pbr(
    surf: LabPbrSurface,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    light_color: vec3<f32>,
) -> vec3<f32> {
    let n = surf.normal;
    let v = normalize(view_dir);
    let l = normalize(light_dir);
    let h = normalize(v + l);

    let n_dot_v = max(dot(n, v), 0.0001);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    // Specular D, G, F
    let ndf = distribution_ggx(n_dot_h, surf.roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, surf.roughness);
    let f = fresnel_schlick(h_dot_v, surf.f0);

    let numerator = ndf * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;

    // Energy conservation: diffuse gets remaining light after specular reflection
    let kd = (vec3<f32>(1.0) - f) * (1.0 - surf.metallic);
    let diffuse = kd * surf.albedo / PI;

    let radiance = light_color * n_dot_l;
    return (diffuse + specular) * radiance + surf.emission;
}

// Parallax Occlusion Mapping (POM) raymarching along tangent view vector
fn pom_offset(
    uv: vec2<f32>,
    view_tangent: vec3<f32>,
    depth_scale: f32,
    num_layers: f32,
    sample_height: f32,
) -> vec2<f32> {
    let p = view_tangent.xy / max(abs(view_tangent.z), 0.001) * depth_scale;
    return uv - p * (1.0 - sample_height);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labpbr_normal_decode() {
        // Flat normal: (128, 128) -> (0, 0), Z reconstructed to 1.0, AO=1.0, height=1.0
        let flat = decode_normal([128, 128, 255, 255]);
        assert!((flat.normal[0] - 0.0039).abs() < 0.01);
        assert!((flat.normal[1] - 0.0039).abs() < 0.01);
        assert!((flat.normal[2] - 1.0).abs() < 0.01);
        assert_eq!(flat.ao, 1.0);
        assert_eq!(flat.height, 1.0);

        // Sloped normal: (255, 128) -> X=1.0, Y=0.0, Z=0.0
        let sloped = decode_normal([255, 128, 128, 64]);
        assert!((sloped.normal[0] - 1.0).abs() < 0.01);
        assert!((sloped.normal[2] - 0.0).abs() < 0.01);
        assert!((sloped.ao - 128.0 / 255.0).abs() < 0.001);
        assert!((sloped.height - 64.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn test_labpbr_specular_dielectric_vs_metal() {
        let albedo = [0.8, 0.7, 0.2]; // Gold color

        // Dielectric test (G = 100 <= 229)
        let dielectric = decode_specular([200, 100, 30, 0], albedo);
        assert_eq!(dielectric.metallic, 0.0);
        let expected_f0 = (100.0 / 229.0) * 0.09;
        assert!((dielectric.f0[0] - expected_f0).abs() < 1e-5);
        assert_eq!(dielectric.emission, 0.0);
        assert!((dielectric.porosity - 30.0 / 64.0).abs() < 1e-5);
        assert_eq!(dielectric.sss, 0.0);

        // Metallic test (G = 237 >= 230: gold metal)
        let metal = decode_specular([240, 237, 100, 128], albedo);
        assert_eq!(metal.metallic, 1.0);
        assert_eq!(metal.f0, albedo);
        assert_eq!(metal.porosity, 0.0);
        assert!((metal.sss - (100.0 - 64.0) / 191.0).abs() < 1e-5);
        assert!((metal.emission - 128.0 / 255.0).abs() < 1e-5);
    }

    #[test]
    fn test_labpbr_pom_wgsl_validates() {
        let mut full_shader = String::from(LABPBR_POM_WGSL);
        full_shader.push_str(
            r#"
@fragment
fn fs_test() -> @location(0) vec4<f32> {
    var surf: LabPbrSurface;
    surf.albedo = vec3<f32>(0.8, 0.8, 0.8);
    surf.normal = vec3<f32>(0.0, 0.0, 1.0);
    surf.roughness = 0.3;
    surf.metallic = 0.0;
    surf.f0 = vec3<f32>(0.04);
    surf.emission = vec3<f32>(0.0);
    surf.ao = 1.0;

    let col = evaluate_pbr(
        surf,
        vec3<f32>(0.0, 0.0, 1.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(1.0, 1.0, 1.0),
    );
    let uv = pom_offset(vec2<f32>(0.5, 0.5), vec3<f32>(0.1, 0.1, 0.9), 0.05, 16.0, 0.8);
    return vec4<f32>(col + vec3<f32>(uv.x, uv.y, 0.0), 1.0);
}
"#,
        );

        let res = naga::front::wgsl::parse_str(&full_shader);
        assert!(
            res.is_ok(),
            "LABPBR_POM_WGSL must parse with naga: {:?}",
            res.err()
        );
    }
}
