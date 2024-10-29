use crate::engine::input::Input;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element};

#[derive(Debug)]
pub enum CursorType {
    Default,
}

pub struct Cursor {
    pub element: Element,
    pub chose_index: usize,
    pub choice_length: usize,
    pub step_length: f64,
    default_y: f64,
    pub cursor_type: CursorType,
}

impl Cursor {
    pub fn empty() -> Cursor {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
            .unwrap();
        Cursor {
            element,
            chose_index: 0,
            choice_length: 0,
            step_length: 0.0,
            default_y: 0.0,
            cursor_type: CursorType::Default,
        }
    }
    pub fn new_with_element(element: Element, step_length: f64) -> Cursor {
        Cursor {
            chose_index: 0,
            // 後から更新可能
            choice_length: 0,
            step_length,
            default_y: element.get_attribute("y").unwrap().parse().unwrap(),
            element,
            cursor_type: CursorType::Default,
        }
    }
    pub fn new(
        document: &Document,
        cursor_id: &str,
        choice_length: usize,
        step_length: f64,
    ) -> Cursor {
        let element = document.get_element_by_id(cursor_id).unwrap();
        let default_y = element.get_attribute("y").unwrap().parse().unwrap();
        Cursor {
            element,
            chose_index: 0,
            choice_length,
            step_length,
            default_y,
            cursor_type: CursorType::Default,
        }
    }

    pub fn update_choice_length(&mut self, choice_length: usize) {
        self.choice_length = choice_length;
        self.chose_index = self.chose_index.min(self.choice_length - 1);
    }

    pub fn reset(&mut self) {
        // TODO
        // カーソル位置を記憶する実装
        self.chose_index = 0;
        match self.cursor_type {
            CursorType::Default => {
                self.element
                    .set_attribute("y", &*self.default_y.to_string())
                    .unwrap();
            }
        }
    }
    pub fn consume(&mut self, input: Input) {
        let new_index = match self.cursor_type {
            CursorType::Default => match input {
                Input::ArrowUp => (self.chose_index + self.choice_length - 1) % self.choice_length,
                Input::ArrowDown => (self.chose_index + 1) % self.choice_length,
                _ => self.chose_index,
            },
        };
        self.chose_index = new_index;
        match self.cursor_type {
            CursorType::Default => {
                let new_y: f64 = self.default_y + new_index as f64 * self.step_length;
                self.element
                    .set_attribute("y", new_y.to_string().as_str())
                    .unwrap();
            }
        }
    }
}

pub struct SvgRenderer {
    target_part_name: String,
    wrapper_element: Option<Element>,
    item_element: Option<Element>,
    message_wrapper_element: Option<Element>,
    message_element: Option<Element>,
    pub cursor: Cursor,
    step_length: f64,
    item_labels: Vec<String>,
    item_x: f64,
    item_y: f64,
}

impl SvgRenderer {
    pub fn new(target_part_name: String, step_length: f64) -> SvgRenderer {
        let mut renderer = SvgRenderer {
            target_part_name,
            wrapper_element: None,
            item_element: None,
            message_wrapper_element: None,
            message_element: None,
            cursor: Cursor::empty(),
            step_length,
            item_labels: vec![],
            item_x: 0.0,
            item_y: 0.0,
        };
        renderer.load();
        renderer
    }
    pub fn load(&mut self) {
        self.load_wrapper_element();
        self.load_item_element();
        self.load_cursor();
        self.load_message_wrapper_element();
        self.load_message_element();
    }
    pub fn load_wrapper_element(&mut self) {
        self.wrapper_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_wrapper_id())
    }
    pub fn get_wrapper_id(&self) -> String {
        format!("render-{}-wrapper", self.target_part_name)
    }

    pub fn load_cursor(&mut self) {
        let cursor_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_cursor_id())
            .unwrap();
        self.cursor = Cursor::new_with_element(cursor_element, self.step_length);
    }

    pub fn get_cursor_id(&self) -> String {
        format!("render-{}-cursor", self.target_part_name)
    }

    pub fn load_item_element(&mut self) {
        self.item_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_item_id());
        if let Some(element) = &self.item_element {
            self.item_x = element.get_attribute("x").unwrap().parse().unwrap();
            self.item_y = element.get_attribute("y").unwrap().parse().unwrap();
        }
    }
    pub fn get_item_id(&self) -> String {
        format!("render-{}-item", self.target_part_name)
    }
    pub fn load_message_wrapper_element(&mut self) {
        self.message_wrapper_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_message_wrapper_id())
    }
    pub fn get_message_wrapper_id(&self) -> String {
        format!("render-{}-message-wrapper", self.target_part_name)
    }

    pub fn load_message_element(&mut self) {
        self.message_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_message_id())
    }
    pub fn get_message_id(&self) -> String {
        format!("render-{}-message", self.target_part_name)
    }

    pub fn get_rendered_id(&self) -> String {
        format!("render-{}-rendered", self.target_part_name)
    }

    pub fn render(&mut self, labels: Vec<String>, descriptions: Vec<String>, description: &str) {
        self.cursor.choice_length = labels.len();
        self.item_labels = labels;
        let document = web_sys::window().unwrap().document().unwrap();
        if let Some(to_remove) = document.get_element_by_id(self.get_rendered_id().as_str()) {
            to_remove.remove();
        }
        let group_element = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "g")
            .unwrap();
        group_element
            .set_attribute("id", self.get_rendered_id().as_str())
            .unwrap();
        if let Some(wrapper_element) = &self.wrapper_element {
            wrapper_element.append_child(&*group_element).unwrap();
            wrapper_element.set_attribute("display", "block").unwrap();
        }

        while let Some(node) = group_element
            .query_selector_all(".item-description")
            .unwrap()
            .item(0)
        {
            let element = node.dyn_into::<Element>().unwrap();
            element.remove();
        }
        for (index, label) in self.item_labels.iter().enumerate() {
            if let Some(item_element) = &self.item_element {
                let node = item_element.clone_node().unwrap();
                let empty_element = document
                    .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
                    .unwrap();
                node.append_child(&*empty_element).unwrap();
                let element = empty_element.parent_element().unwrap();
                element.set_inner_html(label);
                match self.cursor.cursor_type {
                    CursorType::Default => {
                        element
                            .set_attribute("x", &*self.item_x.to_string())
                            .unwrap();
                        element
                            .set_attribute(
                                "y",
                                &*(self.item_y + index as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                    }
                }
                element.set_attribute("display", "block").unwrap();
                group_element.append_child(&*element).unwrap();
                if descriptions.is_empty() {
                    continue;
                }
                match self.cursor.cursor_type {
                    CursorType::Default => {
                        let node = element.clone_node().unwrap();
                        let empty_element = web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
                            .unwrap();
                        node.append_child(&*empty_element).unwrap();
                        let element = empty_element.parent_element().unwrap();
                        element.set_inner_html(descriptions[index].as_str());
                        element.class_list().add_1("item-description").unwrap();
                        element
                            .set_attribute("x", &*(self.item_x + 15.0).to_string())
                            .unwrap();
                        element
                            .set_attribute(
                                "y",
                                &*(self.item_y + index as f64 * self.step_length + 20.0)
                                    .to_string(),
                            )
                            .unwrap();
                        element.set_attribute("font-size", "11").unwrap();
                        element.set_attribute("display", "block").unwrap();
                        group_element.append_child(&*element).unwrap();
                    }
                }
            }
        }
        self.cursor
            .element
            .set_attribute("display", "block")
            .unwrap();
        if let Some(element) = &self.message_wrapper_element {
            let display = if description.is_empty() {
                "none"
            } else {
                "block"
            };
            element.set_attribute("display", display).unwrap();
        }

        if let Some(element) = &self.message_element {
            if !description.is_empty() {
                element.set_inner_html(description);
            }
        }
    }
    pub fn hide(&self) {
        if let Some(element) = &self.wrapper_element {
            element.set_attribute("display", "none").unwrap();
        }
    }
}

pub fn get_element_by_id(id: String) -> Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(&*id)
        .unwrap()
}
