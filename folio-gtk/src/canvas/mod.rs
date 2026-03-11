use std::rc::Rc;
use std::cell::RefCell;
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
/// captured by multiple GTK closures (draw func, key handler, blink timer).
pub struct EditorState {
    pub doc:            Document,
    pub engine:         CrdtEngine,
    pub cursor:         DocPosition,
    /// Toggled by the blink timer; draw func reads this to show/hide cursor.
    pub cursor_visible: bool,
}

impl EditorState {
    pub fn new(doc: Document) -> Self {
        EditorState {
            cursor:         DocPosition::block_start(0),
            engine:         CrdtEngine::new(),
            doc,
            cursor_visible: true,
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

        // A4 page + 80 px padding on all sides.
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

        EditorCanvas { widget: da, state }
    }
}
