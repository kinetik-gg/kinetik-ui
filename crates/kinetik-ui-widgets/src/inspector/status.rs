/// Validation or help status severity for a property-grid row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropertyGridStatusSeverity {
    /// No status is attached to the row.
    #[default]
    None,
    /// Informational row status.
    Info,
    /// Non-blocking warning row status.
    Warning,
    /// Blocking error row status.
    Error,
}

/// Deterministic presentation metadata for row validation or help status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyGridStatusPresentation {
    /// Status severity.
    pub severity: PropertyGridStatusSeverity,
    /// Stable compact status label.
    pub label: &'static str,
    /// True when the row should show a status accent.
    pub accented: bool,
    /// True when the status should be treated as blocking validation.
    pub blocking: bool,
}

impl PropertyGridStatusSeverity {
    /// Returns deterministic presentation metadata for this severity.
    #[must_use]
    pub const fn presentation(self) -> PropertyGridStatusPresentation {
        match self {
            Self::None => PropertyGridStatusPresentation {
                severity: self,
                label: "None",
                accented: false,
                blocking: false,
            },
            Self::Info => PropertyGridStatusPresentation {
                severity: self,
                label: "Info",
                accented: true,
                blocking: false,
            },
            Self::Warning => PropertyGridStatusPresentation {
                severity: self,
                label: "Warning",
                accented: true,
                blocking: false,
            },
            Self::Error => PropertyGridStatusPresentation {
                severity: self,
                label: "Error",
                accented: true,
                blocking: true,
            },
        }
    }
}

/// Data-only validation or help status for a property-grid row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropertyGridRowStatus {
    /// Status severity.
    pub severity: PropertyGridStatusSeverity,
    /// Optional status message owned by the application.
    pub message: Option<String>,
}

impl PropertyGridRowStatus {
    /// Creates a row status with the given severity and no message.
    #[must_use]
    pub const fn severity(severity: PropertyGridStatusSeverity) -> Self {
        Self {
            severity,
            message: None,
        }
    }

    /// Creates an informational row status.
    #[must_use]
    pub fn info(message: impl Into<String>) -> Self {
        Self::severity(PropertyGridStatusSeverity::Info).with_message(message)
    }

    /// Creates a warning row status.
    #[must_use]
    pub fn warning(message: impl Into<String>) -> Self {
        Self::severity(PropertyGridStatusSeverity::Warning).with_message(message)
    }

    /// Creates an error row status.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::severity(PropertyGridStatusSeverity::Error).with_message(message)
    }

    /// Returns this status with an attached message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns true when this status represents a blocking error.
    #[must_use]
    pub const fn is_blocking_error(&self) -> bool {
        matches!(self.severity, PropertyGridStatusSeverity::Error)
    }

    /// Returns deterministic presentation metadata for this status.
    #[must_use]
    pub const fn presentation(&self) -> PropertyGridStatusPresentation {
        self.severity.presentation()
    }

    /// Returns accessible status text including severity and message when present.
    #[must_use]
    pub fn semantic_text(&self) -> Option<String> {
        let presentation = self.presentation();
        if matches!(presentation.severity, PropertyGridStatusSeverity::None) {
            return None;
        }

        Some(match self.message.as_deref() {
            Some(message) if !message.is_empty() => format!("{}: {message}", presentation.label),
            _ => presentation.label.to_owned(),
        })
    }
}
