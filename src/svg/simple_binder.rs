use web_sys::Element;
use crate::bop::state::card_game_shared_state::CardGameSharedState;

#[derive(Clone)]
pub struct SimpleBinder {
    external_value: String,
    element: Element,
    value_func: fn(&mut CardGameSharedState) -> String,
}

impl SimpleBinder {
    pub fn new(element: Element, value_func: fn(&mut CardGameSharedState)->String) -> SimpleBinder{
        SimpleBinder {
            external_value: "".to_string(),
            element,
            value_func,
        }
    }
    pub fn sync(&mut self, card_game_shared_state: &mut CardGameSharedState) {
        let value_func = self.value_func;
        let value = value_func(card_game_shared_state);
        if self.external_value != value {
            self.element.set_inner_html(value.as_str());
            self.external_value = value;
            if !self.external_value.is_empty() {
                self.element.set_attribute("display", "block").unwrap();
            }
        }
    }
}