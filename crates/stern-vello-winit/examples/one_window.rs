//! Minimal application-owned Winit loop with a native GPU texture producer.

use std::{sync::Arc, time::Duration};

use stern_core::{
    PhysicalSize as CorePhysicalSize, Primitive, Rect, ScaleFactor, Size, TextureId,
    TexturePrimitive, ViewportInfo,
};
use stern_render::{RenderFrameInput, RenderImageSampling, RenderResources, TextureResource};
use stern_vello_winit::{
    PresenterDeviceScope, VelloNativeTextureRegistration, VelloPresentStatus, VelloPresenterConfig,
    VelloPresenterError, VelloRecoveryKind, VelloRedrawGuidance, VelloResizeOutcome,
    VelloWindowPresenter, wgpu,
};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const PRODUCER_TEXTURE_ID: TextureId = TextureId::from_raw(1);
const PRODUCER_EXTENT: u32 = 256;
const PRODUCER_LOGICAL_EXTENT: f32 = 256.0;
const PRODUCER_INTERVAL: Duration = Duration::from_millis(500);

struct NativeProducer {
    scope: PresenterDeviceScope,
    texture: wgpu::Texture,
    registration: VelloNativeTextureRegistration,
    revision: u64,
}

struct OneWindowApp {
    presenter: VelloWindowPresenter,
    window: Option<Arc<Window>>,
    resources: RenderResources,
    producer: Option<NativeProducer>,
}

impl OneWindowApp {
    fn new() -> Result<Self, VelloPresenterError> {
        let mut resources = RenderResources::new();
        resources.register_texture(producer_resource());
        Ok(Self {
            presenter: VelloWindowPresenter::new(VelloPresenterConfig::new())?,
            window: None,
            resources,
            producer: None,
        })
    }

    fn recover(&mut self, window: &Window) {
        match pollster::block_on(self.presenter.recover()) {
            Ok(_) => window.request_redraw(),
            Err(error) => eprintln!("presenter recovery failed: {error}"),
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop, window: &Arc<Window>) {
        // Any delivered redraw supersedes an older timeout; only a new
        // `Later` result below may arm the next one-shot deadline.
        event_loop.set_control_flow(ControlFlow::Wait);
        let raw_size = window.inner_size();
        match self.presenter.resize(raw_size) {
            Ok(VelloResizeOutcome::RecoveryRequired(_)) => {
                self.recover(window);
                return;
            }
            Ok(VelloResizeOutcome::ZeroSized | VelloResizeOutcome::Detached) => {
                event_loop.set_control_flow(ControlFlow::Wait);
                return;
            }
            Ok(VelloResizeOutcome::Unchanged | VelloResizeOutcome::Resized) => {}
            Ok(_) => return,
            Err(error) => {
                eprintln!("presenter resize failed: {error}");
                return;
            }
        }

        let scale = ScaleFactor::new(window.scale_factor());
        let logical_size = logical_size(raw_size, scale);
        let viewport = ViewportInfo::new(
            logical_size,
            CorePhysicalSize::new(raw_size.width, raw_size.height),
            scale,
        );
        let scope = match self.presenter.device_scope() {
            Ok(Some(scope)) => scope,
            Ok(None) => {
                self.recover(window);
                return;
            }
            Err(error) => {
                eprintln!("presenter device access failed: {error}");
                return;
            }
        };
        if let Err(error) = self.advance_producer(&scope) {
            eprintln!("native texture producer failed: {error}");
            return;
        }
        let primitives = [producer_primitive(logical_size)];
        let report = match self.presenter.present(RenderFrameInput {
            viewport,
            primitives: &primitives,
            resources: &self.resources,
        }) {
            Ok(report) => report,
            Err(error) => {
                eprintln!("presenter frame failed: {error}");
                return;
            }
        };

        if matches!(
            report.status(),
            VelloPresentStatus::SurfaceLost
                | VelloPresentStatus::SurfaceRecoveryRequired
                | VelloPresentStatus::DeviceRecoveryRequired
        ) {
            self.recover(window);
            return;
        }
        match report.redraw() {
            VelloRedrawGuidance::NextFrame => {
                window.request_redraw();
            }
            VelloRedrawGuidance::Later(delay) => {
                event_loop
                    .set_control_flow(ControlFlow::wait_duration(delay.min(PRODUCER_INTERVAL)));
            }
            _ => {
                event_loop.set_control_flow(ControlFlow::wait_duration(PRODUCER_INTERVAL));
            }
        }
    }

    fn advance_producer(
        &mut self,
        current_scope: &PresenterDeviceScope,
    ) -> Result<(), VelloPresenterError> {
        if self
            .producer
            .as_ref()
            .is_some_and(|producer| producer.scope != *current_scope)
        {
            self.producer = None;
        }

        if self.producer.is_none() {
            let revision = 1;
            let texture = self
                .presenter
                .with_device(current_scope, |presenter_device| {
                    let texture = presenter_device
                        .device()
                        .create_texture(&producer_texture_descriptor());
                    populate_producer_texture(
                        presenter_device.device(),
                        presenter_device.queue(),
                        &texture,
                        revision,
                    );
                    texture
                })?;
            let registration = self.presenter.register_native_texture(
                current_scope,
                &producer_resource(),
                &texture,
                revision,
            )?;
            self.producer = Some(NativeProducer {
                scope: current_scope.clone(),
                texture,
                registration,
                revision,
            });
            return Ok(());
        }

        let producer = self.producer.as_ref().expect("producer was initialized");
        let next_revision = producer.revision.saturating_add(1);
        self.presenter
            .with_device(current_scope, |presenter_device| {
                populate_producer_texture(
                    presenter_device.device(),
                    presenter_device.queue(),
                    &producer.texture,
                    next_revision,
                );
            })?;
        let _ = self
            .presenter
            .update_native_texture(&producer.registration, next_revision)?;
        self.producer
            .as_mut()
            .expect("producer was initialized")
            .revision = next_revision;
        Ok(())
    }
}

fn producer_resource() -> TextureResource {
    TextureResource {
        id: PRODUCER_TEXTURE_ID,
        size: Size::new(PRODUCER_LOGICAL_EXTENT, PRODUCER_LOGICAL_EXTENT),
        sampling: RenderImageSampling::Pixelated,
        snapshot: None,
    }
}

fn producer_texture_descriptor() -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some("stern-one-window-producer"),
        size: wgpu::Extent3d {
            width: PRODUCER_EXTENT,
            height: PRODUCER_EXTENT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    }
}

fn populate_producer_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    revision: u64,
) {
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("stern-one-window-producer-encoder"),
    });
    let attachments = [Some(wgpu::RenderPassColorAttachment {
        view: &view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(producer_color(revision)),
            store: wgpu::StoreOp::Store,
        },
    })];
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stern-one-window-producer-clear"),
            color_attachments: &attachments,
            ..Default::default()
        });
    }
    queue.submit([encoder.finish()]);
}

fn producer_color(revision: u64) -> wgpu::Color {
    match revision % 4 {
        0 => wgpu::Color {
            r: 0.95,
            g: 0.22,
            b: 0.35,
            a: 1.0,
        },
        1 => wgpu::Color {
            r: 0.16,
            g: 0.52,
            b: 0.96,
            a: 1.0,
        },
        2 => wgpu::Color {
            r: 0.16,
            g: 0.78,
            b: 0.48,
            a: 1.0,
        },
        _ => wgpu::Color {
            r: 0.96,
            g: 0.68,
            b: 0.16,
            a: 1.0,
        },
    }
}

fn producer_primitive(logical_size: Size) -> Primitive {
    let inset_x = if logical_size.width > 48.0 { 24.0 } else { 0.0 };
    let inset_y = if logical_size.height > 48.0 {
        24.0
    } else {
        0.0
    };
    Primitive::Texture(TexturePrimitive {
        texture: PRODUCER_TEXTURE_ID,
        rect: Rect::new(
            inset_x,
            inset_y,
            (logical_size.width - inset_x * 2.0).max(1.0),
            (logical_size.height - inset_y * 2.0).max(1.0),
        ),
        source_size: Size::new(PRODUCER_LOGICAL_EXTENT, PRODUCER_LOGICAL_EXTENT),
    })
}

#[allow(clippy::cast_possible_truncation)]
fn logical_size(raw: winit::dpi::PhysicalSize<u32>, scale: ScaleFactor) -> Size {
    Size::new(
        (f64::from(raw.width) / scale.value()) as f32,
        (f64::from(raw.height) / scale.value()) as f32,
    )
}

impl ApplicationHandler for OneWindowApp {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. }) {
            event_loop.set_control_flow(ControlFlow::Wait);
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = if let Some(window) = self.window.clone() {
            window
        } else {
            let window = match event_loop.create_window(
                Window::default_attributes().with_title("Stern Vello presenter example"),
            ) {
                Ok(window) => Arc::new(window),
                Err(error) => {
                    eprintln!("window creation failed: {error}");
                    event_loop.exit();
                    return;
                }
            };
            self.window = Some(Arc::clone(&window));
            window
        };

        match pollster::block_on(self.presenter.resume(Arc::clone(&window))) {
            Ok(_) => window.request_redraw(),
            Err(error) => {
                eprintln!("presenter resume failed: {error}");
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        let _ = self.presenter.suspend();
        self.window = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.presenter.accepts_window(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => match self.presenter.resize(size) {
                Ok(VelloResizeOutcome::RecoveryRequired(
                    VelloRecoveryKind::CreateSurface
                    | VelloRecoveryKind::RecreateSurface
                    | VelloRecoveryKind::RebuildDevice,
                )) => {
                    if let Some(window) = self.window.clone() {
                        self.recover(&window);
                    }
                }
                Ok(_) => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                Err(error) => eprintln!("presenter resize failed: {error}"),
            },
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.clone() {
                    self.redraw(event_loop, &window);
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = OneWindowApp::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
