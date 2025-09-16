use aubrey_common::color::Color;
use aubrey_core::ecs::Entity;
use softbuffer::{Context, Surface};
use aubrey_window::access::{with_window_public as with_window};
use std::num::NonZeroU32;
use winit::window::Window;
use std::cell::RefCell;
use std::collections::HashMap;
use ab_glyph::{FontRef, PxScale, point, Font, Glyph};

// ===== wgpu backend for placeholder rendering =====
mod wgpu_backend {
    use super::*;

    pub struct GpuState {
        pub surface: wgpu::Surface<'static>,
        pub device: wgpu::Device,
        pub queue: wgpu::Queue,
        pub config: wgpu::SurfaceConfiguration,
        pub pipeline: wgpu::RenderPipeline,
        pub bind_group_layout: wgpu::BindGroupLayout,
        pub uniform_buf: wgpu::Buffer,
        pub bind_group: wgpu::BindGroup,
        pub bg_layout_label: &'static str,
    }

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct Params {
        color: [f32; 4],
        resolution: [f32; 2],
        thickness_px: f32,
        _pad: f32,
    }

    fn shader_source() -> &'static str {
        r#"
struct Params {
  color: vec4<f32>,
  resolution: vec2<f32>,
  thickness_px: f32,
  _pad: f32,
};

@group(0) @binding(0)
var<uniform> params: Params;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(3.0, 1.0)
  );
  var out: VsOut;
  let p = pos[vid];
  out.pos = vec4<f32>(p, 0.0, 1.0);
  out.uv = (p * 0.5) + vec2<f32>(0.5, 0.5);
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let res = params.resolution;
  let min_res = min(res.x, res.y);
  let t = params.thickness_px / min_res;
  let uv = in.uv;
  let border = (uv.x < t) || (uv.y < t) || (uv.x > 1.0 - t) || (uv.y > 1.0 - t);
  let diag1 = abs(uv.y - uv.x) < t;
  let diag2 = abs(uv.y - (1.0 - uv.x)) < t;
  if (border || diag1 || diag2) {
    return params.color;
  }
  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
"#
    }

    fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("placeholder-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source().into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("params-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<Params>() as u64) },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("placeholder-pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("placeholder-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[], compilation_options: wgpu::PipelineCompilationOptions::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        (pipeline, bind_group_layout)
    }

    pub fn render_placeholder(win: Entity, color: [f32; 4]) -> Option<()> {
        // get or init state
        super::with_window(win, |window| {
            // SAFETY: we extend window lifetime for surface; wgpu expects 'static surface. We ensure drop after window is alive for the app lifetime.
            let window_static: &'static Window = unsafe { std::mem::transmute::<&Window, &'static Window>(window) };
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            let state = super::GPU_STATES.with(|cell| {
                let mut map = cell.borrow_mut();
                if let Some(state) = map.get(&win) { return state as *const GpuState as usize; }

                let instance = wgpu::Instance::default();
                let surface = instance.create_surface(window_static).expect("create surface");
                let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface), force_fallback_adapter: false,
                })).expect("request adapter");
                let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                }, None)).expect("request device");
                let surface_caps = surface.get_capabilities(&adapter);
                let format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: surface_caps.alpha_modes[0],
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                };
                surface.configure(&device, &config);

                let (pipeline, bgl) = create_pipeline(&device, config.format);
                let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("params-ubo"),
                    size: std::mem::size_of::<Params>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("params-bg"),
                    layout: &bgl,
                    entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() }],
                });
                let new_state = GpuState { surface, device, queue, config, pipeline, bind_group_layout: bgl, uniform_buf, bind_group, bg_layout_label: "params-bgl" };
                map.insert(win, new_state);
                map.get(&win).unwrap() as *const GpuState as usize
            });

            let res: &mut GpuState = super::GPU_STATES.with(|cell| {
                let _map = cell.borrow_mut();
                let ptr = state as *mut GpuState;
                unsafe { &mut *ptr }
            });

            // Reconfigure on resize
            if res.config.width != width || res.config.height != height {
                res.config.width = width; res.config.height = height;
                res.surface.configure(&res.device, &res.config);
            }

            let frame = match res.surface.get_current_texture() { Ok(f) => f, Err(_) => {
                res.surface.configure(&res.device, &res.config);
                res.surface.get_current_texture().ok()?
            }};
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let params = Params { color, resolution: [width as f32, height as f32], thickness_px: 1.5, _pad: 0.0 };
            res.queue.write_buffer(&res.uniform_buf, 0, bytemuck::bytes_of(&params));

            let mut encoder = res.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("placeholder-encoder") });
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("placeholder-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                rpass.set_pipeline(&res.pipeline);
                rpass.set_bind_group(0, &res.bind_group, &[]);
                rpass.draw(0..3, 0..1);
            }
            res.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
            Some(())
        }).flatten()
    }

    pub struct Item { pub x: u32, pub y: u32, pub w: u32, pub h: u32, pub color: [f32; 4], pub thickness_px: f32 }

    pub fn render_batch(win: Entity, items: &[Item]) -> Option<()> {
        super::with_window(win, |window| {
            let window_static: &'static Window = unsafe { std::mem::transmute::<&Window, &'static Window>(window) };
            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            let state = super::GPU_STATES.with(|cell| {
                let mut map = cell.borrow_mut();
                if let Some(state) = map.get(&win) { return state as *const GpuState as usize; }

                let instance = wgpu::Instance::default();
                let surface = instance.create_surface(window_static).expect("create surface");
                let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface), force_fallback_adapter: false,
                })).expect("request adapter");
                let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                }, None)).expect("request device");
                let surface_caps = surface.get_capabilities(&adapter);
                let format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: surface_caps.alpha_modes[0],
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                };
                surface.configure(&device, &config);

                let (pipeline, bgl) = create_pipeline(&device, config.format);
                let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("params-ubo"),
                    size: std::mem::size_of::<Params>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("params-bg"),
                    layout: &bgl,
                    entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() }],
                });
                let new_state = GpuState { surface, device, queue, config, pipeline, bind_group_layout: bgl, uniform_buf, bind_group, bg_layout_label: "params-bgl" };
                map.insert(win, new_state);
                map.get(&win).unwrap() as *const GpuState as usize
            });

            let res: &mut GpuState = super::GPU_STATES.with(|cell| {
                let _map = cell.borrow_mut();
                let ptr = state as *mut GpuState;
                unsafe { &mut *ptr }
            });

            if res.config.width != width || res.config.height != height {
                res.config.width = width; res.config.height = height;
                res.surface.configure(&res.device, &res.config);
            }

            let frame = match res.surface.get_current_texture() { Ok(f) => f, Err(_) => {
                res.surface.configure(&res.device, &res.config);
                res.surface.get_current_texture().ok()?
            }};
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Prepare per-item uniform bind groups
            let mut bgs: Vec<wgpu::BindGroup> = Vec::with_capacity(items.len());
            for it in items {
                let params = Params { color: it.color, resolution: [it.w as f32, it.h as f32], thickness_px: it.thickness_px, _pad: 0.0 };
                let buf = res.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("params-ubo-item"),
                    size: std::mem::size_of::<Params>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                res.queue.write_buffer(&buf, 0, bytemuck::bytes_of(&params));
                let bg = res.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("params-bg-item"),
                    layout: &res.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.as_entire_binding() }],
                });
                bgs.push(bg);
            }

            let mut encoder = res.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("placeholder-batch-encoder") });
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("placeholder-batch-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                rpass.set_pipeline(&res.pipeline);
                for (i, it) in items.iter().enumerate() {
                    rpass.set_bind_group(0, &bgs[i], &[]);
                    // viewport: x,y in pixels; width/height in pixels; depth 0..1
                    rpass.set_viewport(it.x as f32, it.y as f32, it.w as f32, it.h as f32, 0.0, 1.0);
                    rpass.draw(0..3, 0..1);
                }
            }
            res.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
            Some(())
        }).flatten()
    }
}

thread_local! { pub static GPU_STATES: RefCell<HashMap<Entity, wgpu_backend::GpuState>> = RefCell::new(HashMap::new()); }

/// Render a placeholder using wgpu for the given window entity.
pub fn render_placeholder_wgpu(win: Entity, color: [f32; 4]) -> Option<()> {
    wgpu_backend::render_placeholder(win, color)
}

pub struct PlaceholderItem { pub x: u32, pub y: u32, pub w: u32, pub h: u32, pub color: [f32; 4], pub thickness_px: f32 }

pub fn render_placeholders_wgpu(win: Entity, items: &[PlaceholderItem]) -> Option<()> {
    let items2: Vec<wgpu_backend::Item> = items.iter().map(|i| wgpu_backend::Item { x: i.x, y: i.y, w: i.w, h: i.h, color: i.color, thickness_px: i.thickness_px }).collect();
    wgpu_backend::render_batch(win, &items2)
}

fn with_surface<R>(win: Entity, f: impl FnOnce(&mut Surface<&Window, &Window>, (u32, u32)) -> R) -> Option<R> {
    with_window(win, |wnd| {
        let sz = wnd.inner_size();
        let (w, h) = (sz.width, sz.height);
        let ctx = Context::new(wnd).ok()?;
        let mut surf = Surface::new(&ctx, wnd).ok()?;
        let _ = surf.resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap());
        Some(f(&mut surf, (w, h)))
    }).flatten()
}
/// Execute a closure with a CPU frame buffer for the given window entity.
/// The closure receives: (buf, width, height, stride). Buffer format: ARGB8888.
pub fn with_frame(win: Entity, f: impl FnOnce(&mut [u32], usize, usize, usize)) -> Option<()> {
    with_surface(win, |surf, (wpx, hpx)| {
        let mut buf = match surf.buffer_mut() { Ok(b) => b, Err(_) => return (), };
        let width = wpx as usize;
        let height = hpx as usize;
        let stride = width; // tightly packed
        {
            let slice: &mut [u32] = &mut buf;
            f(slice, width, height, stride);
        }
        let _ = buf.present();
    })
}

#[inline]
pub fn pack_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
pub fn pack_color<C: Color>(c: &C) -> u32 {
    let rgba = c.rgba();
    let r = (rgba.r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (rgba.g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (rgba.b.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (rgba.a.clamp(0.0, 1.0) * 255.0) as u8;
    pack_rgba_u8(r, g, b, a)
}

#[inline]
pub fn clear(buf: &mut [u32], width: usize, height: usize, stride: usize, color: u32) {
    for y in 0..height {
        let row = &mut buf[y * stride..y * stride + width];
        for px in row.iter_mut() { *px = color; }
    }
}

#[inline]
pub fn put_pixel(buf: &mut [u32], width: usize, height: usize, stride: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        let ux = x as usize; let uy = y as usize;
        if ux < width && uy < height {
            buf[uy * stride + ux] = color;
        }
    }
}

/// Bresenham's line algorithm
pub fn draw_line(buf: &mut [u32], width: usize, height: usize, stride: usize, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel(buf, width, height, stride, x0, y0, color);
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

pub fn draw_rect_outline(buf: &mut [u32], width: usize, height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 { return; }
    let x0 = x;
    let y0 = y;
    let x1 = x + w - 1;
    let y1 = y + h - 1;
    // top/bottom
    draw_line(buf, width, height, stride, x0, y0, x1, y0, color);
    draw_line(buf, width, height, stride, x0, y1, x1, y1, color);
    // left/right
    draw_line(buf, width, height, stride, x0, y0, x0, y1, color);
    draw_line(buf, width, height, stride, x1, y0, x1, y1, color);
}

pub fn draw_text_mono(buf: &mut [u32], width: usize, height: usize, stride: usize, x: i32, y: i32, text: &str, font_bytes: &[u8], px: f32, color: u32) {
    if let Ok(font) = FontRef::try_from_slice(font_bytes) {
        let scale = PxScale::from(px);
        let ascent = px; // rough baseline
        let line_gap = px * 0.2;
        let mut caret = point(x as f32, y as f32 + ascent);
        for ch in text.chars() {
            if ch == '\n' { caret.x = x as f32; caret.y += px + line_gap; continue; }
            let id = font.glyph_id(ch);
            let sg = Glyph { id, scale, position: caret };
            if let Some(outline) = font.outline_glyph(sg) {
                let bb = outline.px_bounds();
                outline.draw(|gx, gy, cov| {
                    let gx = gx as i32 + bb.min.x as i32;
                    let gy = gy as i32 + bb.min.y as i32;
                    if gx >= 0 && gy >= 0 {
                        let ux = gx as usize; let uy = gy as usize;
                        if ux < width && uy < height {
                            let a = (cov.clamp(0.0, 1.0) * 255.0) as u8;
                            let src = color;
                            let sr = ((src >> 16) & 0xFF) as u8;
                            let sg = ((src >> 8) & 0xFF) as u8;
                            let sb = (src & 0xFF) as u8;
                            let dst = &mut buf[uy * stride + ux];
                            let dr = ((*dst >> 16) & 0xFF) as u8;
                            let dg = ((*dst >> 8) & 0xFF) as u8;
                            let db = (*dst & 0xFF) as u8;
                            let inv_a = 255u16 - a as u16;
                            let r = ((sr as u16 * a as u16 + dr as u16 * inv_a) / 255) as u8;
                            let g = ((sg as u16 * a as u16 + dg as u16 * inv_a) / 255) as u8;
                            let b = ((sb as u16 * a as u16 + db as u16 * inv_a) / 255) as u8;
                            *dst = pack_rgba_u8(r, g, b, 255);
                        }
                    }
                });
                let adv = font.h_advance_unscaled(id) * scale.x;
                caret.x += adv;
            } else {
                // Fallback advance
                caret.x += px * 0.6;
            }
        }
    }
}
