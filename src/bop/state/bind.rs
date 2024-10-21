use crate::bop::state::card_game_shared_state::CardGameSharedState;
use crate::svg::simple_binder::SimpleBinder;

pub fn get_binds() -> Vec<SimpleBinder>{
    let mut binds = vec![];

    let required_input = web_sys::window().unwrap().document().unwrap().get_element_by_id("simple-binder-required-input").unwrap();
    fn required_input_func(card_game_shared_state: &mut CardGameSharedState) -> String{
        format!("{}さん、入札してください。上下: アイテム選択　左右: 金額変更　A: 決定", card_game_shared_state.players[card_game_shared_state.own_player_index].player_name)
    }
    binds.push(SimpleBinder::new(required_input, required_input_func));
    binds
}