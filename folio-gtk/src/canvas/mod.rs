use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Duration;
use gtk4::prelude::*;
use gtk4::{DrawingArea, EventControllerKey};
use glib;
use folio_core::{Document, DocPosition, CrdtEngine};

pub mod layout;
pub mod render;
pub mod cursor;
pub mod selection;
pub mod input;
pub mod hittest;

/// Shared mutable editor state — lives behind Rc<RefCell<>> so it can be
/// captured by multiple GTK closures (draw func, key handler, blink timer,
/// auto-save timer).
pub struct EditorState {
    pub doc:            Document,
    pub engine:         CrdtEngine,
    pub cursor:         DocPosition,
    /// Toggled by the blink timer; draw func reads this to show/hide cursor.
    pub cursor_visible: bool,
    /// Path to the on-disk .folio file, if the document has been saved.
    pub save_path:      Option<PathBuf>,
    /// True if there are unsaved changes since the last save/checkpoint.
    pub dirty:          bool,
}

impl EditorState {
    pub fn new(doc: Document) -> Self {
        EditorState {
            cursor:         DocPosition::block_start(0),
            engine:         CrdtEngine::new(),
            doc,
            cursor_visible: true,
            save_path:      None,
            dirty:          false,
        }
    }

    pub fn with_path(doc: Document, path: PathBuf, engine: CrdtEngine) -> Self {
        EditorState {
            cursor:         DocPosition::block_start(0),
            engine,
            doc,
            cursor_visible: true,
            save_path:      Some(path),
            dirty:          false,
        }
    }
}

pub struct EditorCanvas {
    pub widget: DrawingArea,
    pub state:  Rc<RefCell<EditorState>>,
}

impl EditorCanvas {
    pub fn new(doc: Document) -> Self {
        let state = Rc::new(RefCell::new(EditorState::new(doc)));
        let da    = DrawingArea::new();

        da.set_content_width(874);
        da.set_content_height(1203);
        da.set_focusable(true);

        // ── Draw function ──────────────────────────────────────────────────
        {
            let s = state.clone();
            da.set_draw_func(move |_, cr, _, _| render::draw(cr, &s.borrow()));
        }

        // ── Keyboard input ─────────────────────────────────────────────────
        let kc = EventControllerKey::new();
        {
            let s = state.clone();
            let d = da.clone();
            kc.connect_key_pressed(move |_, key, _, mods| {
                input::handle_key(key, mods, &mut s.borrow_mut(), &d);
                glib::Propagation::Stop
            });
        }
        da.add_controller(kc);

        // ── Cursor blink timer ─────────────────────────────────────────────
        {
            let s = state.clone();
            let d = da.clone();
            glib::timeout_add_local(Duration::from_millis(530), move || {
                s.borrow_mut().cursor_visible ^= true;
                d.queue_draw();
                glib::ControlFlow::Continue
            });
        }

        // ── Auto-save timer (every 2 seconds if dirty) ─────────────────────
        {
            let s = state.clone();
            glib::timeout_add_local(Duration::from_secs(2), move || {
                let mut st = s.borrow_mut();
                if st.dirty {
                    if let Some(ref path) = st.save_path.clone() {
                        let path = path.clone();
                        if let Err(e) = folio_core::format::save_folio(
                            &path, &st.doc, &st.engine, &[]
                        ) {
                            eprintln!("Auto-save failed: {e}");
                        } else {
                            st.dirty = false;
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        EditorCanvas { widget: da, state }
    }

    /// Construct a canvas for a document already loaded from disk.
    pub fn from_loaded(doc: Document, path: PathBuf, engine: CrdtEngine) -> Self {
        let canvas = EditorCanvas::new(doc.clone());
        {
            let mut st = canvas.state.borrow_mut();
            st.save_path = Some(path);
            st.engine    = engine;
        }
        canvas
    }
}
