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

    pub const CUSTOMROLE: &str = "customrole";
    pub const NICKNAME: &str = "nickname";
    pub const CONFIRM_OKAY: &str = "confirm-okay";

    pub const SHOP_CUSTOMROLE: &str = "customrole-shop";
    pub const SHOP_NICKNAME: &str = "nickname-shop";
    pub const SHOP_ENERGYBOOST: &str = "energyboost-shop";
    pub const SHOP_BLINDBOX: &str = "blindbox-shop";

    pub const INVENTORY_NICKNAME: &str = "nickname-inventory";
    pub const INVENTORY_CUSTOMROLE: &str = "customrole-inventory";

    pub const CONFIRM_ORDER_CUSTOMROLE: &str = "customrole-order-confirm";
    pub const CONFIRM_ORDER_NICKNAME: &str = "nickname-order-confirm";
    pub const CONFIRM_ORDER_ENERGYBOOST: &str = "energyboost-order-confirm";
    pub const CONFIRM_ORDER_BLINDBOX: &str = "blindbox-order-confirm";
    pub const CHANGE_NICKNAME: &str = "nickname-change";

    pub const CUSTOMROLE_SUBSCRIBE: &str = "customrole-subscribe";
    pub const CUSTOMROLE_UNSUBSCRIBE: &str = "customrole-unsubscribe";
}

pub mod colors {
    pub const MENU_COLOR: u32 = 0xffbe98;
}
