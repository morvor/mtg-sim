//! Mana payment against a brute-force search (CR 601.2g–h, 106, 107.4): see
//! `mtg_engine::properties::payment`.

use mtg_engine::properties::payment::*;

#[test]
fn the_automatic_payer_finds_every_payment_and_pays_legally() {
    let n = std::env::var("MTG_PAYMENT_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    let (held, payable, violations) = check_many(1, n);
    eprintln!("{held} of {n} boards held ({payable} payable)");
    for v in &violations {
        eprintln!("{v}");
    }
    assert!(violations.is_empty(), "{} violations", violations.len());
}
