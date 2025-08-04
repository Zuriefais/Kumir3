use crate::egui_tools::EguiRenderer;
use crate::gruvbox_egui::gruvbox_dark_theme;
use crate::gui::{KumirGui, VelloWindowOptions};
use crate::kumir_state::KumirState;
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{ScreenDescriptor, wgpu};
use log::info;
use std::sync::{Arc, Mutex};
use std::thread;
use vello::peniko::color::palette;
use vello::wgpu::TextureFormat;
use vello::{AaConfig, Renderer, RendererOptions, Scene};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::Texture;

use winit::event::WindowEvent;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::Window;

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub kumir_gui: KumirGui,
    pub vello_renderer: Renderer,
    pub vello_scene: Arc<Mutex<Scene>>,
    pub vello_texture: Texture,
    pub window: Arc<Window>,
    vello_window_options: Arc<Mutex<VelloWindowOptions>>,
}

fn create_vello_texture(device: &wgpu::Device, width: u32, height: u32) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Vello Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        info!("Creating App State...");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");
        info!("Surface created");
        let width = 1920;
        let height = 1080;

        // let _ = window.request_inner_size(PhysicalSize::new(width, height));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Unable request adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8Unorm;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");
        info!("Supported formats: {:?}", swapchain_capabilities.formats);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, &window);
        egui_renderer.context().set_style(gruvbox_dark_theme());

        let scale_factor = 1.0;
        let vello_window_options: Arc<Mutex<VelloWindowOptions>> = Default::default();

        let vello_texture = create_vello_texture(&device, 100, 100);
        let vello_renderer = Renderer::new(
            &device,
            RendererOptions {
                ..Default::default()
            },
        )
        .expect("Couldn't create renderer");
        let vello_scene = Arc::new(Mutex::new(Scene::new()));
        info!("App State created!!");

        let kumir_state = KumirState::new(Arc::clone(&vello_scene), width, height);
        let kumir_gui = KumirGui::new(
            egui_renderer.context(),
            kumir_state,
            vello_window_options.clone(),
        );

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            kumir_gui,
            vello_renderer,
            vello_scene,
            window,
            vello_texture,
            vello_window_options,
        })
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn handle_redraw(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Frame"); // Zone for the entire frame

        let window = &self.window;
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        let mut vello_window_changed = false;

        #[cfg(unix)]
        tracy_full::zone!("Vello Setup"); // Zone for Vello setup
        if let Ok(mut vello_size) = self.vello_window_options.lock()
            && vello_size.changed
            && vello_size.height != 0
            && vello_size.width != 0
        {
            vello_window_changed = true;
            vello_size.changed = false;
            info!(
                "Vello texture size changed: {}, {}",
                vello_size.width, vello_size.height
            );
            self.vello_texture =
                create_vello_texture(&self.device, vello_size.width, vello_size.height);
        }
        self.vello_scene.lock().unwrap().reset();
        self.kumir_gui.add_shapes_to_scene();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: window.scale_factor() as f32 * self.scale_factor,
        };

        let surface_texture = self.surface.get_current_texture();
        match surface_texture {
            Err(SurfaceError::Outdated) => {
                info!("wgpu surface outdated");
                return;
            }
            Err(_) => {
                surface_texture.expect("Failed to acquire next swap chain texture");
                return;
            }
            Ok(_) => {}
        };
        let surface_texture = surface_texture.unwrap();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let vello_view = self
            .vello_texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(TextureFormat::Rgba8Unorm),
                usage: Some(
                    wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                ),
                ..Default::default()
            });

        if vello_window_changed && let Ok(mut vello_options) = self.vello_window_options.lock() {
            vello_options.texture = self
                .egui_renderer
                .register_native_texture(&self.device, &vello_view);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        #[cfg(unix)]
        tracy_full::zone!("Vello Render"); // Zone for Vello rendering
        self.vello_renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.vello_scene.lock().unwrap(),
                &vello_view,
                &vello::RenderParams {
                    base_color: palette::css::BLACK,
                    width,
                    height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .expect("failed to render to surface");

        #[cfg(unix)]
        tracy_full::zone!("Egui Render"); // Zone for eGUI rendering
        self.egui_renderer.begin_frame(window);
        self.kumir_gui.render_gui();
        self.egui_renderer.end_frame_and_draw(
            &self.device,
            &self.queue,
            &mut encoder,
            window,
            &surface_view,
            screen_descriptor,
        );

        #[cfg(unix)]
        tracy_full::zone!("Queue Submit"); // Zone for queue submission
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
        window.request_redraw();
        #[cfg(unix)]
        tracy_full::frame!();
    }

    pub fn event(&mut self, event: &WindowEvent) {
        self.egui_renderer.handle_input(&self.window, event);
    }
}
