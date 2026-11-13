//! CR 702.117 Surge.

use crate::common_k702_111_124::*;
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const SURGE: CastMethod = CastMethod::Keyword(KeywordKind::Surge);

#[test]
fn surge_can_be_paid_after_you_cast_another_spell_this_turn() {
    cr!("702.117", "702.117a");
    ruling!(
        "Reckless Bushwhacker",
        "Casting a spell for its surge cost doesn’t change its mana cost or its mana value."
    );
    assert_supported_card("Reckless Bushwhacker");
    let mut t = TestGame::new(2);
    // Reckless Bushwhacker: {2}{R} 2/1 haste, surge {1}{R}; "When this creature enters,
    // if its surge cost was paid, other creatures you control get +1/+0 and gain haste
    // until end of turn."
    t.lands(P0, "Mountain", 3);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let bw = t.hand(P0, "Reckless Bushwhacker");
    assert!(!castable(&mut t, P0, bw, SURGE), "no other spell cast yet");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(castable(&mut t, P0, bw, SURGE));
    let spell = t.cast(P0, bw).method(SURGE).go();
    assert_eq!(untapped_lands(&t, P0), 0, "{{1}}{{R}} was paid");
    assert_eq!(t.g.mana_value_of(spell), 3);
    t.resolve_all();
    assert!(t.on_battlefield(bw));
    // Its surge cost was paid: the other creature gets +1/+0 and haste.
    assert_eq!(t.pt(bears), (3, 2));
    assert!(has(&t, bears, KeywordKind::Haste));
    assert_eq!(t.pt(bw), (2, 1));
}

#[test]
fn cast_normally_its_surge_ability_does_nothing() {
    cr!("702.117a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bw = t.hand(P0, "Reckless Bushwhacker");
    t.cast(P0, bw).go();
    t.resolve_all();
    assert!(t.on_battlefield(bw));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn only_spells_cast_by_you_or_a_teammate_this_turn_count() {
    cr!("702.117a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bw = t.hand(P0, "Reckless Bushwhacker");
    // An opponent's spell doesn't count.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(!castable(&mut t, P0, bw, SURGE));
    // Nor does a spell you cast last turn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    let bw = t.hand(P0, "Reckless Bushwhacker");
    assert!(castable(&mut t, P0, bw, SURGE));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, bw, SURGE));
}

#[test]
fn a_teammates_spell_counts() {
    cr!("702.117a");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    t.lands(P0, "Mountain", 2);
    let bw = t.hand(P0, "Reckless Bushwhacker");
    assert!(!castable(&mut t, P0, bw, SURGE));
    // P2 is P0's teammate.
    t.lands(P2, "Mountain", 1);
    let bolt = t.hand(P2, "Lightning Bolt");
    t.cast(P2, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(castable(&mut t, P0, bw, SURGE));
}

#[test]
fn the_other_spell_can_be_countered_or_still_on_the_stack() {
    cr!("702.117a");
    ruling!(
        "Comparative Analysis",
        "The other spell that you or a teammate cast can be one that's resolved, one that was countered, or (for instants with surge) one that's still on the stack."
    );
    assert_supported_card("Comparative Analysis");
    let mut t = TestGame::new(2);
    // Comparative Analysis: {3}{U} instant, surge {2}{U}: "Target player draws two cards."
    t.lands(P0, "Island", 5);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let ca = t.hand(P0, "Comparative Analysis");
    assert!(!castable(&mut t, P0, ca, SURGE));
    t.cast(P0, bears).go();
    // The creature spell is still on the stack.
    assert_eq!(t.stack_len(), 1);
    assert!(castable(&mut t, P0, ca, SURGE));
    let hand = t.hand_size(P0);
    t.cast(P0, ca).method(SURGE).target(Entity::Player(P0)).go();
    assert_eq!(untapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_copy_of_a_spell_cast_for_its_surge_cost_had_its_surge_cost_paid() {
    cr!("702.117a");
    ruling!(
        "Crush of Tentacles",
        "If an instant or sorcery spell cast for its surge cost is copied, the copy is also considered to have had its surge cost paid."
    );
    assert_supported_card("Crush of Tentacles");
    let mut t = TestGame::new(2);
    // Crush of Tentacles: surge {3}{U}{U}; "Return all nonland permanents to their
    // owners' hands. If this spell's surge cost was paid, create an 8/8 blue Octopus
    // creature token."
    t.lands(P0, "Island", 7);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    let crush = t.hand(P0, "Crush of Tentacles");
    let spell = t.cast(P0, crush).method(SURGE).go();
    let twin = t.hand(P0, "Twincast");
    t.cast(P0, twin).target(spell).go();
    // Twincast resolves, copying Crush; the copy resolves and makes an Octopus.
    t.resolve();
    t.resolve();
    let octopi = tokens_of(&t, P0);
    assert_eq!(octopi.len(), 1);
    assert_eq!(t.pt(octopi[0]), (8, 8));
    // Then the original bounces it and makes another.
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(t.g.history.tokens_created.get(&P0).copied(), Some(2));
}
