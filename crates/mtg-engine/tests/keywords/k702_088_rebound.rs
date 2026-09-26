//! CR 702.88 Rebound.

use crate::common_k702_011_017::{assert_supported, give_mana_for};
use crate::common_k702_027_037::next_upkeep;
use crate::common_k702_052_066::run_effect;
use mtg_engine::ability::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Staggershock from `p`'s hand at player `target`.
fn cast_staggershock(t: &mut TestGame, p: PlayerId, target: PlayerId) -> ObjectId {
    give_mana_for(t, p, "Staggershock");
    let card = t.hand(p, "Staggershock");
    t.cast(p, card).target(Entity::Player(target)).go()
}

/// The delayed rebound trigger on the stack, if any.
fn rebound_triggers(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|s| {
            matches!(&t.g.obj(**s).stack.as_ref().unwrap().kind,
                mtg_engine::object::StackKind::Triggered { .. })
        })
        .count()
}

#[test]
fn a_spell_cast_from_hand_is_exiled_and_cast_again_next_upkeep() {
    cr!("702.88", "702.88a");
    ruling!(
        "Staggershock",
        "If you cast a spell with rebound from your hand and it resolves, it isn't put into your graveyard. Rather, it's exiled directly from the stack."
    );
    ruling!(
        "Staggershock",
        "If you cast a card from exile this way, it will go to your graveyard when it resolves, fails to resolve, or is countered. It won't go back to exile."
    );
    assert_supported("Staggershock");
    let mut t = TestGame::new(2);
    let spell = cast_staggershock(&mut t, P0, P1);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(spell), Zone::Exile);
    assert!(!t.in_graveyard(P0, "Staggershock"));
    // Nothing happens during the opponent's upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(rebound_triggers(&t), 0);
    // At the beginning of the caster's next upkeep, it may be cast without paying its
    // mana cost.
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(rebound_triggers(&t), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let recast = t.g.current(spell);
    assert_eq!(t.g.obj(recast).zone, Zone::Stack);
    assert_eq!(
        t.g.obj(recast).stack.as_ref().unwrap().cast.method,
        CastMethod::Free
    );
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Staggershock"));
    // No more rebound.
    next_upkeep(&mut t, P0);
    assert_eq!(rebound_triggers(&t), 0);
}

#[test]
fn rebound_does_nothing_for_a_spell_not_cast_from_its_controllers_hand() {
    cr!("702.88a");
    ruling!(
        "Staggershock",
        "If you cast a spell with rebound from anywhere other than your hand"
    );
    ruling!(
        "Staggershock",
        "Rebound will have no effect on copies of spells because you don't cast them from your hand."
    );
    let mut t = TestGame::new(2);
    let card = t.exile(P0, "Staggershock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(card)],
    );
    // Copy it too.
    let spell = t.g.current(card);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Staggershock"));
    next_upkeep(&mut t, P0);
    assert_eq!(rebound_triggers(&t), 0);
}

#[test]
fn a_countered_rebound_spell_goes_to_the_graveyard() {
    cr!("702.88a");
    ruling!(
        "Staggershock",
        "If a spell with rebound that you cast from your hand doesn't resolve for any reason"
    );
    let mut t = TestGame::new(2);
    let spell = cast_staggershock(&mut t, P0, P1);
    assert!(t.g.counter(spell, None));
    t.settle();
    assert!(t.in_graveyard(P0, "Staggershock"));
    next_upkeep(&mut t, P0);
    assert_eq!(rebound_triggers(&t), 0);
}

#[test]
fn declining_to_cast_it_leaves_it_in_exile_for_good() {
    cr!("702.88a");
    ruling!(
        "Staggershock",
        "If you are unable to cast a card from exile this way, or you choose not to, nothing happens when the delayed triggered ability resolves. The card remains exiled for the rest of the game"
    );
    let mut t = TestGame::new(2);
    let spell = cast_staggershock(&mut t, P0, P1);
    t.resolve();
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Exile);
    next_upkeep(&mut t, P0);
    assert_eq!(rebound_triggers(&t), 0);
    assert_eq!(t.zone(spell), Zone::Exile);
}

#[test]
fn casting_with_rebound_is_an_alternative_cost_and_ignores_sorcery_timing() {
    cr!("702.88b", "601.2f");
    ruling!(
        "Staggershock",
        "Timing restrictions based on the card's type (if it's a sorcery) are ignored."
    );
    assert_supported("Distortion Strike");
    assert_supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Distortion Strike");
    let card = t.hand(P0, "Distortion Strike");
    let spell = t.cast(P0, card).target(bears).go();
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Exile);
    // An opponent's Thalia: noncreature spells cost {1} more, added to the alternative
    // cost of casting it without paying its mana cost.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    next_upkeep(&mut t, P0);
    let lands_untapped = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.is_land() && !o.tapped)
        .count();
    assert!(lands_untapped >= 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    // It's cast during the upkeep (a sorcery), paying {1}.
    let recast = t.g.current(spell);
    assert_eq!(t.g.obj(recast).zone, Zone::Stack);
    assert_eq!(t.g.obj(recast).chars.mana_value(), 1);
    let now_untapped = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.is_land() && !o.tapped)
        .count();
    assert_eq!(now_untapped, lands_untapped - 1);
}

#[test]
fn multiple_instances_of_rebound_are_redundant() {
    cr!("702.88c");
    ruling!(
        "Cast Through Time",
        "Multiple instances of rebound on the same spell are redundant."
    );
    assert_supported("Cast Through Time");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cast Through Time");
    let spell = cast_staggershock(&mut t, P0, P1);
    assert_eq!(
        t.g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == mtg_engine::keywords::KeywordKind::Rebound)
            .count(),
        2
    );
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Exile);
    next_upkeep(&mut t, P0);
    assert_eq!(rebound_triggers(&t), 1);
}

#[test]
fn cast_through_time_gives_instants_rebound_even_after_it_leaves() {
    cr!("702.88a");
    ruling!(
        "Cast Through Time",
        "the delayed triggered ability will allow you to cast it during your next upkeep even if Cast Through Time has left the battlefield by then."
    );
    let mut t = TestGame::new(2);
    let ctt = t.battlefield(P0, "Cast Through Time");
    give_mana_for(&mut t, P0, "Lightning Bolt");
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Exile);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(ctt)],
    );
    next_upkeep(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}
