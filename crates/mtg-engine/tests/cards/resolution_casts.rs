//! Spells cast while an effect resolves (CR 608.2g), by the player the text names, if able
//! or optionally (`oracle/patterns/resolution_casts.rs`, `permission_grammar`).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn omen_machine_the_player_casts_the_nonland_card() {
    cr!("608.2g", "101.3");
    assert_supported(&["Omen Machine"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omen Machine");
    // P1's draw step: Grizzly Bears is cast for free by P1, without being asked.
    t.library_top(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P1);
    // A land is put onto the battlefield.
    t.library_top(P0, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn audacious_swap_the_owner_may_cast_the_card() {
    cr!("608.2g");
    assert_supported(&["Audacious Swap"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let target = t.battlefield(P1, "Hill Giant");
    let s = t.hand(P0, "Audacious Swap");
    // P1's library is only Grizzly Bears; Hill Giant is shuffled in: either is on top.
    t.g.players[P1.idx()].library.clear();
    t.library_top(P1, "Grizzly Bears");
    t.cast(P0, s).target(target).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    let on = t.named_on_battlefield("Grizzly Bears").len() + t.named_on_battlefield("Hill Giant").len();
    assert_eq!(on, 1);
    assert!(t
        .g
        .battlefield
        .iter()
        .all(|o| t.g.obj(*o).chars.name == "Mountain" || t.g.obj(*o).controller == P1));
}

#[test]
fn oskar_casts_the_discarded_card() {
    cr!("608.2g", "601.3");
    assert_supported(&["Oskar, Rubbish Reclaimer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oskar, Rubbish Reclaimer");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.discard(P0, bears, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn discover_the_impossible_casts_only_a_cheap_instant() {
    cr!("601.3e", "608.2g");
    assert_supported(&["Discover the Impossible"]);
    // The exiled card is Craw Wurm: not an instant, so it goes to the hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let wurm = t.library_top(P0, "Craw Wurm");
    let d = t.hand(P0, "Discover the Impossible");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer_yes(P0, true);
    t.cast(P0, d).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    // An instant with mana value 2 or less is cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let bolt = t.library_top(P0, "Lightning Bolt");
    let d = t.hand(P0, "Discover the Impossible");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, d).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn ziatoras_envoy_plays_a_land_from_the_top() {
    cr!("305.1", "608.2g");
    assert_supported(&["Ziatora's Envoy"]);
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Ziatora's Envoy");
    let land = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.attack(&[(z, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.zone(t.g.current(land)), Zone::Battlefield);
}

#[test]
fn jadzi_casts_the_revealed_card_for_one_mana() {
    cr!("118.9", "608.2g");
    assert_supported(&["Jadzi, Oracle of Arcavios // Journey to the Oracle"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jadzi, Oracle of Arcavios // Journey to the Oracle");
    t.lands(P0, "Island", 2);
    t.library_top(P0, "Island");
    t.library_top(P0, "Craw Wurm");
    let opt = t.hand(P0, "Opt");
    // Craw Wurm is revealed as Opt is cast (the trigger resolves first).
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    // Craw Wurm ({4}{G}{G}) cast for {1}: both Islands are tapped (Opt and the {1}).
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert!(t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Island")
        .all(|o| o.tapped));
}

#[test]
fn infamous_cruelclaw_casts_by_discarding_a_card() {
    cr!("118.9", "608.2g");
    assert_supported(&["The Infamous Cruelclaw"]);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "The Infamous Cruelclaw");
    let land = t.hand(P0, "Island");
    t.library_top(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn jetsam_casts_a_spell_from_each_opponents_graveyard_and_exiles_it() {
    cr!("608.2g", "614.1a");
    assert_supported(&["Flotsam // Jetsam"]);
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 6);
    let j = t.hand(P0, "Flotsam // Jetsam");
    let b1 = t.graveyard(P1, "Lightning Bolt");
    let b2 = t.graveyard(P2, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(b1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(b2)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.cast(P0, j)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
    assert!(!t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_graveyard(P2, "Lightning Bolt"));
    assert!(t.in_exile("Lightning Bolt"));
}
