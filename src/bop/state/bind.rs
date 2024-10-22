use crate::bop::state::card_game_shared_state::{BidMessage, CardGameSharedState};
use crate::svg::simple_binder::SimpleBinder;
use crate::svg::svg_renderer::get_element_by_id;

pub fn get_binds() -> Vec<SimpleBinder> {
    let mut binds = vec![];

    let required_input = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("simple-binder-required-input")
        .unwrap();
    fn required_input_func(card_game_shared_state: &mut CardGameSharedState, _: usize) -> String {
        match card_game_shared_state.phase_index {
            1 => format!(
                "{}さん、入札してください。上下: 選択　左右: 金額変更　A: 決定",
                card_game_shared_state.players[card_game_shared_state.own_player_index].player_name
            ),
            2 => format!(
                "{}さん、使用するアイテムを選んでください。上下: 選択　A: 決定",
                card_game_shared_state.players[card_game_shared_state.own_player_index].player_name
            ),

            3 => format!(
                "{}さん、攻撃対象を選んでください。上下: 選択　A: 決定",
                card_game_shared_state.players[card_game_shared_state.own_player_index].player_name
            ),
            _ => "".to_string(),
        }
    }
    binds.push(SimpleBinder::new(required_input, 0, required_input_func));

    fn bid_amount_func(
        card_game_shared_state: &mut CardGameSharedState,
        args_usize: usize,
    ) -> String {
        if let Some(bid_input) = card_game_shared_state.bid_input.get(args_usize) {
            bid_input.bid_amount.to_string()
        } else {
            "".to_string()
        }
    }

    fn current_amount_func(
        card_game_shared_state: &mut CardGameSharedState,
        args_usize: usize,
    ) -> String {
        let amount = BidMessage::current_bid_amount(
            args_usize,
            &card_game_shared_state.temporary_bid_history,
        );
        if amount == 0 {
            "-".to_string()
        } else {
            amount.to_string()
        }
    }

    for n in 0..3 {
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-input-amount-{}", n + 1)),
            n,
            bid_amount_func,
        ));
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-current-amount-{}", n + 1)),
            n,
            current_amount_func,
        ));
    }
    for n in 0..10 {
        fn card_list_a(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(card) = card_game_shared_state.players[0]
                .own_card_list
                .get(args_usize)
            {
                card.card_kind.get_card_name()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-card-list-a-{}", n + 1)),
            n,
            card_list_a,
        ));
        fn card_list_a_description(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(card) = card_game_shared_state.players[0]
                .own_card_list
                .get(args_usize)
            {
                card.card_kind.get_card_description()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-card-list-a-{}-description", n + 1)),
            n,
            card_list_a_description,
        ));
    }
    for n in 0..10 {
        fn card_list_b(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(card) = card_game_shared_state.players[1]
                .own_card_list
                .get(args_usize)
            {
                card.card_kind.get_card_name()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-card-list-b-{}", n + 1)),
            n,
            card_list_b,
        ));
        fn card_list_b_description(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(card) = card_game_shared_state.players[1]
                .own_card_list
                .get(args_usize)
            {
                card.card_kind.get_card_description()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-card-list-b-{}-description", n + 1)),
            n,
            card_list_b_description,
        ));
    }
    for n in 0..2 {
        fn player_info_money(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            let player_state = &card_game_shared_state.players[args_usize].player_state;
            format!(
                "{}(+{})",
                player_state.current_money_amount, player_state.estimated_money_amount
            )
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-player-info-money-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            player_info_money,
        ));
    }
    for n in 0..19 {
        fn scheduled_card(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(card) = card_game_shared_state.bid_scheduled_cards.get(args_usize) {
                card.card_kind.get_card_name()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-scheduled-card-{}", n + 1)),
            n,
            scheduled_card,
        ));
    }
    for n in 0..19 {
        fn scheduled_card_description(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(card) = card_game_shared_state.bid_scheduled_cards.get(args_usize) {
                card.card_kind.get_card_description()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-scheduled-card-description-{}",
                n + 1
            )),
            n,
            scheduled_card_description,
        ));
    }
    for n in 0..2 {
        fn player_status(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            let hp = card_game_shared_state.players[args_usize]
                .player_state
                .current_hp;
            let max_hp = card_game_shared_state.players[args_usize]
                .player_state
                .max_hp;
            format!("{}/{}", hp, max_hp)
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-status-hp-max-hp-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            player_status,
        ));
    }
    for n in 0..2 {
        fn player_status(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            let attack_point = card_game_shared_state.players[args_usize]
                .player_state
                .attack_point;
            attack_point.to_string()
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-status-atk-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            player_status,
        ));
    }
    for n in 0..2 {
        fn player_status(
            card_game_shared_state: &mut CardGameSharedState,
            args_usize: usize,
        ) -> String {
            let defence_point = card_game_shared_state.players[args_usize]
                .player_state
                .defence_point;
            defence_point.to_string()
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-status-def-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            player_status,
        ));
    }
    binds
}
