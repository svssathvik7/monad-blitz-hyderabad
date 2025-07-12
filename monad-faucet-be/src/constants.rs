pub mod faucet {
    pub const MAGNIFICATION_GITHUB_AUTH: u8 = 10; // Either GitHub authenticated or Garden user
    pub const MAGNIFICATION_GARDEN_USER: u8 = 10;
    pub const MAGNIFICATION_NO_AUTH: u8 = 1; // Neither authenticated nor Garden user
    pub const WITHDRAW_LIMIT_DENOMINATOR: f64 = 1_000_000_000.0;
}
