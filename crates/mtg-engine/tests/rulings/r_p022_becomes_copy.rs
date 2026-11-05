//! Rulings batch P022 — permanents that become copies: effects that already applied keep
//! applying (CR 613.1, 611.2c, 707.2), becoming a copy isn't entering (CR 603.6a, 607.5a),
//! copies use last known information (CR 608.2h), "for as long as" durations (CR 611.2b),
//! illegal targets (CR 608.2b), copies of tokens (CR 707.2, 111.3), and several triggers
//! at once (CR 603.3b).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Wall of Stolen Identity ------------------------------------------------------------------

const WALL: &str = "Wall of Stolen Identity";

/// P0's Wall of Stolen Identity enters as a copy of `what`; its "When you do" trigger is
/// put on the stack (not resolved).
fn wall_copies(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported(WALL);
    t.answer_choose(P0, &[obj(what)]);
    let w = t.enter(P0, WALL);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the tap trigger");
    t.g.current(w)
}

#[test]
fn wall_of_stolen_identity_keeps_a_copied_animated_land_tapped() {
    cr!("611.2b", "502.3", "707.2");
    ruling!("Wall of Stolen Identity", "If Wall of Stolen Identity copies a creature that’s normally not a creature (such as Mutavault), the copied permanent will be tapped and will remain tapped even when it’s no longer a creature for as long as you control Wall of Stolen Identity.");
    let mut t = TestGame::new(2);
    // P1's land is a 2/2 creature until end of turn (as Mutavault's ability makes it).
    let land = t.battlefield(P1, "Wastes");
    animate_land(&mut t, land);
    let wall = wall_copies(&mut t, land);
    t.resolve_all();
    assert!(t.obj_now(land).tapped);
    // The Wall copied a land (the animation isn't copiable).
    assert!(!is_creature(&t, wall));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_creature(&t, land));
    assert!(t.obj_now(land).tapped, "it doesn't untap during P1's untap step");
}

#[test]
fn wall_of_stolen_identity_leaving_before_its_trigger_resolves() {
    cr!("611.2b", "502.3");
    ruling!("Wall of Stolen Identity", "If Wall of Stolen Identity leaves the battlefield before its triggered ability has resolved, the copied creature will be tapped, but it will untap as normal.");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = wall_copies(&mut t, giant);
    kill(&mut t, wall);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(giant).tapped);
}

#[test]
fn wall_of_stolen_identity_changing_control_ends_the_lock() {
    cr!("611.2b", "502.3");
    ruling!("Wall of Stolen Identity", "If another player gains control of Wall of Stolen Identity, its effect keeping the creature tapped expires. It won’t keep the creature from untapping anymore, even if you later regain control of Wall of Stolen Identity.");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = wall_copies(&mut t, giant);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    give_control(&mut t, wall, P1);
    give_control(&mut t, wall, P0);
    assert_eq!(t.obj_now(wall).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(giant).tapped);
    // Without the control change the Giant would have stayed tapped.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    wall_copies(&mut t, giant);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(giant).tapped);
}

// --- Effects that already applied --------------------------------------------------------------

#[test]
fn mindlink_mech_keeps_noncopy_effects_after_it_becomes_a_copy() {
    cr!("613.1", "707.2", "702.122a");
    ruling!("Mindlink Mech", "If a continuous effect that isn't a copy effect is modifying Mindlink Mech's power, toughness, or types, that effect will continue to apply to it after it becomes a copy.");
    supported("Mindlink Mech");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mindlink Mech");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Crew 1 with the Bears: the Mech becomes an artifact creature (a noncopy effect).
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    let i = activated_index(&t, mech, "Crew");
    t.activate(P0, mech, i, &[]).expect("crew");
    t.resolve();
    assert!(is_creature(&t, mech));
    // Its trigger waits; Giant Growth on it first.
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(mech)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, mech), "Grizzly Bears");
    // 4/3 by the exception, +3/+3, and still a creature by crewing.
    assert_eq!(t.pt(mech), (7, 6));
    assert!(is_creature(&t, mech));
    assert!(t.obj_now(mech).chars.has_subtype("Vehicle"));
}

#[test]
fn effects_that_applied_before_becoming_a_copy_still_apply() {
    cr!("613.1", "611.2c", "707.2");
    ruling!("Nascent Metamorph", "If an effect begins to apply to Nascent Metamorph before it becomes a copy, that effect will continue to apply.");
    ruling!("Protean Thaumaturge", "If an effect begins to apply to Protean Thaumaturge before it becomes a copy of another creature, that effect will continue to apply.");
    ruling!("True Polymorph", "If an effect begins to apply to the first target permanent before it becomes a copy of another permanent, that effect will continue to apply.");
    ruling!("Saheeli, Sublime Artificer", "If an effect begins to apply to the target artifact before it becomes a copy of another permanent, that effect will continue to apply.");
    // Nascent Metamorph: Giant Growth, then it attacks and becomes a Hill Giant.
    supported("Nascent Metamorph");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Nascent Metamorph");
    giant_growth(&mut t, P0, m);
    stack_library(&mut t, P1, &["Hill Giant"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(m, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Hill Giant");
    assert_eq!(t.pt(m), (6, 6));
    // Protean Thaumaturge: Giant Growth, then an enchantment enters.
    supported("Protean Thaumaturge");
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Protean Thaumaturge");
    let giant = t.battlefield(P1, "Hill Giant");
    giant_growth(&mut t, P0, pt);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(name_of(&t, pt), "Hill Giant");
    assert_eq!(t.pt(pt), (7, 7));
    // True Polymorph on a Giant Grown Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    giant_growth(&mut t, P0, bears);
    cast_spell(
        &mut t,
        P0,
        "True Polymorph",
        "{4}{U}{U}",
        &[obj(bears), obj(giant)],
        None,
        None,
    );
    assert_eq!(name_of(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (6, 6));
    // Saheeli's −2: a Giant Grown Ornithopter becomes a copy of Hill Giant (an artifact).
    supported("Saheeli, Sublime Artificer");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli, Sublime Artificer");
    t.g.add_counters(Entity::Object(saheeli), counters::LOYALTY, 5, None);
    let thopter = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P0, "Hill Giant");
    giant_growth(&mut t, P0, thopter);
    assert_eq!(t.pt(thopter), (3, 5));
    t.answer_targets(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, saheeli, "becomes a copy").expect("−2");
    t.resolve_all();
    assert_eq!(name_of(&t, thopter), "Hill Giant");
    assert!(is_artifact(&t, thopter));
    assert_eq!(t.pt(thopter), (6, 6));
}

// --- Mirrors activated in response to themselves -------------------------------------------

#[test]
fn mirage_mirror_keeps_the_effect_of_an_ability_it_had_as_an_earlier_copy() {
    cr!("613.1", "611.2c", "707.2");
    ruling!("Mirage Mirror", "If an effect begins to apply to Mirage Mirror before it becomes a copy of another permanent, that effect will continue to apply. For example, if Mirage Mirror is activated twice in response to itself targeting first Rampaging Hippo then Frilled Sandwalla, the ability it has while it's a copy of Frilled Sandwalla can be activated and its effect will continue to apply while Mirage Mirror is a copy of Rampaging Hippo.");
    supported("Mirage Mirror");
    supported("Rampaging Hippo");
    supported("Frilled Sandwalla");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Mirage Mirror");
    let hippo = t.battlefield(P1, "Rampaging Hippo");
    let sandwalla = t.battlefield(P1, "Frilled Sandwalla");
    t.lands(P0, "Wastes", 4);
    t.lands(P0, "Forest", 2);
    let i = activated_index(&t, mirror, "becomes a copy");
    t.activate(P0, mirror, i, &[obj(hippo)]).expect("first");
    t.activate(P0, mirror, i, &[obj(sandwalla)]).expect("second");
    t.resolve();
    assert_eq!(name_of(&t, mirror), "Frilled Sandwalla");
    let pump = activated_index(&t, mirror, "+2/+2");
    t.activate(P0, mirror, pump, &[]).expect("Sandwalla's ability");
    t.resolve();
    assert_eq!(t.pt(mirror), (3, 3));
    t.resolve_all();
    assert_eq!(name_of(&t, mirror), "Rampaging Hippo");
    assert_eq!(t.pt(mirror), (7, 8));
    assert!(has_kw(&t, mirror, KeywordKind::Trample));
}

#[test]
fn mirror_of_the_forebears_keeps_the_effect_of_an_ability_it_had_as_an_earlier_copy() {
    cr!("613.1", "611.2c", "707.2");
    ruling!("Mirror of the Forebears", "If an effect begins to apply to Mirror of the Forebears before it becomes a copy of another permanent, that effect will continue to apply. For example, if Mirror of the Forebears is activated twice in response to itself targeting first Spirit of the Hearth then Temur Sabertooth, the ability it has while it’s a copy of Temur Sabertooth can be activated and its effect will continue to apply while Mirror of the Forebears is a copy of Spirit of the Hearth.");
    supported("Mirror of the Forebears");
    supported("Spirit of the Hearth");
    supported("Temur Sabertooth");
    let mut t = TestGame::new(2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Cat");
    let mirror = t.enter(P0, "Mirror of the Forebears");
    t.settle();
    let spirit = t.battlefield(P0, "Spirit of the Hearth");
    let sabertooth = t.battlefield(P0, "Temur Sabertooth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Forest", 1);
    let i = activated_index(&t, mirror, "becomes a copy");
    t.activate(P0, mirror, i, &[obj(spirit)]).expect("first");
    t.activate(P0, mirror, i, &[obj(sabertooth)]).expect("second");
    t.resolve();
    assert_eq!(name_of(&t, mirror), "Temur Sabertooth");
    // "{1}{G}: You may return another creature you control to its owner's hand. If you
    // do, this creature gains indestructible until end of turn."
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let a = activated_index(&t, mirror, "indestructible");
    t.activate(P0, mirror, a, &[]).expect("Sabertooth's ability");
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(has_kw(&t, mirror, KeywordKind::Indestructible));
    t.resolve_all();
    assert_eq!(name_of(&t, mirror), "Spirit of the Hearth");
    assert!(has_kw(&t, mirror, KeywordKind::Indestructible));
    assert!(is_artifact(&t, mirror));
}

// --- Not entering ---------------------------------------------------------------------------

#[test]
fn deceiver_of_form_copies_dont_enter_and_keep_counters_and_auras() {
    cr!("603.6a", "707.2", "613.1");
    ruling!("Deceiver of Form", "If a creature you control becomes a copy of the revealed creature card, it’s neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won’t trigger.");
    ruling!("Deceiver of Form", "If applicable, creatures you control copy the printed values of the revealed creature card. They will retain any counters they had on them. Any Auras and/or Equipment that were attached with them will remain so.");
    supported("Deceiver of Form");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deceiver of Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    attach_new(&mut t, P0, "Holy Strength", obj(bears));
    // Soul Warden (P1's) would see a creature entering; Elvish Visionary's enters
    // trigger would draw.
    t.battlefield(P1, "Soul Warden");
    stack_library(&mut t, P0, &["Elvish Visionary"]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Elvish Visionary");
    // A 1/1 with a +1/+1 counter and Holy Strength (+1/+2).
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.pt(bears), (3, 4));
    assert_eq!(crate::r_s06_common::attached_to(&t, t.g.attachments_of(obj(t.g.current(bears)))[0]), Some(obj(t.g.current(bears))));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn unstable_shapeshifter_doesnt_get_an_as_enters_choice() {
    cr!("607.5a", "614.12");
    ruling!("Unstable Shapeshifter", "If a creature enters that has some characteristic or ability set when it enters, the copy does not get to make that choice (because it is not entering.");
    supported("Unstable Shapeshifter");
    supported("Voice of All");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    choose_color(&mut t, P1, Color::Red);
    let voice = t.enter(P1, "Voice of All");
    t.resolve_all();
    assert_eq!(name_of(&t, shifter), "Voice of All");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.g.protected_from(voice, bolt));
    assert!(!t.g.protected_from(shifter, bolt));
}

#[test]
fn unstable_shapeshifter_copying_another_copies_the_next_creature_twice() {
    cr!("707.9a", "707.3", "603.2");
    ruling!("Unstable Shapeshifter", "If an Unstable Shapeshifter is on the battlefield when another Unstable Shapeshifter enters, the first one gets the second one's triggered ability, so it will copy the next creature that enters twice. It will only do so the next time a creature enters, because the second \"becomes a copy\" effect to resolve will erase the extra triggered ability.");
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Unstable Shapeshifter");
    let second = t.enter(P0, "Unstable Shapeshifter");
    t.resolve_all();
    assert_eq!(abilities_with(&t, first, "becomes a copy"), 2);
    // Hill Giant enters: the first triggers twice, the second once.
    t.enter(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(name_of(&t, first), "Hill Giant");
    assert_eq!(name_of(&t, second), "Hill Giant");
    assert_eq!(abilities_with(&t, first, "becomes a copy"), 1);
    // The next creature: one trigger each.
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn absorb_identity_copies_the_returned_creature_as_it_last_existed() {
    cr!("608.2h", "707.2");
    ruling!("Absorb Identity", "If Shapeshifters you control do become copies of the creature, they'll use the copiable values of the creature as it last existed on the battlefield. They don't copy whether that creature was tapped or untapped, whether it had any counters on it, whether it had any Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on.");
    supported("Absorb Identity");
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let giant = dressed_giant(&mut t, P1);
    t.answer_yes(P0, true);
    cast_spell(&mut t, P0, "Absorb Identity", "{1}{U}", &[obj(giant)], None, None);
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_printed_giant(&t, sharer, Some((3, 3)), true);
}

// --- Several triggers ------------------------------------------------------------------------

#[test]
fn cemetery_puca_triggers_for_each_creature_dying_together() {
    cr!("603.2c", "603.3b", "707.9a");
    ruling!("Cemetery Puca", "If multiple creatures are put into their owners’ graveyards at the same time, Cemetery Puca’s ability will trigger once for each of those creatures. You choose the order that those abilities will resolve. It’s possible to pay {1}, have Cemetery Puca become a copy of something, activate an activated ability of that creature, then pay {1} again, for example. It will end up a copy of the last creature card paid for.");
    supported("Cemetery Puca");
    supported("Frilled Sandwalla");
    for (order, last) in [([0, 1], "Hill Giant"), ([1, 0], "Frilled Sandwalla")] {
        let mut t = TestGame::new(2);
        let puca = t.battlefield(P0, "Cemetery Puca");
        let giant = t.battlefield(P1, "Hill Giant");
        let sandwalla = t.battlefield(P1, "Frilled Sandwalla");
        t.lands(P0, "Wastes", 3);
        t.lands(P0, "Forest", 1);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        // Wrath of God-style: both die at once (Pyroclasm, 2 damage each... the Giant
        // has 3 toughness, so destroy both together).
        let ids = vec![giant, sandwalla];
        for id in &ids {
            t.g.destroy(*id, None);
        }
        t.g.flush_events();
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.settle();
        assert_eq!(t.stack_len(), 2, "one trigger per creature");
        t.resolve();
        if last == "Hill Giant" {
            assert_eq!(name_of(&t, puca), "Frilled Sandwalla");
            let i = activated_index(&t, puca, "+2/+2");
            t.activate(P0, puca, i, &[]).expect("pump");
            t.resolve();
            assert_eq!(t.pt(puca), (3, 3));
            t.resolve_all();
            assert_eq!(name_of(&t, puca), "Hill Giant");
            assert_eq!(t.pt(puca), (5, 5));
        } else {
            assert_eq!(name_of(&t, puca), "Hill Giant");
            t.resolve_all();
            assert_eq!(name_of(&t, puca), "Frilled Sandwalla");
        }
        assert_eq!(abilities_with(&t, puca, "you may pay {1}"), 1);
    }
}

#[test]
fn lazav_triggers_for_each_creature_card_and_you_order_them() {
    cr!("603.2c", "603.3b");
    ruling!("Lazav, Dimir Mastermind", "If multiple creature cards are put into an opponent's graveyard at the same time, you choose the order that Lazav's triggered abilities go on the stack.");
    supported("Lazav, Dimir Mastermind");
    for (order, last) in [([0, 1], "Hill Giant"), ([1, 0], "Grizzly Bears")] {
        let mut t = TestGame::new(2);
        let lazav = t.battlefield(P0, "Lazav, Dimir Mastermind");
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.g.destroy(giant, None);
        t.g.destroy(bears, None);
        t.g.flush_events();
        let from = t.asked().len();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::Order { .. })));
        t.resolve_all();
        // Its name stays Lazav's; the last resolved copy decides the rest.
        assert_eq!(name_of(&t, lazav), "Lazav, Dimir Mastermind");
        let pt = if last == "Hill Giant" { (3, 3) } else { (2, 2) };
        assert_eq!(t.pt(lazav), pt, "{last}");
    }
}

#[test]
fn spirit_of_resilience_triggers_once_for_cards_leaving_together() {
    cr!("603.2c");
    ruling!("Spirit of Resilience", "If multiple cards leave your graveyard at the same time, Spirit of Resilience's ability will trigger only once.");
    supported("Spirit of Resilience");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Spirit of Resilience");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Bojuka Bog");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.counters(spirit, counters::PLUS1), 1, "{}", t.dump_log());
}

// --- Illegal targets ----------------------------------------------------------------------

#[test]
fn copy_abilities_with_an_illegal_target_do_nothing() {
    cr!("608.2b");
    ruling!("Shapesharer", "If either target becomes an illegal target, the ability will resolve but have no effect.");
    ruling!("True Polymorph", "If either target is an illegal target for True Polymorph as it resolves, the spell will do nothing.");
    // Shapesharer targeting itself and a Hill Giant that leaves.
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{2}{U}");
    let i = activated_index(&t, sharer, "becomes a copy");
    t.activate(P0, sharer, i, &[obj(sharer), obj(giant)]).expect("activation");
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(name_of(&t, sharer), "Shapesharer");
    // True Polymorph: Grizzly Bears into a Hill Giant that leaves; then a Bears that
    // leaves (the Giant stays a Giant).
    for gone in [1, 0] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        lands_for(&mut t, P0, "{4}{U}{U}");
        let tp = t.hand(P0, "True Polymorph");
        t.cast(P0, tp).target(obj(bears)).target(obj(giant)).go();
        kill(&mut t, [bears, giant][gone]);
        t.resolve_all();
        if gone == 1 {
            assert_eq!(name_of(&t, bears), "Grizzly Bears");
        } else {
            assert_eq!(name_of(&t, giant), "Hill Giant");
            assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
        }
    }
}

#[test]
fn the_animus_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!("The Animus", "If either one of the targets of The Animus’s last ability is illegal when the ability begins to resolve, nothing will happen when the ability resolves.");
    supported("The Animus");
    supported("Isamaru, Hound of Konda");
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let animus = t.battlefield(P0, "The Animus");
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        let card = t.exile(P1, "Hill Giant");
        t.g.add_counters(Entity::Object(card), "memory", 1, None);
        let i = activated_index(&t, animus, "becomes a copy");
        t.activate(P0, animus, i, &[obj(isamaru), obj(card)]).expect("activation");
        if remove {
            // The card leaves exile before the ability resolves.
            t.g.move_object(card, Zone::Graveyard(P1), mtg_engine::events::MoveCause::Effect, None);
        }
        t.resolve_all();
        let name = if remove { "Isamaru, Hound of Konda" } else { "Hill Giant" };
        assert_eq!(name_of(&t, isamaru), name);
    }
}

// --- Copying tokens and enchantment creatures ------------------------------------------------

#[test]
fn the_ever_changing_dane_copying_a_token_copies_its_original_values() {
    cr!("707.2", "111.3", "608.2h");
    ruling!("The Ever-Changing 'Dane", "If The Ever-Changing ’Dane becomes a copy of a token creature, it copies the original characteristics of that token as defined by the effect that put it onto the battlefield. It won’t become a token creature.");
    supported("The Ever-Changing 'Dane");
    let mut t = TestGame::new(2);
    let dane = t.battlefield(P0, "The Ever-Changing 'Dane");
    let wolf = dressed_wolf(&mut t, P0);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(wolf)]);
    let i = activated_index(&t, dane, "Sacrifice another creature");
    t.activate(P0, dane, i, &[]).expect("activation");
    t.resolve_all();
    assert!(is_wolf(&t, dane));
    assert_eq!(t.pt(dane), (2, 2));
    assert!(!t.obj_now(dane).is_token());
}

#[test]
fn protean_thaumaturge_can_target_the_enchantment_creature_that_entered() {
    cr!("115.1", "707.2");
    ruling!("Protean Thaumaturge", "If an enchantment creature enters the battlefield under your control, Protean Thaumaturge’s ability can target it.");
    supported("Protean Thaumaturge");
    supported("Nyxborn Rollicker");
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Protean Thaumaturge");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    let nyx = t.enter(P0, "Nyxborn Rollicker");
    t.answer_targets(P0, &[obj(nyx)]);
    t.resolve_all();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().any(|c| c.contains(&obj(nyx))), "{offered:?}");
    assert_eq!(name_of(&t, pt), "Nyxborn Rollicker");
}

#[test]
fn kaya_cant_choose_a_card_no_longer_in_exile() {
    cr!("608.2b", "400.7");
    ruling!("Kaya, Spirits' Justice", "If a creature card that was exiled is no longer in exile when Kaya, Spirits' Justice's first ability resolves (perhaps because of Pull from Eternity or a similar effect), you can't choose that creature card with that ability.");
    supported("Kaya, Spirits' Justice");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kaya, Spirits' Justice");
        let wolf = wolf_token(&mut t, P0);
        let card = t.graveyard(P0, "Hill Giant");
        t.answer_targets(P0, &[obj(wolf)]);
        let exiled = t.g.move_object(card, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1, "Kaya's trigger");
        if leaves {
            let e = exiled.unwrap();
            t.g.move_object(e, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
        }
        let e = exiled.unwrap();
        t.answer_choose(P0, &[obj(e)]);
        t.resolve_all();
        let name = if leaves { "Wolf" } else { "Hill Giant" };
        assert_eq!(name_of(&t, wolf), name);
    }
}
