//! A copy has the original's copiable values (CR 707.2–707.4, 707.8–707.10): see
//! `mtg_engine::properties::copies`.

use mtg_engine::properties::copies::*;

#[test]
fn copies_of_a_sample_of_cards_have_their_copiable_values() {
    let t1 = std::time::Instant::now();
    let cards = match std::env::var("MTG_COPY_CARD") {
        Ok(name) => supported(&[name]),
        Err(_) => sample(0, 20),
    };
    eprintln!("{} cards compiled in {:?}", cards.len(), t1.elapsed());
    let (copies, mismatches, skipped) = check_cards(&cards);
    eprintln!(
        "{} cards, {copies} copies, {:?} not made in {:?}",
        cards.len(),
        skipped,
        t1.elapsed()
    );
    for m in &mismatches {
        eprintln!("{m}");
    }
    assert!(mismatches.is_empty(), "{} mismatches", mismatches.len());
}
