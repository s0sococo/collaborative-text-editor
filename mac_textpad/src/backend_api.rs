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

#[cfg(test)]
mod tests {
    use super::*;
    struct MockBackend {
        text: String,
    }
    impl Default for MockBackend {
        fn default() -> Self {
            Self {
                text: String::new(),
            }
        }
    }

    impl DocBackend for MockBackend {
        fn apply_intent(&mut self, intent: Intent) -> FrontendUpdate {
            match intent {
                Intent::ReplaceAll { text } => {
                    self.text = text;
                }
                Intent::InsertAt { pos, text } => {
                    let pos = pos.min(self.text.chars().count());
                    let byt = nth_char_to_byte(&self.text, pos);
                    self.text.insert_str(byt, &text);
                }
                Intent::DeleteRange { start, end } => {
                    let len = self.text.chars().count();
                    let s = start.min(len);
                    let e = end.min(len);
                    if s < e {
                        let sb = nth_char_to_byte(&self.text, s);
                        let eb = nth_char_to_byte(&self.text, e);
                        self.text.replace_range(sb..eb, "");
                    }
                }
                Intent::MoveCursor { .. } => {}
            }
            FrontendUpdate {
                full_text: Some(self.text.clone()),
                remote_cursors: vec![],
            }
        }

        fn render_text(&self) -> String {
            self.text.clone()
        }
    }

    fn nth_char_to_byte(s: &str, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        let mut count = 0usize;
        for (i, _) in s.char_indices() {
            if count == n {
                return i;
            }
            count += 1;
        }
        s.len()
    }

    #[test]
    fn mock_backend_basic_flow() {
        let mut b = MockBackend::default();
        b.apply_intent(Intent::ReplaceAll { text: "abc".into() });
        b.apply_intent(Intent::InsertAt {
            pos: 1,
            text: "X".into(),
        });
        b.apply_intent(Intent::DeleteRange { start: 2, end: 3 });
        assert_eq!(b.render_text(), "aXc");
    }
}
