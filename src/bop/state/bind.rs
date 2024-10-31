use crate::bop::state::bop_shared_state::BoPSharedState;
use crate::bop::state::message::BidMessage;
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
    fn required_input_func(bop_shared_state: &mut BoPSharedState, _: usize) -> String {
        if bop_shared_state.phase_index == 4 {
            return if bop_shared_state.players[0].is_lose() {
                format!("{}さんの勝利です", bop_shared_state.players[1].player_name)
            } else {
                format!("{}さんの勝利です", bop_shared_state.players[0].player_name)
            }
        }
        if bop_shared_state.input_is_guard {
            return format!(
                "{}{}が考えています...",
                bop_shared_state.players
                    [bop_shared_state.opponent_player_index(bop_shared_state.own_player_index)]
                .player_name,
                if bop_shared_state.has_cpu && bop_shared_state.own_player_index == 0 {
                    "(CPU)"
                } else {
                    "さん"
                }
            );
        }
        match bop_shared_state.phase_index {
            1 => format!(
                "{}さん、入札してください。上下: 選択　左右: 金額変更　A: 決定",
                bop_shared_state.players[bop_shared_state.own_player_index].player_name
            ),
            2 => format!(
                "{}さん、使用するアイテムを選んでください。上下: 選択　A: 決定　Z: スキップ",
                bop_shared_state.players[bop_shared_state.own_player_index].player_name
            ),

            3 => format!(
                "{}さん、攻撃対象を選んでください。",
                bop_shared_state.players[bop_shared_state.own_player_index].player_name
            ),
            _ => "".to_string(),
        }
    }
    binds.push(SimpleBinder::new(required_input, 0, required_input_func));

    fn bid_amount_func(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
        if let Some(bid_input) = bop_shared_state.bid_input.get(args_usize) {
            bid_input.bid_amount.to_string()
        } else {
            "".to_string()
        }
    }

    fn current_amount_func(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
        let amount =
            BidMessage::current_bid_amount(args_usize, &bop_shared_state.temporary_bid_history);
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
    for n in 0..11 {
        fn item_list_a(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            if let Some(item) = bop_shared_state.players[0].own_item_list.get(args_usize) {
                item.item_kind.get_name()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-item-list-a-{}", n + 1)),
            n,
            item_list_a,
        ));
        fn item_list_a_description(
            bop_shared_state: &mut BoPSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(item) = bop_shared_state.players[0].own_item_list.get(args_usize) {
                item.item_kind.get_description()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-item-list-a-{}-description", n + 1)),
            n,
            item_list_a_description,
        ));
    }
    for n in 0..10 {
        fn bop_list_b(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            if let Some(item) = bop_shared_state.players[1].own_item_list.get(args_usize) {
                item.item_kind.get_name()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-item-list-b-{}", n + 1)),
            n,
            bop_list_b,
        ));
        fn item_list_b_description(
            bop_shared_state: &mut BoPSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(item) = bop_shared_state.players[1].own_item_list.get(args_usize) {
                item.item_kind.get_description()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-item-list-b-{}-description", n + 1)),
            n,
            item_list_b_description,
        ));
    }
    for n in 0..2 {
        fn player_info_money(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            let player_state = &bop_shared_state.players[args_usize].player_status;
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
        fn scheduled_item(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            if let Some(item) = bop_shared_state.bid_scheduled_items.get(args_usize) {
                item.item_kind.get_name()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!("simple-binder-scheduled-item-{}", n + 1)),
            n,
            scheduled_item,
        ));
    }
    for n in 0..19 {
        fn scheduled_item_description(
            bop_shared_state: &mut BoPSharedState,
            args_usize: usize,
        ) -> String {
            if let Some(item) = bop_shared_state.bid_scheduled_items.get(args_usize) {
                item.item_kind.get_description()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-scheduled-item-description-{}",
                n + 1
            )),
            n,
            scheduled_item_description,
        ));
    }
    for n in 0..2 {
        fn player_status(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            let hp = bop_shared_state.players[args_usize]
                .player_status
                .current_hp;
            let max_hp = bop_shared_state.players[args_usize].player_status.max_hp;
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
        fn player_status(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            let attack_point = bop_shared_state.players[args_usize]
                .player_status
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
        fn player_status(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            let defence_point = bop_shared_state.players[args_usize]
                .player_status
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
    for n in 0..2 {
        fn use_item_cursor(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            if bop_shared_state.phase_index == 2 && bop_shared_state.own_player_index == args_usize
            {
                "👉".to_string()
            } else {
                "".to_string()
            }
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "use-item-cursor-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            use_item_cursor,
        ));
    }

    for n in 0..2 {
        fn initiative(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            if bop_shared_state.initiatives_to_player_index[0] == args_usize {
                "先攻"
            } else {
                "後攻"
            }
            .to_string()
        }
        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-initiative-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            initiative,
        ));
    }

    for n in 0..2 {
        fn damage(bop_shared_state: &mut BoPSharedState, args_usize: usize) -> String {
            let own_player_state = &bop_shared_state.players[args_usize].player_status;
            let opponent_player_state = &bop_shared_state.players
                [bop_shared_state.opponent_player_index(args_usize)]
            .player_status;
            own_player_state
                .get_damage(opponent_player_state.attack_point)
                .to_string()
        }

        binds.push(SimpleBinder::new(
            get_element_by_id(format!(
                "simple-binder-status-damage-{}",
                if n == 0 { "a" } else { "b" }
            )),
            n,
            damage,
        ));
    }

    fn bid_cursor(bop_shared_state: &mut BoPSharedState, _: usize) -> String {
        if bop_shared_state.phase_index == 1 {
            "👉".to_string()
        } else {
            "".to_string()
        }
    }
    binds.push(SimpleBinder::new(
        get_element_by_id("render-game-main-bid-cursor".to_string()),
        0,
        bid_cursor,
    ));
    binds
}
