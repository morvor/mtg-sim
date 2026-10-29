//! Rulings batch P053 — power/toughness boosts and setting ("enlarge"): Embiggen, Haunted
//! Plate Mail, Built to Smash, Cankerous Thirst, Growth Cycle, Vigilant Sentry, Season of
//! Gathering, Colossification, Colossus Hammer, Crystalline Nautilus, Wild Might, emblems,
//! Fortifying Draught, Garruk, Unleashed, Giant Inheritance and Gimli's Fury.

use crate::r_p053_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::{counters, CardType, ColorSet, Supertype};
use mtg_engine::*;

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    crate::r_s06_common::has_kw(t, id, kw)
}

#[test]
fn embiggen_counts_every_type_and_cant_target_a_brushwagg() {
    cr!("205.3", "115.1");
    ruling!(
        "Embiggen",
        "All subtypes count, including those that aren’t creature types."
    );
    ruling!("Embiggen", "Brushwaggs know what they did.");
    supported("Embiggen");
    // Razorfield Ripper: Artifact Creature — Equipment Rhino: four types, +4/+4.
    let mut t = TestGame::new(2);
    let ripper = t.battlefield(P0, "Razorfield Ripper");
    let wagg = t.battlefield(P0, "Brushwagg");
    t.lands(P0, "Forest", 1);
    let e = t.hand(P0, "Embiggen");
    let from = t.asked().len();
    t.cast(P0, e).target(obj(ripper)).go();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(ripper)));
    assert!(!offered[0].contains(&obj(wagg)));
    t.resolve_all();
    assert_eq!(t.pt(ripper), (7, 7));
    // A legendary one counts its supertype too: Tuvasa (Legendary Creature — Merfolk
    // Shaman) gets +4/+4.
    let mut t = TestGame::new(2);
    let tuvasa = t.battlefield(P0, "Tuvasa the Sunlit");
    t.lands(P0, "Forest", 1);
    let e = t.hand(P0, "Embiggen");
    t.cast(P0, e).target(obj(tuvasa)).go();
    t.resolve_all();
    assert_eq!(t.pt(tuvasa), (5, 5));
}

#[test]
fn haunted_plate_mail_resolves_even_if_you_gain_a_creature_in_response() {
    cr!("602.5b", "608.2");
    ruling!(
        "Haunted Plate Mail",
        "Although you must control no creatures to activate the \"animation\" ability, having a creature somehow come under your control in response to that ability won't stop it from resolving."
    );
    supported("Haunted Plate Mail");
    let mut t = TestGame::new(2);
    let mail = t.battlefield(P0, "Haunted Plate Mail");
    activate_containing(&mut t, P0, mail, "Spirit").unwrap();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Grizzly Bears");
    assert!(!can_activate_text(&mut t, P0, mail, "Spirit"));
    t.resolve_all();
    assert!(t.obj_now(mail).is(CardType::Creature));
    assert_eq!(t.pt(mail), (4, 4));
}

#[test]
fn pump_spells_with_a_conditional_keyword_check_as_they_resolve() {
    cr!("608.2h", "608.2c");
    ruling!(
        "Built to Smash",
        "An artifact creature gains trample in addition to getting +3/+3, not instead of it."
    );
    ruling!(
        "Gimli's Fury",
        "Gimli's Fury checks whether the creature is legendary as it resolves. If the target isn't legendary as the spell is cast but becomes one before the spell resolves, that creature will gain trample."
    );
    supported("Built to Smash");
    supported("Gimli's Fury");
    // Built to Smash on an attacking artifact creature and on a nonartifact one.
    for (name, artifact) in [("Razorfield Ripper", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let base = t.pt(c);
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        t.resolve_all();
        let base = if artifact { t.pt(c) } else { base };
        t.lands(P0, "Mountain", 1);
        let s = t.hand(P0, "Built to Smash");
        t.cast(P0, s).target(obj(c)).go();
        t.resolve_all();
        assert_eq!(t.pt(c), (base.0 + 3, base.1 + 3), "{name}");
        assert_eq!(has_kw(&t, c, KeywordKind::Trample), artifact, "{name}");
    }
    // Gimli's Fury: the Bears become legendary before it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Gimli's Fury");
    t.cast(P0, s).target(obj(bears)).go();
    crate::r_s26_common::modify_until_eot(
        &mut t,
        bears,
        vec![mtg_engine::ability::Modification::AddSupertypes(vec![
            Supertype::Legendary,
        ])],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 4));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
}

#[test]
fn cankerous_thirst_two_targets_and_choices_on_resolution() {
    cr!("115.3", "608.2c", "601.2h");
    ruling!(
        "Cankerous Thirst",
        "As you cast Cankerous Thirst, you must target a creature for the first effect and you must target a creature for the second effect. Both targets may be the same. As Cankerous Thirst resolves, you decide whether to apply either effect."
    );
    supported("Cankerous Thirst");
    // {B}{G} spent; both target the same Hill Giant; apply only the -3/-3.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Cankerous Thirst");
    let spell = t.cast(P0, c).targets(&[obj(giant), obj(giant)]).go();
    assert_eq!(
        crate::r_s25_common::targets_of(&t, spell),
        vec![obj(giant), obj(giant)]
    );
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let yes_no = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 2);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn growth_cycle_doesnt_count_itself() {
    cr!("608.2h", "608.2n");
    ruling!(
        "Growth Cycle",
        "Because Growth Cycle is still on the stack as you perform its instructions, it isn’t in your graveyard and won’t add to the bonus the creature gets."
    );
    supported("Growth Cycle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Growth Cycle");
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Growth Cycle");
    t.cast(P0, c).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2 + 3 + 2, 2 + 3 + 2));
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn vigilant_sentry_damage_can_become_lethal_below_threshold() {
    cr!("704.5g", "702.109a");
    ruling!(
        "Vigilant Sentry",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Vigilant Sentry may become lethal if the number of cards in your graveyard falls below seven."
    );
    supported("Vigilant Sentry");
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Vigilant Sentry");
    let cards: Vec<_> = (0..7).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.g.recompute();
    assert_eq!(t.pt(sentry), (3, 3));
    damage(&mut t, sentry, 2, sentry);
    assert!(t.on_battlefield(sentry));
    crate::r_s05_common::move_to(&mut t, cards[0], Zone::Exile);
    t.g.recompute();
    t.settle();
    assert!(!t.on_battlefield(sentry));
}

#[test]
fn season_of_gathering_choices_are_made_as_it_resolves() {
    cr!("700.2", "608.2c");
    ruling!(
        "Season of Gathering",
        "Choices made for the first and second modes of Season of Gathering aren't made until the spell is resolving."
    );
    supported("Season of Gathering");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Forest", 6);
    let c = t.hand(P0, "Season of Gathering");
    // Mode 0 (+1/+1 counter) and mode 1 (destroy artifacts or enchantments).
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    let from = t.asked().len();
    t.cast(P0, c).go();
    let asked_at_cast = asked_debug(&t, from);
    assert!(
        !t.asked()[from..].iter().any(|(_, d)| matches!(
            d,
            Decision::ChooseEntities { .. } | Decision::ChooseOption { .. }
        )),
        "{asked_at_cast:?}"
    );
    // On resolution: the Bears get the counter; enchantment is chosen.
    let opts = |_: &mtg_engine::game::Game, d: &Decision| match d {
        Decision::ChooseOption { options, .. } => options
            .iter()
            .position(|o| o.to_lowercase().contains("enchantment"))
            .map(Answer::Index),
        _ => None,
    };
    crate::r_s03_common::respond(&mut t, P0, opts);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(!t.on_battlefield(anthem));
    assert!(t.on_battlefield(ring));
}

#[test]
fn equipment_and_auras_that_boost_regardless_of_the_creature() {
    cr!("613.4c", "613.1f", "701.26a");
    ruling!(
        "Colossification",
        "Colossification can be attached to a creature that's already tapped. Its middle ability won't have any effect, but its last ability certainly will."
    );
    ruling!(
        "Colossus Hammer",
        "Colossus Hammer can be attached to a creature that didn't have flying to begin with. It just won't lose flying."
    );
    supported("Colossification");
    supported("Colossus Hammer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P0, "Forest", 7);
    let c = t.hand(P0, "Colossification");
    t.cast(P0, c).target(obj(bears)).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.pt(bears), (22, 22));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hammer = attach_new(&mut t, P0, "Colossus Hammer", obj(bears));
    assert_eq!(attached_to(&t, hammer), Some(obj(bears)));
    assert_eq!(t.pt(bears), (12, 12));
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
}

#[test]
fn crystalline_nautilus_sacrifices_itself_as_creature_or_aura() {
    cr!("702.103b", "603.2");
    ruling!(
        "Crystalline Nautilus",
        "Crystalline Nautilus's middle ability functions as long as Crystalline Nautilus is on the battlefield, whether it's a creature or an Aura."
    );
    supported("Crystalline Nautilus");
    // As a creature: targeted by Giant Growth, sacrificed.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Crystalline Nautilus");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(n)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(n));
    // As an Aura: targeting the Nautilus itself sacrifices it (Disenchant); targeting the
    // enchanted creature sacrifices the creature (its granted ability).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let n = t.hand(P0, "Crystalline Nautilus");
    let bestow = crate::r_s07_common::cast_methods(&mut t, P0, n)
        .into_iter()
        .find(|m| format!("{m:?}").contains("Bestow"))
        .expect("bestow");
    t.cast(P0, n).method(bestow).target(obj(bears)).go();
    t.resolve_all();
    let aura = t.named_on_battlefield("Crystalline Nautilus")[0];
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert_eq!(t.pt(bears), (6, 6));
    t.lands(P1, "Plains", 2);
    let dis = t.hand(P1, "Disenchant");
    t.cast(P1, dis).target(obj(aura)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(aura));
    assert!(t.in_graveyard(P0, "Crystalline Nautilus"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn wild_might_each_player_may_pay() {
    cr!("118.12", "608.2c");
    ruling!(
        "Wild Might",
        "Each player gets the option to pay when the spell resolves."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    t.lands(P1, "Mountain", 2);
    t.lands(P2, "Mountain", 2);
    let c = t.hand(P0, "Wild Might");
    t.cast(P0, c).target(obj(bears)).go();
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    t.answer_yes(P2, true);
    t.resolve_all();
    let askers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert!(askers.contains(&P0) && askers.contains(&P1) && askers.contains(&P2), "{askers:?}");
    // P2 paid: no extra +4/+4.
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P2), 0);
}

fn emblems(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::Emblem && o.controller == p && o.zone == Zone::Command)
        .map(|o| o.id)
        .collect()
}

#[test]
fn emblems_arent_permanents_and_have_no_characteristics() {
    cr!("114.1", "114.3", "114.4");
    ruling!(
        "Elspeth, Knight-Errant",
        "Emblems behave similarly to enchantments: They have an ability that, in a general sense, continually affects the game. The primary difference between them is that emblems aren’t permanents and don’t exist on the battlefield."
    );
    ruling!(
        "Koth of the Hammer",
        "Emblems behave similarly to enchantments: They have an ability that, in a general sense, continually affects the game."
    );
    ruling!(
        "Venser, the Sojourner",
        "Emblems behave similarly to enchantments: They have an ability that, in a general sense, continually affects the game."
    );
    for (name, idx, check) in [
        ("Elspeth, Knight-Errant", 2usize, "indestructible"),
        ("Koth of the Hammer", 2, "mountains"),
        ("Venser, the Sojourner", 2, "exile target permanent"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P0, name);
        put_counters(&mut t, pw, counters::LOYALTY, 8);
        t.activate(P0, pw, idx, &[]).unwrap();
        t.resolve_all();
        let e = emblems(&t, P0);
        assert_eq!(e.len(), 1, "{name}");
        let o = t.g.obj(e[0]);
        assert!(!t.g.battlefield.contains(&e[0]), "{name}");
        assert!(o.zone == Zone::Command, "{name}: {:?}", o.zone);
        assert!(o.chars.card_types.is_empty(), "{name}");
        assert_eq!(o.chars.colors, ColorSet::NONE, "{name}");
        assert!(
            o.chars.abilities.iter().any(|a| a.text.contains(check)),
            "{name}: {:?}",
            o.chars.abilities.iter().map(|a| a.text.clone()).collect::<Vec<_>>()
        );
        // A Wrath of God doesn't touch it; it stays for the rest of the game.
        t.lands(P0, "Plains", 4);
        let w = t.hand(P0, "Wrath of God");
        t.cast(P0, w).go();
        t.resolve_all();
        t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
        assert_eq!(emblems(&t, P0).len(), 1, "{name}");
    }
}

#[test]
fn fortifying_draught_counts_life_gained_ignoring_life_lost() {
    cr!("119.3", "608.2h");
    ruling!(
        "Fortifying Draught",
        "Fortifying Draught counts the total amount of life gained without considering any life you lost during that turn."
    );
    supported("Fortifying Draught");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p009_common::lose(&mut t, P0, 2);
    crate::r_p009_common::gain(&mut t, P0, 3);
    t.lands(P0, "Forest", 1);
    let c = t.hand(P0, "Fortifying Draught");
    t.cast(P0, c).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.pt(bears), (7, 7));
}

#[test]
fn garruk_unleashed_gets_one_loyalty_counter() {
    cr!("606.4");
    ruling!(
        "Garruk, Unleashed",
        "Garruk's second ability gives him only one loyalty counter, no matter how many opponents control how many more creatures than you do."
    );
    supported("Garruk, Unleashed");
    let mut t = TestGame::new(3);
    let g = t.battlefield(P0, "Garruk, Unleashed");
    put_counters(&mut t, g, counters::LOYALTY, 4);
    for p in [P1, P2] {
        for _ in 0..3 {
            t.battlefield(p, "Grizzly Bears");
        }
    }
    let before = t.counters(g, counters::LOYALTY);
    t.activate(P0, g, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(g, counters::LOYALTY), before - 2 + 1);
}

#[test]
fn giant_inheritance_returns_only_if_still_in_the_graveyard() {
    cr!("400.7", "608.2b");
    ruling!(
        "Giant Inheritance",
        "Giant Inheritance will return to its owner's hand from the graveyard only if it's still in the graveyard when its ability resolves."
    );
    supported("Giant Inheritance");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let gi = attach_new(&mut t, P0, "Giant Inheritance", obj(bears));
        destroy(&mut t, gi);
        assert_eq!(t.stack_len(), 1);
        let in_gy = t.g.current(gi);
        if leaves {
            // It leaves the graveyard and comes back before the ability resolves.
            let exiled = crate::r_s05_common::move_to(&mut t, in_gy, Zone::Exile).unwrap();
            crate::r_s05_common::move_to(&mut t, exiled, Zone::Graveyard(P0));
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Giant Inheritance"), !leaves);
        assert_eq!(t.in_graveyard(P0, "Giant Inheritance"), leaves);
    }
}
