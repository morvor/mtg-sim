//! Rulings batch P041 — prevention shields and "source of your choice" effects: what is
//! chosen or targeted (CR 609.7, 615.7–615.9), when the source's qualities are checked,
//! spells that become permanents (CR 609.7b), several shields and several sources at
//! once, and effects that keep applying after their source changes or leaves.

use crate::r_p041_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

// --- Chosen sources --------------------------------------------------------------------------

#[test]
fn healing_grace_targets_only_the_recipient() {
    cr!("609.7a", "615.7");
    ruling!(
        "Healing Grace",
        "Healing Grace only targets the creature, planeswalker, or player that will receive the damage prevention “shield.” The source isn’t targeted."
    );
    supported("Healing Grace");
    // Black Knight has protection from white: it can't be targeted by Healing Grace, but
    // it can be the chosen source.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "Black Knight");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(knight)]);
    cast_resolve(&mut t, P0, "Healing Grace", &[pl(P0)]);
    let targets = crate::r_p016_common::target_choices(&t, P0, from, "");
    assert_eq!(targets.len(), 1, "one target");
    assert!(!targets[0].contains(&obj(knight)));
    assert_eq!(t.life(P0), 23);
    damage(&mut t, knight, pl(P0), 2);
    assert_eq!(t.life(P0), 23, "the Knight's damage was prevented");
    // The shield had 3: 1 is left.
    damage(&mut t, knight, pl(P0), 2);
    assert_eq!(t.life(P0), 22);
}

/// P1 casts Flametongue Kavu (red; "When this creature enters, it deals 4 damage to target
/// creature") and its trigger targets P0's Craw Wurm (6/4); `respond` runs while the Kavu
/// spell is on the stack. Returns whether the Wurm was dealt damage.
fn kavu_hits_wurm(respond: impl FnOnce(&mut TestGame, ObjectId)) -> bool {
    supported("Flametongue Kavu");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.set_step(P1, Step::PrecombatMain);
    let kavu = cast_new(&mut t, P1, "Flametongue Kavu", &[]);
    respond(&mut t, kavu);
    t.answer_targets(P1, &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Flametongue Kavu").len(), 1);
    assert!(t.stack_len() == 0);
    damage_marked(&t, wurm) > 0 || !t.on_battlefield(wurm)
}

#[test]
fn a_permanent_spell_chosen_as_the_source_keeps_being_prevented_as_a_permanent() {
    cr!("609.7b", "615.9");
    ruling!(
        "Burrenton Forge-Tender",
        "If a permanent spell is chosen as the source of damage, Burrenton Forge-Tender’s prevention effect continues to apply for the rest of the turn to the permanent that spell becomes once it has entered the battlefield."
    );
    ruling!(
        "Pay No Heed",
        "If the chosen source is a permanent spell, damage dealt later in the turn by the permanent it becomes will also be prevented."
    );
    ruling!(
        "Shieldmage Elder",
        "If the ability is activated targeting a spell that becomes a permanent, it also prevents any damage from that permanent this turn."
    );
    supported("Burrenton Forge-Tender");
    supported("Pay No Heed");
    supported("Shieldmage Elder");
    // Without prevention, the Kavu's trigger damages the Wurm.
    assert!(kavu_hits_wurm(|_, _| {}));
    // Burrenton Forge-Tender, choosing the Kavu spell.
    assert!(!kavu_hits_wurm(|t, kavu| {
        let tender = t.battlefield(P0, "Burrenton Forge-Tender");
        t.answer_choose(P0, &[obj(kavu)]);
        activate(t, P0, tender, 0, &[]);
        t.resolve();
        assert_eq!(t.stack_len(), 1, "only the Kavu spell is left");
    }));
    // Pay No Heed, choosing the Kavu spell.
    assert!(!kavu_hits_wurm(|t, kavu| {
        t.answer_choose(P0, &[obj(kavu)]);
        cast_new(t, P0, "Pay No Heed", &[]);
        t.resolve();
        assert_eq!(t.stack_len(), 1);
    }));
    // Shieldmage Elder (tapping two Wizards), targeting the Kavu spell.
    assert!(!kavu_hits_wurm(|t, kavu| {
        let elder = t.battlefield(P0, "Shieldmage Elder");
        t.battlefield(P0, "Prodigal Sorcerer");
        activate(t, P0, elder, 1, &[obj(kavu)]);
        t.resolve();
        assert_eq!(t.stack_len(), 1);
    }));
}

#[test]
fn burrenton_forge_tender_checks_the_sources_color_as_it_deals_damage() {
    cr!("609.7a", "615.9");
    ruling!(
        "Burrenton Forge-Tender",
        "If the red source you chose changes colors before it deals damage, the damage-prevention effect doesn’t apply."
    );
    supported("Cerulean Wisps");
    for wisps in [false, true] {
        let mut t = TestGame::new(2);
        let tender = t.battlefield(P0, "Burrenton Forge-Tender");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_choose(P0, &[obj(giant)]);
        activate_resolve(&mut t, P0, tender, 0, &[]);
        if wisps {
            // The Giant becomes blue until end of turn.
            cast_resolve(&mut t, P0, "Cerulean Wisps", &[obj(giant)]);
        }
        damage(&mut t, giant, pl(P0), 3);
        assert_eq!(t.life(P0), if wisps { 17 } else { 20 }, "blue: {wisps}");
    }
}

#[test]
fn consulate_surveillance_and_auriok_replica_prevent_only_the_damage_to_you() {
    cr!("609.7a", "615.1a");
    ruling!(
        "Consulate Surveillance",
        "If the chosen source would deal damage to you and one or more other players or permanents, only the damage that would be dealt to you is prevented."
    );
    ruling!(
        "Auriok Replica",
        "If the chosen source would deal damage to you and to other creatures, players, and/or planeswalkers, only the damage that would be dealt to you is prevented."
    );
    supported("Consulate Surveillance");
    supported("Auriok Replica");
    supported("Arc Lightning");
    for replica in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        let prevention = if replica {
            t.lands(P0, "Plains", 1);
            t.battlefield(P0, "Auriok Replica")
        } else {
            let s = t.enter(P0, "Consulate Surveillance");
            t.resolve_all();
            s
        };
        // P1's Arc Lightning: 2 damage to P0, 1 to P0's Wurm.
        t.set_step(P1, Step::PrecombatMain);
        lands_for_cost(&mut t, P1, "Arc Lightning");
        let arc = t.hand(P1, "Arc Lightning");
        t.answer_targets(P1, &[pl(P0), obj(wurm)]);
        divide(&mut t, P1, &[2, 1]);
        let arc = t.cast_with(P1, arc, &[]).unwrap();
        t.answer_choose(P0, &[obj(arc)]);
        activate(&mut t, P0, prevention, 0, &[]);
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "Replica: {replica}");
        assert_eq!(damage_marked(&t, wurm), 1, "Replica: {replica}");
    }
}

#[test]
fn goblin_furrier_that_left_the_battlefield_prevents_nothing() {
    cr!("608.2h", "611.3a", "615.1a");
    ruling!(
        "Goblin Furrier",
        "If a spell or ability is cast or activated that would have Goblin Furrier deal damage to a snow creature, and Goblin Furrier leaves the battlefield before it resolves, the damage won’t be prevented."
    );
    supported("Goblin Furrier");
    supported("Hermetic Study");
    supported("Boreal Druid");
    // Hermetic Study: "Enchanted creature has '{T}: This creature deals 1 damage to any
    // target.'" The Furrier aims it at P1's snow 1/1.
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let furrier = t.battlefield(P0, "Goblin Furrier");
        let study = t.battlefield(P0, "Hermetic Study");
        assert!(t.g.attach(study, obj(furrier)));
        t.g.recompute();
        let druid = t.battlefield(P1, "Boreal Druid");
        activate(&mut t, P0, furrier, 0, &[obj(druid)]);
        if leaves {
            exile_now(&mut t, furrier);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(druid), !leaves, "Furrier left: {leaves}");
    }
}

// --- Divided shields, several sources at once -------------------------------------------------

/// The damage events dealt to `to` this turn, by source.
fn dealt_by(t: &TestGame, source: ObjectId, to: Entity) -> u32 {
    damage_events(t)
        .iter()
        .filter(|(s, e, _)| *s == source && *e == to)
        .map(|(_, _, n)| *n)
        .sum()
}

/// P0 casts Remedy with 2 of the prevention on P0 and 3 on `other`.
fn remedy(t: &mut TestGame, other: ObjectId) -> ObjectId {
    lands_for_cost(t, P0, "Remedy");
    let card = t.hand(P0, "Remedy");
    t.answer_targets(P0, &[pl(P0), obj(other)]);
    divide(t, P0, &[2, 3]);
    t.cast_with(P0, card, &[]).unwrap()
}

/// Angel of Salvation enters under P0's control with 2 of the prevention on P0 and 3 on
/// `other`; the trigger is on the stack.
fn angel(t: &mut TestGame, other: ObjectId) {
    t.answer_targets(P0, &[pl(P0), obj(other)]);
    divide(t, P0, &[2, 3]);
    t.enter(P0, "Angel of Salvation");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn a_shield_and_several_sources_at_once_the_recipient_chooses_which_is_prevented() {
    cr!("615.7");
    ruling!(
        "Remedy",
        "If multiple sources would deal damage to an affected target at once, that target or that target’s controller chooses which source’s damage to prevent for each “shield.”"
    );
    ruling!(
        "Angel of Salvation",
        "If multiple sources would deal damage to an affected target at once, that target or that target's controller chooses which source's damage to prevent for each \"shield.\""
    );
    ruling!(
        "Benevolent Ancestor",
        "If the target would be dealt damage by multiple sources at once, that player or that permanent’s controller chooses one of those sources at the time damage would be dealt, and 1 of that damage from that source is prevented."
    );
    supported("Remedy");
    supported("Angel of Salvation");
    // A 2-damage shield on P0 (or 1 from the Ancestor); a Giant and a Bears each deal 2 to
    // P0 at once. P0 orders the damage: the Bears' damage first, then the Giant's.
    for (kind, shield) in [("Remedy", 2), ("Angel", 2), ("Ancestor", 1)] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        match kind {
            "Remedy" => {
                remedy(&mut t, wurm);
            }
            "Angel" => angel(&mut t, wurm),
            _ => {
                let a = t.battlefield(P0, "Benevolent Ancestor");
                activate(&mut t, P0, a, 0, &[pl(P0)]);
            }
        }
        t.resolve_all();
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
        let from = t.asked().len();
        damage_at_once(&mut t, &[(giant, pl(P0), 2), (bears, pl(P0), 2)], true);
        let orders = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::Order { .. }))
            .count();
        assert_eq!(orders, 1, "{kind}: P0 chose");
        assert_eq!(dealt_by(&t, bears, pl(P0)), 2 - shield, "{kind}");
        assert_eq!(dealt_by(&t, giant, pl(P0)), 2, "{kind}");
        assert_eq!(t.life(P0), 20 - 4 + shield as i32, "{kind}");
    }
}

#[test]
fn divided_shields_for_illegal_targets_arent_created() {
    cr!("608.2b", "601.2d", "615.7");
    ruling!(
        "Remedy",
        "If some of the targets become illegal, the original division of prevention effects still applies, but the effects that would have been created for illegal targets aren’t created at all."
    );
    ruling!(
        "Angel of Salvation",
        "If some of the targets become illegal for Angel of Salvation's ability, the original division of prevention effects still applies, but the effects that would have been created for illegal targets aren't created at all."
    );
    // 2 on P0 and 3 on a Bears that is exiled before the spell or ability resolves: P0's
    // shield is still 2.
    for angel_of_salvation in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if angel_of_salvation {
            angel(&mut t, bears);
        } else {
            remedy(&mut t, bears);
        }
        exile_now(&mut t, bears);
        t.resolve_all();
        let giant = t.battlefield(P1, "Hill Giant");
        damage(&mut t, giant, pl(P0), 5);
        assert_eq!(t.life(P0), 17, "Angel: {angel_of_salvation}");
    }
}

#[test]
fn oko_with_an_illegal_target_doesnt_prevent_damage() {
    cr!("608.2b", "120.3c");
    ruling!(
        "Oko, the Trickster",
        "If the target creature is an illegal target by the time Oko's second ability tries to resolve, the ability won't resolve. Damage that would be dealt to Oko won't be prevented."
    );
    supported("Oko, the Trickster");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let oko = t.battlefield(P0, "Oko, the Trickster");
        let bears = t.battlefield(P0, "Grizzly Bears");
        activate(&mut t, P0, oko, 1, &[obj(bears)]);
        if illegal {
            exile_now(&mut t, bears);
        }
        t.resolve_all();
        let giant = t.battlefield(P1, "Hill Giant");
        damage(&mut t, giant, obj(oko), 2);
        if illegal {
            assert!(!t.obj_now(oko).is(CardType::Creature));
            assert_eq!(t.counters(oko, "loyalty"), 2);
        } else {
            assert_eq!(t.obj_now(oko).name(), "Grizzly Bears");
            assert_eq!(t.counters(oko, "loyalty"), 4);
            assert_eq!(damage_marked(&t, oko), 0);
        }
    }
}

#[test]
fn two_dazzling_reflections_prevent_the_next_two_times() {
    cr!("615.8");
    ruling!(
        "Dazzling Reflection",
        "If two Dazzling Reflections targeting the same creature have resolved in the same turn, only one prevention effect will apply the next time it would deal damage. The other effect will apply if the creature would deal damage a second time later that turn."
    );
    supported("Dazzling Reflection");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Dazzling Reflection", &[obj(giant)]);
    cast_resolve(&mut t, P0, "Dazzling Reflection", &[obj(giant)]);
    assert_eq!(t.life(P0), 26);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 26);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 26);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn two_dark_spheres_apply_one_after_the_other() {
    cr!("615.8", "616.1f");
    ruling!(
        "Dark Sphere",
        "So if the source would deal 5 damage after two of these abilities have resolved, the first one prevents 2 damage, reducing it to 3 damage, then the second one prevents a further 1 damage, reducing the total damage dealt to 2."
    );
    supported("Dark Sphere");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    for _ in 0..2 {
        let sphere = t.battlefield(P0, "Dark Sphere");
        t.answer_choose(P0, &[obj(giant)]);
        activate_resolve(&mut t, P0, sphere, 0, &[]);
    }
    damage(&mut t, giant, pl(P0), 5);
    assert_eq!(t.life(P0), 18);
    assert_eq!(damage_events(&t), vec![(giant, pl(P0), 2)]);
}

#[test]
fn zealot_il_vec_prevents_its_combat_damage_even_if_its_damage_was_prevented() {
    cr!("603.12", "615.1a");
    ruling!(
        "Zealot il-Vec",
        "If you choose to have Zealot il-Vec deal 1 damage to a creature and that damage is prevented or replaced, Zealot il-Vec’s combat damage will still be prevented."
    );
    supported("Zealot il-Vec");
    let mut t = TestGame::new(2);
    let zealot = t.battlefield(P0, "Zealot il-Vec");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ancestor = t.battlefield(P1, "Benevolent Ancestor");
    activate_resolve(&mut t, P1, ancestor, 0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    fight_it_out(&mut t, &[zealot], &[]);
    assert_eq!(damage_marked(&t, bears), 0, "the 1 damage was prevented");
    assert!(damage_events(&t).is_empty());
    assert_eq!(t.life(P1), 20, "the Zealot's combat damage was prevented");
}

// --- Force Bubble, Bone Mask, Reverse Damage --------------------------------------------------

#[test]
fn force_bubble_counts_damage_and_is_sacrificed_when_its_trigger_resolves() {
    cr!("614.1a", "603.8");
    ruling!(
        "Force Bubble",
        "If you take 6 damage at once, it will replace all 6 damage and put 6 counters on it. Then the second ability will trigger. Any additional damage you take before this ability resolves also results in counters, but once the ability resolves, this card is sacrificed."
    );
    supported("Force Bubble");
    let mut t = TestGame::new(2);
    let bubble = t.battlefield(P0, "Force Bubble");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, pl(P0), 6);
    assert_eq!(t.counters(bubble, "depletion"), 6);
    assert_eq!(t.stack_len(), 1, "the sacrifice trigger");
    damage(&mut t, giant, pl(P0), 2);
    assert_eq!(t.counters(bubble, "depletion"), 8);
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert!(!t.on_battlefield(bubble));
    damage(&mut t, giant, pl(P0), 2);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn force_bubbles_one_choice_for_damage_dealt_at_once() {
    cr!("616.1", "616.1e");
    ruling!(
        "Force Bubble",
        "If you control more than one, each time you take damage you can decide which one replaces the damage, and that one replaces all of the damage. You can’t split the damage between them."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Force Bubble");
    let b = t.battlefield(P0, "Force Bubble");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, giant, pl(P0), 2);
    let first = (t.counters(a, "depletion"), t.counters(b, "depletion"));
    assert!(first == (2, 0) || first == (0, 2), "{first:?}");
    // Combat damage from two sources at once: one bubble replaces all of it.
    damage_at_once(&mut t, &[(giant, pl(P0), 1), (bears, pl(P0), 1)], true);
    let both = (t.counters(a, "depletion"), t.counters(b, "depletion"));
    assert_eq!(both.0 + both.1, 4, "{both:?}");
    assert!(both.0 == first.0 + 2 || both.1 == first.1 + 2, "not split: {both:?}");
    assert_eq!(t.life(P0), 20);
}

#[test]
fn bone_mask_still_prevents_with_too_few_cards_in_the_library() {
    cr!("615.5", "609.3");
    ruling!(
        "Bone Mask",
        "If you do not have enough cards in your library, the damage is still prevented and all the cards in your library are exiled."
    );
    supported("Bone Mask");
    let mut t = TestGame::new(2);
    let mask = t.battlefield(P0, "Bone Mask");
    t.lands(P0, "Wastes", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    while t.library_size(P0) > 2 {
        let top = *t.g.player(P0).library.last().unwrap();
        t.g.exile_object(top, None);
    }
    let exiled = t.g.exile.len();
    t.answer_choose(P0, &[obj(giant)]);
    activate_resolve(&mut t, P0, mask, 0, &[]);
    damage(&mut t, giant, pl(P0), 5);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.g.exile.len(), exiled + 2);
}

#[test]
fn reverse_damage_reverses_only_the_next_time() {
    cr!("615.8", "615.5");
    ruling!(
        "Reverse Damage",
        "It only affects damage dealt by the source one time. If the source damages you a second time this turn, the damage will not be reversed."
    );
    supported("Reverse Damage");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Reverse Damage", &[]);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 23);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 20);
}

// --- What a shield applies to -----------------------------------------------------------------

#[test]
fn kitsune_palliator_shields_only_what_exists_as_it_resolves() {
    cr!("615.11");
    ruling!(
        "Kitsune Palliator",
        "Kitsune Palliator's ability creates prevention shields for every creature and player when it resolves. It doesn't affect creatures that enter (or become creatures) later."
    );
    supported("Kitsune Palliator");
    let mut t = TestGame::new(2);
    let palliator = t.battlefield(P0, "Kitsune Palliator");
    let wurm = t.battlefield(P1, "Craw Wurm");
    activate_resolve(&mut t, P0, palliator, 0, &[]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, obj(bears), 1);
    damage(&mut t, src, obj(wurm), 1);
    damage(&mut t, src, pl(P0), 1);
    damage(&mut t, src, pl(P1), 1);
    assert_eq!(damage_marked(&t, bears), 1, "the Bears entered later");
    assert_eq!(damage_marked(&t, wurm), 0);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn lithomancers_focus_prevents_damage_from_colorless_sources_arriving_later() {
    cr!("615.1a", "611.2c");
    ruling!(
        "Lithomancer's Focus",
        "Lithomancer’s Focus will prevent damage to the creature from any colorless source that turn, even if that source didn’t exist or wasn’t colorless as Lithomancer’s Focus resolved."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Lithomancer's Focus", &[obj(wurm)]);
    let memnite = t.battlefield(P1, "Memnite");
    damage(&mut t, memnite, obj(wurm), 3);
    assert_eq!(damage_marked(&t, wurm), 0);
    // A colored source isn't covered.
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, bears, obj(wurm), 3);
    assert_eq!(damage_marked(&t, wurm), 3);
}

#[test]
fn shields_on_a_permanent_outlast_their_source_and_its_changes() {
    cr!("611.2a", "615.1a");
    ruling!(
        "Dovin, Hand of Control",
        "Once Dovin’s loyalty ability resolves, the damage prevention continues to apply even if Dovin leaves the battlefield."
    );
    ruling!(
        "Kiora, the Crashing Wave",
        "Once the first ability resolves, the damage prevention continues to apply even if that permanent changes controllers."
    );
    supported("Dovin, Hand of Control");
    supported("Kiora, the Crashing Wave");
    supported("Act of Treason");
    // Dovin's −1 on P1's Giant, then Dovin is exiled.
    let mut t = TestGame::new(2);
    let dovin = t.battlefield(P0, "Dovin, Hand of Control");
    let giant = t.battlefield(P1, "Hill Giant");
    activate_resolve(&mut t, P0, dovin, 0, &[obj(giant)]);
    exile_now(&mut t, dovin);
    let bears = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, giant, pl(P0), 3);
    damage(&mut t, bears, obj(giant), 2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(damage_marked(&t, giant), 0);
    damage(&mut t, bears, pl(P1), 2);
    assert_eq!(t.life(P1), 18, "other damage is dealt");
    // Kiora's +1 on P1's Giant, then P0 gains control of it.
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, the Crashing Wave");
    let giant = t.battlefield(P1, "Hill Giant");
    activate_resolve(&mut t, P0, kiora, 0, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Act of Treason", &[obj(giant)]);
    assert_eq!(t.obj_now(giant).controller, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, giant, pl(P1), 3);
    damage(&mut t, bears, obj(giant), 2);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_marked(&t, giant), 0);
}

#[test]
fn wellgabber_apothecary_keeps_preventing_after_the_creature_untaps() {
    cr!("608.2h", "615.1a");
    ruling!(
        "Wellgabber Apothecary",
        "Once the ability has resolved, all damage that would be dealt to that creature for the rest of the turn will be prevented, regardless of whether it’s tapped or untapped at the time that damage would be dealt."
    );
    supported("Wellgabber Apothecary");
    let mut t = TestGame::new(2);
    let apothecary = t.battlefield(P0, "Wellgabber Apothecary");
    t.lands(P0, "Plains", 2);
    let kithkin = t.battlefield(P0, "Burrenton Forge-Tender");
    t.g.objects[kithkin.0 as usize].tapped = true;
    activate_resolve(&mut t, P0, apothecary, 0, &[obj(kithkin)]);
    t.g.objects[kithkin.0 as usize].tapped = false;
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, obj(kithkin), 3);
    assert!(t.on_battlefield(kithkin));
    assert_eq!(damage_marked(&t, kithkin), 0);
}

#[test]
fn kurbis_keeps_preventing_after_the_creature_loses_its_counters() {
    cr!("608.2h", "615.1a");
    ruling!(
        "Kurbis, Harvest Celebrant",
        "Once the activated ability resolves, damage will be prevented even if the creature later loses all of its +1/+1 counters."
    );
    supported("Kurbis, Harvest Celebrant");
    let mut t = TestGame::new(2);
    let kurbis = t.battlefield(P0, "Kurbis, Harvest Celebrant");
    t.g.add_counters(obj(kurbis), "+1/+1", 2, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    t.g.recompute();
    activate_resolve(&mut t, P0, kurbis, 0, &[obj(bears)]);
    assert_eq!(t.counters(kurbis, "+1/+1"), 1);
    t.g.remove_counters(obj(bears), "+1/+1", 1);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, obj(bears), 3);
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, bears), 0);
}

#[test]
fn delaying_shield_replaces_damage_that_cant_be_prevented() {
    cr!("614.1a", "615.12");
    ruling!(
        "Delaying Shield",
        "Note that the damage is replaced and is not “prevented”. This means it can replace damage that “can’t be prevented”."
    );
    supported("Delaying Shield");
    supported("Leyline of Punishment");
    let mut t = TestGame::new(2);
    let shield = t.battlefield(P0, "Delaying Shield");
    t.battlefield(P1, "Leyline of Punishment");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(shield, "delay"), 3);
}

// --- Combat ------------------------------------------------------------------------------------

#[test]
fn cover_of_winter_prevents_x_in_each_combat_damage_step() {
    cr!("510.4", "615.10");
    ruling!(
        "Cover of Winter",
        "If a creature has double strike, Cover of Winter prevents X of the combat damage it would deal during each combat damage step."
    );
    supported("Cover of Winter");
    supported("Ridgetop Raptor");
    let mut t = TestGame::new(2);
    let cover = t.battlefield(P0, "Cover of Winter");
    t.g.add_counters(obj(cover), "age", 1, None);
    let raptor = t.battlefield(P1, "Ridgetop Raptor");
    to_beginning_of_combat(&mut t, P1);
    attack_with(&mut t, &[(raptor, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 18, "2 - 1 in each of the two steps");
    assert_eq!(damage_events(&t).len(), 2);
}

#[test]
fn woolly_razorbacks_ice_doesnt_prevent_damage_dealt_to_it() {
    cr!("510.2", "615.1a");
    ruling!(
        "Woolly Razorback",
        "Having an ice counter on Woolly Razorback doesn’t prevent combat damage that would be dealt to it."
    );
    supported("Woolly Razorback");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let razorback = t.battlefield(P1, "Woolly Razorback");
    t.g.add_counters(obj(razorback), "ice", 3, None);
    t.g.recompute();
    fight_it_out(&mut t, &[wurm], &[(razorback, wurm)]);
    assert_eq!(t.counters(razorback, "ice"), 2);
    assert_eq!(damage_marked(&t, razorback), 6);
    assert_eq!(damage_marked(&t, wurm), 0, "the Razorback's own damage is prevented");
}

#[test]
fn camel_prevents_damage_from_an_animated_desert_in_combat() {
    cr!("615.1a", "506.4");
    ruling!("Camel", "It does prevent damage from animated Deserts in combat.");
    supported("Camel");
    supported("Hostile Desert");
    let mut t = TestGame::new(2);
    let camel = t.battlefield(P0, "Camel");
    let desert = t.battlefield(P1, "Hostile Desert");
    t.lands(P1, "Mountain", 4);
    t.graveyard(P1, "Forest");
    t.set_step(P0, Step::BeginningOfCombat);
    activate_resolve(&mut t, P1, desert, 1, &[]);
    assert!(t.obj_now(desert).is(CardType::Creature));
    attack_with(&mut t, &at_p1(&[camel]));
    block_and_finish(&mut t, P1, &[(desert, camel)]);
    assert!(t.on_battlefield(camel));
    assert_eq!(damage_marked(&t, camel), 0);
    assert!(damage_events(&t).is_empty());
}

#[test]
fn cautery_slivers_prevention_targets_only_players_planeswalkers_and_slivers() {
    cr!("115.1", "115.2");
    ruling!(
        "Cautery Sliver",
        "Note that the first ability can affect any kind of creature, but the second ability can affect only Slivers. Both abilities can affect players."
    );
    supported("Cautery Sliver");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Cautery Sliver");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    let from = t.asked().len();
    // The damage ability, then the prevention ability (each asks for its target).
    let _ = t.activate(P0, sliver, 0, &[obj(bears)]);
    let damage_targets = crate::r_p016_common::target_choices(&t, P0, from, "");
    assert!(damage_targets[0].contains(&obj(bears)));
    assert!(damage_targets[0].contains(&pl(P1)));
    t.resolve_all();
    assert!(!t.on_battlefield(bears) || damage_marked(&t, bears) == 1);
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Cautery Sliver");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    let from = t.asked().len();
    let _ = t.activate(P0, sliver, 1, &[pl(P0)]);
    let prevent_targets = crate::r_p016_common::target_choices(&t, P0, from, "");
    assert!(!prevent_targets[0].contains(&obj(bears)), "not a Sliver");
    assert!(prevent_targets[0].contains(&pl(P0)));
    assert!(prevent_targets[0].contains(&pl(P1)));
}
