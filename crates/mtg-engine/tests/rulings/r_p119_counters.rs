//! Rulings batch P119 — "+1/+1 counters matter" statics, replacement effects and checks:
//! counting creatures rather than counters, replacement effects that add counters (CR
//! 614.1a, 614.12), permanents entering at the same time (CR 614.12, 603.6a), checks made
//! as damage is dealt (CR 615), untapping during other players' untap steps (CR 502.3),
//! cost reductions (CR 601.2f), and values fixed on resolution (CR 608.2h).

use crate::r_p119_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- counting creatures and counters --------------------------------------------------------

#[test]
fn counts_of_creatures_and_counters() {
    cr!("208.2a", "613.4c", "122.1a");
    ruling!(
        "Fungal Behemoth",
        "Any +1/+1 counters on Fungal Behemoth itself count toward its base power and toughness, then give it a power and toughness bonus. For example, if there are two +1/+1 counters on Fungal Behemoth and three on other creatures you control, Fungal Behemoth is a 7/7 creature."
    );
    ruling!(
        "Armorcraft Judge",
        "Armorcraft Judge's ability counts the number of creatures, not the number of counters. A creature with more than one +1/+1 counter won't cause you to draw more than one card."
    );
    ruling!(
        "High Sentinels of Arashin",
        "High Sentinels of Arashin gets +1/+1 per creature, not per +1/+1 counter. It doesn't matter how many +1/+1 counters are on any other creature you control as long as there's one or more."
    );
    for name in [
        "Fungal Behemoth",
        "Armorcraft Judge",
        "High Sentinels of Arashin",
    ] {
        supported(name);
    }
    let mut t = TestGame::new(2);
    let fb = t.battlefield(P0, "Fungal Behemoth");
    with_counters(&mut t, P0, "Grizzly Bears", 3);
    give_plus1(&mut t, fb, 2);
    assert_eq!(t.pt(fb), (7, 7));

    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Grizzly Bears", 3);
    with_counters(&mut t, P0, "Hill Giant", 1);
    let hand = t.hand_size(P0);
    t.enter(P0, "Armorcraft Judge");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);

    let mut t = TestGame::new(2);
    let hs = t.battlefield(P0, "High Sentinels of Arashin");
    with_counters(&mut t, P0, "Grizzly Bears", 3);
    t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(hs), (4, 5));
}

#[test]
fn primordial_hydra_counts_counters_not_power_for_trample() {
    cr!("613.1f", "611.3a");
    ruling!(
        "Primordial Hydra",
        "Consider only the number of +1/+1 counters on Primordial Hydra when determining if it has trample, not its power and toughness."
    );
    supported("Primordial Hydra");
    let mut t = TestGame::new(2);
    let h = with_counters(&mut t, P0, "Primordial Hydra", 9);
    giant_growth(&mut t, h);
    assert_eq!(t.pt(h), (12, 12));
    assert!(!has_kw(&mut t, h, KeywordKind::Trample));
    give_plus1(&mut t, h, 1);
    assert!(has_kw(&mut t, h, KeywordKind::Trample));
}

#[test]
fn vigean_hydropon_cant_attack_but_has_no_defender() {
    cr!("508.1c", "702.3b");
    ruling!(
        "Vigean Hydropon",
        "Even though this creature can't attack, it doesn't have defender."
    );
    supported("Vigean Hydropon");
    let mut t = TestGame::new(2);
    let h = with_counters(&mut t, P0, "Vigean Hydropon", 5);
    assert!(!has_kw(&mut t, h, KeywordKind::Defender));
    assert!(!crate::r_s02_common::can_attack(&mut t, h));
}

// --- replacement effects adding counters -----------------------------------------------------

#[test]
fn hardened_scales_apply_one_after_the_other() {
    cr!("614.1a", "616.1");
    ruling!(
        "Hardened Scales",
        "Each additional Hardened Scales you control will increase the number of +1/+1 counters placed on a creature you control by one."
    );
    supported("Hardened Scales");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    t.battlefield(P0, "Hardened Scales");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    assert_eq!(plus1(&t, bears), 3);
}

#[test]
fn lifecrafters_gift_counters_are_separate_events() {
    cr!("614.1a", "122.6");
    ruling!(
        "Lifecrafter's Gift",
        "Each +1/+1 counter put on the target creature is a separate event. Replacement effects may affect each of these counters being put on the creature."
    );
    supported("Lifecrafter's Gift");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = with_counters(&mut t, P0, "Hill Giant", 1);
    assert_eq!(plus1(&t, giant), 2);
    cast_resolve(&mut t, P0, "Lifecrafter's Gift", &[obj(bears)]);
    assert_eq!(plus1(&t, bears), 4);
    assert_eq!(plus1(&t, giant), 4);
}

#[test]
fn entering_with_counters_and_the_permanents_own_replacement_effect() {
    cr!("614.12", "614.1c");
    ruling!(
        "Mowu, Loyal Companion",
        "Because Mowu's replacement effect affects only Mowu, it will apply if Mowu somehow enters the battlefield with one or more +1/+1 counters on it and give Mowu an additional +1/+1 counter."
    );
    ruling!(
        "Conclave Mentor",
        "Conclave Mentor's first ability doesn't apply to itself if it's somehow entering the battlefield with a +1/+1 counter on it."
    );
    ruling!(
        "Kami of Whispered Hopes",
        "However, if Kami of Whispered Hopes somehow enters the battlefield with +1/+1 counters it, its first ability won't apply to itself."
    );
    ruling!(
        "Ozolith, the Shattered Spire",
        "However, if Ozolith, the Shattered Spire somehow enters the battlefield with +1/+1 counters it, its first ability won't apply to itself."
    );
    // (card, counters it has after entering with one)
    let cases = [
        ("Mowu, Loyal Companion", 2),
        ("Conclave Mentor", 1),
        ("Kami of Whispered Hopes", 1),
        ("Ozolith, the Shattered Spire", 1),
    ];
    for (name, n) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let c = enter_with(&mut t, P0, name, 1, false);
        assert_eq!(plus1(&t, c), n, "{name}");
        // It applies to others once it's on the battlefield.
        if n == 1 {
            let bears = t.battlefield(P0, "Grizzly Bears");
            give_plus1(&mut t, bears, 1);
            assert_eq!(plus1(&t, bears), 2, "{name}");
        }
    }
}

#[test]
fn entering_with_counters_alongside_a_creature_entering_at_the_same_time() {
    cr!("614.12", "614.1c");
    ruling!(
        "Bramblewood Paragon",
        "If Bramblewood Paragon enters at the same time as another Warrior (due to Living End, for example), that creature doesn't get a +1/+1 counter."
    );
    ruling!(
        "Oona's Blackguard",
        "If Oona's Blackguard enters at the same time as another Rogue (due to Living End, for example), that creature doesn't get a +1/+1 counter."
    );
    ruling!(
        "Ascendant Acolyte",
        "If Ascendant Acolyte is entering the battlefield at the same time as other creatures, it doesn't count any counters those other creatures would enter the battlefield with."
    );
    for (lord, other) in [
        ("Bramblewood Paragon", "Elvish Warrior"),
        ("Oona's Blackguard", "Krovikan Scoundrel"),
    ] {
        supported(lord);
        let mut t = TestGame::new(2);
        let ids = enter_together(&mut t, &[(P0, lord), (P0, other)]);
        assert_eq!(plus1(&t, ids[1]), 0, "{other} with {lord}");
        let later = t.enter(P0, other);
        assert_eq!(plus1(&t, later), 1, "{other} after {lord}");
    }
    supported("Ascendant Acolyte");
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Grizzly Bears", 2);
    let ids = enter_together(&mut t, &[(P0, "Ascendant Acolyte"), (P0, "Mossborn Hydra")]);
    assert_eq!(plus1(&t, ids[1]), 1);
    assert_eq!(plus1(&t, ids[0]), 2);
}

#[test]
fn mossborn_hydra_entering_with_lands_triggers_for_each() {
    cr!("603.6a");
    ruling!(
        "Mossborn Hydra",
        "If Mossborn Hydra somehow enters at the same time as one or more other lands you control, its last ability triggers for each of those lands."
    );
    supported("Mossborn Hydra");
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[(P0, "Mossborn Hydra"), (P0, "Forest"), (P0, "Forest")],
    );
    assert_eq!(stack(&t), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, ids[0]), 4);
}

// --- checks as damage is dealt ------------------------------------------------------------

/// The Bears (with a +1/+1 counter) attack; the counter is removed before damage if
/// `remove`. Returns the cards P0 drew.
fn counter_at_damage(name: &str, remove: bool) -> usize {
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::DeclareBlockers);
    if remove {
        let b = t.g.current(bears);
        t.g.remove_counters(Entity::Object(b), "+1/+1", 1);
    }
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::EndOfCombat);
    t.hand_size(P0) - hand
}

#[test]
fn combat_damage_triggers_check_for_a_counter_as_damage_is_dealt() {
    cr!("603.2", "510.3a");
    ruling!(
        "Bred for the Hunt",
        "A creature that deals combat damage to a player must have a +1/+1 counter on it at the time damage is dealt in order for Bred for the Hunt's ability to trigger."
    );
    ruling!(
        "Haliya, Ascendant Cadet",
        "A creature that deals combat damage to a player must have a +1/+1 counter on it at the time damage is dealt in order for Haliya, Ascendant Cadet’s last ability to trigger."
    );
    for name in ["Bred for the Hunt", "Haliya, Ascendant Cadet"] {
        supported(name);
        assert_eq!(counter_at_damage(name, false), 1, "{name}");
        assert_eq!(counter_at_damage(name, true), 0, "{name}");
    }
}

#[test]
fn hindervines_checks_for_counters_as_damage_is_dealt() {
    cr!("615.1a", "510.1");
    ruling!(
        "Hindervines",
        "Hindervines checks whether a creature has a +1/+1 counter on it at the moment it deals damage. It doesn't matter whether a creature had a +1/+1 counter, or was even on the battlefield, when Hindervines resolved."
    );
    supported("Hindervines");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_resolve(&mut t, P0, "Hindervines", &[]);
    let elves = t.battlefield(P0, "Elite Vanguard");
    give_plus1(&mut t, elves, 1);
    attack_with(
        &mut t,
        &[
            (bears, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    t.advance_to(P0, Step::DeclareBlockers);
    give_plus1(&mut t, bears, 1);
    t.advance_to(P0, Step::EndOfCombat);
    // The Bears (3) and the Vanguard (3) deal damage; the Giant doesn't.
    assert_eq!(t.life(P1), 14);
}

// --- untapping during other players' untap steps ------------------------------------------

#[test]
fn ivorytusk_fortress_untaps_during_other_players_untap_steps() {
    cr!("502.3", "502.4");
    ruling!(
        "Ivorytusk Fortress",
        "Each creature you control with a +1/+1 counter on it untaps at the same time as the active player’s permanents. You can’t choose to not untap them at that time."
    );
    ruling!(
        "Ivorytusk Fortress",
        "Effects that state a creature you control doesn’t untap during your untap step won’t apply during another player’s untap step."
    );
    ruling!(
        "Ivorytusk Fortress",
        "Controlling more than one Ivorytusk Fortress doesn’t allow you to untap any creature more than once during a single untap step."
    );
    supported("Ivorytusk Fortress");
    for fortresses in [1, 2] {
        let mut t = TestGame::new(2);
        for _ in 0..fortresses {
            t.battlefield(P0, "Ivorytusk Fortress");
        }
        let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
        let giant = t.battlefield(P0, "Hill Giant");
        let claus = t.battlefield(P0, "Claustrophobia");
        t.g.attach(claus, obj(bears));
        for id in [bears, giant] {
            t.g.objects[id.0 as usize].tapped = true;
        }
        t.g.recompute();
        let from = n_asked(&t);
        t.advance_to(P1, Step::Upkeep);
        assert!(!t.obj_now(bears).tapped, "{fortresses}");
        assert!(t.obj_now(giant).tapped, "no counter");
        assert!(!t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    }
}

// --- costs ----------------------------------------------------------------------------------

#[test]
fn hamza_reduces_only_generic_mana() {
    cr!("601.2f", "118.7c");
    ruling!(
        "Hamza, Guardian of Arashin",
        "Hamza's first ability affects only generic mana costs. It can't reduce the total cost to cast the spell below {G}{W}."
    );
    supported("Hamza, Guardian of Arashin");
    for (lands, ok) in [
        (&["Forest", "Plains"][..], true),
        (&["Wastes", "Wastes"][..], false),
    ] {
        let mut t = TestGame::new(2);
        for _ in 0..6 {
            with_counters(&mut t, P0, "Grizzly Bears", 1);
        }
        for l in lands {
            t.lands(P0, l, 1);
        }
        let h = t.hand(P0, "Hamza, Guardian of Arashin");
        assert_eq!(t.cast(P0, h).try_go().is_ok(), ok, "{lands:?}");
    }
}

#[test]
fn dyadrine_counts_the_mana_actually_spent() {
    cr!("601.2h", "107.3");
    ruling!(
        "Dyadrine, Synthesis Amalgam",
        "Dyadrine’s second ability cares about the amount of mana that was actually paid to cast Dyadrine. Any effects that increase or decrease the cost to cast it (including the “commander tax” in a Commander game) will also be taken into account."
    );
    supported("Dyadrine, Synthesis Amalgam");
    for animar in [false, true] {
        let mut t = TestGame::new(2);
        if animar {
            with_counters(&mut t, P0, "Animar, Soul of Elements", 1);
        }
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Wastes", if animar { 1 } else { 2 });
        let d = t.hand(P0, "Dyadrine, Synthesis Amalgam");
        t.cast(P0, d).x(2).go();
        t.resolve_all();
        let d = t.named_on_battlefield("Dyadrine, Synthesis Amalgam")[0];
        assert_eq!(plus1(&t, d), if animar { 3 } else { 4 });
    }
}

#[test]
fn arcbound_tracker_counts_spells_cast_before_it_was_on_the_battlefield() {
    cr!("603.2", "601.2i");
    ruling!(
        "Arcbound Tracker",
        "Arcbound Tracker will consider spells you cast before it was on the battlefield. For example, if Arcbound Tracker is the first spell you cast in a turn, each subsequent spell you cast after it's on the battlefield will cause the last ability to trigger."
    );
    supported("Arcbound Tracker");
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Arcbound Tracker", &[]);
    let tracker = t.named_on_battlefield("Arcbound Tracker")[0];
    assert_eq!(plus1(&t, tracker), 2);
    cast_resolve(&mut t, P0, "Opt", &[]);
    assert_eq!(plus1(&t, tracker), 3);
}

// --- values and choices on resolution ---------------------------------------------------------

#[test]
fn sunbringers_touch_counts_your_hand_as_it_resolves() {
    cr!("608.2h", "701.39a");
    ruling!(
        "Sunbringer's Touch",
        "Count the number of cards in your hand as Sunbringer’s Touch resolves to determine the value of X."
    );
    supported("Sunbringer's Touch");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.hand(P0, "Hill Giant");
    }
    cast_resolve(&mut t, P0, "Sunbringer's Touch", &[]);
    assert_eq!(plus1(&t, bears), 3);
    assert!(has_kw(&mut t, bears, KeywordKind::Trample));
}

#[test]
fn nissas_judgment_affects_the_targets_still_legal() {
    cr!("608.2b");
    ruling!(
        "Nissa's Judgment",
        "As Nissa’s Judgment resolves, if at least one of its targets is still legal, it will resolve, affecting only targets that are still legal at that time."
    );
    supported("Nissa's Judgment");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_slots(
        &mut t,
        P0,
        "Nissa's Judgment",
        &[&[obj(a), obj(b)], &[obj(giant)]],
    );
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(plus1(&t, b), 1);
    assert!(
        t.in_graveyard(P1, "Hill Giant"),
        "the 3/3 Bears dealt 3 damage"
    );
}

#[test]
fn basris_lieutenant_can_target_itself() {
    cr!("115.5");
    ruling!(
        "Basri's Lieutenant",
        "Basri's Lieutenant can be the target of its own enters-the-battlefield triggered ability."
    );
    let mut t = TestGame::new(2);
    let from = n_asked(&t);
    let l = t.enter(P0, "Basri's Lieutenant");
    t.resolve_all();
    assert!(crate::r_s02_common::target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&obj(l))));
    assert_eq!(plus1(&t, l), 1);
}

#[test]
fn elrond_counts_the_cards_actually_looked_at() {
    cr!("701.22a", "608.2h");
    ruling!(
        "Elrond, Master of Healing",
        "Elrond, Master of Healing's first ability cares about the number of cards you actually looked at."
    );
    supported("Elrond, Master of Healing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elrond, Master of Healing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let lib = t.g.players[P0.idx()].library.clone();
    for id in &lib[1..] {
        t.g.move_object(
            *id,
            mtg_engine::object::Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            None,
        );
    }
    assert_eq!(t.library_size(P0), 1);
    let from = n_asked(&t);
    cast_new(&mut t, P0, "Magma Jet", &[Entity::Player(P1)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    let maxes: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, text, .. } if !text.contains("any target") => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes.last(), Some(&1), "{maxes:?}");
    assert_eq!((plus1(&t, bears), plus1(&t, giant)), (1, 0));
}

#[test]
fn forgotten_ancient_moves_counters_without_targeting() {
    cr!("115.1", "608.2c");
    ruling!(
        "Forgotten Ancient",
        "Forgotten Ancient's last ability doesn't target any creatures. You choose how many +1/+1 counters will be moved (and onto which creatures) as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let fa = with_counters(&mut t, P0, "Forgotten Ancient", 3);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let from = n_asked(&t);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(plus1(&t, scout), 2, "a hexproof creature it doesn't target");
    assert_eq!(plus1(&t, fa), 1);
}

#[test]
fn rite_of_the_serpent_snake_if_the_creature_survives_with_a_counter() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Rite of the Serpent",
        "If Rite of the Serpent resolves but the creature isn’t destroyed (perhaps because it has indestructible or it regenerated), you’ll get a Snake token if the creature has a +1/+1 counter on it."
    );
    supported("Rite of the Serpent");
    let mut t = TestGame::new(2);
    let myr = with_counters(&mut t, P1, "Darksteel Myr", 1);
    cast_resolve(&mut t, P0, "Rite of the Serpent", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(tokens_named(&t, P0, "Snake"), 1);
}

#[test]
fn blaster_moves_all_its_counters_if_it_has_fewer_than_x() {
    cr!("608.2c", "701.66a");
    ruling!(
        "Blaster, Combat DJ // Blaster, Morale Booster",
        "If Blaster, Morale Booster has fewer than X +1/+1 counters on it as its activated ability resolves, all of its +1/+1 counters are moved."
    );
    supported("Blaster, Combat DJ // Blaster, Morale Booster");
    let mut t = TestGame::new(2);
    let blaster = enter_with(
        &mut t,
        P0,
        "Blaster, Combat DJ // Blaster, Morale Booster",
        0,
        true,
    );
    t.resolve_all();
    let blaster = t.g.current(blaster);
    assert_eq!(plus1(&t, blaster), 3, "modular 3");
    t.g.objects[blaster.0 as usize].summoning_sick = false;
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Wastes", 5);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.activate(P0, blaster, 0, &[obj(thopter)]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, thopter), 3);
    assert_eq!(plus1(&t, blaster), 0);
}

#[test]
fn cloud_cant_attach_an_equipment_that_cant_equip_it() {
    cr!("301.5c", "701.3b");
    ruling!(
        "Cloud, Ex-SOLDIER",
        "You can't use Cloud's second ability to try to attach an Equipment to Cloud if that Equipment can't legally be attached to Cloud."
    );
    supported("Cloud, Ex-SOLDIER");
    // An Equipment that's also a creature (without reconfigure) can't equip a creature.
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Animated Blade",
        "Artifact Creature — Equipment Construct",
        "{2}",
        Some((1, 1)),
        "Equip {1}",
    );
    let blade = t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    t.answer_targets(P0, &[obj(blade)]);
    let cloud = t.enter(P0, "Cloud, Ex-SOLDIER");
    t.resolve_all();
    assert!(t.obj_now(blade).attached_to.is_none());
    assert!(t.on_battlefield(cloud));
    // A normal Equipment is attached.
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Bonesplitter");
    t.answer_targets(P0, &[obj(sword)]);
    let cloud = t.enter(P0, "Cloud, Ex-SOLDIER");
    t.resolve_all();
    assert_eq!(t.obj_now(sword).attached_to, Some(obj(cloud)));
}
