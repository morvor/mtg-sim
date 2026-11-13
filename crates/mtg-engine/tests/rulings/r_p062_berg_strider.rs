//! Rulings batch P062 — Berg Strider: "When this creature enters, tap target artifact or
//! creature an opponent controls. If {S} was spent to cast this spell, that permanent
//! doesn't untap during its controller's next untap step." {S} is mana from a snow source
//! (CR 107.4h) spent to cast it (CR 601.2h); a permanent that wasn't cast, or a copy of a
//! spell, had no mana spent to cast it (CR 707.10).

use crate::r_p062_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Berg Strider paying with five `land`s, its enters ability targeting `target`
/// (if given; otherwise its target is answered later).
fn cast_strider(t: &mut TestGame, land: &str, target: Option<ObjectId>) -> ObjectId {
    supported("Berg Strider");
    t.lands(P0, land, 5);
    let card = t.hand(P0, "Berg Strider");
    if let Some(target) = target {
        t.answer_targets(P0, &[obj(target)]);
    }
    t.cast_with(P0, card, &[]).expect("cast Berg Strider");
    card
}

#[test]
fn berg_strider_cast_with_snow_mana_keeps_an_already_tapped_target_tapped() {
    cr!("107.4h", "601.2h", "502.3");
    ruling!(
        "Berg Strider",
        "The enters-the-battlefield ability can target an artifact or creature an opponent controls that’s already tapped. If snow mana was spent to cast Berg Strider, the target won’t untap during its controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    cast_strider(&mut t, "Snow-Covered Island", Some(bears));
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
    // Without snow mana, the target is tapped but untaps as normal.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_strider(&mut t, "Island", Some(giant));
    t.resolve_all();
    assert!(is_tapped(&t, giant));
    untaps_next(&mut t, giant, P1);
}

#[test]
fn more_snow_mana_doesnt_keep_the_target_tapped_longer() {
    cr!("107.4h", "502.3");
    ruling!(
        "Berg Strider",
        "Spending more than one snow mana to cast Berg Strider won’t cause the artifact or creature to remain tapped for additional turns."
    );
    // All five mana spent came from snow sources.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_strider(&mut t, "Snow-Covered Island", Some(giant));
    t.resolve_all();
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn berg_strider_put_onto_the_battlefield_doesnt_stop_untapping() {
    cr!("601.2h", "502.3");
    ruling!(
        "Berg Strider",
        "If Berg Strider enters the battlefield without being cast, no mana was spent to cast it. The enters-the-battlefield ability may still tap the target, but that permanent won’t be stopped from untapping during its controller’s next untap step."
    );
    supported("Berg Strider");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    crate::r_s05_common::enter(&mut t, P0, "Berg Strider");
    t.resolve_all();
    assert!(is_tapped(&t, giant));
    untaps_next(&mut t, giant, P1);
}

#[test]
fn a_copy_of_the_berg_strider_spell_had_no_mana_spent_on_it() {
    cr!("707.10", "502.3");
    ruling!(
        "Berg Strider",
        "Similarly, if an effect copies the creature spell that becomes Berg Strider, no mana was spent to cast the copy. The target won’t be stopped from untapping during its controller’s next untap step."
    );
    supported("Lithoform Engine");
    // Berg Strider is cast with snow mana; Lithoform Engine ("{4}, {T}: Copy target
    // permanent spell you control.") copies it. The copy's ability targets the Giant, the
    // original's the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let engine = t.battlefield(P0, "Lithoform Engine");
    let spell = cast_strider(&mut t, "Snow-Covered Island", Some(bears));
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(spell)]);
    activate_containing(&mut t, P0, engine, "permanent spell").expect("copy it");
    t.clear_answers();
    // The copy resolves first; its enters ability targets the Giant.
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Berg Strider").len(), 2);
    assert!(is_tapped(&t, bears));
    assert!(is_tapped(&t, giant));
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, giant), "the copy's target untapped");
    assert!(is_tapped(&t, bears), "the original's target stayed tapped");
}
