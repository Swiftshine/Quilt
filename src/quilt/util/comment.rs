use crate::quilt::common::Point2D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Comment {
    /// The file this comment is for, e.g. "file.mapbin"
    pub file: String,
    pub position: Point2D,
    pub contents: String,

    #[serde(skip)] // editor-only, don't include in file
    pub is_selected: bool,
}
