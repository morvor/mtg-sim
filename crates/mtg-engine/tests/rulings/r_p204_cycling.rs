//! Rulings batch P204 — cycling and typecycling (CR 702.29), and tiered (Restoration
//! Magic).

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s04_cycling::{
    check_cycling_is_an_activated_ability, check_typecycling_is_cycling, counter_top_with,
    kinds_on_stack, typecycle,
};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn typecycling_triggers_cycle_abilities_and_is_stopped_with_cycling() {
    cr!("702.29e", "702.29f");
    ruling!(
        "Elvish Aberration",
        "Any ability that triggers whenever a card is cycled will trigger if you activate a forestcycling ability."
    );
    ruling!(
        "Wirewood Guardian",
        "Forestcycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on Forestcycling this card."
    );
    ruling!(
        "Chartooth Cougar",
        "Mountaincycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on Mountaincycling this card."
    );
    ruling!(
        "Eternal Dragon",
        "Plainscycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on Plainscycling this card."
    );
    ruling!(
        "Homing Sliver",
        "Slivercycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on Slivercycling a card."
    );
    ruling!(
        "Vedalken Aethermage",
        "Wizardcycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on Wizardcycling this card."
    );
    for name in [
        "Elvish Aberration",
        "Wirewood Guardian",
        "Chartooth Cougar",
        "Eternal Dragon",
        "Homing Sliver",
        "Vedalken Aethermage",
    ] {
        check_typecycling_is_cycling(name);
    }
}

#[test]
fn typecycling_is_an_activated_ability() {
    cr!("702.29e", "602.1", "112.1");
    ruling!(
        "Wirewood Guardian",
        "Forestcycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with Forestcycling."
    );
    ruling!(
        "Chartooth Cougar",
        "Mountaincycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with Mountaincycling."
    );
    ruling!(
        "Eternal Dragon",
        "Plainscycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with Plainscycling."
    );
    ruling!(
        "Twisted Abomination",
        "Swampcycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with Swampcycling."
    );
    ruling!(
        "Vedalken Aethermage",
        "Wizardcycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with Wizardcycling."
    );
    for name in [
        "Wirewood Guardian",
        "Chartooth Cougar",
        "Eternal Dragon",
        "Twisted Abomination",
        "Vedalken Aethermage",
    ] {
        check_cycling_is_an_activated_ability(name);
    }
}

#[test]
fn typecycling_searches_for_a_card_of_the_type_instead_of_drawing() {
    cr!("702.29e", "701.23a", "205.3i");
    ruling!(
        "Elvish Aberration",
        "The card you find can be a basic Forest or any land card with the Forest land type."
    );
    ruling!(
        "Wirewood Guardian",
        "Unlike the normal cycling ability, Forestcycling doesn’t allow you to draw a card. Instead, it lets you search your library for a Forest card."
    );
    ruling!(
        "Chartooth Cougar",
        "Unlike the normal cycling ability, Mountaincycling doesn't allow you to draw a card. Instead, it lets you search your library for a Mountain card."
    );
    ruling!(
        "Eternal Dragon",
        "Unlike the normal cycling ability, Plainscycling doesn't allow you to draw a card. Instead, it lets you search your library for a Plains card."
    );
    ruling!(
        "Vedalken Aethermage",
        "Unlike the normal cycling ability, Wizardcycling doesn't allow you to draw a card. Instead, it lets you search your library for a Wizard card."
    );
    // (card, a card of the type to find, another of the type, a card not of the type)
    for (name, find, also, not) in [
        ("Elvish Aberration", "Stomping Ground", "Forest", "Mountain"),
        ("Wirewood Guardian", "Forest", "Savannah", "Plains"),
        ("Chartooth Cougar", "Mountain", "Stomping Ground", "Forest"),
        ("Eternal Dragon", "Plains", "Savannah", "Island"),
        ("Vedalken Aethermage", "Prodigal Sorcerer", "Snapcaster Mage", "Merfolk Looter"),
    ] {
        let mut t = TestGame::new(2);
        let f = t.library_top(P0, find);
        let a = t.library_top(P0, also);
        let n = t.library_top(P0, not);
        let offered = typecycle(&mut t, name, Some(f));
        assert!(offered.contains(&Entity::Object(f)), "{name}");
        assert!(offered.contains(&Entity::Object(a)), "{name}");
        assert!(!offered.contains(&Entity::Object(n)), "{name}");
        assert!(t.in_hand(P0, find), "{name}");
    }
}

#[test]
fn a_cycle_trigger_without_a_legal_target_doesnt_stop_cycling() {
    cr!("702.29a", "702.29c", "603.3d");
    ruling!(
        "Rampaging War Mammoth",
        "You can cycle a card even if it has a triggered ability from cycling that won't have a legal target."
    );
    // Rampaging War Mammoth: "Cycling {X}{2}{R}. When you cycle this card, destroy up to X
    // target artifacts." No artifacts: it can be cycled; the card is drawn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Rampaging War Mammoth");
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(1));
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Hand(P0));
    // Either ability can be countered alone: the cycling ability is Stifled under the
    // trigger, which still destroys the artifact.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 4);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Rampaging War Mammoth");
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(1));
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    let cycling = t.g.stack[0];
    stifle(&mut t, cycling);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.zone(top), Zone::Library(P0));
    // The trigger is Stifled: the card is still drawn.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 4);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Rampaging War Mammoth");
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(1));
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    counter_top_with(&mut t, P1, "Stifle");
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.zone(top), Zone::Hand(P0));
}

/// P1 casts Stifle targeting the activated or triggered ability `ability`, and it resolves.
fn stifle(t: &mut TestGame, ability: ObjectId) {
    give_mana_for(t, P1, "Stifle");
    let c = t.hand(P1, "Stifle");
    t.cast(P1, c).target(Entity::Object(ability)).go();
    t.resolve();
}

#[test]
fn a_cast_or_cycle_trigger_resolves_even_if_the_spell_or_cycling_ability_is_gone() {
    cr!("702.29c", "603.2", "603.4");
    ruling!(
        "Drownyard Lurker",
        "If Drownyard Lurker or its cycling ability are countered or otherwise leave the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    ruling!(
        "Warped Tusker",
        "If Warped Tusker or its cycling ability are countered or otherwise leave the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    // "When you cast or cycle this card, create a 0/1 colorless Eldrazi Spawn creature
    // token."
    for name in ["Drownyard Lurker", "Warped Tusker"] {
        supported(name);
        // Cycled: the trigger is above the cycling ability; the cycling ability is
        // Stifled; a Spawn is still created.
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 2);
        let top = t.library_top(P0, "Hill Giant");
        let card = t.hand(P0, name);
        cycle(&mut t, P0, card, 0).unwrap();
        t.settle();
        assert_eq!(kinds_on_stack(&t), "AT", "{name}");
        let cycling = t.g.stack[0];
        stifle(&mut t, cycling);
        assert_eq!(kinds_on_stack(&t), "T", "{name}");
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Spawn").len(), 1, "{name}");
        assert_eq!(t.zone(top), Zone::Library(P0));
        // Cast: the trigger resolves first; the spell is countered under it.
        let mut t = TestGame::new(2);
        t.lands(P0, "Wastes", 7);
        let card = t.hand(P0, name);
        t.cast(P0, card).go();
        t.settle();
        assert_eq!(kinds_on_stack(&t), "ST", "{name}");
        let spell = t.g.stack[0];
        give_mana_for(&mut t, P1, "Cancel");
        let cancel = t.hand(P1, "Cancel");
        t.cast(P1, cancel).target(Entity::Object(spell)).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert_eq!(with_subtype(&t, P0, "Spawn").len(), 1, "{name}");
    }
}

#[test]
fn astral_drifts_trigger_resolves_before_the_card_is_drawn() {
    cr!("702.29c", "603.3", "610.3");
    ruling!(
        "Astral Drift",
        "Astral Drift's triggered ability resolves before the cycling ability does. You won't draw until after you choose the target and choose whether to exile the creature."
    );
    supported("Astral Drift");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Astral Drift");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    // Exiled before the draw.
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.zone(top), Zone::Library(P0));
    t.resolve();
    assert_eq!(t.zone(top), Zone::Hand(P0));
    // It returns at the beginning of the next end step.
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn hollow_one_doesnt_let_you_discard_but_counts_cycled_cards() {
    cr!("601.2f", "702.29a");
    ruling!(
        "Hollow One",
        "Hollow One's first ability doesn't give you permission to discard cards. You'll need another effect that instructs or allows you to discard them, such as a cycling ability."
    );
    supported("Hollow One");
    // Hollow One {5}: "This spell costs {2} less to cast for each card you've cycled or
    // discarded this turn."
    let mut t = TestGame::new(2);
    let others: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, "Craw Wurm")).collect();
    let hollow = t.hand(P0, "Hollow One");
    t.lands(P0, "Wastes", 3);
    // No legal action discards a card.
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    for a in t.g.legal_actions(P0) {
        if let mtg_engine::decision::Action::Activate { source, .. } = a {
            assert!(!others.contains(&source));
        }
    }
    // Cycling two cards (Barren Moor, {B}): it costs {1}.
    t.lands(P0, "Swamp", 2);
    for _ in 0..2 {
        let moor = t.hand(P0, "Barren Moor");
        cycle(&mut t, P0, moor, 0).unwrap();
        t.resolve_all();
    }
    t.cast(P0, hollow).go();
    assert_eq!(untapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hollow One").len(), 1);
}

#[test]
fn ichor_slick_cycled_is_cast_with_madness_before_the_draw() {
    cr!("702.35a", "702.29a", "603.3");
    ruling!(
        "Ichor Slick",
        "If you cycle Ichor Slick, you may cast it for its madness cost. You choose whether or not to cast it with madness before you draw a card from the cycling ability."
    );
    supported("Ichor Slick");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Ichor Slick");
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    // Discarded into exile; the madness trigger is above the cycling ability.
    assert!(t.in_exile("Ichor Slick"));
    assert_eq!(kinds_on_stack(&t), "AT");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    // Cast before the draw.
    assert_eq!(kinds_on_stack(&t), "AS");
    assert_eq!(t.zone(top), Zone::Library(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.zone(top), Zone::Hand(P0));
}

#[test]
fn vile_manifestation_counts_cycling_cards_however_they_got_there() {
    cr!("702.29a", "613.4c");
    ruling!(
        "Vile Manifestation",
        "Vile Manifestation’s first ability counts only the cards with cycling abilities in your graveyard. It doesn’t care whether or not they were cycled to get there."
    );
    supported("Vile Manifestation");
    // "This creature gets +1/+0 for each card with cycling in your graveyard." (0/4)
    let mut t = TestGame::new(2);
    let vile = t.battlefield(P0, "Vile Manifestation");
    // Put into the graveyard without cycling: two cards with cycling, a typecycling card,
    // and one without.
    t.graveyard(P0, "Barren Moor");
    t.graveyard(P0, "Ichor Slick");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Barren Moor");
    t.g.recompute();
    assert_eq!(t.pt(vile), (2, 4));
    // A cycled one counts as well.
    t.lands(P0, "Swamp", 1);
    let moor = t.hand(P0, "Barren Moor");
    cycle(&mut t, P0, moor, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(vile), (3, 4));
}

#[test]
fn restoration_magic_cura_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "702.183a");
    ruling!(
        "Restoration Magic",
        "If you chose the Cura mode and the target permanent is an illegal target when Restoration Magic tries to resolve, it won't resolve and none of its effects will happen. You won't gain 3 life."
    );
    supported("Restoration Magic");
    for kill in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Plains", 2);
        let card = t.hand(P0, "Restoration Magic");
        t.cast(P0, card).modes(&[1]).target(bears).go();
        if kill {
            t.g.destroy(bears, None);
            t.settle();
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if kill { 20 } else { 23 }, "{kill}");
        assert_eq!(untapped_lands(&t, P0), 0, "the Cura cost {{1}} was paid");
    }
}

#[test]
fn shark_typhoon_cycled_for_x_0_makes_a_0_0_shark_that_dies() {
    cr!("107.3e", "702.29c", "704.5f");
    ruling!(
        "Shark Typhoon",
        "You can choose 0 as the value of X in Shark Typhoon's cycling cost. The last ability will trigger, and you'll create a 0/0 blue Shark creature token with flying."
    );
    supported("Shark Typhoon");
    // "Cycling {X}{1}{U}. When you cycle this card, create an X/X blue Shark creature token
    // with flying."
    for x in [0i64, 3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul Warden");
        t.lands(P0, "Island", 5);
        let card = t.hand(P0, "Shark Typhoon");
        t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(x));
        cycle(&mut t, P0, card, 0).unwrap();
        t.settle();
        assert_eq!(kinds_on_stack(&t), "AT");
        assert_eq!(untapped_lands(&t, P0), 3 - x as usize);
        t.resolve_all();
        // Soul Warden: "Whenever another creature enters, you gain 1 life." The Shark was
        // created either way.
        assert_eq!(t.life(P0), 21, "{x}");
        let sharks = with_subtype(&t, P0, "Shark");
        if x == 0 {
            assert!(sharks.is_empty());
        } else {
            assert_eq!(sharks.len(), 1);
            assert_eq!(t.pt(sharks[0]), (3, 3));
            assert!(t
                .obj_now(sharks[0])
                .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
        }
    }
}
