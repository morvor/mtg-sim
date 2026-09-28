//! Rulings batch S22 — the Masteries ("You may pay {2}{U} rather than pay this spell's
//! mana cost. If the {2}{U} cost was paid, ..."): whether that alternative cost was paid
//! (CR 118.9), which can't be combined with another alternative cost such as casting it
//! without paying its mana cost (CR 118.9a); cost increases and reductions apply to it
//! (CR 601.2f) and it still counts as paid.

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::casting::CastOption;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The option to cast `card` for its own alternative cost ("{2}{U}").
fn own_alternative(t: &TestGame, card: ObjectId) -> CastOption {
    t.g.cast_options(P0, card)
        .into_iter()
        .find(|o| {
            matches!(o.method, CastMethod::Alternative(_))
                && o.alt_cost.as_ref().is_some_and(|c| c.mana.is_some())
        })
        .expect("its alternative cost")
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn ingenious_mastery_cast_without_paying_its_mana_cost_hasnt_paid_its_cost() {
    cr!("118.9a", "118.9", "107.3b", "601.2b");
    ruling!(
        "Ingenious Mastery",
        "If you choose to pay one alternative cost, you can’t pay any other alternative costs. For example, if an effect lets you cast a “Mastery” spell “without paying its mana cost,” you can’t also choose to pay its given alternative cost."
    );
    supported("Ingenious Mastery");
    supported("Omniscience");
    // "If the {2}{U} cost was paid, you draw three cards, then an opponent creates two
    // Treasure tokens and they scry 2. If that cost wasn't paid, you draw X cards."
    // Cast with Omniscience ("You may cast spells from your hand without paying their
    // mana costs."): the {2}{U} cost isn't paid too, and X is 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "Ingenious Mastery");
    let free = t
        .g
        .cast_options(P0, m)
        .into_iter()
        .find(|o| o.alt_cost.as_ref().is_some_and(|c| c.mana.is_none()))
        .expect("Omniscience's option");
    let hand = t.hand_size(P0);
    t.cast(P0, m).method(free.method).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 0);
    assert_eq!(t.hand_size(P0), hand - 1, "X = 0 cards drawn");
    assert_eq!(treasures(&t, P1), 0);
    // For its own alternative cost {2}{U}: three cards, and two Treasures for P1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "Ingenious Mastery");
    let alt = own_alternative(&t, m);
    let hand = t.hand_size(P0);
    t.cast(P0, m).method(alt.method).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(treasures(&t, P1), 2);
}

#[test]
fn ingenious_mastery_cost_changes_apply_to_the_alternative_cost_which_still_counts_as_paid() {
    cr!("601.2f", "118.9", "118.9a");
    ruling!(
        "Ingenious Mastery",
        "If an effect increases or decreases the cost of spells you cast, that cost increase or decrease is applied to the alternative cost you chose to pay. In that case, the cost was still paid for the purposes of the effect, even if you paid more or less for it when it was cast."
    );
    supported("Ingenious Mastery");
    // Thalia, Guardian of Thraben ("Noncreature spells cost {1} more to cast."): {3}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 4);
    let m = t.hand(P0, "Ingenious Mastery");
    let alt = own_alternative(&t, m);
    let hand = t.hand_size(P0);
    t.cast(P0, m).method(alt.method).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(treasures(&t, P1), 2);
    // Goblin Electromancer ("Instant and sorcery spells you cast cost {1} less to
    // cast."): {1}{U}.
    supported("Goblin Electromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Island", 2);
    let m = t.hand(P0, "Ingenious Mastery");
    let alt = own_alternative(&t, m);
    let hand = t.hand_size(P0);
    t.cast(P0, m).method(alt.method).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(treasures(&t, P1), 2);
    // Cast normally with X = 2: two cards, no Treasures.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let m = t.hand(P0, "Ingenious Mastery");
    let hand = t.hand_size(P0);
    t.cast(P0, m).x(2).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 5);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(treasures(&t, P1), 0);
}

#[test]
fn baleful_mastery_mana_value_and_the_opponent_chosen_as_it_resolves() {
    cr!("202.3", "118.9c", "608.2d");
    ruling!(
        "Baleful Mastery",
        "The mana value of a spell on the stack is determined by its mana cost, not any alternative costs you used to pay for it."
    );
    ruling!(
        "Baleful Mastery",
        "In a multiplayer game, you choose which opponent takes the prescribed action as the spell resolves."
    );
    supported("Baleful Mastery");
    // "You may pay {1}{B} rather than pay this spell's mana cost. If the {1}{B} cost was
    // paid, an opponent draws a card. Exile target creature or planeswalker." Three
    // players: P0 chooses P2 to draw.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let m = t.hand(P0, "Baleful Mastery");
    let alt = own_alternative(&t, m);
    let spell = t
        .cast(P0, m)
        .method(alt.method)
        .target(Entity::Object(giant))
        .go();
    assert_eq!(t.g.mana_value_of(spell), 4);
    assert_eq!(tapped_lands(&t, P0), 2);
    let (h1, h2) = (t.hand_size(P1), t.hand_size(PlayerId(2)));
    t.answer_choose(P0, &[Entity::Player(PlayerId(2))]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.hand_size(P1), h1);
    assert_eq!(t.hand_size(PlayerId(2)), h2 + 1);
    // Cast for {3}{B}: no opponent draws.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::B, 4);
    let m = t.hand(P0, "Baleful Mastery");
    let h1 = t.hand_size(P1);
    t.cast(P0, m).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn baleful_mastery_a_copy_resolves_as_though_the_cost_was_paid() {
    cr!("707.10", "118.9");
    ruling!(
        "Baleful Mastery",
        "If you copy a \"Mastery\" spell and the alternative cost was paid, the copy will resolve as though the cost was paid."
    );
    supported("Baleful Mastery");
    supported("Twincast");
    // Twincast ("Copy target instant or sorcery spell. You may choose new targets for
    // the copy."): the copy exiles Grizzly Bears and P1 draws twice in all.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let m = t.hand(P0, "Baleful Mastery");
    let alt = own_alternative(&t, m);
    let spell = t
        .cast(P0, m)
        .method(alt.method)
        .target(Entity::Object(giant))
        .go();
    add_mana(&mut t, P0, ManaType::U, 2);
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(Entity::Object(spell)).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let h1 = t.hand_size(P1);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P1), h1 + 2);
}
