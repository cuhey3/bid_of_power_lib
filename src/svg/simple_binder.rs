use crate::bop::state::bop_shared_state::BoPSharedState;
use web_sys::Element;

#[derive(Clone)]
pub struct SimpleBinder {
    external_value: String,
    args_usize: usize,
    element: Element,
    value_func: fn(&mut BoPSharedState, usize) -> String,
}

impl SimpleBinder {
    pub fn new(
        element: Element,
        args_usize: usize,
        value_func: fn(&mut BoPSharedState, args_usize: usize) -> String,
    ) -> SimpleBinder {
        SimpleBinder {
            external_value: "".to_string(),
            element,
            args_usize,
            value_func,
        }
    }
    pub fn sync(&mut self, bop_shared_state: &mut BoPSharedState) -> &mut SimpleBinder {
        let value_func = self.value_func;
        let value = value_func(bop_shared_state, self.args_usize);
        if self.external_value != value {
            self.element.set_inner_html(value.as_str());
            self.external_value = value;
            if !self.external_value.is_empty() {
                self.element.set_attribute("display", "block").unwrap();
            }
        }
        self
    }
}
