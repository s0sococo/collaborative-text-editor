//! Backend API - boundary between editor and CRDT logic.

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
    pub site_id: String,      // peer identifier
    pub pos: usize,           // caret position in text
    pub color_rgba: [f32; 4], // color to display caret
}

/// Update from the backend to the editor
///
/// full text snapshot + remote carets
#[derive(Debug, Clone, PartialEq)]
pub struct FrontendUpdate {
    pub full_text: Option<String>,
    pub remote_cursors: Vec<RemoteCursor>,
}

impl FrontendUpdate {
    pub fn empty() -> Self {
        Self {
            full_text: None,
            remote_cursors: Vec::new(),
        }
    }
}

/// trait - document enginge must implement
pub trait DocBackend: Send {
    // apply intent from editor, return update for editor
    fn apply_intent(&mut self, intent: Intent) -> FrontendUpdate;

    // apply remote update from other peers, return update for editor, default empty
    fn apply_remote(&mut self, _bytes: &[u8]) -> FrontendUpdate {
        FrontendUpdate::empty()
    }

    /// Current full text (used for initial paint and saving)
    fn render_text(&self) -> String;

    // current remote cursor states , default empty
    fn remote_cursors(&self) -> Vec<RemoteCursor> {
        Vec::new()
    }
}