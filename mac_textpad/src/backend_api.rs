//! Backend API - boundary between editor and CRDT logic.

/// intencja uzytkownika w edytorze
/// uzytkownik moze chciec wstawic tekst, usunac tekst, przesunac kursor
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

///  remotecursor do wyswietlania pozycji innych uzytkownikow
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteCursor {
    pub site_id: String,      // unikalny identyfikator uzytkownika
    pub pos: usize,           // pozycja kursora
    pub color_rgba: [f32; 4], // kolor kursora w formacie RGBA
}

/// Aktualizacja dla frontendu - zwracana przez backend po zastosowaniu intencji
#[derive(Debug, Clone, PartialEq)]
pub struct FrontendUpdate {
    pub full_text: Option<String>, // pelny tekst do zaktualizowania w edytorze
    pub remote_cursors: Vec<RemoteCursor>, // aktualizacje pozycji zdalnych kursorow
}

// pusta aktualizacja
impl FrontendUpdate {
    pub fn empty() -> Self {
        Self {
            full_text: None,
            remote_cursors: Vec::new(),
        }
    }
}

///backend sluzy do zarzadzania dokumentem i synchronizacji
///
/// Trait for document backend - to jest cos w stylu interfejsu, ktory musi byc zaimplementowany
///  przez kazdy backend (narzucone jest ze to moze byc zaimplementowane tylko dla
///  struktur ktore sa Send)
pub trait DocBackend: Send {
    // ta metoda dostaje "intencje" z edytora i zwraca aktualizacje dla edytora
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

pub struct MockBackend {
    text: String,
}

// implementacja traitu Default dla MockBackend, ktory zmusza do implementacji metody default
impl Default for MockBackend {
    fn default() -> Self {
        Self {
            // pusty tekst na start
            text: "".into(),
        }
    }
}

// implementujaemy trait DocBackend dla MockBackend
// backend bedzie musial byc w przyszlosci podmieniony
// zmuszamy do implementacji apply_intent i render_text
impl DocBackend for MockBackend {
    // kiedy dostaniemy intencje, to zaktualizujemy tekst i zwrocimy aktualizacje
    fn apply_intent(&mut self, intent: Intent) -> FrontendUpdate {
        match intent {
            Intent::ReplaceAll { text } => self.text = text,
            Intent::MoveCursor { pos } => {
                // nie robimy nic z ruchem kursora w mocku
                let _ = pos;
            }
            _ => {} // reszta narazie nie jest zaimplementowana
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
