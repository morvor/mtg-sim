//! Checks from the register sweep of Nov 10, 2026 (docs/agents/items/register-sweep-1.md):
//! behaviors the round-trip items' "Remaining" sections described as "not observable" and
//! that the sweep verified in play, and the compiler reading the sweep's reading of the
//! equivalence list exposed (the grandeur cost).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// "You may discard one or more land cards. When you do, ..." is compiled with the count
/// "any number": a player who accepts the offer and chooses no card has not discarded one
/// or more cards, so the reflexive ability doesn't trigger, exactly as if the offer had been
/// declined (CR 603.12: a reflexive triggered ability triggers when the player takes the
/// action).
#[test]
fn choosing_no_card_for_an_optional_one_or_more_discard_is_declining() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    // Accept the offer, then choose no land card.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.enter(P0, "Loamcrafter Faun");
    t.settle();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    t.resolve();
    assert_eq!(t.hand_size(P0), 2, "no land card was discarded");
    assert_eq!(
        t.stack_len(),
        0,
        "no reflexive trigger for a discard that didn't happen: {}",
        t.dump_log()
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Choosing one card does trigger it, and the Bears come back.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.hand(P0, "Forest");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Loamcrafter Faun");
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the reflexive trigger: {}", t.dump_log());
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Forest"));
}

/// The grandeur cost "Discard another card named ~" asks for a card with that name, the
/// printed one, as every other "card named ~" does (Squadron Hawk, Relentless Rats), not
/// for a card that shares a name with the object as it is now. With Spy Kit equipped,
/// Baru, Fist of Krosa has every nonlegendary creature card's name (CR 612.7), so Baru and
/// a Grizzly Bears card have the same name (CR 201.2a); the Bears card still isn't a card
/// named Baru, Fist of Krosa and can't pay the cost. Another Baru can. (The compiler had
/// read the cost as "with the same name as ~", which the round trip's equivalence list
/// hid until register-sweep-1 read it.)
#[test]
fn grandeur_asks_for_a_card_with_the_printed_name_not_one_sharing_a_gained_name() {
    cr!("207.2c", "612.7", "201.2a");
    let mut t = TestGame::new(2);
    let baru = t.battlefield(P0, "Baru, Fist of Krosa");
    let kit = t.battlefield(P0, "Spy Kit");
    assert!(t.g.attach(kit, baru.into()));
    t.recompute();
    assert!(
        t.obj_now(baru).chars.has_name("Grizzly Bears"),
        "Spy Kit gives Baru the Bears' name"
    );
    t.hand(P0, "Grizzly Bears");
    // The Bears card shares a name with Baru now, but it isn't a card named Baru, Fist of
    // Krosa: the grandeur cost can't be paid.
    assert!(t.activate(P0, baru, 0, &[]).is_err(), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Another card named Baru, Fist of Krosa pays it: an X/X Wurm, X the lands you control.
    let other = t.hand(P0, "Baru, Fist of Krosa");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.activate(P0, baru, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Baru, Fist of Krosa"));
    assert!(
        t.in_hand(P0, "Grizzly Bears"),
        "the Bears card wasn't discarded"
    );
    t.resolve();
    let wurms = t.named_on_battlefield("Wurm");
    assert_eq!(wurms.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(wurms[0]), (2, 2));
}
