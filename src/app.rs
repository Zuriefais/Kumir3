use crate::egui_tools::EguiRenderer;
use crate::executors::add_shapes_to_scene;
use crate::gruvbox_egui::gruvbox_dark_theme;
use crate::gui::{KumirGui, VelloWindowSize};
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{ScreenDescriptor, wgpu};
use log::info;
use std::sync::{Arc, RwLock};
use vello::peniko::Color;
use vello::peniko::color::palette;
use vello::util::RenderContext;
use vello::wgpu::TextureFormat;
use vello::{AaConfig, Renderer, RendererOptions, Scene};
use wgpu::Texture;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub kumir_gui: KumirGui,
    pub vello_renderer: Renderer,
    pub vello_scene: Scene,
    pub vello_context: RenderContext,
    pub vello_texture: Texture,
    pub texture_blitter: wgpu::util::TextureBlitter,
    vello_window_size: Arc<RwLock<VelloWindowSize>>,
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
    async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

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
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
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

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);
        egui_renderer.context().set_style(gruvbox_dark_theme());

        let scale_factor = 1.0;
        let vello_window_size: Arc<RwLock<VelloWindowSize>> = Default::default();
        let kumir_gui = KumirGui::new(egui_renderer.context(), vello_window_size.clone());

        let vello_texture = create_vello_texture(&device, 100, 100);
        let vello_renderer = Renderer::new(
            &device,
            RendererOptions {
                ..Default::default()
            },
        )
        .expect("Couldn't create renderer");
        let vello_scene = Scene::new();
        let vello_context = RenderContext::new();
        let texture_blitter = wgpu::util::TextureBlitter::new(&device, *swapchain_format);

        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            kumir_gui,
            vello_renderer,
            vello_scene,
            vello_context,
            vello_texture,
            texture_blitter,
            vello_window_size,
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn handle_redraw(&mut self, window: &Window) {
        let width = self.surface_config.width;
        let height = self.surface_config.height;

        if let Ok(mut vello_size) = self.vello_window_size.write() {
            if vello_size.changed && vello_size.height != 0 && vello_size.height != 0 {
                vello_size.changed = false;
                self.vello_texture =
                    create_vello_texture(&self.device, vello_size.width, vello_size.height);
            }
        }
        self.vello_scene.reset();

        // Re-add the objects to draw to the scene.
        add_shapes_to_scene(&mut self.vello_scene, width, height);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: window.scale_factor() as f32 * self.scale_factor,
        };

        let surface_texture = self.surface.get_current_texture();

        match surface_texture {
            Err(SurfaceError::Outdated) => {
                // Ignoring outdated to allow resizing and minimization
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
        let vello_texture_egui = self
            .egui_renderer
            .register_native_texture(&self.device, &vello_view);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            self.vello_renderer
                .render_to_texture(
                    &self.device,
                    &self.queue,
                    &self.vello_scene,
                    &vello_view,
                    &vello::RenderParams {
                        base_color: palette::css::BLACK, // Background color
                        width,
                        height,
                        antialiasing_method: AaConfig::Msaa16,
                    },
                )
                .expect("failed to render to surface");
            // self.texture_blitter
            //     .copy(&self.device, &mut encoder, &vello_view, &surface_view);
            self.egui_renderer.begin_frame(window);
            self.kumir_gui.render_gui(vello_texture_egui);
            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn resumed(&mut self) {}
}

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        Self {
            instance,
            state: None,
            window: None,
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1920;
        let initial_height = 1080;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.state.as_mut().unwrap().resize_surface(width, height);
        }
    }

    fn handle_redraw(&mut self) {
        // Attempt to handle minimizing window
        if let Some(window) = self.window.as_ref() {
            if let Some(min) = window.is_minimized() {
                if min {
                    info!("Window is minimized");
                    return;
                }
            }
            if let Some(state) = &mut self.state {
                state.handle_redraw(window);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // let egui render to process the event first
        self.state
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(self.window.as_ref().unwrap(), &event);

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
