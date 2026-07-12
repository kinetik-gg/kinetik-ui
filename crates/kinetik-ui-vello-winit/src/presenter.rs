use std::sync::Arc;

use kinetik_ui_render::RenderFrameInput;
use kinetik_ui_vello::VelloRenderer;
use vello::{
    Renderer, RendererOptions,
    util::{RenderContext, RenderSurface},
    wgpu::{
        CommandEncoderDescriptor, CurrentSurfaceTexture, SurfaceTexture, TextureViewDescriptor,
    },
};
use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowId},
};

use crate::{
    PresenterDevice, PresenterDeviceScope, VelloAttachOutcome, VelloAttachmentStatus,
    VelloPresentReport, VelloPresentStatus, VelloPresenterConfig, VelloPresenterError,
    VelloPresenterStatus, VelloRecoveryKind, VelloRecoveryOutcome, VelloRedrawGuidance,
    VelloResizeOutcome, VelloSuspendOutcome,
    device::{DeviceAuthority, DeviceInbox},
    frame::{
        AcquiredFrame, DriveFailure, DrivenFrame, PresentOperations, drive_present,
        report_for_driven,
    },
    lifecycle::{
        DEVICE_REBUILD_SEQUENCE, DropAction, Extent, LifecycleState, ResizePlan, ResumePlan,
    },
};

struct GpuState {
    renderer: Renderer,
    context: RenderContext,
    dev_id: usize,
    inbox: DeviceInbox,
}

enum PresentAttempt {
    Driven(DrivenFrame),
    DeviceLost,
}

/// Presenter for one live Vello surface attached to one Winit window.
pub struct VelloWindowPresenter {
    config: VelloPresenterConfig,
    authority: DeviceAuthority,
    lifecycle: LifecycleState<WindowId>,
    surface: Option<RenderSurface<'static>>,
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    toolkit: VelloRenderer,
    pending_error: Option<VelloPresenterError>,
}

impl VelloWindowPresenter {
    /// Creates a detached presenter without initializing a GPU or surface.
    ///
    /// # Errors
    ///
    /// Returns [`VelloPresenterError::GenerationExhausted`] if a unique opaque
    /// presenter identity cannot be allocated.
    pub fn new(config: VelloPresenterConfig) -> Result<Self, VelloPresenterError> {
        Ok(Self {
            config,
            authority: DeviceAuthority::new()?,
            lifecycle: LifecycleState::new(),
            surface: None,
            window: None,
            gpu: None,
            toolkit: VelloRenderer::new(),
            pending_error: None,
        })
    }

    /// Returns the retained configuration used for recovery.
    #[must_use]
    pub const fn config(&self) -> &VelloPresenterConfig {
        &self.config
    }

    /// Returns the attached window, if any.
    #[must_use]
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// Returns the exact attached window ID, if any.
    #[must_use]
    pub fn window_id(&self) -> Option<WindowId> {
        self.lifecycle.window()
    }

    /// Returns whether the presenter currently owns this exact window.
    #[must_use]
    pub fn accepts_window(&self, window_id: WindowId) -> bool {
        self.lifecycle.window() == Some(window_id)
    }

    /// Returns a non-mutating status snapshot.
    ///
    /// Call [`Self::device_scope`] before native work when callback visibility
    /// must be current.
    #[must_use]
    pub fn status(&self) -> VelloPresenterStatus {
        let scope = if self.lifecycle.attachment_status() == VelloAttachmentStatus::Detached
            || self.lifecycle.recovery().is_some()
        {
            None
        } else {
            self.authority.scope()
        };
        VelloPresenterStatus::new(
            self.lifecycle.attachment_status(),
            self.lifecycle.recovery(),
            scope,
        )
    }

    /// Polls callback inboxes and returns the current usable device scope.
    ///
    /// # Errors
    ///
    /// Returns an actionable callback, overflow, or generation error before
    /// exposing a scope.
    pub fn device_scope(&mut self) -> Result<Option<PresenterDeviceScope>, VelloPresenterError> {
        self.poll_device_events()?;
        self.return_pending_error()?;
        if self.lifecycle.attachment_status() == VelloAttachmentStatus::Detached
            || self.lifecycle.recovery().is_some()
        {
            return Ok(None);
        }
        Ok(self.authority.scope())
    }

    /// Borrows the exact current device and queue after validating a scope.
    ///
    /// The closure is never called for a foreign presenter, stale generation,
    /// detached presenter, pending recovery, or callback-reported failure.
    ///
    /// # Errors
    ///
    /// Returns a typed scope, device-availability, callback, overflow, or
    /// generation error.
    pub fn with_device<R>(
        &mut self,
        scope: &PresenterDeviceScope,
        use_device: impl FnOnce(PresenterDevice<'_>) -> R,
    ) -> Result<R, VelloPresenterError> {
        self.poll_device_events()?;
        self.return_pending_error()?;
        self.authority.validate(scope)?;
        if self.lifecycle.attachment_status() == VelloAttachmentStatus::Detached
            || self.lifecycle.recovery().is_some()
        {
            return Err(VelloPresenterError::DeviceUnavailable);
        }
        let gpu = self
            .gpu
            .as_ref()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let device = &gpu.context.devices[gpu.dev_id];
        Ok(use_device(PresenterDevice::new(
            scope.clone(),
            &device.device,
            &device.queue,
        )))
    }

    /// Attaches a Winit window and initializes a non-zero surface as needed.
    ///
    /// Redundant resume of the same window is idempotent. A different window is
    /// rejected while attached.
    ///
    /// # Errors
    ///
    /// Returns a typed window, callback, initialization, recovery, or
    /// generation error.
    pub async fn resume(
        &mut self,
        window: Arc<Window>,
    ) -> Result<VelloAttachOutcome, VelloPresenterError> {
        self.poll_device_events()?;
        self.return_pending_error()?;
        let window_id = window.id();
        let extent = Extent::from(window.inner_size());
        match self.lifecycle.resume(window_id, extent)? {
            ResumePlan::AlreadyAttached => {
                let _ = self.resize(PhysicalSize::new(extent.width, extent.height))?;
                Ok(VelloAttachOutcome::AlreadyAttached)
            }
            ResumePlan::AttachedZeroSized => {
                self.window = Some(window);
                Ok(VelloAttachOutcome::AttachedZeroSized)
            }
            ResumePlan::Recover(_) => {
                self.window = Some(window);
                let outcome = self.recover().await?;
                let (VelloRecoveryOutcome::SurfaceReady { device_scope, .. }
                | VelloRecoveryOutcome::DeviceRebuilt { device_scope }) = outcome
                else {
                    return Err(VelloPresenterError::DeviceUnavailable);
                };
                Ok(VelloAttachOutcome::AttachedPresentable { device_scope })
            }
        }
    }

    /// Drops the surface before releasing presenter window ownership.
    #[must_use]
    pub fn suspend(&mut self) -> VelloSuspendOutcome {
        let plan = self.lifecycle.suspend();
        if plan.actions.is_empty() {
            return VelloSuspendOutcome::AlreadyDetached;
        }
        for action in plan.actions {
            match action {
                DropAction::Surface => {
                    self.surface.take();
                }
                DropAction::Window => {
                    debug_assert!(self.surface.is_none());
                    self.window.take();
                }
            }
        }
        VelloSuspendOutcome::Suspended
    }

    /// Applies one raw physical window extent without fabricating a 1x1 size.
    ///
    /// # Errors
    ///
    /// Returns callback, overflow, recovery, or generation errors before GPU
    /// configuration.
    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<VelloResizeOutcome, VelloPresenterError> {
        self.poll_device_events()?;
        self.return_pending_error()?;
        match self.lifecycle.resize(Extent::from(size)) {
            ResizePlan::Outcome(outcome) => Ok(outcome),
            ResizePlan::ZeroSized { drop_surface } => {
                if drop_surface {
                    self.surface.take();
                }
                Ok(VelloResizeOutcome::ZeroSized)
            }
            ResizePlan::Configure { extent, force } => {
                self.configure_existing_surface(extent, force)?;
                Ok(VelloResizeOutcome::Resized)
            }
        }
    }

    /// Creates/recreates the surface or rebuilds the complete device path once.
    ///
    /// # Errors
    ///
    /// Returns callback, initialization, recovery, or generation errors. A
    /// failed attempt remains pending and never exposes old native handles.
    pub async fn recover(&mut self) -> Result<VelloRecoveryOutcome, VelloPresenterError> {
        self.poll_device_events()?;
        self.return_pending_error()?;
        let Some(kind) = self.lifecycle.recovery() else {
            return Ok(VelloRecoveryOutcome::NotNeeded);
        };
        let Some(window) = self.window.clone() else {
            if kind == VelloRecoveryKind::RebuildDevice {
                self.surface.take();
                self.drop_gpu_state();
            }
            return Ok(VelloRecoveryOutcome::DeferredDetached(kind));
        };
        let extent = self.lifecycle.desired();
        if extent.is_zero() {
            if kind == VelloRecoveryKind::RebuildDevice {
                self.surface.take();
                self.drop_gpu_state();
            }
            return Ok(VelloRecoveryOutcome::DeferredZeroSized(kind));
        }

        if kind == VelloRecoveryKind::RebuildDevice {
            debug_assert_eq!(DEVICE_REBUILD_SEQUENCE.len(), 7);
            self.surface.take();
            self.drop_gpu_state();
            let (gpu, surface, scope) = self.create_fresh_gpu_surface(window, extent, true).await?;
            self.gpu = Some(gpu);
            self.surface = Some(surface);
            self.lifecycle.mark_surface_ready(extent);
            return Ok(VelloRecoveryOutcome::DeviceRebuilt {
                device_scope: scope,
            });
        }

        self.surface.take();
        if self.gpu.is_none() {
            let (gpu, surface, scope) =
                self.create_fresh_gpu_surface(window, extent, false).await?;
            self.gpu = Some(gpu);
            self.surface = Some(surface);
            self.lifecycle.mark_surface_ready(extent);
            return Ok(VelloRecoveryOutcome::SurfaceReady {
                device_changed: false,
                device_scope: scope,
            });
        }

        let gpu = self
            .gpu
            .as_mut()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let surface = gpu
            .context
            .create_surface(
                window,
                extent.width,
                extent.height,
                self.config.present_mode(),
            )
            .await
            .map_err(VelloPresenterError::recovery)?;
        let changed = surface.dev_id != gpu.dev_id;
        let scope = if changed {
            let device = &gpu.context.devices[surface.dev_id].device;
            let renderer = Renderer::new(device, RendererOptions::default())
                .map_err(VelloPresenterError::recovery)?;
            let scope = self.authority.replace()?;
            let inbox = DeviceInbox::install(device, scope.clone());
            gpu.renderer = renderer;
            gpu.dev_id = surface.dev_id;
            gpu.inbox = inbox;
            scope
        } else {
            self.authority
                .scope()
                .ok_or(VelloPresenterError::DeviceUnavailable)?
        };
        self.surface = Some(surface);
        self.lifecycle.mark_surface_ready(extent);
        Ok(VelloRecoveryOutcome::SurfaceReady {
            device_changed: changed,
            device_scope: scope,
        })
    }

    /// Performs one synchronous acquire/render/blit/notify/present attempt.
    ///
    /// # Errors
    ///
    /// Returns actionable validation, Vello render, callback, overflow, device,
    /// or generation failures. It never retries acquisition in the same call.
    pub fn present(
        &mut self,
        input: RenderFrameInput<'_>,
    ) -> Result<VelloPresentReport, VelloPresenterError> {
        if let Some(report) = self.preflight_present()? {
            return Ok(report);
        }

        let configured = self
            .lifecycle
            .configured()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        match self.run_present_attempt(input, configured)? {
            PresentAttempt::Driven(driven) => {
                match &driven {
                    DrivenFrame::Presented {
                        suboptimal: true, ..
                    } => self.lifecycle.mark_reconfigure(),
                    DrivenFrame::AcquiredExtentOutdated | DrivenFrame::Outdated => {
                        self.lifecycle.mark_surface_ready(configured);
                    }
                    DrivenFrame::Lost => {
                        self.surface.take();
                        self.lifecycle.mark_surface_lost();
                    }
                    _ => {}
                }
                report_for_driven(driven, self.config.timeout_retry())
            }
            PresentAttempt::DeviceLost => Ok(VelloPresentReport::new(
                VelloPresentStatus::DeviceRecoveryRequired,
                VelloRedrawGuidance::None,
                None,
            )),
        }
    }

    fn preflight_present(&mut self) -> Result<Option<VelloPresentReport>, VelloPresenterError> {
        self.poll_device_events()?;
        self.return_pending_error()?;
        if self.lifecycle.attachment_status() == VelloAttachmentStatus::Detached {
            return Ok(Some(VelloPresentReport::new(
                VelloPresentStatus::Detached,
                VelloRedrawGuidance::ExternalEvent,
                None,
            )));
        }
        let raw_size = self
            .window
            .as_ref()
            .ok_or(VelloPresenterError::DeviceUnavailable)?
            .inner_size();
        let report = match self.resize(raw_size)? {
            VelloResizeOutcome::ZeroSized => Some(VelloPresentReport::new(
                VelloPresentStatus::ZeroSized,
                VelloRedrawGuidance::NonZeroResize,
                None,
            )),
            VelloResizeOutcome::RecoveryRequired(VelloRecoveryKind::RebuildDevice) => {
                Some(VelloPresentReport::new(
                    VelloPresentStatus::DeviceRecoveryRequired,
                    VelloRedrawGuidance::None,
                    None,
                ))
            }
            VelloResizeOutcome::RecoveryRequired(_) => Some(VelloPresentReport::new(
                VelloPresentStatus::SurfaceRecoveryRequired,
                VelloRedrawGuidance::NextFrame,
                None,
            )),
            VelloResizeOutcome::Detached => Some(VelloPresentReport::new(
                VelloPresentStatus::Detached,
                VelloRedrawGuidance::ExternalEvent,
                None,
            )),
            VelloResizeOutcome::Unchanged | VelloResizeOutcome::Resized => None,
        };
        Ok(report)
    }

    fn run_present_attempt(
        &mut self,
        input: RenderFrameInput<'_>,
        configured: Extent,
    ) -> Result<PresentAttempt, VelloPresenterError> {
        let current_scope = self
            .authority
            .scope()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let surface = self
            .surface
            .as_ref()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let gpu = self
            .gpu
            .as_mut()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let window = self
            .window
            .as_ref()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let mut post_render_error = None;
        let driven = {
            let mut operations = RealPresentOperations {
                surface,
                gpu,
                toolkit: &mut self.toolkit,
                window,
                config: &self.config,
                current_scope,
                post_render_error: &mut post_render_error,
            };
            drive_present(&mut operations, input, configured)
        };
        match driven {
            Ok(driven) => Ok(PresentAttempt::Driven(driven)),
            Err(DriveFailure::DeviceLostAfterRender) => {
                self.transition_device_loss()?;
                Ok(PresentAttempt::DeviceLost)
            }
            Err(DriveFailure::Render(error)) => {
                Err(post_render_error.unwrap_or_else(|| VelloPresenterError::render(error)))
            }
        }
    }

    async fn create_fresh_gpu_surface(
        &mut self,
        window: Arc<Window>,
        extent: Extent,
        rebuilding: bool,
    ) -> Result<(GpuState, RenderSurface<'static>, PresenterDeviceScope), VelloPresenterError> {
        let mut context = RenderContext::new();
        let surface = context
            .create_surface(
                window,
                extent.width,
                extent.height,
                self.config.present_mode(),
            )
            .await
            .map_err(|error| {
                if rebuilding {
                    VelloPresenterError::recovery(error)
                } else {
                    VelloPresenterError::initialization(error)
                }
            })?;
        let device = &context.devices[surface.dev_id].device;
        let renderer = Renderer::new(device, RendererOptions::default()).map_err(|error| {
            if rebuilding {
                VelloPresenterError::recovery(error)
            } else {
                VelloPresenterError::initialization(error)
            }
        })?;
        let scope = if rebuilding {
            self.authority.activate()
        } else if self.authority.scope().is_some() {
            self.authority.replace()?
        } else {
            self.authority.activate()
        };
        let inbox = DeviceInbox::install(device, scope.clone());
        let dev_id = surface.dev_id;
        Ok((
            GpuState {
                renderer,
                context,
                dev_id,
                inbox,
            },
            surface,
            scope,
        ))
    }

    fn configure_existing_surface(
        &mut self,
        extent: Extent,
        force: bool,
    ) -> Result<(), VelloPresenterError> {
        if extent.is_zero() {
            return Err(VelloPresenterError::Validation {
                message: "zero-sized surfaces are never configured".into(),
            });
        }
        let gpu = self
            .gpu
            .as_ref()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let surface = self
            .surface
            .as_mut()
            .ok_or(VelloPresenterError::DeviceUnavailable)?;
        let current = Extent {
            width: surface.config.width,
            height: surface.config.height,
        };
        if current != extent {
            gpu.context
                .resize_surface(surface, extent.width, extent.height);
        } else if force {
            gpu.context.configure_surface(surface);
        }
        self.lifecycle.mark_surface_ready(extent);
        Ok(())
    }

    fn poll_device_events(&mut self) -> Result<bool, VelloPresenterError> {
        let Some(gpu) = self.gpu.as_ref() else {
            return Ok(false);
        };
        let Some(current) = self.authority.scope() else {
            return Ok(false);
        };
        let events = gpu.inbox.drain_current(&current);
        if events.lost {
            self.transition_device_loss()?;
            return Ok(true);
        }
        if events.overflow > 0 {
            self.pending_error = Some(VelloPresenterError::UncapturedErrorOverflow {
                dropped: events.overflow,
            });
        } else if let Some(error) = events.error {
            self.pending_error = Some(VelloPresenterError::UncapturedGpu(error));
        }
        Ok(false)
    }

    fn transition_device_loss(&mut self) -> Result<(), VelloPresenterError> {
        if self.authority.invalidate()? {
            self.surface.take();
            self.drop_gpu_state();
            self.pending_error = None;
            self.lifecycle.mark_device_lost();
        }
        Ok(())
    }

    fn drop_gpu_state(&mut self) {
        if let Some(GpuState {
            renderer,
            context,
            dev_id: _,
            inbox,
        }) = self.gpu.take()
        {
            drop(inbox);
            drop(renderer);
            drop(context);
        }
    }

    fn return_pending_error(&mut self) -> Result<(), VelloPresenterError> {
        match self.pending_error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

struct RealPresentOperations<'a> {
    surface: &'a RenderSurface<'static>,
    gpu: &'a mut GpuState,
    toolkit: &'a mut VelloRenderer,
    window: &'a Arc<Window>,
    config: &'a VelloPresenterConfig,
    current_scope: PresenterDeviceScope,
    post_render_error: &'a mut Option<VelloPresenterError>,
}

impl PresentOperations for RealPresentOperations<'_> {
    type Frame = SurfaceTexture;
    type RenderError = vello::Error;

    fn acquire(&mut self) -> AcquiredFrame<Self::Frame> {
        match self.surface.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => AcquiredFrame::Success(frame),
            CurrentSurfaceTexture::Suboptimal(frame) => AcquiredFrame::Suboptimal(frame),
            CurrentSurfaceTexture::Timeout => AcquiredFrame::Timeout,
            CurrentSurfaceTexture::Occluded => AcquiredFrame::Occluded,
            CurrentSurfaceTexture::Outdated => AcquiredFrame::Outdated,
            CurrentSurfaceTexture::Lost => AcquiredFrame::Lost,
            CurrentSurfaceTexture::Validation => AcquiredFrame::Validation,
        }
    }

    fn acquired_extent(&mut self, frame: &Self::Frame) -> Extent {
        Extent {
            width: frame.texture.width(),
            height: frame.texture.height(),
        }
    }

    fn drop_frame(&mut self, frame: Self::Frame) {
        drop(frame);
    }

    fn reconfigure(&mut self) {
        self.gpu.context.configure_surface(self.surface);
    }

    fn encode_scene(
        &mut self,
        input: RenderFrameInput<'_>,
    ) -> kinetik_ui_render::RenderFrameOutput {
        self.toolkit.submit_frame(input)
    }

    fn render_vello(&mut self) -> Result<(), Self::RenderError> {
        let device = &self.gpu.context.devices[self.gpu.dev_id];
        self.gpu.renderer.render_to_texture(
            &device.device,
            &device.queue,
            self.toolkit.scene(),
            &self.surface.target_view,
            &self
                .config
                .render_params(self.surface.config.width, self.surface.config.height),
        )
    }

    fn blit_submit(&mut self, frame: &Self::Frame) {
        let device = &self.gpu.context.devices[self.gpu.dev_id];
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = device
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("kinetik-ui-vello-winit-blit"),
            });
        self.surface.blitter.copy(
            &device.device,
            &mut encoder,
            &self.surface.target_view,
            &view,
        );
        device.queue.submit([encoder.finish()]);
    }

    fn pre_present_notify(&mut self) {
        self.window.pre_present_notify();
    }

    fn present(&mut self, frame: Self::Frame) {
        frame.present();
    }

    fn current_device_lost_after_render_failure(&mut self) -> bool {
        let events = self.gpu.inbox.drain_current(&self.current_scope);
        if let Some(error) = events.error {
            self.post_render_error
                .get_or_insert(VelloPresenterError::UncapturedGpu(error));
        }
        if !events.lost && events.overflow > 0 {
            *self.post_render_error = Some(VelloPresenterError::UncapturedErrorOverflow {
                dropped: events.overflow,
            });
        }
        events.lost
    }
}
