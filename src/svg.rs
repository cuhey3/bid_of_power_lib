use crate::svg::element_wrapper::ElementWrapper;
use web_sys::Document;

pub mod element_wrapper;
pub mod simple_binder;
pub mod svg_renderer;

pub struct SharedElements {
    pub message: ElementWrapper,
    pub document: Document,
}

impl SharedElements {
    pub fn new() -> SharedElements {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        SharedElements {
            message: ElementWrapper::new(document.get_element_by_id("message").unwrap()),
            document,
        }
    }
}
