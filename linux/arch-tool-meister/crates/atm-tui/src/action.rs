use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    CommandCompleted {
        success: bool,
        output: String,
        error: String,
    },
    CommandStarted {
        command: String,
        description: String,
    },
    ShowLoading {
        message: String,
    },
    HideLoading,
    UpdateProgress {
        percentage: u16,
        message: String,
    },
    VersionDetected {
        module: String,
        version_info: VersionInfo,
    },
    ShowStatus {
        message: String,
        status_type: StatusType,
    },
    ClearStatus,
    AwaitingConfirmation {
        message: String,
        result_type: StatusType,
    },
    ConfirmationReceived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub stable_version: Option<String>,
    pub insiders_version: Option<String>,
    pub stable_installed: bool,
    pub insiders_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}
