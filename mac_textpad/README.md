## App level architecture
Based on visualization from document 2. 
```
App (eframe)
│
├── Sidebar
│   ├── Document list (click → open)
│   └── Menu buttons (New, Share, Delete)
│
└── EditorView
    ├── Text area (backed by your `DocBackend` API)
    ├── Local caret
    └── Remote carets (colored)
```