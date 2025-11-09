//! Backend API - boundary between editor and CRDT logic.

use core::str;

/// Intent coming from the editor to the editor engine
///
/// Position bases - CRDT can translate positions to IDs
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// insert 'text' at 'pos' (cursor)
    InsertAt { pos: usize, text: String },
    /// Delete [Start, End)
    DeleteRange { start: usize, end: usize },
    /// Local Caret Movement
    MoveCursor { pos: usize },
    /// replace entire text with 'text' - ex. opening a file
    ReplaceAll { text: String },
}


/// Remote peers caret in editor
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteCursor {
    pub site_id: String, // peer identifier
    pub pos: usize, // caret position in text
    pub color_rgba:[f32; 4] // color to display caret
}


