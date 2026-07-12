use std::{cell::Cell, time::Duration};

use kinetik_ui_core::{Color, PhysicalSize as CorePhysicalSize, ScaleFactor, Size, ViewportInfo};
use kinetik_ui_render::{RenderDiagnostic, RenderFrameInput, RenderFrameOutput, RenderResources};
use vello::wgpu::{DeviceLostReason, PresentMode};

use crate::{
    InvalidColorChannel, PresenterGpuError, PresenterGpuErrorKind, VelloPresentStatus,
    VelloPresenterConfig, VelloPresenterError, VelloRecoveryKind, VelloRedrawGuidance,
    VelloResizeOutcome, VelloWindowPresenter,
    device::{DeviceAuthority, DeviceEvent, DeviceInbox},
    frame::{
        AcquiredFrame, DriveFailure, DrivenFrame, FrameOperation, PresentOperations, drive_present,
        report_for_driven,
    },
    lifecycle::{
        DEVICE_REBUILD_SEQUENCE, DeviceRecoveryAction, DropAction, Extent, LifecycleState,
        ResizePlan, ResumePlan,
    },
};

#[derive(Debug, Clone, Copy)]
enum FakeAcquire {
    Success(Extent),
    Suboptimal(Extent),
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

struct FakeOperations {
    acquire: FakeAcquire,
    operations: Vec<FrameOperation>,
    render_fails: bool,
    current_loss_after_failure: bool,
    output: RenderFrameOutput,
}

impl FakeOperations {
    fn new(acquire: FakeAcquire) -> Self {
        Self {
            acquire,
            operations: Vec::new(),
            render_fails: false,
            current_loss_after_failure: false,
            output: RenderFrameOutput {
                primitive_count: 0,
                diagnostics: Vec::new(),
            },
        }
    }
}

impl PresentOperations for FakeOperations {
    type Frame = Extent;
    type RenderError = &'static str;

    fn acquire(&mut self) -> AcquiredFrame<Self::Frame> {
        self.operations.push(FrameOperation::Acquire);
        match self.acquire {
            FakeAcquire::Success(frame) => AcquiredFrame::Success(frame),
            FakeAcquire::Suboptimal(frame) => AcquiredFrame::Suboptimal(frame),
            FakeAcquire::Timeout => AcquiredFrame::Timeout,
            FakeAcquire::Occluded => AcquiredFrame::Occluded,
            FakeAcquire::Outdated => AcquiredFrame::Outdated,
            FakeAcquire::Lost => AcquiredFrame::Lost,
            FakeAcquire::Validation => AcquiredFrame::Validation,
        }
    }

    fn acquired_extent(&mut self, frame: &Self::Frame) -> Extent {
        self.operations.push(FrameOperation::ValidateAcquiredExtent);
        *frame
    }

    fn drop_frame(&mut self, _frame: Self::Frame) {
        self.operations.push(FrameOperation::DropAcquired);
    }

    fn reconfigure(&mut self) {
        self.operations.push(FrameOperation::Reconfigure);
    }

    fn encode_scene(&mut self, _input: RenderFrameInput<'_>) -> RenderFrameOutput {
        self.operations.push(FrameOperation::EncodeScene);
        self.output.clone()
    }

    fn render_vello(&mut self) -> Result<(), Self::RenderError> {
        self.operations.push(FrameOperation::VelloRenderSubmit);
        if self.render_fails {
            Err("injected Vello failure")
        } else {
            Ok(())
        }
    }

    fn blit_submit(&mut self, _frame: &Self::Frame) {
        self.operations.push(FrameOperation::BlitSubmit);
    }

    fn pre_present_notify(&mut self) {
        self.operations.push(FrameOperation::PrePresentNotify);
    }

    fn present(&mut self, _frame: Self::Frame) {
        self.operations.push(FrameOperation::Present);
    }

    fn current_device_lost_after_render_failure(&mut self) -> bool {
        self.operations.push(FrameOperation::PollAfterRenderFailure);
        self.current_loss_after_failure
    }
}

fn input(resources: &RenderResources, extent: Extent) -> RenderFrameInput<'_> {
    RenderFrameInput {
        viewport: ViewportInfo::new(
            Size::new(1.0, 1.0),
            CorePhysicalSize::new(extent.width, extent.height),
            ScaleFactor::ONE,
        ),
        primitives: &[],
        resources,
    }
}

#[test]
fn successful_frame_uses_the_exact_present_order_and_preserves_diagnostics() {
    let extent = Extent {
        width: 800,
        height: 600,
    };
    let resources = RenderResources::new();
    let mut fake = FakeOperations::new(FakeAcquire::Success(extent));
    fake.output = RenderFrameOutput {
        primitive_count: 7,
        diagnostics: vec![RenderDiagnostic::MissingImage(
            kinetik_ui_core::ImageId::from_raw(9),
        )],
    };

    let driven = drive_present(&mut fake, input(&resources, extent), extent).unwrap();
    let report = report_for_driven(driven, Duration::from_millis(16)).unwrap();

    assert_eq!(
        fake.operations,
        vec![
            FrameOperation::Acquire,
            FrameOperation::ValidateAcquiredExtent,
            FrameOperation::EncodeScene,
            FrameOperation::VelloRenderSubmit,
            FrameOperation::BlitSubmit,
            FrameOperation::PrePresentNotify,
            FrameOperation::Present,
        ]
    );
    assert_eq!(report.status(), VelloPresentStatus::Presented);
    assert_eq!(report.redraw(), VelloRedrawGuidance::UseApplicationRequest);
    assert_eq!(report.frame_output(), Some(&fake.output));
}

#[test]
fn stale_frame_extent_skips_acquire_and_every_scene_or_gpu_operation() {
    let configured = Extent {
        width: 800,
        height: 600,
    };
    let stale = Extent {
        width: 801,
        height: 600,
    };
    let resources = RenderResources::new();
    let mut fake = FakeOperations::new(FakeAcquire::Success(configured));

    let driven = drive_present(&mut fake, input(&resources, stale), configured).unwrap();
    let report = report_for_driven(driven, Duration::from_millis(16)).unwrap();

    assert!(fake.operations.is_empty());
    assert_eq!(report.status(), VelloPresentStatus::FrameExtentOutdated);
    assert_eq!(report.redraw(), VelloRedrawGuidance::NextFrame);
    assert!(report.frame_output().is_none());
}

#[test]
fn acquired_extent_mismatch_drops_before_one_reconfigure_and_never_reacquires() {
    let configured = Extent {
        width: 800,
        height: 600,
    };
    let acquired = Extent {
        width: 799,
        height: 600,
    };
    let resources = RenderResources::new();
    let mut fake = FakeOperations::new(FakeAcquire::Success(acquired));

    let driven = drive_present(&mut fake, input(&resources, configured), configured).unwrap();

    assert_eq!(driven, DrivenFrame::AcquiredExtentOutdated);
    assert_eq!(
        fake.operations,
        vec![
            FrameOperation::Acquire,
            FrameOperation::ValidateAcquiredExtent,
            FrameOperation::DropAcquired,
            FrameOperation::Reconfigure,
        ]
    );
}

#[test]
fn acquisition_result_and_redraw_matrix_is_literal() {
    let extent = Extent {
        width: 640,
        height: 360,
    };
    let resources = RenderResources::new();
    let cases = [
        (
            FakeAcquire::Timeout,
            VelloPresentStatus::Timeout,
            VelloRedrawGuidance::Later(Duration::from_millis(25)),
            vec![FrameOperation::Acquire],
        ),
        (
            FakeAcquire::Occluded,
            VelloPresentStatus::Occluded,
            VelloRedrawGuidance::ExternalEvent,
            vec![FrameOperation::Acquire],
        ),
        (
            FakeAcquire::Outdated,
            VelloPresentStatus::Outdated,
            VelloRedrawGuidance::NextFrame,
            vec![FrameOperation::Acquire, FrameOperation::Reconfigure],
        ),
        (
            FakeAcquire::Lost,
            VelloPresentStatus::SurfaceLost,
            VelloRedrawGuidance::NextFrame,
            vec![FrameOperation::Acquire],
        ),
    ];

    for (acquire, status, redraw, operations) in cases {
        let mut fake = FakeOperations::new(acquire);
        let driven = drive_present(&mut fake, input(&resources, extent), extent).unwrap();
        let report = report_for_driven(driven, Duration::from_millis(25)).unwrap();
        assert_eq!(report.status(), status);
        assert_eq!(report.redraw(), redraw);
        assert!(report.frame_output().is_none());
        assert_eq!(fake.operations, operations);
    }

    let mut validation = FakeOperations::new(FakeAcquire::Validation);
    let driven = drive_present(&mut validation, input(&resources, extent), extent).unwrap();
    assert!(matches!(
        report_for_driven(driven, Duration::from_millis(25)),
        Err(VelloPresenterError::Validation { .. })
    ));
    assert_eq!(validation.operations, vec![FrameOperation::Acquire]);
}

#[test]
fn suboptimal_frame_presents_once_and_reports_next_frame() {
    let extent = Extent {
        width: 640,
        height: 360,
    };
    let resources = RenderResources::new();
    let mut fake = FakeOperations::new(FakeAcquire::Suboptimal(extent));

    let driven = drive_present(&mut fake, input(&resources, extent), extent).unwrap();
    let report = report_for_driven(driven, Duration::from_millis(16)).unwrap();

    assert_eq!(report.status(), VelloPresentStatus::PresentedSuboptimal);
    assert_eq!(report.redraw(), VelloRedrawGuidance::NextFrame);
    assert_eq!(
        fake.operations
            .iter()
            .filter(|operation| **operation == FrameOperation::Acquire)
            .count(),
        1
    );
    assert_eq!(fake.operations.last(), Some(&FrameOperation::Present));
}

#[test]
fn suboptimal_reconfigure_precedes_the_next_single_acquire() {
    let extent = Extent {
        width: 640,
        height: 360,
    };
    let resources = RenderResources::new();
    let mut lifecycle = LifecycleState::new();
    lifecycle.resume(1_u8, extent).unwrap();
    lifecycle.mark_surface_ready(extent);
    let mut first = FakeOperations::new(FakeAcquire::Suboptimal(extent));
    let first_result = drive_present(&mut first, input(&resources, extent), extent).unwrap();
    assert!(matches!(
        first_result,
        DrivenFrame::Presented {
            suboptimal: true,
            ..
        }
    ));
    lifecycle.mark_reconfigure();
    assert_eq!(
        lifecycle.resize(extent),
        ResizePlan::Configure {
            extent,
            force: true,
        }
    );

    let mut second = FakeOperations::new(FakeAcquire::Success(extent));
    second.operations.push(FrameOperation::Reconfigure);
    drive_present(&mut second, input(&resources, extent), extent).unwrap();

    assert_eq!(second.operations[0], FrameOperation::Reconfigure);
    assert_eq!(second.operations[1], FrameOperation::Acquire);
    assert_eq!(
        second
            .operations
            .iter()
            .filter(|operation| **operation == FrameOperation::Acquire)
            .count(),
        1
    );
}

#[test]
fn render_failure_drops_frame_polls_inbox_and_does_not_blit_notify_or_present() {
    let extent = Extent {
        width: 640,
        height: 360,
    };
    let resources = RenderResources::new();
    let mut fake = FakeOperations::new(FakeAcquire::Success(extent));
    fake.render_fails = true;

    let error = drive_present(&mut fake, input(&resources, extent), extent).unwrap_err();

    assert_eq!(error, DriveFailure::Render("injected Vello failure"));
    assert_eq!(
        fake.operations,
        vec![
            FrameOperation::Acquire,
            FrameOperation::ValidateAcquiredExtent,
            FrameOperation::EncodeScene,
            FrameOperation::VelloRenderSubmit,
            FrameOperation::DropAcquired,
            FrameOperation::PollAfterRenderFailure,
        ]
    );
}

#[test]
fn current_loss_after_render_failure_wins_recovery_transition() {
    let extent = Extent {
        width: 320,
        height: 200,
    };
    let resources = RenderResources::new();
    let mut fake = FakeOperations::new(FakeAcquire::Success(extent));
    fake.render_fails = true;
    fake.current_loss_after_failure = true;

    assert_eq!(
        drive_present(&mut fake, input(&resources, extent), extent),
        Err(DriveFailure::DeviceLostAfterRender)
    );
    assert_eq!(
        fake.operations.last(),
        Some(&FrameOperation::PollAfterRenderFailure)
    );
}

#[test]
fn lifecycle_proves_zero_restore_resize_and_surface_before_window_drop() {
    let mut lifecycle = LifecycleState::new();
    assert_eq!(
        lifecycle.resume(7_u8, Extent::ZERO).unwrap(),
        ResumePlan::AttachedZeroSized
    );
    assert_eq!(
        lifecycle.resize(Extent {
            width: 800,
            height: 600,
        }),
        ResizePlan::Outcome(VelloResizeOutcome::RecoveryRequired(
            VelloRecoveryKind::CreateSurface
        ))
    );
    lifecycle.mark_surface_ready(Extent {
        width: 800,
        height: 600,
    });
    assert_eq!(
        lifecycle.resize(Extent {
            width: 800,
            height: 600,
        }),
        ResizePlan::Outcome(VelloResizeOutcome::Unchanged)
    );
    assert_eq!(
        lifecycle.resize(Extent {
            width: 801,
            height: 600,
        }),
        ResizePlan::Configure {
            extent: Extent {
                width: 801,
                height: 600,
            },
            force: false,
        }
    );
    lifecycle.mark_surface_ready(Extent {
        width: 801,
        height: 600,
    });
    assert_eq!(
        lifecycle.suspend().actions,
        vec![DropAction::Surface, DropAction::Window]
    );
    assert!(lifecycle.suspend().actions.is_empty());
}

#[test]
fn lifecycle_rejects_wrong_window_and_distinguishes_reconfigure_recreate_rebuild() {
    let extent = Extent {
        width: 800,
        height: 600,
    };
    let mut lifecycle = LifecycleState::new();
    assert_eq!(
        lifecycle.resume(1_u8, extent).unwrap(),
        ResumePlan::Recover(VelloRecoveryKind::CreateSurface)
    );
    assert_eq!(
        lifecycle.resume(1_u8, extent).unwrap(),
        ResumePlan::AlreadyAttached
    );
    assert!(matches!(
        lifecycle.resume(2_u8, extent),
        Err(VelloPresenterError::WrongWindow)
    ));
    lifecycle.mark_surface_ready(extent);
    lifecycle.mark_reconfigure();
    assert_eq!(
        lifecycle.resize(extent),
        ResizePlan::Configure {
            extent,
            force: true,
        }
    );
    lifecycle.mark_surface_lost();
    assert_eq!(
        lifecycle.recovery(),
        Some(VelloRecoveryKind::RecreateSurface)
    );
    lifecycle.mark_device_lost();
    assert_eq!(lifecycle.recovery(), Some(VelloRecoveryKind::RebuildDevice));
    assert_eq!(lifecycle.desired(), extent);
}

#[test]
fn whole_device_rebuild_order_drops_every_old_owner_before_creation_and_configure() {
    assert_eq!(
        DEVICE_REBUILD_SEQUENCE,
        [
            DeviceRecoveryAction::DropSurface,
            DeviceRecoveryAction::DropRenderer,
            DeviceRecoveryAction::DropContext,
            DeviceRecoveryAction::CreateContext,
            DeviceRecoveryAction::CreateSurface,
            DeviceRecoveryAction::CreateRenderer,
            DeviceRecoveryAction::ConfigureSurface,
        ]
    );
}

#[test]
fn scope_validation_rejects_foreign_and_stale_before_closure_invocation() {
    let mut first = DeviceAuthority::for_test(1, 9, false);
    let mut second = DeviceAuthority::for_test(2, 9, false);
    let first_scope = first.activate();
    let foreign_same_generation = second.activate();
    let called = Cell::new(false);
    let validate_then_call = |authority: &DeviceAuthority, scope, called: &Cell<bool>| {
        authority.validate(scope)?;
        called.set(true);
        Ok::<_, VelloPresenterError>(())
    };

    assert_eq!(
        validate_then_call(&first, &foreign_same_generation, &called),
        Err(VelloPresenterError::ForeignPresenterScope)
    );
    assert!(!called.get());
    first.invalidate().unwrap();
    assert_eq!(
        validate_then_call(&first, &first_scope, &called),
        Err(VelloPresenterError::StaleDeviceScope)
    );
    assert!(!called.get());
}

#[test]
fn same_device_preserves_scope_and_replacement_advances_checked_generation() {
    let mut authority = DeviceAuthority::for_test(7, 3, false);
    let first = authority.activate();
    let same = authority.activate();
    assert_eq!(same, first);
    let replacement = authority.replace().unwrap();
    assert_ne!(replacement, first);
    assert_eq!(
        authority.validate(&first),
        Err(VelloPresenterError::StaleDeviceScope)
    );

    let mut exhausted = DeviceAuthority::for_test(8, u64::MAX, true);
    assert_eq!(
        exhausted.invalidate(),
        Err(VelloPresenterError::GenerationExhausted)
    );
}

#[test]
fn callback_inbox_preserves_current_error_and_ignores_stale_events() {
    let mut current_authority = DeviceAuthority::for_test(1, 4, false);
    let mut stale_authority = DeviceAuthority::for_test(1, 3, false);
    let current = current_authority.activate();
    let stale = stale_authority.activate();
    let (inbox, sender) = DeviceInbox::for_test();
    sender.send(DeviceEvent::Error {
        scope: stale,
        error: PresenterGpuError::new(PresenterGpuErrorKind::Internal, "stale"),
    });
    sender.send(DeviceEvent::Error {
        scope: current.clone(),
        error: PresenterGpuError::new(PresenterGpuErrorKind::Validation, "exact message"),
    });
    sender.send(DeviceEvent::Lost {
        scope: DeviceAuthority::for_test(99, 1, true).scope().unwrap(),
        reason: DeviceLostReason::Unknown,
        message: "foreign loss".into(),
    });

    let events = inbox.drain_current(&current);

    assert!(!events.lost);
    let error = events.error.unwrap();
    assert_eq!(error.kind(), PresenterGpuErrorKind::Validation);
    assert_eq!(error.message(), "exact message");
    assert_eq!(events.overflow, 0);
}

#[test]
fn current_loss_applies_once_and_failed_rebuild_keeps_desired_resize_pending() {
    let initial = Extent {
        width: 800,
        height: 600,
    };
    let desired = Extent {
        width: 1024,
        height: 768,
    };
    let mut authority = DeviceAuthority::for_test(5, 2, false);
    let scope = authority.activate();
    let (inbox, sender) = DeviceInbox::for_test();
    sender.send(DeviceEvent::Lost {
        scope,
        reason: DeviceLostReason::Unknown,
        message: "device removed".into(),
    });
    let events = inbox.drain_current(&authority.scope().unwrap());
    assert!(events.lost);
    assert!(authority.invalidate().unwrap());
    assert!(!authority.invalidate().unwrap());

    let mut lifecycle = LifecycleState::new();
    lifecycle.resume(1_u8, initial).unwrap();
    lifecycle.mark_surface_ready(initial);
    assert_eq!(
        lifecycle.resize(desired),
        ResizePlan::Configure {
            extent: desired,
            force: false,
        }
    );
    lifecycle.mark_device_lost();
    assert_eq!(lifecycle.desired(), desired);
    assert_eq!(lifecycle.recovery(), Some(VelloRecoveryKind::RebuildDevice));
    assert_eq!(
        lifecycle.resize(desired),
        ResizePlan::Outcome(VelloResizeOutcome::RecoveryRequired(
            VelloRecoveryKind::RebuildDevice
        ))
    );
}

#[test]
fn callback_inbox_is_bounded_and_reports_overflow() {
    let mut authority = DeviceAuthority::for_test(1, 1, false);
    let scope = authority.activate();
    let (inbox, sender) = DeviceInbox::for_test();
    for index in 0..33 {
        sender.send(DeviceEvent::Error {
            scope: scope.clone(),
            error: PresenterGpuError::new(
                PresenterGpuErrorKind::Validation,
                format!("error {index}"),
            ),
        });
    }

    let events = inbox.drain_current(&scope);

    assert_eq!(events.error.unwrap().message(), "error 0");
    assert_eq!(events.overflow, 1);
}

#[test]
fn config_validates_modes_color_channels_and_propagates_values() {
    let color = Color::rgba(0.25, 0.5, 0.75, 0.8);
    let config = VelloPresenterConfig::new()
        .with_present_mode(PresentMode::AutoVsync)
        .unwrap()
        .with_base_color(color)
        .unwrap()
        .with_timeout_retry(Duration::from_millis(42))
        .unwrap();
    assert_eq!(config.present_mode(), PresentMode::AutoVsync);
    assert_eq!(config.base_color(), color);
    assert_eq!(config.timeout_retry(), Duration::from_millis(42));
    let params = config.render_params(1920, 1080);
    assert_eq!(params.width, 1920);
    assert_eq!(params.height, 1080);
    assert_eq!(params.antialiasing_method, config.antialiasing_method());
    assert_eq!(
        params.base_color.components.map(f32::to_bits),
        [0.25, 0.5, 0.75, 0.8].map(f32::to_bits)
    );
    assert!(matches!(
        VelloPresenterConfig::new().with_present_mode(PresentMode::Fifo),
        Err(VelloPresenterError::Validation { .. })
    ));

    for (color, channel) in [
        (
            Color::rgba(f32::NAN, 0.0, 0.0, 1.0),
            InvalidColorChannel::Red,
        ),
        (Color::rgba(0.0, -0.1, 0.0, 1.0), InvalidColorChannel::Green),
        (Color::rgba(0.0, 0.0, 1.1, 1.0), InvalidColorChannel::Blue),
        (
            Color::rgba(0.0, 0.0, 0.0, f32::INFINITY),
            InvalidColorChannel::Alpha,
        ),
    ] {
        assert!(matches!(
            VelloPresenterConfig::new().with_base_color(color),
            Err(VelloPresenterError::InvalidBaseColor { channel: actual }) if actual == channel
        ));
    }
}

#[test]
fn constructing_presenter_is_gpu_free_and_detached() {
    let presenter = VelloWindowPresenter::new(VelloPresenterConfig::new()).unwrap();
    assert_eq!(
        presenter.status().attachment(),
        crate::VelloAttachmentStatus::Detached
    );
    assert!(presenter.status().device_scope().is_none());
    assert!(presenter.window().is_none());
    assert!(presenter.window_id().is_none());
}
