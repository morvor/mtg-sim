//! Rulings batch S25 — a copy of a spell with {X} has the value of X chosen as the
//! original was cast (CR 707.10, 107.3).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Blaze ("Blaze deals X damage to any target.") with X = `x` at P1, paying with
/// a Mountain and Wastes. Returns the spell.
fn blaze_at_p1(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", x as usize);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(x).target(Entity::Player(P1)).go()
}

/// The copy of `blaze` on the stack has its X, and the two deal 2·X damage to P1.
fn copy_has_the_same_x(t: &mut TestGame, blaze: ObjectId, x: i32) {
    let copy = spell_copies(t)[0];
    assert_eq!(x_of(t, copy), Some(x));
    assert_eq!(x_of(t, copy), x_of(t, blaze));
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 * x);
}

#[test]
fn a_spell_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Twincast",
        "If the spell that's copied has an X whose value was determined as it was cast, the copy will have the same value of X."
    );
    supported("Twincast");
    supported("Blaze");
    let mut t = TestGame::new(2);
    let blaze = blaze_at_p1(&mut t, 3);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(blaze)]);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    copy_has_the_same_x(&mut t, blaze, 3);
}

#[test]
fn a_cast_trigger_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Double Vision",
        "If the spell that's copied has an X whose value was determined as it was cast, the copy has the same value of X."
    );
    supported("Double Vision");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Double Vision");
    let blaze = blaze_at_p1(&mut t, 3);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    copy_has_the_same_x(&mut t, blaze, 3);
}

#[test]
fn kitsa_s_copy_has_the_same_x() {
    cr!("707.10", "107.3", "702.108a");
    ruling!(
        "Kitsa, Otterball Elite",
        "If the spell that’s copied has an X whose value was determined as it was cast, the copy will have the same value of X."
    );
    supported("Kitsa, Otterball Elite");
    // "{2}, {T}: Copy target instant or sorcery spell you control. You may choose new
    // targets for the copy. Activate only if Kitsa's power is 3 or greater."
    let mut t = TestGame::new(2);
    let kitsa = t.battlefield(P0, "Kitsa, Otterball Elite");
    // Giant Growth (and prowess) make Kitsa big enough.
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(kitsa)]);
    t.resolve_all();
    assert!(t.pt(kitsa).0 >= 3);
    let blaze = blaze_at_p1(&mut t, 2);
    // Prowess triggers (below the copy ability).
    t.settle();
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(blaze)]);
    activate_containing(&mut t, P0, kitsa, "Copy target").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    copy_has_the_same_x(&mut t, blaze, 2);
}

#[test]
fn a_goggles_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Pyromancer's Goggles",
        "If the copied spell has an X whose value was determined as it was cast, the copy has the same value of X."
    );
    supported("Pyromancer's Goggles");
    // "{T}: Add {R}. When that mana is spent to cast a red instant or sorcery spell, copy
    // that spell and you may choose new targets for the copy."
    let mut t = TestGame::new(2);
    let goggles = t.battlefield(P0, "Pyromancer's Goggles");
    assert!(tap_for_mana(&mut t, P0, goggles, "Add {R}"));
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, card).x(2).target(Entity::Player(P1)).go();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    copy_has_the_same_x(&mut t, blaze, 2);
}

#[test]
fn a_bonus_round_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Bonus Round",
        "If the spell that’s copied has an X whose value was determined as it was cast, the copy has the same value of X."
    );
    supported("Bonus Round");
    // "Until end of turn, whenever a player casts an instant or sorcery spell, that player
    // copies it and may choose new targets for the copy."
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Bonus Round", &[]);
    t.resolve_all();
    let blaze = blaze_at_p1(&mut t, 3);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    copy_has_the_same_x(&mut t, blaze, 3);
}

#[test]
fn a_guildmage_copy_has_the_same_x() {
    cr!("707.10", "107.3", "202.3e");
    ruling!(
        "League Guildmage",
        "If the spell that's copied has an X whose value was determined as it was cast (like Banefire does), the copy will have the same value of X."
    );
    supported("League Guildmage");
    // "{X}{R}, {T}: Copy target instant or sorcery spell you control with mana value X.
    // You may choose new targets for the copy." Blaze with X = 2 has mana value 3 on the
    // stack.
    let mut t = TestGame::new(2);
    let guildmage = t.battlefield(P0, "League Guildmage");
    let blaze = blaze_at_p1(&mut t, 2);
    assert_eq!(t.g.mana_value_of(blaze), 3);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(blaze)]);
    activate_containing(&mut t, P0, guildmage, "Copy target").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    copy_has_the_same_x(&mut t, blaze, 2);
}
