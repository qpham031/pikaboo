pub const HOUR_IN_SEC_STR: &str = "3600";
pub const MINUTE_IN_SEC_STR: &str = "60";
pub const MINUTE_IN_SEC: u64 = 60;
pub const HOUR_IN_SEC: u64 = MINUTE_IN_SEC * 60;
pub const DAY_IN_SEC: u64 = HOUR_IN_SEC * 24;
pub const MONTH_IN_SEC: u64 = DAY_IN_SEC * 30;

pub mod interact {
    pub const PIKABOO: &str = "pikaboo";
    pub const PIKABOO_QUICK: &str = "pikaboo-quick";
    pub const PIKABOO_MOD: &str = "pikaboo-mod";

    pub const INVENTORY: &str = "inventory";
    pub const SHOP: &str = "shop";
    pub const ZONES: &str = "zones";
    pub const GAMES: &str = "games";
    pub const FAQS: &str = "faqs";
    pub const ABOUT: &str = "about";

    pub const CUSTOM_ROLE: &str = "customrole";
    pub const NICKNAME: &str = "nickname";
    pub const CONFIRM_OKAY: &str = "confirm-okay";

    pub const SHOP_CUSTOM_ROLE: &str = "shop-custom-role";
    pub const SHOP_NICKNAME: &str = "shop-nickname";
    pub const TRADE_CUSTOM_ROLE: &str = "trade-custom-role";
    pub const TRADE_NICKNAME: &str = "trade-nickname";

    pub const CUSTOM_ROLE_SUBCRIBE: &str = "custom-role-subscribe";
    pub const CUSTOM_ROLE_UNSUBCRIBE: &str = "custom-role-unsubscribe";
    pub const CONFIRM_TRADE_CUSTOM_ROLE: &str = "confirm-trade-custom-role";
    pub const CONFIRM_TRADE_NICKNAME: &str = "confirm-trade-nickname";
}

pub mod colors {
    pub const MENU_COLOR: u32 = 0xffbe98;
}
