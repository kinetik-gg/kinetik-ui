use kinetik_ui_core::PhysicalSize as CorePhysicalSize;
use winit::dpi::PhysicalSize;

use crate::{VelloAttachmentStatus, VelloPresenterError, VelloRecoveryKind, VelloResizeOutcome};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Extent {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Extent {
    pub(crate) const ZERO: Self = Self {
        width: 0,
        height: 0,
    };

    pub(crate) const fn is_zero(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

impl From<PhysicalSize<u32>> for Extent {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<CorePhysicalSize> for Extent {
    fn from(size: CorePhysicalSize) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResumePlan {
    AlreadyAttached,
    AttachedZeroSized,
    Recover(VelloRecoveryKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DropAction {
    Surface,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeviceRecoveryAction {
    DropSurface,
    DropRenderer,
    DropContext,
    CreateContext,
    CreateSurface,
    CreateRenderer,
    ConfigureSurface,
}

pub(crate) const DEVICE_REBUILD_SEQUENCE: [DeviceRecoveryAction; 7] = [
    DeviceRecoveryAction::DropSurface,
    DeviceRecoveryAction::DropRenderer,
    DeviceRecoveryAction::DropContext,
    DeviceRecoveryAction::CreateContext,
    DeviceRecoveryAction::CreateSurface,
    DeviceRecoveryAction::CreateRenderer,
    DeviceRecoveryAction::ConfigureSurface,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SuspendPlan {
    pub(crate) actions: Vec<DropAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResizePlan {
    Outcome(VelloResizeOutcome),
    ZeroSized { drop_surface: bool },
    Configure { extent: Extent, force: bool },
}

#[derive(Debug)]
pub(crate) struct LifecycleState<W> {
    window: Option<W>,
    desired: Extent,
    configured: Option<Extent>,
    recovery: Option<VelloRecoveryKind>,
    reconfigure: bool,
}

impl<W: Copy + Eq> LifecycleState<W> {
    pub(crate) const fn new() -> Self {
        Self {
            window: None,
            desired: Extent::ZERO,
            configured: None,
            recovery: None,
            reconfigure: false,
        }
    }

    pub(crate) fn resume(
        &mut self,
        window: W,
        extent: Extent,
    ) -> Result<ResumePlan, VelloPresenterError> {
        if let Some(current) = self.window {
            return if current == window {
                Ok(ResumePlan::AlreadyAttached)
            } else {
                Err(VelloPresenterError::WrongWindow)
            };
        }

        self.window = Some(window);
        self.desired = extent;
        if extent.is_zero() {
            if self.recovery != Some(VelloRecoveryKind::RebuildDevice) {
                self.recovery = None;
            }
            return Ok(ResumePlan::AttachedZeroSized);
        }
        if self.recovery.is_none() {
            self.recovery = Some(VelloRecoveryKind::CreateSurface);
        }
        Ok(ResumePlan::Recover(
            self.recovery.expect("recovery was selected"),
        ))
    }

    pub(crate) fn suspend(&mut self) -> SuspendPlan {
        let mut actions = Vec::new();
        if self.configured.take().is_some() {
            actions.push(DropAction::Surface);
        }
        if self.window.take().is_some() {
            actions.push(DropAction::Window);
        }
        self.desired = Extent::ZERO;
        self.reconfigure = false;
        if self.recovery != Some(VelloRecoveryKind::RebuildDevice) {
            self.recovery = None;
        }
        SuspendPlan { actions }
    }

    pub(crate) fn resize(&mut self, extent: Extent) -> ResizePlan {
        if self.window.is_none() {
            return ResizePlan::Outcome(VelloResizeOutcome::Detached);
        }
        let unchanged = self.desired == extent;
        self.desired = extent;
        if extent.is_zero() {
            let drop_surface = self.configured.take().is_some();
            self.reconfigure = false;
            if self.recovery != Some(VelloRecoveryKind::RebuildDevice) {
                self.recovery = None;
            }
            return ResizePlan::ZeroSized { drop_surface };
        }
        if let Some(recovery) = self.recovery {
            return ResizePlan::Outcome(VelloResizeOutcome::RecoveryRequired(recovery));
        }
        let Some(configured) = self.configured else {
            self.recovery = Some(VelloRecoveryKind::CreateSurface);
            return ResizePlan::Outcome(VelloResizeOutcome::RecoveryRequired(
                VelloRecoveryKind::CreateSurface,
            ));
        };
        if unchanged && configured == extent && !self.reconfigure {
            return ResizePlan::Outcome(VelloResizeOutcome::Unchanged);
        }
        ResizePlan::Configure {
            extent,
            force: self.reconfigure || configured == extent,
        }
    }

    pub(crate) fn mark_surface_ready(&mut self, extent: Extent) {
        self.configured = Some(extent);
        self.recovery = None;
        self.reconfigure = false;
    }

    pub(crate) fn mark_reconfigure(&mut self) {
        if self.configured.is_some() {
            self.reconfigure = true;
        }
    }

    pub(crate) fn mark_surface_lost(&mut self) {
        self.configured = None;
        self.reconfigure = false;
        self.recovery = Some(VelloRecoveryKind::RecreateSurface);
    }

    pub(crate) fn mark_device_lost(&mut self) {
        self.configured = None;
        self.reconfigure = false;
        self.recovery = Some(VelloRecoveryKind::RebuildDevice);
    }

    pub(crate) const fn window(&self) -> Option<W> {
        self.window
    }

    pub(crate) const fn desired(&self) -> Extent {
        self.desired
    }

    pub(crate) const fn configured(&self) -> Option<Extent> {
        self.configured
    }

    pub(crate) const fn recovery(&self) -> Option<VelloRecoveryKind> {
        self.recovery
    }

    pub(crate) fn attachment_status(&self) -> VelloAttachmentStatus {
        if self.window.is_none() {
            VelloAttachmentStatus::Detached
        } else if self.desired.is_zero() {
            VelloAttachmentStatus::ZeroSized
        } else if self.configured.is_some() && self.recovery.is_none() {
            VelloAttachmentStatus::Presentable
        } else {
            VelloAttachmentStatus::RecoveryPending
        }
    }
}
