//! CR 702.120 Escalate.

use crate::common_k702_111_124::*;
use mtg_engine::ability::Duration;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn escalate_costs_more_for_each_mode_beyond_the_first() {
    cr!("702.120", "702.120a");
    ruling!(
        "Borrowed Hostility",
        "Additional costs don’t affect a spell’s mana value."
    );
    assert_supported_card("Borrowed Hostility");
    let mut t = TestGame::new(2);
    // Borrowed Hostility: {R} instant, escalate {3}; "Choose one or both — • Target
    // creature gets +3/+0 until end of turn. • Target creature gains first strike until
    // end of turn."
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Borrowed Hostility");
    // Both modes cost {3}{R}: not enough mana.
    let r = t.cast(P0, c).modes(&[0, 1]).target(bears).target(bears).try_go();
    assert!(r.is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, c).modes(&[0, 1]).target(bears).target(bears).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
    assert!(has(&t, bears, KeywordKind::FirstStrike));
}

#[test]
fn one_mode_costs_only_the_mana_cost() {
    cr!("702.120a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Borrowed Hostility");
    t.cast(P0, c).modes(&[1]).target(bears).go();
    assert_eq!(untapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert!(has(&t, bears, KeywordKind::FirstStrike));
}

#[test]
fn a_nonmana_escalate_cost_is_paid_once_per_extra_mode() {
    cr!("702.120a");
    assert_supported_card("Collective Brutality");
    let mut t = TestGame::new(2);
    // Collective Brutality: {1}{B}, escalate—discard a card; "• Target opponent reveals
    // their hand ... discards [an instant or sorcery card]. • Target creature gets -2/-2
    // until end of turn. • Target opponent loses 2 life and you gain 2 life."
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Llanowar Elves");
    t.hand(P0, "Hill Giant");
    let c = t.hand(P0, "Collective Brutality");
    t.cast(P0, c)
        .modes(&[1, 2])
        .target(bears)
        .target(Entity::Player(P1))
        .go();
    // One mode beyond the first: one card discarded.
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    // All three modes: two cards discarded.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Llanowar Elves");
    t.hand(P0, "Hill Giant");
    let c = t.hand(P0, "Collective Brutality");
    t.cast(P0, c)
        .modes(&[0, 1, 2])
        .target(Entity::Player(P1))
        .target(bears)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.hand_size(P0), 1);
    // Without enough cards to discard, the extra modes can't be chosen.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Collective Brutality");
    let r = t
        .cast(P0, c)
        .modes(&[1, 2])
        .target(bears)
        .target(Entity::Player(P1))
        .try_go();
    assert!(r.is_err());
}

#[test]
fn escalate_can_tap_a_creature_you_just_got() {
    cr!("702.120a");
    ruling!(
        "Collective Effort",
        "You can tap any untapped creature you control to pay the escalate cost, including one you haven't controlled continuously since the beginning of the turn."
    );
    assert_supported_card("Collective Effort");
    let mut t = TestGame::new(2);
    // Collective Effort: {1}{W}{W}, escalate—tap an untapped creature you control;
    // "• Destroy target creature with power 4 or greater. • Destroy target enchantment.
    // • Put a +1/+1 counter on each creature target player controls."
    t.lands(P0, "Plains", 3);
    let sick = t.battlefield_sick(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Collective Effort");
    t.cast(P0, c)
        .modes(&[0, 2])
        .target(giant)
        .target(Entity::Player(P0))
        .go();
    assert!(t.obj(sick).tapped);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.pt(sick), (3, 3));
}

#[test]
fn cost_reductions_apply_to_the_total_including_escalate_costs() {
    cr!("702.120a");
    ruling!(
        "Borrowed Hostility",
        "Effects that reduce the cost of spells reduce the total cost, including any escalate costs added."
    );
    let mut t = TestGame::new(2);
    // Goblin Electromancer: "Instant and sorcery spells you cast cost {1} less to cast."
    // {R} + {3} - {1} = {2}{R} (the reduction applies after the escalate cost is added).
    t.battlefield(P0, "Goblin Electromancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Borrowed Hostility");
    t.cast(P0, c).modes(&[0, 1]).target(bears).target(bears).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn escalate_is_paid_even_without_paying_the_mana_cost() {
    cr!("702.120a");
    ruling!(
        "Borrowed Hostility",
        "If an effect allows you to cast a spell that has escalate without paying its mana cost, you pay escalate costs for that spell if you choose more than one mode."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Borrowed Hostility");
    mtg_engine::casting::grant_play_permission(&mut t.g, P0, vec![c], Duration::EndOfTurn, true, None);
    t.cast(P0, c)
        .method(CastMethod::Free)
        .modes(&[0, 1])
        .target(bears)
        .target(bears)
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
}
