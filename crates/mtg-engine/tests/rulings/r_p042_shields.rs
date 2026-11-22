//! Rulings batch P042 — prevention shields and "of your choice" effects: what is targeted
//! and what is chosen as the effect resolves (CR 609.7, 615.7), values and sets locked in
//! on resolution (CR 608.2h, 611.2c), what is checked when the damage would be dealt, divided
//! shields (CR 601.2d), and the parts of these spells that happen as they resolve.

use crate::r_p042_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

// --- Chosen, not targeted ----------------------------------------------------------------------

#[test]
fn chosen_sources_arent_targeted_and_are_chosen_as_it_resolves() {
    cr!("609.7a", "115.1", "702.11b", "702.16b");
    ruling!(
        "Pay No Heed",
        "Pay No Heed doesn’t target anything. You can choose a source with hexproof, for example. You choose the damage source as Pay No Heed resolves."
    );
    ruling!(
        "Auriok Replica",
        "You choose a source of damage as the ability resolves. Since the ability doesn’t target the source, you could choose a source with protection from artifacts, for example."
    );
    ruling!(
        "Ajani's Aid",
        "The last ability of Ajani’s Aid doesn’t target anything. You choose a source of damage as the ability resolves."
    );
    ruling!(
        "Burrenton Forge-Tender",
        "The last ability of Burrenton Forge-Tender doesn’t target anything. You choose a source of damage as the ability resolves, if able."
    );
    supported("Ajani's Aid");
    supported("Auriok Replica");
    // (card, P1's source). Slippery Bogle has hexproof; Tel-Jilad Chosen has protection
    // from artifacts; Raging Goblin is red.
    for (card, source) in [
        ("Pay No Heed", "Slippery Bogle"),
        ("Auriok Replica", "Tel-Jilad Chosen"),
        ("Ajani's Aid", "Slippery Bogle"),
        ("Burrenton Forge-Tender", "Raging Goblin"),
    ] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P1, source);
        let from = t.asked().len();
        if card == "Pay No Heed" {
            cast_new(&mut t, P0, card, &[]);
        } else {
            let perm = t.battlefield(P0, card);
            if card == "Auriok Replica" {
                t.lands(P0, "Plains", 1);
            }
            activate(&mut t, P0, perm, 0, &[]);
        }
        let asked_before: Vec<_> = t.asked()[from..].to_vec();
        assert!(
            !asked_before.iter().any(|(_, d)| matches!(
                d,
                Decision::ChooseTargets { .. } | Decision::ChooseEntities { .. }
            )),
            "{card}: nothing targeted or chosen as it's put on the stack"
        );
        t.answer_choose(P0, &[obj(src)]);
        t.resolve_all();
        let chose = t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. }));
        assert!(chose, "{card}: chosen as it resolved");
        // The damage is combat damage for Ajani's Aid.
        damage_at_once(&mut t, &[(src, pl(P0), 1)], true);
        assert_eq!(t.life(P0), 20, "{card}");
        assert_eq!(prevented_events(&t), vec![(src, pl(P0), 1)], "{card}");
    }
}

#[test]
fn burrenton_forge_tender_can_be_activated_with_no_source_to_choose() {
    cr!("609.7a", "602.2");
    ruling!(
        "Burrenton Forge-Tender",
        "You can activate it even if there is no source to choose."
    );
    let mut t = TestGame::new(2);
    let tender = t.battlefield(P0, "Burrenton Forge-Tender");
    activate_resolve(&mut t, P0, tender, 0, &[]);
    assert!(
        t.in_graveyard(P0, "Burrenton Forge-Tender"),
        "the cost was paid"
    );
    assert_eq!(t.stack_len(), 0, "it resolved, choosing nothing");
}

#[test]
fn ajanis_aid_with_no_creature_enables_revolt() {
    cr!("609.7a", "701.21a");
    ruling!(
        "Ajani's Aid",
        "You can activate the last ability even if there is no creature about to deal combat damage, or even no creature at all. No damage will be prevented, but it will enable your revolt abilities."
    );
    supported("Fatal Push");
    let mut t = TestGame::new(2);
    let aid = t.battlefield(P0, "Ajani's Aid");
    activate_resolve(&mut t, P0, aid, 0, &[]);
    assert!(t.in_graveyard(P0, "Ajani's Aid"));
    // Fatal Push with revolt destroys a creature with mana value 4 (Hill Giant).
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Fatal Push", &[obj(giant)]);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn healing_grace_one_source_and_the_life_gain_on_resolution() {
    cr!("609.7a", "615.7");
    ruling!(
        "Healing Grace",
        "You choose only one source, even if that source won’t deal 3 damage."
    );
    ruling!(
        "Healing Grace",
        "You gain 3 life as Healing Grace resolves, even if there’s no damage to prevent."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Healing Grace", &[pl(P0)]);
    let chosen: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max as usize),
            _ => None,
        })
        .collect();
    assert_eq!(chosen, vec![1], "one source");
    assert_eq!(t.life(P0), 23, "gained as it resolved");
    damage(&mut t, bears, pl(P0), 1);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 20, "the Giant's damage isn't prevented");
    damage(&mut t, bears, pl(P0), 2);
    assert_eq!(t.life(P0), 20, "the Bears' next 2 are");
}

// --- Things that happen as the spell resolves ---------------------------------------------------

#[test]
fn draws_and_life_gain_happen_on_resolution_not_on_prevention() {
    cr!("608.2c");
    ruling!(
        "Festival of the Guildpact",
        "You draw a card when Festival of the Guildpact resolves, not when damage is prevented."
    );
    ruling!(
        "Withstand",
        "You draw a card when Withstand resolves, not when damage is prevented."
    );
    ruling!(
        "Bandage",
        "You draw the card when Bandage resolves, not when the damage is actually prevented."
    );
    ruling!(
        "Dazzling Reflection",
        "You gain the life as soon as Dazzling Reflection resolves, not as the damage is prevented."
    );
    supported("Festival of the Guildpact");
    supported("Dazzling Reflection");
    for card in [
        "Festival of the Guildpact",
        "Withstand",
        "Bandage",
        "Dazzling Reflection",
    ] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let (hand, life) = (t.hand_size(P0), t.life(P0));
        match card {
            "Festival of the Guildpact" => {
                t.lands(P0, "Plains", 3);
                let c = t.hand(P0, card);
                t.cast(P0, c).x(2).go();
                t.resolve_all();
            }
            "Dazzling Reflection" => {
                cast_resolve(&mut t, P0, card, &[obj(giant)]);
            }
            _ => {
                cast_resolve(&mut t, P0, card, &[pl(P0)]);
            }
        }
        assert!(prevented_events(&t).is_empty(), "{card}");
        if card == "Dazzling Reflection" {
            assert_eq!(t.life(P0), life + 3, "{card}: gained on resolution");
        } else {
            assert_eq!(t.hand_size(P0), hand + 1, "{card}: drew on resolution");
        }
        damage(&mut t, giant, pl(P0), 1);
        assert_eq!(prevented_events(&t).len(), 1, "{card}");
        if card == "Dazzling Reflection" {
            assert_eq!(t.life(P0), life + 3, "{card}: no more life");
        } else {
            assert_eq!(t.hand_size(P0), hand + 1, "{card}: no draw on prevention");
        }
    }
}

#[test]
fn heal_draws_at_the_next_upkeep_whether_or_not_damage_was_prevented() {
    cr!("603.7a");
    ruling!(
        "Heal",
        "You draw a card at the beginning of the next turn’s upkeep after Heal resolves, regardless of whether 1 damage was actually prevented."
    );
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Heal", &[pl(P0)]);
    assert!(prevented_events(&t).is_empty());
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(
        t.hand_size(P0),
        hand + 1,
        "P0 drew in the next turn's upkeep"
    );
}

#[test]
fn samite_censer_bearer_shields_each_creature_you_control_as_it_resolves() {
    cr!("608.2h", "615.7");
    ruling!(
        "Samite Censer-Bearer",
        "This ability sets up a separate 1-point damage prevention shield on each creature you control at the time the ability resolves."
    );
    supported("Samite Censer-Bearer");
    let mut t = TestGame::new(2);
    let bearer = t.battlefield(P0, "Samite Censer-Bearer");
    let a = t.battlefield(P0, "Craw Wurm");
    let b = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Plains", 1);
    activate_resolve(&mut t, P0, bearer, 0, &[]);
    let later = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    // Each shield is separate: 1 prevented from each of a and b, none for the newcomer.
    damage_at_once(
        &mut t,
        &[
            (giant, obj(a), 2),
            (giant, obj(b), 2),
            (giant, obj(later), 2),
        ],
        false,
    );
    assert_eq!(
        (
            damage_marked(&t, a),
            damage_marked(&t, b),
            damage_marked(&t, later)
        ),
        (1, 1, 2)
    );
    // Each shield is used up.
    damage(&mut t, giant, obj(a), 1);
    assert_eq!(damage_marked(&t, a), 2);
}

#[test]
fn battlefield_medic_counts_clerics_as_it_resolves() {
    cr!("608.2h", "615.7");
    ruling!(
        "Battlefield Medic",
        "The value of X is determined when this resolves, not later when damage would be dealt."
    );
    supported("Battlefield Medic");
    let mut t = TestGame::new(2);
    let medic = t.battlefield(P0, "Battlefield Medic");
    let wurm = t.battlefield(P0, "Craw Wurm");
    activate(&mut t, P0, medic, 0, &[obj(wurm)]);
    // A second Cleric arrives before it resolves: X = 2.
    t.battlefield(P0, "Battlefield Medic");
    t.resolve_all();
    // A third arrives after: X stays 2.
    t.battlefield(P1, "Battlefield Medic");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, obj(wurm), 3);
    assert_eq!(damage_marked(&t, wurm), 1);
}

// --- Divided shields -------------------------------------------------------------------------------

#[test]
fn serra_hymn_divides_on_activation_and_works_with_no_counters() {
    cr!("601.2d", "602.2b", "615.7");
    ruling!(
        "Serra's Hymn",
        "You count the number of counters, then divide up the prevention during the announcement of the ability. Each targeted creature receives the appropriate amount of damage prevention when the ability resolves."
    );
    ruling!(
        "Serra's Hymn",
        "You can activate the ability (choosing no targets) even if there are no verse counters on it. It doesn’t do anything useful if you do this."
    );
    supported("Serra's Hymn");
    // Three verse counters: 2 to the Wurm, 1 to P0, divided while activating.
    let mut t = TestGame::new(2);
    let hymn = t.battlefield(P0, "Serra's Hymn");
    crate::r_s29_common::put_counters(&mut t, hymn, "verse", 3);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm), pl(P0)]);
    divide(&mut t, P0, &[2, 1]);
    let from = t.asked().len();
    let src = t.g.current(hymn);
    t.activate(P0, src, 0, &[]).unwrap();
    let divided = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::Divide { total, .. } => Some(*total),
        _ => None,
    });
    assert_eq!(divided, Some(3), "divided as it was activated");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let giant = t.battlefield(P1, "Hill Giant");
    damage_at_once(&mut t, &[(giant, obj(wurm), 3), (giant, pl(P0), 3)], false);
    assert_eq!((damage_marked(&t, wurm), t.life(P0)), (1, 18));
    // No counters: activated with no targets; nothing happens.
    let mut t = TestGame::new(2);
    let hymn = t.battlefield(P0, "Serra's Hymn");
    t.answer_targets(P0, &[]);
    t.activate(P0, hymn, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Serra's Hymn"));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn remedy_and_angel_of_salvation_divide_as_put_on_the_stack_at_least_1_each() {
    cr!("601.2d", "603.3d");
    ruling!(
        "Remedy",
        "You divide the prevention effects as you cast Remedy, not as it resolves. Each target must be assigned at least 1 prevention “shield.” You can’t choose more than five targets and assign none to a target."
    );
    ruling!(
        "Angel of Salvation",
        "You divide the prevention effects as Angel of Salvation's triggered ability is put onto the stack, not as it resolves. Each target must be assigned at least 1 prevention \"shield.\" You can't choose more than five targets and assign none to a target."
    );
    for angel in [false, true] {
        let mut t = TestGame::new(2);
        let creatures: Vec<ObjectId> = (0..6).map(|_| t.battlefield(P0, "Craw Wurm")).collect();
        let six: Vec<Entity> = creatures.iter().map(|c| obj(*c)).collect();
        // Six candidates; at most five targets may be chosen.
        t.answer_targets(P0, &six[..5]);
        divide(&mut t, P0, &[1, 1, 1, 1, 1]);
        let from = t.asked().len();
        if angel {
            t.enter(P0, "Angel of Salvation");
            t.g.flush_events();
            t.settle();
        } else {
            cast_new(&mut t, P0, "Remedy", &[]);
        }
        assert_eq!(t.stack_len(), 1, "Angel: {angel}");
        let asked = t.asked()[from..].to_vec();
        let max_targets = asked
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseTargets { max, .. } => Some(*max),
                _ => None,
            })
            .expect("targets chosen");
        assert_eq!(max_targets, 5, "Angel: {angel}");
        let divide_asked = asked.iter().find_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        });
        assert_eq!(
            divide_asked,
            Some((5, 5, 1)),
            "Angel: {angel}: divided before resolving, at least 1 each"
        );
        t.resolve_all();
        let giant = t.battlefield(P1, "Hill Giant");
        for c in &creatures[..5] {
            damage(&mut t, giant, obj(*c), 2);
            assert_eq!(damage_marked(&t, *c), 1, "Angel: {angel}");
        }
    }
}

// --- Checked as it resolves, not as damage is dealt --------------------------------------------

#[test]
fn guard_dogs_compares_colors_only_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Guard Dogs",
        "You choose the target on activation and choose the permanent which is compared for color on resolution."
    );
    ruling!(
        "Guard Dogs",
        "You only check colors on resolution and not later when the damage prevention actually is applied."
    );
    supported("Guard Dogs");
    // P0 compares their red Raging Goblin with P1's red Hill Giant; the Giant then becomes
    // blue: its combat damage is still prevented. Compared with P0's white Guard Dogs, it
    // isn't.
    for (compare_goblin, prevented) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let dogs = t.battlefield(P0, "Guard Dogs");
        let goblin = t.battlefield(P0, "Raging Goblin");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Plains", 3);
        let from = t.asked().len();
        activate(&mut t, P0, dogs, 0, &[obj(giant)]);
        let chosen_early = t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. }));
        assert!(!chosen_early, "the permanent is chosen on resolution");
        t.answer_choose(P0, &[obj(if compare_goblin { goblin } else { dogs })]);
        t.resolve_all();
        cast_resolve(&mut t, P0, "Cerulean Wisps", &[obj(giant)]);
        assert!(t.obj_now(giant).chars.colors.contains(Color::Blue));
        damage_at_once(&mut t, &[(giant, pl(P0), 3)], true);
        assert_eq!(
            t.life(P0),
            if prevented { 20 } else { 17 },
            "goblin: {compare_goblin}"
        );
    }
}

#[test]
fn wandering_mage_checks_cleric_or_wizard_only_on_activation_and_resolution() {
    cr!("608.2b");
    ruling!(
        "Wandering Mage",
        "The second ability checks that the targeted creature is a Cleric or Wizard only at the time the ability is activated and at the time it resolves. It doesn’t check at the time the damage is actually prevented."
    );
    supported("Wandering Mage");
    // The target (Prodigal Sorcerer, a Wizard) becomes a Frog after the ability resolves:
    // still prevented. If it becomes a Frog before, the ability doesn't resolve.
    for frog_first in [false, true] {
        let mut t = TestGame::new(2);
        let mage = t.battlefield(P0, "Wandering Mage");
        let wizard = t.battlefield(P0, "Prodigal Sorcerer");
        t.lands(P0, "Island", 2);
        activate(&mut t, P0, mage, 1, &[obj(wizard)]);
        if frog_first {
            cast_resolve(&mut t, P1, "Turn to Frog", &[obj(wizard)]);
        } else {
            t.resolve_all();
            cast_resolve(&mut t, P1, "Turn to Frog", &[obj(wizard)]);
        }
        t.resolve_all();
        assert!(!t
            .obj_now(wizard)
            .chars
            .subtypes
            .iter()
            .any(|s| s == "Wizard"));
        let giant = t.battlefield(P1, "Hill Giant");
        damage(&mut t, giant, obj(wizard), 1);
        // (A 1/1 Frog: read the events, it may have died.)
        assert_eq!(
            damage_dealt_to(&t, obj(wizard)),
            if frog_first { 1 } else { 0 },
            "Frog first: {frog_first}"
        );
        assert_eq!(prevented_events(&t).len(), if frog_first { 0 } else { 1 });
    }
}

#[test]
fn wellgabber_apothecary_target_must_be_tapped_on_activation_and_resolution_only() {
    cr!("608.2b");
    ruling!(
        "Wellgabber Apothecary",
        "The targeted creature must be tapped when the ability is activated and when it resolves. It doesn’t need to remain tapped in between."
    );
    // 0: untapped then tapped again before it resolves (works); 1: untapped at resolution
    // (doesn't resolve).
    for case in 0..2 {
        let mut t = TestGame::new(2);
        let apo = t.battlefield(P0, "Wellgabber Apothecary");
        let lord = t.battlefield(P0, "Lord of Atlantis");
        t.g.tap(lord);
        t.lands(P0, "Plains", 2);
        activate(&mut t, P0, apo, 0, &[obj(lord)]);
        t.g.untap(lord);
        if case == 0 {
            t.g.tap(lord);
        }
        t.resolve_all();
        let giant = t.battlefield(P1, "Hill Giant");
        damage(&mut t, giant, obj(lord), 1);
        assert_eq!(
            damage_marked(&t, lord),
            if case == 0 { 0 } else { 1 },
            "case {case}"
        );
    }
}

#[test]
fn inquisitors_snare_keeps_preventing_after_the_creature_leaves() {
    cr!("615.1a", "113.7a");
    ruling!(
        "Inquisitor's Snare",
        "The prevention effect applies even after the creature leaves the battlefield."
    );
    supported("Inquisitor's Snare");
    supported("Goblin Arsonist");
    // P1's attacking Goblin Arsonist (red) is destroyed by the Snare; its "When this
    // creature dies, you may have it deal 1 damage to any target" damage is prevented.
    let mut t = TestGame::new(2);
    let arsonist = t.battlefield(P1, "Goblin Arsonist");
    to_beginning_of_combat(&mut t, P1);
    attack_with(&mut t, &[(arsonist, pl(P0))]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[pl(P0)]);
    cast_resolve(&mut t, P0, "Inquisitor's Snare", &[obj(arsonist)]);
    assert!(t.in_graveyard(P1, "Goblin Arsonist"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(
        prevented_events(&t).len(),
        1,
        "its dies-trigger damage was prevented"
    );
}

#[test]
fn shadowbane_prevents_only_damage_to_you_and_your_creatures() {
    cr!("615.1a", "615.8");
    ruling!(
        "Shadowbane",
        "This only prevents damage the chosen source would deal to you or a creature you control. If the creature would deal damage to another player, a planeswalker, or a creature you don't control, it does not get prevented."
    );
    supported("Shadowbane");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let pw = t.battlefield(P0, "Gideon Jura");
    t.answer_choose(P0, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Shadowbane", &[]);
    damage(&mut t, giant, pl(P1), 1);
    damage(&mut t, giant, obj(theirs), 1);
    damage(&mut t, giant, obj(pw), 1);
    assert_eq!(
        (
            t.life(P1),
            damage_marked(&t, theirs),
            t.counters(pw, "loyalty")
        ),
        (19, 1, 5)
    );
    assert!(prevented_events(&t).is_empty());
    // Still waiting for the next time: damage to P0's Wurm.
    damage(&mut t, giant, obj(wurm), 3);
    assert_eq!(damage_marked(&t, wurm), 0);
}

#[test]
fn ethersworn_shieldmage_checks_artifact_creatures_as_damage_would_be_dealt() {
    cr!("615.1");
    ruling!(
        "Ethersworn Shieldmage",
        "When Ethersworn Shieldmage’s triggered ability resolves, it doesn’t lock in which permanents will be affected. For the rest of the turn, any time damage would be dealt to a permanent, check whether it’s an artifact creature at that time. If it is, that damage is prevented."
    );
    supported("Ethersworn Shieldmage");
    supported("Tezzeret's Touch");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let walker = t.battlefield(P1, "Ornithopter");
    t.enter(P0, "Ethersworn Shieldmage");
    t.resolve_all();
    // After it resolves: an artifact creature that arrives, and an artifact that becomes
    // a creature, are covered; a creature that isn't an artifact isn't.
    let memnite = t.battlefield(P1, "Memnite");
    let giant = t.battlefield(P1, "Hill Giant");
    let touch = t.battlefield(P0, "Tezzeret's Touch");
    let rock = t.battlefield(P0, "Mind Stone");
    assert!(t.g.attach(touch, obj(rock)));
    t.g.recompute();
    assert!(t.obj_now(rock).is(mtg_engine::types::CardType::Creature));
    for target in [walker, memnite, rock] {
        damage(&mut t, giant, obj(target), 1);
        assert!(t.on_battlefield(target));
        assert_eq!(damage_marked(&t, target), 0);
    }
    damage(&mut t, giant, obj(wurm), 1);
    assert_eq!(damage_marked(&t, wurm), 1, "not an artifact");
    cast_resolve(&mut t, P0, "Turn to Frog", &[obj(memnite)]);
    damage(&mut t, giant, obj(memnite), 1);
    assert_eq!(damage_marked(&t, memnite), 0, "still an artifact creature");
}

#[test]
fn surge_of_salvation_hexproof_is_locked_in_the_prevention_isnt() {
    cr!("611.2c", "615.1");
    ruling!(
        "Surge of Salvation",
        "The set of permanents that gain hexproof is determined as Surge of Salvation resolves. Permanents you begin to control later in the turn won't gain hexproof. However, the damage prevention effect will apply to all creatures you control throughout the turn, even if you didn't control them (or they weren't creatures) as Surge of Salvation resolved."
    );
    supported("Surge of Salvation");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Craw Wurm");
    let stolen = t.battlefield(P1, "Craw Wurm");
    let goblin = t.battlefield(P1, "Raging Goblin");
    cast_resolve(&mut t, P0, "Surge of Salvation", &[]);
    assert!(hexproof(&t, mine));
    cast_resolve(&mut t, P0, "Act of Treason", &[obj(stolen)]);
    assert_eq!(t.obj_now(stolen).controller, P0);
    assert!(!hexproof(&t, stolen), "not when it resolved");
    damage(&mut t, goblin, obj(stolen), 2);
    damage(&mut t, goblin, obj(mine), 2);
    assert_eq!((damage_marked(&t, stolen), damage_marked(&t, mine)), (0, 0));
    assert_eq!(prevented_events(&t).len(), 2);
}

fn hexproof(t: &TestGame, id: ObjectId) -> bool {
    crate::r_p125_common::hexproof(t, id)
}

#[test]
fn avacyn_prevents_all_damage_of_the_color_checked_as_it_is_dealt() {
    cr!("609.7b", "615.1a");
    ruling!(
        "Avacyn, Guardian Angel",
        "Both activated abilities prevent all damage of the chosen color, not just combat damage."
    );
    ruling!(
        "Avacyn, Guardian Angel",
        "For each activated ability, you choose the color as the ability resolves. However, the color of a source that would deal damage to the permanent or player is checked only when it would deal that damage, not as Avacyn’s ability resolves. For example, if you choose blue, damage a blue creature would deal is prevented, even if that creature wasn’t blue (or wasn’t on the battlefield) when Avacyn’s ability resolved."
    );
    supported("Avacyn, Guardian Angel");
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Avacyn, Guardian Angel");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Plains", 2);
    let from = t.asked().len();
    activate(&mut t, P0, avacyn, 0, &[obj(wurm)]);
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })),
        "no color chosen yet"
    );
    crate::r_p125_common::choose_color(&mut t, P0, Color::Blue);
    t.resolve_all();
    // A creature that wasn't on the battlefield, and one that becomes blue later.
    let giant = t.battlefield(P1, "Hill Giant");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.set_step(P1, Step::PrecombatMain);
    activate_resolve(&mut t, P1, sorcerer, 0, &[obj(wurm)]);
    assert_eq!(
        damage_marked(&t, wurm),
        0,
        "noncombat damage from a blue source"
    );
    damage(&mut t, giant, obj(wurm), 1);
    assert_eq!(damage_marked(&t, wurm), 1, "red: not prevented");
    cast_resolve(&mut t, P1, "Cerulean Wisps", &[obj(giant)]);
    to_beginning_of_combat(&mut t, P1);
    t.attack(&[(giant, pl(P0))], &[(wurm, giant)]);
    assert_eq!(
        combat_damage(&t, giant, obj(wurm)),
        0,
        "now blue: prevented"
    );
    assert_eq!(damage_marked(&t, wurm), 1);
}

#[test]
fn prevent_all_damage_effects_arent_limited_to_combat_damage() {
    cr!("615.1a");
    ruling!(
        "Solitary Confinement",
        "Solitary Confinement prevents all damage that would be dealt to you, not just combat damage."
    );
    ruling!(
        "Soul Parry",
        "Soul Parry prevents all damage the targeted creatures would deal this turn, not just combat damage."
    );
    supported("Solitary Confinement");
    supported("Earthquake");
    // Solitary Confinement (P0 has shroud): P1's Earthquake, noncombat damage to each
    // player.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solitary Confinement");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let quake = t.hand(P1, "Earthquake");
    let quake = t.cast(P1, quake).x(2).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    assert_eq!(prevented_events(&t), vec![(quake, pl(P0), 2)]);
    // Soul Parry on Prodigal Sorcerer and a Giant: the Sorcerer's ability damage too.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(sorcerer), obj(giant)]);
    cast_resolve(&mut t, P0, "Soul Parry", &[]);
    t.set_step(P1, Step::PrecombatMain);
    activate_resolve(&mut t, P1, sorcerer, 0, &[pl(P0)]);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 20);
    assert_eq!(prevented_events(&t).len(), 2);
}

#[test]
fn solitary_confinement_sacrificed_in_upkeep_you_draw_that_turn() {
    cr!("504.1", "614.10");
    ruling!(
        "Solitary Confinement",
        "The draw step happens after the upkeep step. If you choose not to pay Solitary Confinement's upkeep cost, you'll sacrifice it. That turn's draw step will happen."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solitary Confinement");
    t.hand(P0, "Grizzly Bears");
    // P1's turn passes; P0 doesn't discard in the upkeep.
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Solitary Confinement"));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), hand + 1, "drew in the draw step");
}

#[test]
fn hidden_retreat_damage_to_anything_and_only_spells() {
    cr!("115.1", "615.1a");
    ruling!(
        "Hidden Retreat",
        "This can be used on damage to a creature, player, or planeswalker."
    );
    ruling!(
        "Hidden Retreat",
        "This can be used on instant or sorcery spells on the stack, but not on abilities."
    );
    supported("Hidden Retreat");
    supported("Arc Lightning");
    let mut t = TestGame::new(2);
    let retreat = t.battlefield(P0, "Hidden Retreat");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let pw = t.battlefield(P0, "Gideon Jura");
    t.hand(P0, "Grizzly Bears");
    // P1's Arc Lightning: 1 to P0, 1 to the Wurm, 1 to Gideon.
    t.set_step(P1, Step::PrecombatMain);
    lands_for_cost(&mut t, P1, "Arc Lightning");
    let card = t.hand(P1, "Arc Lightning");
    t.answer_targets(P1, &[pl(P0), obj(wurm), obj(pw)]);
    divide(&mut t, P1, &[1, 1, 1]);
    let arc = t.cast_with(P1, card, &[]).unwrap();
    let card = t.g.players[0].hand[0];
    t.answer_choose(P0, &[obj(card)]);
    activate(&mut t, P0, retreat, 0, &[obj(arc)]);
    t.resolve_all();
    assert_eq!(
        (
            t.life(P0),
            damage_marked(&t, wurm),
            t.counters(pw, "loyalty")
        ),
        (20, 0, 6)
    );
    // An ability (Prodigal Sorcerer's) on the stack isn't a legal target.
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.hand(P0, "Grizzly Bears");
    t.g.objects[sorcerer.0 as usize].summoning_sick = false;
    t.answer_targets(P1, &[pl(P0)]);
    t.g.turn.priority = Some(P1);
    let _ =
        t.g.activate_ability(P1, sorcerer, t.obj_now(sorcerer).chars.abilities[0].uid);
    assert_eq!(t.stack_len(), 1);
    let ability = *t.g.stack.last().unwrap();
    let from2 = t.asked().len();
    let r = t.activate(P0, retreat, 0, &[Entity::Object(ability)]);
    let offered: Vec<Vec<Entity>> = crate::r_p016_common::target_choices(&t, P0, from2, "");
    assert!(
        r.is_err()
            || offered
                .iter()
                .all(|c| !c.contains(&Entity::Object(ability))),
        "the ability isn't a legal target"
    );
}
