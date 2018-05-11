//! Models contains all structures that are used in different
//! modules of the app

pub mod attributes;
pub mod authorization;
pub mod base_product;
pub mod category;
pub mod currency_exchange;
pub mod elastic;
pub mod product;
pub mod status;
pub mod store;
pub mod user_role;
pub mod validation_rules;
pub mod wizard_store;

pub use self::attributes::*;
pub use self::authorization::*;
pub use self::base_product::*;
pub use self::category::*;
pub use self::currency_exchange::*;
pub use self::elastic::*;
pub use self::product::*;
pub use self::status::*;
pub use self::store::*;
pub use self::user_role::*;
pub use self::validation_rules::*;
pub use self::wizard_store::*;
