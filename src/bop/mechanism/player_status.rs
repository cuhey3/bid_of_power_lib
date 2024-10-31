#[derive(Clone, Debug)]
pub struct PlayerStatus {
    pub max_hp: u32,
    pub current_hp: u32,
    pub attack_point: u32,
    pub defence_point: u32,
    pub current_money_amount: u32,
    pub estimated_money_amount: u32,
}

impl PlayerStatus {
    pub fn init() -> PlayerStatus {
        PlayerStatus {
            max_hp: 50,
            current_hp: 50,
            attack_point: 10,
            defence_point: 5,
            current_money_amount: 5,
            estimated_money_amount: 3,
        }
    }

    pub fn update_current_hp(&mut self, amount_of_change: i32) {
        let mut updated_hp = self.current_hp as i32 + amount_of_change;
        updated_hp = updated_hp.max(0);
        updated_hp = updated_hp.min(self.max_hp as i32);
        self.current_hp = updated_hp as u32;
    }

    pub fn get_amount(&self, status_name: &String) -> u32 {
        match status_name.as_str() {
            "ATK" => self.attack_point,
            "DEF" => self.defence_point,
            "HP" => self.current_hp,
            "MHP" => self.max_hp,
            "Money" => self.current_money_amount,
            "Gain" => self.estimated_money_amount,
            _ => {
                panic!()
            }
        }
    }

    pub fn set_amount(&mut self, status_name: &String, amount: u32) {
        match status_name.as_str() {
            "ATK" => self.attack_point = amount,
            "DEF" => self.defence_point = amount,
            "HP" => self.current_hp = amount.min(self.max_hp),
            "MHP" => self.max_hp = amount,
            "Money" => self.current_money_amount = amount,
            "Gain" => self.estimated_money_amount = amount,
            _ => {
                panic!()
            }
        }
    }
    pub fn capped_update_status(&mut self, status_name: &String, amount_of_change: i32) {
        if let "HP" = status_name.as_str() {
            self.update_current_hp(amount_of_change);
            return;
        }
        let current_amount = self.get_amount(&status_name);
        let updated_amount = (current_amount as i32 + amount_of_change).max(0) as u32;
        self.set_amount(&status_name, updated_amount);
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp == 0
    }

    pub fn get_damage(&self, attack_point: u32) -> u32 {
        if self.defence_point >= attack_point {
            // attack_point が 0 ならダメージは 0
            // そうでなければ最低保証ダメージは 1
            attack_point.min(1)
        } else {
            attack_point - self.defence_point
        }
    }
}
