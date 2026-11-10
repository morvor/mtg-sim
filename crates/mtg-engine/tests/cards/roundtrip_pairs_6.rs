//! roundtrip-pairs-6 (docs/agents/items/roundtrip-pairs-6.md): untriaged round-trip
//! mismatches, checked in play where a mismatch could have meant the engine does something
//! other than what the card says.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spider_punk_abilities_cant_be_countered() {
    cr!("701.6a");
    // "Spells and abilities can't be countered." is compiled as a restriction on every
    // object (`Restriction::CantBeCountered(Filter::Any)`, rendered "permanents"): an
    // activated ability on the stack can't be countered either.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Spider-Punk");
    let tim = t.battlefield(P0, "Prodigal Sorcerer");
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.activate(P0, tim, 0, &[Entity::Player(P1)]).unwrap();
    let ability = *t.g.stack.last().unwrap();
    t.cast(P1, stifle).target(ability).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "the ability resolved");
    assert!(t.in_graveyard(P1, "Stifle"));
}

#[test]
fn aura_graft_with_no_legal_place_leaves_the_aura_where_it_is() {
    cr!("701.3b");
    ruling!(
        "Aura Graft",
        "If there is no legal place to move the enchantment, then it doesn't move but you still control it."
    );
    // "Attach it to another permanent it can enchant": the choice is among permanents
    // other than the one it's attached to that it can legally enchant
    // (`Filter::CanBeAttachedBy`); with none, it stays, and you control it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let spell = t.hand(P0, "Aura Graft");
    t.lands(P0, "Island", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(aura)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(aura).controller, P0);
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn galvanoth_leaves_a_noninstant_card_on_top() {
    ruling!(
        "Galvanoth",
        "If the card you look at isn't an instant or sorcery card, it remains on top of your library."
    );
    // "You may look at the top card of your library. You may cast it without paying its
    // mana cost if it's an instant or sorcery spell." (rendered as a condition on the top
    // card): a creature card stays where it is.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galvanoth");
    let card = t.library_top(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.library_top(P0), Some(t.g.current(card)));
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn blinding_beam_lasts_through_the_targeted_players_next_untap_step() {
    cr!("502.3", "611.2a");
    // "Creatures don't untap during target player's next untap step." is compiled as
    // "creatures target player controls don't untap" through the next untap step: the
    // targeted player's, whoever's turn it is cast on (`turn::untap_restricted_player`).
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.library_top(P0, "Forest");
        t.library_top(P1, "Forest");
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let beam = t.hand(P0, "Blinding Beam");
    // Cast during P1's turn, after P1's untap step.
    t.set_step(P1, Step::PrecombatMain);
    t.g.tap(bears);
    t.cast(P0, beam).modes(&[1]).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    // P0's untap step passes: not P1's.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    // P1's next untap step: the Bears stay tapped.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "doesn't untap during P1's next untap step");
    // The untap step after that: they untap.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn tinybones_checks_hands_as_the_ability_resolves() {
    cr!("608.2b");
    ruling!(
        "Tinybones, Trinket Thief",
        "Tinybones checks which opponents have no cards in hand only as its last ability resolves."
    );
    // "{4}{B}{B}: Each opponent with no cards in hand loses 10 life." (rendered "each
    // opponent ifs they have no cards in hand"): an opponent who casts their last card in
    // response has no cards in hand as it resolves; one who keeps a card doesn't lose.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let tiny = t.battlefield(P0, "Tinybones, Trinket Thief");
    t.lands(P0, "Swamp", 6);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P2, "Grizzly Bears");
    t.activate(P0, tiny, 0, &[]).unwrap();
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 10, "no cards in hand as the ability resolves");
    assert_eq!(t.life(P2), 20, "a card in hand");
}

#[test]
fn chain_of_plasma_lets_a_targeted_player_discard() {
    cr!("115.4");
    // "~ deals 3 damage to any target. Then that player or that permanent's controller
    // may discard a card.": `ControllerOf(Target(0))` of a player target is that player.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let chain = t.hand(P0, "Chain of Plasma");
    t.hand(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.cast(P0, chain).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the targeted player discarded");
}
