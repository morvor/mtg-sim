//! Rulings batch P168, legendary synergies (part 2): We Ride at Dawn and "your commander"
//! (CR 903.3d) and convoke (CR 702.51), Bard Class (CR 716), Éomer's extra combat (CR
//! 500.8), Spider-Verse and the legend rule (CR 704.5j), legendary spells (CR 205.4e),
//! the Ring-bearer (CR 701.54), Serah Farron's cost reduction (CR 601.2f), cast triggers
//! resolving first (CR 603.3), last known information (CR 608.2h) and target uniqueness
//! (CR 115.3).

use crate::r_s01_common::*;
use crate::r_s06_common::{activate_containing, give_control};
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Casts the real card `name` from `p`'s hand, giving `p` the lands for its mana cost.
fn cast_new(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    t.cast(p, c).go()
}

fn choices_since(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. }
            | Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// We Ride at Dawn
// ---------------------------------------------------------------------------

/// Makes `id` a commander (the designation of its card, CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.g.obj(id).owner;
    let name = t.g.obj(id).card.as_ref().unwrap().name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
}

fn mercenaries(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Mercenary").len()
}

#[test]
fn we_ride_at_dawn_follows_the_commanders_owner() {
    cr!("903.3d", "603.2");
    ruling!(
        "We Ride at Dawn",
        "If you control another player’s commander, you won’t create a Mercenary when you attack with that creature. However, if another player controls your commander, you’ll create a Mercenary when it attacks."
    );
    supported("We Ride at Dawn");
    // P0 attacks with P1's commander: nothing.
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    t.battlefield(P0, "We Ride at Dawn");
    t.battlefield(P0, "Fervor");
    let theirs = t.battlefield(P1, "Isamaru, Hound of Konda");
    make_commander(&mut t, theirs);
    give_control(&mut t, theirs, P0);
    t.attack(&[(theirs, pl(P1))], &[]);
    t.settle();
    t.resolve_all();
    assert_eq!(mercenaries(&t, P0), 0);
    // P1 attacks with P0's commander: P0 creates one.
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    t.battlefield(P0, "We Ride at Dawn");
    t.battlefield(P1, "Fervor");
    let mine = t.battlefield(P0, "Isamaru, Hound of Konda");
    make_commander(&mut t, mine);
    give_control(&mut t, mine, P1);
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(mine, pl(P0))]);
    t.resolve_all();
    assert_eq!(mercenaries(&t, P0), 1);
    assert_eq!(mercenaries(&t, P1), 0);
}

#[test]
fn we_ride_at_dawn_triggers_for_each_commander() {
    cr!("903.3c", "603.2");
    ruling!(
        "We Ride at Dawn",
        "If you have two commanders and they’re both declared as attackers, We Ride at Dawn’s last ability will trigger twice."
    );
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    t.battlefield(P0, "We Ride at Dawn");
    let a = t.battlefield(P0, "Tymna the Weaver");
    let b = t.battlefield(P0, "Thrasios, Triton Hero");
    make_commander(&mut t, a);
    make_commander(&mut t, b);
    crate::r_s01_common::attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    t.resolve_all();
    assert_eq!(mercenaries(&t, P0), 2);
}

#[test]
fn we_ride_at_dawn_convoke_can_tap_a_summoning_sick_creature() {
    cr!("702.51a", "302.6");
    ruling!(
        "We Ride at Dawn",
        "You can tap any untapped creature you control to convoke a spell, even one you haven’t controlled continuously since the beginning of your most recent turn."
    );
    // Reki ({2}{G}): a Forest and two Grizzly Bears that just arrived.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "We Ride at Dawn");
    t.lands(P0, "Forest", 1);
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Grizzly Bears");
    let reki = t.hand(P0, "Reki, the History of Kamigawa");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.cast(P0, reki).go();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
}

// ---------------------------------------------------------------------------
// Bard Class
// ---------------------------------------------------------------------------

/// A Bard Class of P0's at `level`.
fn bard_class(t: &mut TestGame, level: u32) -> ObjectId {
    let class = t.battlefield(P0, "Bard Class");
    for n in 2..=level {
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Forest", 1);
        if n == 3 {
            t.lands(P0, "Wastes", 3);
        }
        activate_containing(t, P0, class, &format!("Level {n}")).unwrap();
        t.resolve_all();
    }
    class
}

#[test]
fn two_bard_classes_give_two_counters() {
    cr!("614.1c", "614.12");
    ruling!(
        "Bard Class",
        "If you control multiple Bard Classes, legendary creatures you control enter the battlefield with that many additional +1/+1 counters."
    );
    supported("Bard Class");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bard Class");
    t.battlefield(P0, "Bard Class");
    let isamaru = t.enter(P0, "Isamaru, Hound of Konda");
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(isamaru, "+1/+1"), 2);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn bard_class_level_two_reduces_only_colored_mana() {
    cr!("601.2f", "716.2a");
    ruling!(
        "Bard Class",
        "Unlike most abilities that reduce the costs of spells, Bard Class's Level 2 ability cannot be applied to generic mana costs. For example, a spell that costs {2}{G} to cast would instead cost {2} to cast."
    );
    for (wastes, ok) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        bard_class(&mut t, 2);
        t.lands(P0, "Wastes", wastes);
        let reki = t.hand(P0, "Reki, the History of Kamigawa");
        assert_eq!(t.cast(P0, reki).try_go().is_ok(), ok, "{wastes}");
    }
}

#[test]
fn bard_class_level_three_cards_must_be_played_this_turn() {
    cr!("716.2a", "611.2a");
    ruling!(
        "Bard Class",
        "If you do not play the cards exiled with Bard Classes Level 3 ability on the turn they are exiled, they will remain exiled indefinitely and cannot be played on a later turn."
    );
    let mut t = TestGame::new(2);
    bard_class(&mut t, 3);
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Mountain"]);
    cast_new(&mut t, P0, "Isamaru, Hound of Konda", &[]);
    t.resolve_all();
    let bolt = t.g.current(lib[0]);
    assert_eq!(t.zone(bolt), Zone::Exile);
    // This turn it could be cast; on P0's next turn it can't, and it stays exiled.
    t.lands(P0, "Mountain", 1);
    assert!(crate::r_s02_common::can_cast(&mut t, P0, bolt, CastMethod::Normal));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, bolt, CastMethod::Normal));
    assert_eq!(t.zone(bolt), Zone::Exile);
}

#[test]
fn bard_class_lets_either_face_of_an_exiled_mdfc_be_cast() {
    cr!("712.11", "716.2a");
    ruling!(
        "Bard Class",
        "If you exile a modal double-faced card this way, you may cast either face of that card."
    );
    supported("Valentin, Dean of the Vein // Lisette, Dean of the Root");
    for face in [0u8, 1] {
        let mut t = TestGame::new(2);
        bard_class(&mut t, 3);
        let lib = stack_library(
            &mut t,
            P0,
            &["Valentin, Dean of the Vein // Lisette, Dean of the Root", "Mountain"],
        );
        cast_new(&mut t, P0, "Isamaru, Hound of Konda", &[]);
        t.resolve_all();
        let mdfc = t.g.current(lib[0]);
        assert_eq!(t.zone(mdfc), Zone::Exile);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 2);
        let method = if face == 0 {
            CastMethod::Normal
        } else {
            CastMethod::Half(1)
        };
        t.cast(P0, mdfc).method(method).go();
        t.resolve_all();
        let name = if face == 0 {
            "Valentin, Dean of the Vein"
        } else {
            "Lisette, Dean of the Root"
        };
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{face}");
    }
}

// ---------------------------------------------------------------------------
// Éomer, Marshal of Rohan
// ---------------------------------------------------------------------------

#[test]
fn eomer_sees_legends_dying_with_it() {
    cr!("603.10a", "510.2");
    ruling!(
        "Éomer, Marshal of Rohan",
        "If Éomer, Marshal of Rohan and one or more other attacking legendary creatures you control die at the same time (probably due to combat damage), Éomer, Marshal of Rohan's last ability will trigger."
    );
    supported("Éomer, Marshal of Rohan");
    let mut t = TestGame::new(2);
    let eomer = t.battlefield(P0, "Éomer, Marshal of Rohan");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.attack(&[(eomer, pl(P1)), (isamaru, pl(P1))], &[(wurm, eomer), (ogre, isamaru)]);
    t.resolve_all();
    assert!(!t.on_battlefield(eomer));
    assert!(!t.on_battlefield(isamaru));
    // The ability triggered and resolved: the Bears were untapped.
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn eomers_extra_combat_comes_with_no_main_phase() {
    cr!("500.8", "505.1a");
    ruling!(
        "Éomer, Marshal of Rohan",
        "Notably, Éomer, Marshal of Rohan's triggered ability doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    let mut t = TestGame::new(2);
    let eomer = t.battlefield(P0, "Éomer, Marshal of Rohan");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.attack(&[(eomer, pl(P1)), (isamaru, pl(P1))], &[(ogre, isamaru)]);
    t.resolve_all();
    let mut steps = vec![];
    let turn = t.g.turn.number;
    t.g.run_until(10_000, |g| {
        if steps.last() != Some(&g.turn.step) {
            steps.push(g.turn.step);
        }
        g.turn.step == Step::PostcombatMain || g.turn.number != turn
    });
    // From the first combat's end: straight to the next beginning of combat.
    assert_eq!(steps[0], Step::EndOfCombat);
    assert_eq!(steps[1], Step::BeginningOfCombat);
    assert_eq!(*steps.last().unwrap(), Step::PostcombatMain);
    assert_eq!(
        steps.iter().filter(|s| **s == Step::PostcombatMain).count(),
        1
    );
}

// ---------------------------------------------------------------------------
// Spider-Verse
// ---------------------------------------------------------------------------

fn twin(spider: bool) -> mtg_engine::card::CardDef {
    custom_card(
        "Twin Hero",
        if spider {
            "Legendary Creature — Spider Human"
        } else {
            "Legendary Creature — Human"
        },
        "{1}",
        Some((2, 2)),
        "",
    )
}

#[test]
fn spider_verse_legend_rule_skips_spiders() {
    cr!("704.5j");
    ruling!(
        "Spider-Verse",
        "If, while you control Spider-Verse, you also control a legendary Spider permanent and a single legendary non-Spider permanent with the same name, the legend rule will not cause you to put either one into your graveyard."
    );
    ruling!(
        "Spider-Verse",
        "If, while you control Spider-Verse, you also control more than one legendary non-Spider permanent and at least one legendary Spider permanent all with the same name, the legend rule applies to only the non-Spider permanents. You must choose one of the non-Spider permanents to keep and put the rest of the non-Spider permanents with that name into your graveyard. You may not choose one of the Spider permanents to keep and you may not put any of the Spiders into your graveyard this way."
    );
    supported("Spider-Verse");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spider-Verse");
    let spider = t.custom(P0, twin(true), Zone::Battlefield);
    let a = t.custom(P0, twin(false), Zone::Battlefield);
    t.settle();
    assert!(t.on_battlefield(spider) && t.on_battlefield(a));
    // A second non-Spider: one of the two non-Spiders goes; the Spider can't be chosen.
    let b = t.custom(P0, twin(false), Zone::Battlefield);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(b)]);
    t.settle();
    assert!(t.on_battlefield(spider));
    assert!(t.on_battlefield(b));
    assert!(!t.on_battlefield(a));
    let c = choices_since(&t, from);
    assert_eq!(c.len(), 1);
    assert!(!c[0].contains(&obj(spider)));
}

#[test]
fn spider_verse_copies_once_each_turn() {
    cr!("603.2", "707.10");
    ruling!(
        "Spider-Verse",
        "Once you choose to copy a spell with Spider-Verse's last ability, that ability won't trigger again for the duration of the turn. Any instances of the ability already on the stack when you choose to copy a spell will have no effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spider-Verse");
    let a = t.graveyard(P0, "Think Twice");
    let b = t.graveyard(P0, "Think Twice");
    let c = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 9);
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    t.cast(P0, a).method(flashback.clone()).go();
    t.settle();
    // In response, a second one: two triggers on the stack.
    t.cast(P0, b).method(flashback.clone()).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "copy it"), 2);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    // The first trigger copied the second Think Twice; the other one did nothing: three
    // draws.
    assert_eq!(t.hand_size(P0), hand + 3);
    // A third one this turn: no trigger.
    t.cast(P0, c).method(flashback).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "copy it"), 0);
}

// ---------------------------------------------------------------------------
// Reki, Merry, legendary sorceries
// ---------------------------------------------------------------------------

#[test]
fn reki_draws_for_legendary_spells_of_any_type_but_not_lands() {
    cr!("205.4e", "305.1", "601.2i");
    ruling!("Reki, the History of Kamigawa", "Lands aren’t spells.");
    ruling!(
        "Reki, the History of Kamigawa",
        "Legendary spells are spells of any type that have the supertype legendary."
    );
    supported("Reki, the History of Kamigawa");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reki, the History of Kamigawa");
    let urborg = t.hand(P0, "Urborg, Tomb of Yawgmoth");
    t.play_land(P0, urborg).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Shadowspear", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Jaya's Immolating Inferno", &[pl(P1)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn merry_and_the_other_legend_can_attack_different_players() {
    cr!("508.1b", "802.1");
    ruling!(
        "Merry, Esquire of Rohan",
        "Merry and the other attacking legendary creature don't have to be attacking the same player, planeswalker, or battle."
    );
    supported("Merry, Esquire of Rohan");
    let mut t = TestGame::new(3);
    let merry = t.battlefield(P0, "Merry, Esquire of Rohan");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let hand = t.hand_size(P0);
    crate::r_s01_common::attack_with(&mut t, &[(merry, pl(P1)), (isamaru, pl(P2))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn legendary_sorceries_can_be_cast_any_number_of_times() {
    cr!("205.4e", "100.2a");
    ruling!(
        "Jaya's Immolating Inferno",
        "Other than the casting restriction, the legendary supertype on a sorcery carries no additional rules. You may cast any number of legendary sorceries in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    ruling!(
        "Kamahl's Druidic Vow",
        "Other than the casting restriction, the legendary supertype on a sorcery carries no additional rules. You may cast any number of legendary sorceries in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    ruling!(
        "Karn's Temporal Sundering",
        "Other than the casting restriction, the legendary supertype on a sorcery carries no additional rules. You may cast any number of legendary sorceries in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    ruling!(
        "Primevals' Glorious Rebirth",
        "Other than the casting restriction, the legendary supertype on a sorcery carries no additional rules. You may cast any number of legendary sorceries in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    ruling!(
        "Urza's Ruinous Blast",
        "Other than the casting restriction, the legendary supertype on a sorcery carries no additional rules. You may cast any number of legendary sorceries in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    ruling!(
        "Yawgmoth's Vile Offering",
        "Other than the casting restriction, the legendary supertype on a sorcery carries no additional rules. You may cast any number of legendary sorceries in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    let sorceries = [
        "Jaya's Immolating Inferno",
        "Kamahl's Druidic Vow",
        "Karn's Temporal Sundering",
        "Primevals' Glorious Rebirth",
        "Urza's Ruinous Blast",
        "Yawgmoth's Vile Offering",
    ];
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    for name in sorceries {
        supported(name);
        for _ in 0..2 {
            let targets: Vec<Entity> = match name {
                "Jaya's Immolating Inferno" | "Karn's Temporal Sundering" => vec![pl(P1)],
                _ => vec![],
            };
            give_mana_for(&mut t, P0, name);
            let c = t.hand(P0, name);
            for e in &targets {
                t.answer_targets(P0, &[*e]);
            }
            t.g.turn.priority = Some(P0);
            let mut b = t.cast(P0, c);
            if name == "Jaya's Immolating Inferno" || name == "Kamahl's Druidic Vow" {
                b = b.x(0);
            }
            assert!(b.try_go().is_ok(), "{name}");
            t.resolve_all();
        }
    }
    // A deck with four of each, and one with five of one.
    let mut deck: Vec<_> = sorceries
        .iter()
        .flat_map(|n| std::iter::repeat_n(card(n), 4))
        .collect();
    deck.extend(std::iter::repeat_n(card("Mountain"), 36));
    assert!(mtg_engine::deck::check_constructed(&deck).is_empty());
    deck.push(card("Urza's Ruinous Blast"));
    assert!(!mtg_engine::deck::check_constructed(&deck).is_empty());
}

// ---------------------------------------------------------------------------
// Ringsight
// ---------------------------------------------------------------------------

#[test]
fn ringsight_searches_by_the_colors_of_legendary_creatures() {
    cr!("701.54a", "105.2c");
    ruling!(
        "Ringsight",
        "Ringsight won't let you search for a colorless card, even if you control a colorless legendary creature."
    );
    ruling!(
        "Ringsight",
        "Your Ring-bearer is legendary, so you can search for a card that shares a color with the Ring-bearer that you chose as the Ring tempted you."
    );
    supported("Ringsight");
    // Only a colorless legendary creature: neither Sol Ring nor Divination can be found.
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, Silver Golem");
    let sol = t.library_top(P0, "Sol Ring");
    let div = t.library_top(P0, "Divination");
    t.answer_choose(P0, &[obj(karn)]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Ringsight", &[]);
    t.resolve_all();
    assert!(choices_since(&t, from)
        .iter()
        .all(|c| !c.contains(&obj(sol)) && !c.contains(&obj(div))));
    assert_eq!(t.zone(t.g.current(sol)), Zone::Library(P0));
    assert_eq!(t.zone(t.g.current(div)), Zone::Library(P0));
    // A blue nonlegendary creature chosen as Ring-bearer: a blue card can be found.
    let mut t = TestGame::new(2);
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let div = t.library_top(P0, "Divination");
    t.answer_choose(P0, &[obj(merfolk)]);
    t.answer_choose(P0, &[obj(div)]);
    cast_new(&mut t, P0, "Ringsight", &[]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(div)), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------
// Serah Farron
// ---------------------------------------------------------------------------

#[test]
fn serah_farron_checks_her_condition_twice() {
    cr!("603.4");
    ruling!(
        "Serah Farron // Crystallized Serah",
        "Serah Farron's last ability checks at the moment it would trigger to see if you control two or more other legendary creatures. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't control two or more other legendary creatures at that time, the ability won't resolve and none of its effects will happen."
    );
    supported("Serah Farron // Crystallized Serah");
    fn game(others: usize) -> (TestGame, ObjectId, Vec<ObjectId>) {
        let mut t = TestGame::new(2);
        let serah = t.battlefield(P0, "Serah Farron // Crystallized Serah");
        let legends = ["Isamaru, Hound of Konda", "Brimaz, King of Oreskos"][..others]
            .iter()
            .map(|n| t.battlefield(P0, n))
            .collect();
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        (t, serah, legends)
    }
    let transformed = |t: &TestGame, s: ObjectId| t.obj_now(t.g.current(s)).chars.name.as_str() == "Crystallized Serah";
    // One other legend: no trigger.
    let (t, _, _) = game(1);
    assert_eq!(t.stack_len(), 0);
    // Two: it triggers; one leaves before it resolves: nothing.
    let (mut t, serah, legends) = game(2);
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, legends[1], Zone::Hand(P0));
    t.resolve_all();
    assert!(!transformed(&t, serah));
    // Both still there: it transforms.
    let (mut t, serah, _) = game(2);
    t.resolve_all();
    assert!(transformed(&t, serah));
}

#[test]
fn serah_farron_reduces_only_generic_mana_of_the_total_cost() {
    cr!("601.2f", "202.3");
    ruling!(
        "Serah Farron // Crystallized Serah",
        "The cost reduction applies only to generic mana in the total cost of legendary creature spells you cast."
    );
    ruling!(
        "Serah Farron // Crystallized Serah",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Serah Farron or Crystallized Serah's first abilities). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Josu Vess, Lich Knight");
    // Isamaru ({W}) still costs {W}; Brimaz ({1}{W}{W}) costs {W}{W}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serah Farron // Crystallized Serah");
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    assert!(t.cast(P0, isamaru).try_go().is_err());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serah Farron // Crystallized Serah");
    t.lands(P0, "Plains", 2);
    let brimaz = t.hand(P0, "Brimaz, King of Oreskos");
    assert!(t.cast(P0, brimaz).try_go().is_ok());
    // Josu Vess kicked: {2}{B}{B} plus {5}{B}, less {2}: eight mana; mana value 4.
    for (wastes, ok) in [(4, false), (5, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Serah Farron // Crystallized Serah");
        t.lands(P0, "Swamp", 3);
        t.lands(P0, "Wastes", wastes);
        let josu = t.hand(P0, "Josu Vess, Lich Knight");
        let r = t.cast(P0, josu).kicked(true).try_go();
        assert_eq!(r.is_ok(), ok, "{wastes}");
        if let Ok(spell) = r {
            assert_eq!(t.g.mana_value_of(spell), 4);
            assert_eq!(t.g.obj(spell).stack.as_ref().unwrap().cast.times_kicked, 1);
        }
    }
}

// ---------------------------------------------------------------------------
// The Circle of Loyalty, Bag End Porter, Rakdos Joins Up, Jaya, Kethis, Time of Heroes
// ---------------------------------------------------------------------------

#[test]
fn circle_of_loyalty_cast_trigger() {
    cr!("603.2", "603.3", "601.2i");
    ruling!(
        "The Circle of Loyalty",
        "The Circle of Loyalty's triggered ability won't trigger when you cast it because it's not on the battlefield yet."
    );
    ruling!(
        "The Circle of Loyalty",
        "The Circle of Loyalty's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("The Circle of Loyalty");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "The Circle of Loyalty", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Knight").is_empty());
    // Isamaru, countered: the Knight is still created.
    let spell = cast_new(&mut t, P0, "Isamaru, Hound of Konda", &[]);
    t.settle();
    assert_eq!(t.g.stack[0], spell);
    assert_eq!(t.stack_len(), 2);
    cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.resolve();
    assert_eq!(t.zone(t.g.current(spell)), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Knight").len(), 1);
}

#[test]
fn bag_end_porter_fixes_x_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Bag End Porter",
        "The value of X is determined only as Bag End Porter's triggered ability resolves. It won't change later in the turn if the number of legendary creatures you control changes."
    );
    supported("Bag End Porter");
    let mut t = TestGame::new(2);
    let porter = t.battlefield(P0, "Bag End Porter");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    crate::r_s01_common::attack_with(&mut t, &[(porter, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(porter), (5, 5));
    t.battlefield(P0, "Brimaz, King of Oreskos");
    t.g.recompute();
    assert_eq!(t.pt(porter), (5, 5));
}

#[test]
fn rakdos_joins_up_uses_the_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Rakdos Joins Up",
        "Use the power of the legendary creature as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Rakdos Joins Up");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rakdos Joins Up");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.add_counters(obj(isamaru), "+1/+1", 2, None);
    t.answer_targets(P0, &[pl(P1)]);
    t.g.destroy(isamaru, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn jayas_immolating_inferno_cant_target_a_player_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Jaya's Immolating Inferno",
        "You can't target the same target more than once to have Jaya's Immolating Inferno deal more damage to it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Mountain", 4);
    let jaya = t.hand(P0, "Jaya's Immolating Inferno");
    let from = t.asked().len();
    t.answer_targets(P0, &[pl(P1), pl(P1)]);
    let r = t.cast(P0, jaya).x(2).try_go();
    t.resolve_all();
    // However the answer is treated, P1 is dealt 2 damage at most once.
    assert!(t.life(P1) >= 18, "{r:?}");
    let slots: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(slots, vec![3]);
}

#[test]
fn kethis_casts_from_the_graveyard_paying_all_costs() {
    cr!("601.2f", "118.9", "702.33a");
    ruling!(
        "Kethis, the Hidden Hand",
        "You’ll still pay all costs for a legendary spell you cast this way, including additional costs. You may also pay alternative costs."
    );
    /// Kethis's second ability has resolved with `card` in P0's graveyard.
    fn game(card: &str) -> (TestGame, ObjectId) {
        let mut t = TestGame::new(2);
        let kethis = t.battlefield(P0, "Kethis, the Hidden Hand");
        let c = t.graveyard(P0, card);
        let a = t.graveyard(P0, "Brimaz, King of Oreskos");
        let b = t.graveyard(P0, "Bosco, Just a Bear");
        t.answer_choose(P0, &[obj(a), obj(b)]);
        activate_containing(&mut t, P0, kethis, "Exile two").unwrap();
        t.resolve_all();
        (t, c)
    }
    // Josu Vess kicked from the graveyard: {2}{B}{B} + {5}{B}, less Kethis's {1}.
    for (wastes, ok) in [(5, false), (6, true)] {
        let (mut t, josu) = game("Josu Vess, Lich Knight");
        t.lands(P0, "Swamp", 3);
        t.lands(P0, "Wastes", wastes);
        let r = t.cast(P0, josu).kicked(true).try_go();
        assert_eq!(r.is_ok(), ok, "{wastes}");
    }
    // Ragavan's dash (an alternative cost) from the graveyard: {1}{R} less {1}.
    let (mut t, ragavan) = game("Ragavan, Nimble Pilferer");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, ragavan)
        .method(CastMethod::Keyword(KeywordKind::Dash))
        .go();
    t.resolve_all();
    let r = t.named_on_battlefield("Ragavan, Nimble Pilferer");
    assert_eq!(r.len(), 1);
}

#[test]
fn time_of_heroes_gives_one_bonus_whatever_the_level() {
    cr!("613.4c", "122.1");
    ruling!(
        "Time of Heroes",
        "Each creature you control with a level counter on it gets just +2/+2, regardless of how many level counters it has. It doesn’t get the bonus for each counter."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Time of Heroes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "level", 3, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(other), (2, 2));
}
