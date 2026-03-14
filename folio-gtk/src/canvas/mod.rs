use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Duration;
use gtk4::prelude::*;
use gtk4::{DrawingArea, EventControllerKey, GestureClick, EventControllerMotion};
use glib;
use folio_core::{Document, DocPosition, CrdtEngine, InlineAttr};
use crate::canvas::layout::LayoutCache;
use crate::canvas::selection::SelectionState;

pub mod layout;
pub mod render;
pub mod cursor;
pub mod selection;
pub mod input;
pub mod hittest;

pub struct EditorState {
    pub doc:            Document,
    pub engine:         CrdtEngine,
    pub cursor:         DocPosition,
    pub cursor_visible: bool,
    pub save_path:      Option<PathBuf>,
    pub dirty:          bool,
    /// Current text selection. None = cursor only (no selection).
    pub selection:      Option<SelectionState>,
    /// Mouse-drag anchor — set on button press, cleared on release.
    pub drag_anchor:    Option<DocPosition>,
    /// Formatting attributes to apply to the NEXT characters typed.
    /// When the user clicks Bold with no selection, this set gains Bold.
    /// Each typed character is inserted then has these attrs applied.
    pub pending_attrs:  Vec<InlineAttr>,
    /// Layout cache — RefCell so render can rebuild it while holding &EditorState.
    pub layout_cache:   RefCell<Option<LayoutCache>>,
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
            selection:      None,
            drag_anchor:    None,
            pending_attrs:  Vec::new(),
            layout_cache:   RefCell::new(None),
        }
    }

    /// Invalidate the layout cache after any edit.
    pub fn invalidate_layout(&self) {
        *self.layout_cache.borrow_mut() = None;
    }

    /// Toggle a pending inline attribute for typing mode.
    /// If the attr is already pending, remove it; otherwise add it.
    pub fn toggle_pending_attr(&mut self, attr: InlineAttr) {
        if let Some(pos) = self.pending_attrs.iter().position(|a| a == &attr) {
            self.pending_attrs.remove(pos);
        } else {
            self.pending_attrs.push(attr);
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
        da.set_can_focus(true);

        // ── Draw ───────────────────────────────────────────────────────────
        {
            let s = state.clone();
            da.set_draw_func(move |widget, cr, _, _| render::draw(widget, cr, &s.borrow()));
        }

        // ── Keyboard ──────────────────────────────────────────────────────
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

        // ── Mouse click ───────────────────────────────────────────────────
        let click = GestureClick::new();
        {
            let s = state.clone();
            let d = da.clone();
            click.connect_pressed(move |_, _n, x, y| {
                d.grab_focus();
                let mut st = s.borrow_mut();
                if let Some(pos) = hittest::xy_to_position(&st, x, y) {
                    st.cursor      = pos;
                    st.selection   = None;
                    st.drag_anchor = Some(pos);
                    st.cursor_visible = true;
                    st.invalidate_layout();
                }
                d.queue_draw();
            });
        }
        {
            let s = state.clone();
            click.connect_released(move |_, _, _, _| {
                s.borrow_mut().drag_anchor = None;
            });
        }
        da.add_controller(click);

        // ── Mouse motion (drag-select) ────────────────────────────────────
        let motion = EventControllerMotion::new();
        {
            let s = state.clone();
            let d = da.clone();
            motion.connect_motion(move |_, x, y| {
                let needs_redraw = {
                    let mut st = s.borrow_mut();
                    if st.drag_anchor.is_none() { return; }
                    if let Some(pos) = hittest::xy_to_position(&st, x, y) {
                        let anchor = st.drag_anchor.unwrap();
                        if pos != anchor {
                            st.selection = Some(SelectionState { anchor, active: pos });
                            st.cursor    = pos;
                        } else {
                            st.selection = None;
                            st.cursor    = anchor;
                        }
                        true
                    } else { false }
                };
                if needs_redraw { d.queue_draw(); }
            });
        }
        da.add_controller(motion);

        // ── Cursor blink ──────────────────────────────────────────────────
        {
            let s = state.clone();
            let d = da.clone();
            glib::timeout_add_local(Duration::from_millis(530), move || {
                s.borrow_mut().cursor_visible ^= true;
                d.queue_draw();
                glib::ControlFlow::Continue
            });
        }

        // ── Auto-save (2 s, only when dirty + path set) ───────────────────
        {
            let s = state.clone();
            glib::timeout_add_local(Duration::from_secs(2), move || {
                let mut st = s.borrow_mut();
                if st.dirty {
                    if let Some(path) = st.save_path.clone() {
                        match folio_core::format::save_folio(&path, &st.doc, &st.engine, &[]) {
                            Ok(_)  => st.dirty = false,
                            Err(e) => eprintln!("Auto-save failed: {e}"),
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        EditorCanvas { widget: da, state }
    }

    /// Build a canvas for a document loaded from disk.
    pub fn from_loaded(doc: Document, path: PathBuf, engine: CrdtEngine) -> Self {
        let canvas = EditorCanvas::new(doc);
        {
            let mut st   = canvas.state.borrow_mut();
            st.save_path = Some(path);
            st.engine    = engine;
        }
        canvas
    }
}
