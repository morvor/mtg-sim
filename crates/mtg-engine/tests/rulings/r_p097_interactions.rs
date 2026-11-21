//! Rulings batch P097 — interactions among the "low mana value matters" cards: modes
//! performed in order (CR 700.2, 608.2c) with "until" exile returning mid-resolution
//! (CR 610.3) and regeneration shields used up (CR 701.19), leaves-the-battlefield
//! look-back (CR 603.10a), copies of spells and "mana spent to cast" (CR 707.10, 601.2h),
//! replacement effects entering the battlefield (CR 614.12), partial target legality
//! (CR 608.2b), emblems that animate Vehicles (CR 613.4b, 702.122), and the turns a player
//! has taken in a restarted game (CR 727).

use crate::r_p097_common::*;
use crate::r_s01_common::attack_with;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Austere Command choosing `modes`.
fn austere_command(t: &mut TestGame, modes: &[usize]) {
    pool(t, P0, &[(ManaType::W, 2), (ManaType::C, 4)]);
    let s = t.hand(P0, "Austere Command");
    t.cast(P0, s).modes(modes).go();
    t.resolve_all();
}

#[test]
fn austere_commands_modes_happen_in_order_for_dies_triggers() {
    cr!("700.2", "608.2c", "603.10a");
    ruling!(
        "Austere Command",
        "Each of the chosen modes happens sequentially. If a permanent has an ability that triggers whenever it or another permanent is destroyed, it will see permanents destroyed at the same time as it or before it, but not permanents destroyed by later modes."
    );
    supported("Austere Command");
    let mut t = TestGame::new(2);
    // "Whenever this creature or another creature dies, target player loses 1 life and
    // you gain 1 life."
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    austere_command(&mut t, &[2, 3]);
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    // Blood Artist and Grizzly Bears, not Craw Wurm.
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    assert_eq!(t.life(P0), 22);
}

#[test]
fn austere_command_destroys_a_card_returned_by_an_earlier_mode() {
    cr!("610.3a", "608.2c", "700.2");
    ruling!(
        "Austere Command",
        "If a card is exiled \"until\" another permanent leaves the battlefield, the exiled card returns to the battlefield immediately after that permanent leaves the battlefield during Austere Command's resolution, and it may be destroyed by a later mode."
    );
    ruling!(
        "Celebrate the Mountain-king",
        "The exiled cards return to the battlefield immediately after Celebrate the Mountain-king leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    for source in ["Grasp of Fate", "Celebrate the Mountain-king"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.enter(P0, source);
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile);
        // Destroy all enchantments, then all creatures with mana value 3 or less.
        austere_command(&mut t, &[1, 2]);
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "{source}");
        assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    }
}

#[test]
fn austere_command_uses_up_a_regeneration_shield_per_mode() {
    cr!("701.19a", "701.19c", "700.2");
    ruling!(
        "Austere Command",
        "If the first and last modes are chosen, an artifact creature with mana value 4 or greater will have to be regenerated twice to survive."
    );
    supported("Regenerate");
    for shields in [1, 2] {
        let mut t = TestGame::new(2);
        let j = t.battlefield(P0, "Juggernaut");
        for _ in 0..shields {
            pool(&mut t, P0, &[(ManaType::G, 1), (ManaType::C, 1)]);
            let r = t.hand(P0, "Regenerate");
            t.cast(P0, r).target(j).go();
            t.resolve_all();
        }
        austere_command(&mut t, &[0, 3]);
        assert_eq!(t.on_battlefield(j), shields == 2, "{shields}: {}", t.dump_log());
    }
}

#[test]
fn driver_of_the_dead_can_return_a_creature_that_died_with_it() {
    cr!("603.10a", "700.4");
    ruling!(
        "Driver of the Dead",
        "If Driver of the Dead dies at the same time as a creature with mana value 2 or less, you can target that card and return it to the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Driver of the Dead");
    let bears = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::W, 2), (ManaType::C, 2)]);
    let s = t.hand(P0, "Day of Judgment");
    t.cast(P0, s).go();
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(t.g.current(bears))]);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
}

#[test]
fn a_copy_of_driver_of_the_dead_can_return_itself() {
    cr!("707.2", "603.10a", "202.3");
    ruling!(
        "Driver of the Dead",
        "If a creature card with mana value 2 or less becomes a copy of Driver of the Dead and dies, it can be the target of its own ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Driver of the Dead");
    t.answer_yes(P0, true);
    let driver = t.named_on_battlefield("Driver of the Dead")[0];
    t.answer_choose(P0, &[Entity::Object(driver)]);
    let image = t.enter(P0, "Phantasmal Image");
    t.resolve_all();
    assert_eq!(t.obj_now(image).chars.name.as_str(), "Driver of the Dead");
    t.g.destroy(image, None);
    let in_gy = t.g.current(image);
    assert_eq!(t.zone(in_gy), Zone::Graveyard(P0));
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(in_gy)]);
    // It returns, entering as a copy of Driver of the Dead again.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(driver)]);
    t.resolve_all();
    let candidates = target_candidates(&t, from);
    assert!(
        candidates.iter().any(|c| c.contains(&Entity::Object(in_gy))),
        "{candidates:?}"
    );
    assert!(t.on_battlefield(in_gy), "{}", t.dump_log());
    assert_eq!(t.obj_now(in_gy).chars.name.as_str(), "Driver of the Dead");
}

/// The candidates of Astelli Reclaimer's enters trigger(s) asked since `from`.
fn reclaimer_candidates(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                text, candidates, ..
            } if text.contains("noncreature, nonland permanent card") => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect()
}

/// A game with Mox Amber (mana value 0), Sol Ring (1), Coat of Arms (5) and Mindslaver
/// (6) in P0's graveyard.
fn reclaimer_game() -> (TestGame, [ObjectId; 4]) {
    supported("Astelli Reclaimer");
    let mut t = TestGame::new(2);
    let ids = [
        t.graveyard(P0, "Mox Amber"),
        t.graveyard(P0, "Sol Ring"),
        t.graveyard(P0, "Coat of Arms"),
        t.graveyard(P0, "Mindslaver"),
    ];
    (t, ids)
}

#[test]
fn a_copy_of_astelli_reclaimer_had_no_mana_spent_on_it() {
    cr!("707.10", "601.2h", "107.3m");
    ruling!(
        "Astelli Reclaimer",
        "If a copy of Astelli Reclaimer is created on the stack, no mana was spent to cast the copy, so X will be 0."
    );
    supported("Double Major");
    let (mut t, [mox, ring, ..]) = reclaimer_game();
    pool(&mut t, P0, &[(ManaType::W, 2), (ManaType::C, 3)]);
    let r = t.hand(P0, "Astelli Reclaimer");
    let spell = t.cast(P0, r).go();
    pool(&mut t, P0, &[(ManaType::G, 1), (ManaType::U, 1)]);
    let dm = t.hand(P0, "Double Major");
    t.cast(P0, dm).target(spell).go();
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(mox)]);
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.resolve_all();
    let c = reclaimer_candidates(&t, from);
    assert_eq!(c.len(), 2, "{}", t.dump_log());
    // The token's trigger: mana value 0 or less.
    assert_eq!(c[0], vec![Entity::Object(mox)]);
    // The card's: 5 or less.
    assert!(c[1].contains(&Entity::Object(ring)));
    assert!(t.on_battlefield(mox) && t.on_battlefield(ring));
}

#[test]
fn astelli_reclaimer_cast_without_paying_its_mana_cost_has_x_zero() {
    cr!("601.2h", "118.9");
    ruling!(
        "Astelli Reclaimer",
        "If an effect allows you to cast Astelli Reclaimer without paying its mana cost, X will be 0."
    );
    let (mut t, [mox, ..]) = reclaimer_game();
    t.battlefield(P0, "Omniscience");
    let r = t.hand(P0, "Astelli Reclaimer");
    t.cast(P0, r).method(CastMethod::Free).go();
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(reclaimer_candidates(&t, from), vec![vec![Entity::Object(mox)]]);
}

#[test]
fn mana_paid_for_a_counter_unless_isnt_spent_to_cast_astelli() {
    cr!("601.2h", "118.12");
    ruling!(
        "Astelli Reclaimer",
        "If an effect requires you to pay some amount of mana to prevent Astelli Reclaimer from being countered"
    );
    let (mut t, [_, _, coat, slaver]) = reclaimer_game();
    // Five mana for Astelli Reclaimer, three more for Mana Leak.
    pool(&mut t, P0, &[(ManaType::W, 2), (ManaType::C, 3)]);
    let r = t.hand(P0, "Astelli Reclaimer");
    let spell = t.cast(P0, r).go();
    pool(&mut t, P0, &[(ManaType::C, 3)]);
    pool(&mut t, P1, &[(ManaType::U, 1), (ManaType::C, 1)]);
    let leak = t.hand(P1, "Mana Leak");
    t.cast(P1, leak).target(spell).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Stack, "{}", t.dump_log());
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(coat)]);
    t.resolve_all();
    let c = reclaimer_candidates(&t, from);
    assert!(c[0].contains(&Entity::Object(coat)));
    assert!(!c[0].contains(&Entity::Object(slaver)));
}

#[test]
fn recommission_counter_for_an_artifact_that_will_be_a_creature() {
    cr!("614.12", "614.1c");
    ruling!(
        "Recommission",
        "If a noncreature artifact is returned to the battlefield this way and another effect would cause that artifact to be a creature while it is on the battlefield"
    );
    supported("Recommission");
    for march in [false, true] {
        let mut t = TestGame::new(2);
        if march {
            t.battlefield(P0, "March of the Machines");
        }
        let ring = t.graveyard(P0, "Sol Ring");
        pool(&mut t, P0, &[(ManaType::W, 1), (ManaType::C, 1)]);
        let s = t.hand(P0, "Recommission");
        t.cast(P0, s).target(ring).go();
        t.resolve_all();
        assert!(t.on_battlefield(ring));
        assert_eq!(t.counters(ring, "+1/+1"), march as u32, "{}", t.dump_log());
        if march {
            assert_eq!(t.pt(ring), (2, 2));
        }
    }
}

#[test]
fn commands_resolve_if_any_target_is_still_legal() {
    cr!("608.2b", "700.2");
    ruling!(
        "Ojutai's Command",
        "If all targets for the chosen modes become illegal before the Command resolves, the spell won’t resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    ruling!(
        "Silumgar's Command",
        "If all targets for the chosen modes become illegal before the Command resolves, the spell won’t resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    // Ojutai's Command: return a creature card + gain 4 life; the card is gone.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::W, 1), (ManaType::U, 1), (ManaType::C, 2)]);
    let s = t.hand(P0, "Ojutai's Command");
    t.cast(P0, s).modes(&[0, 1]).target(bears).go();
    t.g.move_object(bears, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Silumgar's Command: bounce Hill Giant + Grizzly Bears gets -3/-3; the Giant is gone.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::B, 1), (ManaType::C, 3)]);
    let s = t.hand(P0, "Silumgar's Command");
    t.cast(P0, s).modes(&[1, 2]).target(giant).target(bears).go();
    t.g.move_object(giant, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Both gone: nothing.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::B, 1), (ManaType::C, 3)]);
    let s = t.hand(P0, "Silumgar's Command");
    t.cast(P0, s).modes(&[1, 2]).target(giant).target(bears).go();
    t.g.move_object(giant, Zone::Exile, events::MoveCause::Effect, None);
    t.g.move_object(bears, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.on_battlefield(wall));
    assert!(t.in_graveyard(P0, "Silumgar's Command"));
}

#[test]
fn daring_waverider_doesnt_exile_a_spell_that_went_to_hand() {
    cr!("614.1a", "702.27a", "400.7");
    ruling!(
        "Daring Waverider",
        "If an instant or sorcery card you cast this way goes to a zone other than exile or a graveyard"
    );
    supported("Daring Waverider");
    let mut t = TestGame::new(2);
    let capsize = t.graveyard(P0, "Capsize");
    let bears = t.battlefield(P1, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::C, 3)]);
    t.answer_targets(P0, &[Entity::Object(capsize)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Daring Waverider");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P1));
    // Buyback: back to P0's hand.
    assert_eq!(t.zone(capsize), Zone::Hand(P0), "{}", t.dump_log());
    // Discarded later this turn, it goes to the graveyard.
    let c = t.g.current(capsize);
    t.g.discard(P0, c, None);
    t.settle();
    assert_eq!(t.zone(capsize), Zone::Graveyard(P0));
}

#[test]
fn kellan_joins_up_triggers_for_each_legendary_creature_entering() {
    cr!("603.2c", "603.6a");
    ruling!(
        "Kellan Joins Up",
        "If multiple legendary creatures enter the battlefield under your control at the same time, Kellan Joins Up's last ability will trigger once for each of those creatures."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kellan Joins Up");
    let bears = t.battlefield(P0, "Grizzly Bears");
    enter_together(
        &mut t,
        &[("Isamaru, Hound of Konda", P0), ("Seifer Almasy", P0)],
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn tezzerets_emblem_animates_vehicles_as_0_0_without_crewing_them() {
    cr!("613.4b", "702.122a", "613.1d");
    ruling!(
        "Tezzeret, Cruel Captain",
        "If the emblem’s ability causes a Vehicle to become an artifact creature, it doesn’t count as “crewing” that Vehicle"
    );
    ruling!(
        "Tezzeret, Cruel Captain",
        "If the emblem’s ability causes a Vehicle to become an artifact creature, its base power and toughness will be set to 0/0. Crewing that Vehicle will not restore its power and toughness."
    );
    supported("Tezzeret, Cruel Captain");
    supported("Mobilizer Mech");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Cruel Captain");
    t.g.objects[tez.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    activate_containing(&mut t, P0, tez, "emblem").expect("−7");
    t.resolve_all();
    // "Whenever this Vehicle becomes crewed, up to one other target Vehicle you control
    // becomes an artifact creature until end of turn."
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mech)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(mech).is(CardType::Creature));
    assert!(t.obj_now(mech).chars.has_subtype("Robot"));
    assert_eq!(t.pt(mech), (3, 3), "{}", t.dump_log());
    // Not crewed: its trigger didn't trigger.
    assert!(!t.dump_log().contains("Resolving Mobilizer Mech"));
    // Crewed now: still 0/0 base.
    assert!(crate::r_s04_common::crew(&mut t, P0, mech, &[giant]));
    t.resolve_all();
    assert!(t.dump_log().contains("Resolving Mobilizer Mech"));
    assert_eq!(t.pt(mech), (3, 3));
}

// Scryfall's data attaches Starting Town's rulings to Squall, SeeD Mercenary (misattached
// rulings, logged in docs/RULING_DISCREPANCIES.md): they are cited under Squall and tested
// on Starting Town. Squall takes part where it can (returning a Starting Town on its
// controller's turn, `starting_town_counts_the_turns_you_have_taken` in r_p098_misc.rs);
// it can't make one enter on another player's turn, and the restart test has no use for it.
#[test]
fn starting_town_enters_tapped_on_other_players_turns() {
    cr!("614.1c", "500.7");
    ruling!(
        "Squall, SeeD Mercenary",
        "If Starting Town enters during another player's turn, it'll enter tapped no matter how many turns they've taken."
    );
    supported("Starting Town");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Starting Town");
    assert!(!tapped(&t, a));
    t.set_step(P1, Step::PrecombatMain);
    let b = t.enter(P0, "Starting Town");
    assert!(tapped(&t, b));
}

#[test]
fn starting_town_counts_the_turns_of_a_restarted_game() {
    cr!("727.1", "614.1c");
    ruling!(
        "Squall, SeeD Mercenary",
        "If an effect restarts the game, Starting Town will enter tapped in the new game unless it's your first, second, or third turn of that game."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let next_main = |t: &mut TestGame| {
        t.advance_to(P0, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
    };
    for _ in 0..4 {
        next_main(&mut t);
    }
    assert_eq!(t.g.player(P0).turns_taken, 5);
    let town = t.hand(P0, "Starting Town");
    t.play_land(P0, town).unwrap();
    assert!(tapped(&t, town));
    let k = t.battlefield(P0, "Karn Liberated");
    t.g.objects[k.0 as usize]
        .counters
        .insert("loyalty".into(), 30);
    t.activate(P0, k, 2, &[]).unwrap();
    t.g.resolve_top();
    next_main(&mut t);
    assert_eq!(t.g.player(P0).turns_taken, 1);
    let town = t.hand(P0, "Starting Town");
    t.play_land(P0, town).unwrap();
    assert!(!tapped(&t, town));
}

#[test]
fn an_aura_put_onto_the_battlefield_cant_enchant_a_permanent_entering_with_it() {
    cr!("303.4g", "303.4f");
    ruling!(
        "Genesis Ultimatum",
        "An Aura being put onto the battlefield this way can't enchant anything else that is being put onto the battlefield at the same time."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P0,
        &["Pacifism", "Grizzly Bears", "Forest", "Island", "Swamp"],
    );
    let giant = t.battlefield(P1, "Hill Giant");
    pool(
        &mut t,
        P0,
        &[(ManaType::G, 2), (ManaType::U, 3), (ManaType::R, 2)],
    );
    let s = t.hand(P0, "Genesis Ultimatum");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(cards[0]), Entity::Object(cards[1])]);
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(t.on_battlefield(cards[1]));
    assert!(t.on_battlefield(cards[0]), "{}", t.dump_log());
    assert_eq!(t.obj_now(cards[0]).attached_to, Some(Entity::Object(giant)));
    let bears = [cards[1], t.g.current(cards[1])];
    let mut asked = false;
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseEntities { candidates, .. } = d {
            if candidates.contains(&Entity::Object(giant)) {
                asked = true;
                assert!(bears.iter().all(|b| !candidates.contains(&Entity::Object(*b))));
            }
        }
    }
    assert!(asked, "the Aura's host was chosen");
}

#[test]
fn historic_cast_triggers_need_a_cast() {
    cr!("700.6", "603.2", "601.2i");
    ruling!(
        "Elena, Turk Recruit",
        "An ability that triggers \"whenever you cast a historic spell\" doesn't trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Glóin, Dwarf Emissary",
        "An ability that triggers \"whenever you cast a historic spell\" doesn't trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Lingering Phantom",
        "An ability that triggers \"whenever you cast a historic spell\" doesn't trigger if a historic card is put onto the battlefield without being cast."
    );
    for case in ["Elena, Turk Recruit", "Glóin, Dwarf Emissary", "Lingering Phantom"] {
        supported(case);
        let mut t = TestGame::new(2);
        if case == "Lingering Phantom" {
            t.graveyard(P0, case);
        } else {
            t.battlefield(P0, case);
        }
        t.enter(P0, "Isamaru, Hound of Konda");
        t.settle();
        assert_eq!(t.stack_len(), 0, "{case}");
        pool(&mut t, P0, &[(ManaType::C, 1)]);
        let a = t.hand(P0, "Mox Amber");
        t.cast(P0, a).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{case}");
    }
}

#[test]
fn barrowin_works_with_a_dungeon_completed_before_it_was_there() {
    cr!("309.7", "603.4");
    ruling!(
        "Barrowin of Clan Undurr",
        "Barrowin of Clan Undurr's last ability works even if it wasn't on the battlefield when you completed a dungeon."
    );
    for completed in [false, true] {
        let mut t = TestGame::new(2);
        if completed {
            t.g.players[0].dungeons_completed = 1;
            t.g.players[0].completed_dungeons.push("Tomb of Annihilation".into());
        }
        let b = t.battlefield(P0, "Barrowin of Clan Undurr");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_choose(P0, &[Entity::Object(bears)]);
        attack_with(&mut t, &[(b, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), completed, "{}", t.dump_log());
    }
}

#[test]
fn a_permanent_card_is_an_artifact_creature_enchantment_land_or_planeswalker() {
    cr!("110.4", "115.1");
    ruling!(
        "Nissa, Vital Force",
        "A permanent card is an artifact, creature, enchantment, land, or planeswalker card."
    );
    supported("Nissa, Vital Force");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nissa, Vital Force");
    let permanents: Vec<Entity> = [
        "Sol Ring",
        "Grizzly Bears",
        "Pacifism",
        "Forest",
        "Chandra, Torch of Defiance",
    ]
    .iter()
    .map(|c| Entity::Object(t.graveyard(P0, c)))
    .collect();
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let div = t.graveyard(P0, "Divination");
    let from = t.asked().len();
    activate_containing(&mut t, P0, n, "Return target permanent card").expect("−3");
    let c = target_candidates(&t, from);
    let mut got = c[0].clone();
    got.sort_by_key(|e| format!("{e:?}"));
    let mut want = permanents.clone();
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(got, want);
    assert!(!c[0].contains(&Entity::Object(bolt)) && !c[0].contains(&Entity::Object(div)));
}
