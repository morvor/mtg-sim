//! Item `rulings-residue-keywords` — combat, costs, replacement and layered effects:
//! exert and additional combat phases (CR 500.8, 701.43), attaching (301.5c), several
//! replacement effects (616.1), conditions checked as a spell is cast (601.3, 702.8),
//! simultaneous entering (603.6a, 702.82c), first strike and creatures entering attacking
//! (510.4), mana spent (601.2h), blocking restrictions (509.1b), control changes and
//! combat (506.4), damage modification order (616.1, 615), divided damage (601.2d),
//! timestamps of player effects (613.7), sacrificing (701.21), delirium counting
//! (608.2c), and demonstrate's copies (702.144).

use crate::r_residue_keywords_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The object (followed across zone changes) is modified until end of turn, as an
/// effect of P1's.
fn modify(t: &mut TestGame, id: ObjectId, m: Modification) {
    run_from(
        t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![m],
            duration: Duration::EndOfTurn,
        },
        &[obj(t.g.current(id))],
    );
}

#[test]
fn surge_engine_abilities_check_defender_and_blue_however_they_changed() {
    cr!("602.5b", "702.3b", "105.3");
    ruling!(
        "Surge Engine",
        "If Surge Engine loses defender in some way other than its first ability, its second ability may still be activated. Likewise, if it becomes blue in some way other than its second ability, its last ability may still be activated. Note that an effect that allows a creature to \"attack as though it didn't have defender\" doesn't actually cause it to lose defender."
    );
    supported("Surge Engine");
    let abil = |t: &TestGame, e: ObjectId, needle: &str| -> u64 {
        t.obj_now(e)
            .chars
            .abilities
            .iter()
            .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
            .unwrap()
            .uid
    };
    let activatable = |t: &mut TestGame, e: ObjectId, needle: &str| -> bool {
        t.g.turn.priority = Some(P0);
        t.g.recompute();
        let uid = abil(t, e, needle);
        t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, mtg_engine::decision::Action::Activate { source, ability }
                if *source == e && *ability == uid)
        })
    };
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Surge Engine");
    t.lands(P0, "Island", 8);
    // With defender, not blue: neither.
    assert!(!activatable(&mut t, e, "becomes blue"));
    assert!(!activatable(&mut t, e, "Draw three"));
    // "Attack as though it didn't have defender" (Wakestone Gargoyle): still has it.
    let g = t.battlefield(P0, "Wakestone Gargoyle");
    t.lands(P0, "Plains", 1);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has(&mut t, e, KeywordKind::Defender));
    assert!(!activatable(&mut t, e, "becomes blue"));
    // Another effect removes defender: its second ability can be activated.
    modify(
        &mut t,
        e,
        Modification::RemoveKeyword(KeywordKind::Defender),
    );
    assert!(activatable(&mut t, e, "becomes blue"));
    // Another effect makes it blue: its last ability can be activated.
    modify(
        &mut t,
        e,
        Modification::SetColors(ColorSet::single(Color::Blue)),
    );
    assert!(activatable(&mut t, e, "Draw three"));
}

#[test]
fn rohirrim_chargers_equipment_enters_unattached_if_the_creature_left() {
    cr!("301.5e", "702.16d");
    ruling!(
        "Rohirrim Chargers",
        "If the exerted creature is no longer on the battlefield or the Equipment can't legally be attached to that creature, that Equipment will enter the battlefield unattached."
    );
    supported("Rohirrim Chargers");
    // The exerted Chargers stays (the Sword is attached to it), leaves the battlefield,
    // or gains protection from artifacts (the Sword can't legally be attached to it).
    for case in ["stays", "left", "protection"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Rohirrim Chargers");
        stack_library(&mut t, P0, &["Forest", "Short Sword"]);
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        match case {
            "left" => {
                move_to(&mut t, c, Zone::Hand(P0));
            }
            "protection" => modify(
                &mut t,
                c,
                Modification::AddKeyword(Keyword::with_filter(
                    KeywordKind::Protection,
                    Filter::Type(CardType::Artifact),
                )),
            ),
            _ => {}
        }
        t.resolve_all();
        let sword = t.named_on_battlefield("Short Sword");
        assert_eq!(sword.len(), 1, "{case}: {}", t.dump_log());
        let attached = t.obj_now(sword[0]).attached_to;
        if case == "stays" {
            assert_eq!(attached, Some(obj(c)), "{case}");
        } else {
            assert!(attached.is_none(), "{case}: {attached:?}");
            // It entered unattached: it wasn't attached first and then unattached (by the
            // state-based action of CR 702.16d).
            assert!(
                !t.g.turn_events.iter().any(|e| matches!(
                    e,
                    mtg_engine::events::Event::Attached { obj, .. }
                        | mtg_engine::events::Event::Unattached { obj, .. }
                        if t.g.current(*obj) == sword[0]
                )),
                "{case}: {}",
                t.dump_log()
            );
        }
    }
}

#[test]
fn combat_celebrant_gives_a_combat_even_if_it_dies() {
    cr!("500.8", "701.43a", "603.12");
    ruling!(
        "Combat Celebrant",
        "If you exert Combat Celebrant, you get an additional combat phase even if Combat Celebrant doesn't survive the first combat phase."
    );
    supported("Combat Celebrant");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Combat Celebrant");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[(giant, c)]);
    assert!(!t.on_battlefield(c));
    let turn = t.g.turn.number;
    t.advance_to_step(Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, turn);
    assert_eq!(t.g.turn.combat_phases, 2);
}

#[test]
fn several_combat_celebrants_give_several_combats_but_untap_only_once() {
    cr!("500.8", "701.43a");
    ruling!(
        "Combat Celebrant",
        "If you exert multiple Combat Celebrants in one combat phase, you'll have that many additional combat phases, but all of your creatures are untapped only during the current combat phase. You'll need to exert the Combat Celebrants one at a time, in multiple combat phases, to untap your attacking creatures and attack with them in each combat step."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Combat Celebrant");
    let b = t.battlefield(P0, "Combat Celebrant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    // Untapped now, during this combat.
    assert!(!tapped(&t, bears));
    block_and_finish(&mut t, P1, &[]);
    // The second combat: the Bears attack again (and stay tapped).
    t.advance_to_step(Step::BeginningOfCombat);
    assert_eq!(t.g.turn.combat_phases, 2);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    // A third combat, with nothing untapping the Bears.
    t.advance_to_step(Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, 1);
    assert_eq!(t.g.turn.combat_phases, 3);
    t.resolve_all();
    assert!(tapped(&t, bears));
}

#[test]
fn two_twists_and_turns_scry_twice_before_exploring() {
    cr!("616.1", "614.6", "701.44a");
    ruling!(
        "Twists and Turns // Mycoid Maze",
        "If you control two copies of Twists and Turns and a creature you control would explore, instead you scry 1, then scry 1 again, then that creature explores. If you control three copies of Twists and Turns, you'd scry 1 an additional time before that creature explores, and so on."
    );
    supported("Twists and Turns // Mycoid Maze");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twists and Turns // Mycoid Maze");
    t.battlefield(P0, "Twists and Turns // Mycoid Maze");
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.enter(P0, "Merfolk Branchwalker");
    t.resolve_all();
    let scries = count_asked(&t, from, |d| matches!(d, Decision::Scry { .. }));
    assert_eq!(scries, 2);
    // Then it explored: a land revealed and put into the hand.
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn dragon_grip_ferocious_is_checked_only_as_it_is_cast() {
    cr!("601.3", "601.2a", "702.8a");
    ruling!(
        "Dragon Grip",
        "If you're using Dragon Grip's ferocious ability to cast it as though it had flash, you must control a creature with power 4 or greater only as you begin the casting process and put Dragon Grip on the stack. Losing control of that creature while casting Dragon Grip (perhaps because you sacrificed it to activate a mana ability), or in response to Dragon Grip won't affect Dragon Grip on the stack."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    lands_for_cost(&mut t, P0, "Dragon Grip");
    let g = t.hand(P0, "Dragon Grip");
    let spell = t.cast(P0, g).target(bears).go();
    // The 4-power creature leaves in response; Dragon Grip stays and resolves.
    move_to(&mut t, wurm, Zone::Hand(P0));
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(pt_now(&mut t, bears), (4, 2));
    // The 4-power creature is sacrificed for mana while Dragon Grip is cast: still cast.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let brute = t.custom(
        P0,
        custom_card(
            "Mana Brute",
            "Creature — Beast",
            "{3}",
            Some((4, 4)),
            "Sacrifice this creature: Add {R}.",
        ),
        Zone::Battlefield,
    );
    t.lands(P0, "Mountain", 2);
    let g = t.hand(P0, "Dragon Grip");
    let spell = t.cast(P0, g).target(bears).go();
    assert!(!t.on_battlefield(brute), "{}", t.dump_log());
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(pt_now(&mut t, bears), (4, 2));
    // Without one as it's cast: no flash.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Dragon Grip");
    let g = t.hand(P0, "Dragon Grip");
    assert!(t.cast(P0, g).target(bears).try_go().is_err());
}

#[test]
fn see_the_unwritten_checks_ferocious_first_and_creatures_enter_together() {
    cr!("608.2c", "603.6a", "207.2c");
    ruling!(
        "See the Unwritten",
        "You determine whether the ferocious ability applies before any creature cards are put onto the battlefield. If you put two creature cards onto the battlefield this way, those creatures enter the battlefield at the same time."
    );
    // No 4-power creature beforehand: one creature, even though the Dreadmaw would be one.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Colossal Dreadmaw", "Grizzly Bears"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "See the Unwritten", &[]);
    t.resolve_all();
    let maxes: Vec<usize> = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max as usize),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![1], "{}", t.dump_log());
    assert_eq!(creatures(&t, P0).len(), 1);
    // With one: the Bears and Soul Warden enter together; the Warden sees the Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Soul Warden"]);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    cast_new(&mut t, P0, "See the Unwritten", &[]);
    t.resolve_all();
    assert_eq!(creatures(&t, P0).len(), 3);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn ravenous_dinosaurs_cant_devour_creatures_entering_with_them() {
    cr!("702.82a", "702.82c", "614.12");
    ruling!(
        "Ravenous Gigantotherium",
        "Ravenous Gigantotherium can't devour creatures entering the battlefield at the same time as it."
    );
    ruling!(
        "Ravenous Tyrannosaurus",
        "Ravenous Tyrannosaurus can't devour creatures entering the battlefield at the same time as it."
    );
    for name in ["Ravenous Gigantotherium", "Ravenous Tyrannosaurus"] {
        supported(name);
        let mut t = TestGame::new(2);
        let old = t.battlefield(P0, "Hill Giant");
        let from = t.asked().len();
        let ids = enter_together(&mut t, &[(P0, name), (P0, "Grizzly Bears")]);
        let offered: Vec<Vec<Entity>> = asked_since(&t, from)
            .into_iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseEntities { candidates, .. } => Some(candidates),
                _ => None,
            })
            .collect();
        assert!(!offered.is_empty(), "{name}");
        for c in &offered {
            assert!(c.contains(&obj(old)), "{name}: {c:?}");
            assert!(ids.iter().all(|i| !c.contains(&obj(*i))), "{name}");
        }
    }
}

#[test]
fn shark_shredder_returned_first_striker_deals_regular_damage_once() {
    cr!("510.4", "702.7c", "702.4c", "508.4");
    ruling!(
        "Shark Shredder, Killer Clone",
        "If Shark Shredder deals combat damage to a player during the first strike combat damage step and its last ability returns a creature with first strike, the returned creature will still deal combat damage during the regular combat damage step. If the returned creature has double strike, it will deal combat damage only once during that regular combat damage step."
    );
    supported("Shark Shredder, Killer Clone");
    for (name, dmg) in [("Youthful Knight", 2), ("Fencing Ace", 1)] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Shark Shredder, Killer Clone");
        let card = t.graveyard(P1, name);
        t.answer_targets(P0, &[obj(card)]);
        attack_with(&mut t, &[(s, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
        assert_eq!(t.life(P1), 20 - 4 - dmg, "{name}");
    }
}

#[test]
fn wurmquake_counts_the_mana_actually_spent() {
    cr!("601.2f", "601.2h", "702.34a");
    ruling!(
        "Wurmquake",
        "You can't choose to spend more mana to cast Wurmquake than you're asked to just to make a bigger Wurm. In most cases, it's a 6/6 Phyrexian Wurm the first time, and a 10/10 Phyrexian Wurm when you cast Wurmquake with flashback. However, if there are any additional costs to cast it, the mana spent to pay those costs counts."
    );
    supported("Wurmquake");
    let wurm = |t: &TestGame| {
        let w = with_subtype(t, P0, "Wurm");
        assert_eq!(w.len(), 1);
        t.pt(w[0])
    };
    // Normally, with extra lands around: 6/6.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 12);
    let w = t.hand(P0, "Wurmquake");
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(wurm(&t), (6, 6));
    // With flashback: 10/10.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 12);
    let w = t.graveyard(P0, "Wurmquake");
    t.cast(P0, w)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert_eq!(wurm(&t), (10, 10));
    // A cost increase (Thalia: "Noncreature spells cost {1} more"): 7/7.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Forest", 12);
    let w = t.hand(P0, "Wurmquake");
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(wurm(&t), (7, 7));
}

#[test]
fn cloud_spirit_without_flying_can_block_nothing() {
    cr!("509.1b", "702.9b");
    ruling!(
        "Cloud Spirit",
        "If it loses Flying, it will be unable to block any creatures."
    );
    supported("Cloud Spirit");
    let mut t = TestGame::new(2);
    let cs = t.battlefield(P1, "Cloud Spirit");
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify(&mut t, cs, Modification::RemoveKeyword(KeywordKind::Flying));
    attack_with(
        &mut t,
        &[(drake, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    let opts = mtg_engine::combat::block_options(&t.g, &[P1]);
    assert!(
        opts.iter().all(|(b, a)| *b != cs || a.is_empty()),
        "{opts:?}"
    );
    // With flying, it could block the Drake.
    let mut t = TestGame::new(2);
    let cs = t.battlefield(P1, "Cloud Spirit");
    let drake = t.battlefield(P0, "Wind Drake");
    attack_with(&mut t, &[(drake, Entity::Player(P1))]);
    let opts = mtg_engine::combat::block_options(&t.g, &[P1]);
    assert!(opts.iter().any(|(b, a)| *b == cs && a.contains(&drake)));
}

#[test]
fn inniaz_gives_away_an_attacker_and_resolves_whatever_the_attackers_became() {
    cr!("506.4", "603.2");
    ruling!(
        "Inniaz, the Gale Force",
        "If you give away a creature you control with flying that attacked, it’s removed from combat. It will still be tapped if it was tapped, so it won’t be able to block."
    );
    ruling!(
        "Inniaz, the Gale Force",
        "Inniaz’s ability resolves even if some or all of the attacking creatures lose flying or leave combat before it resolves."
    );
    supported("Inniaz, the Gale Force");
    let mut t = TestGame::new(2);
    let inniaz = t.battlefield(P0, "Inniaz, the Gale Force");
    let d1 = t.battlefield(P0, "Wind Drake");
    let d2 = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (inniaz, Entity::Player(P1)),
            (d1, Entity::Player(P1)),
            (d2, Entity::Player(P1)),
        ],
    );
    assert_eq!(t.stack_len(), 1);
    // In response one Drake loses flying and the other leaves combat (to its owner's hand).
    modify(&mut t, d1, Modification::RemoveKeyword(KeywordKind::Flying));
    move_to(&mut t, d2, Zone::Hand(P0));
    // P0 takes P1's Bears and gives P1 the attacking Drake.
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(d1)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0, "{}", t.dump_log());
    assert_eq!(controller(&mut t, d1), P1);
    assert!(!is_attacking(&t, d1));
    assert!(tapped(&t, d1));
}

/// Four card types in P0's graveyard (delirium), The Rollercrusher Ride on the battlefield.
fn rollercrusher(t: &mut TestGame) {
    supported("The Rollercrusher Ride");
    delirium_graveyard(t, P0);
    t.battlefield(P0, "The Rollercrusher Ride");
}

#[test]
fn rollercrusher_ride_doubles_divided_damage_after_dividing() {
    cr!("601.2d", "614.1a", "120.3");
    ruling!(
        "The Rollercrusher Ride",
        "If damage dealt by a source you control is being divided among multiple permanents and/or players while you have delirium, that damage is divided before doubling."
    );
    let mut t = TestGame::new(2);
    rollercrusher(&mut t);
    let a = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    cast_new(&mut t, P0, "Forked Bolt", &[]);
    t.resolve_all();
    assert_eq!(damage_on(&t, a), 2, "{}", t.dump_log());
    assert_eq!(t.life(P1), 18);
}

#[test]
fn rollercrusher_ride_and_prevention_in_the_damaged_players_order() {
    cr!("616.1", "616.1e", "615.1a");
    ruling!(
        "The Rollercrusher Ride",
        "If another effect (or effects) modifies how much damage a source you control would deal while you have delirium—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects apply. If all of the damage is prevented, the effect of The Rollercrusher Ride's first ability no longer applies."
    );
    let mut outcomes = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        rollercrusher(&mut t);
        // P1 prevents the next 3 damage to themself (Healing Salve).
        t.set_step(P1, Step::PrecombatMain);
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_new(&mut t, P1, "Healing Salve", &[Entity::Player(P1)]);
        t.resolve_all();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        let from = t.asked().len();
        cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
        t.resolve_all();
        // P1, the player being dealt damage, chose the order.
        assert!(asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseReplacement { .. })));
        assert!(!asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. })));
        outcomes.push(t.life(P1));
    }
    outcomes.sort();
    // Doubled then 3 prevented: 3 damage. Prevented first: all of it; nothing to double.
    assert_eq!(outcomes, vec![17, 20]);
}

#[test]
fn winter_and_reliquary_tower_apply_in_timestamp_order() {
    cr!("613.7", "613.7a", "402.2");
    ruling!(
        "Winter, Misanthropic Guide",
        "If multiple effects modify a player's hand size, apply them in timestamp order. For example, if you put Winter onto the battlefield and then your opponent plays Reliquary Tower (a land with \"You have no maximum hand size\"), that opponent would have no maximum hand size no matter when you started having delirium. However, if the permanents entered in the opposite order, that opponent's maximum hand size would be seven minus the number of card types among cards in your graveyard if you had delirium. At any time you didn't have delirium, they would have no maximum hand size."
    );
    supported("Winter, Misanthropic Guide");
    let max = |t: &mut TestGame| {
        t.g.recompute();
        t.g.player(P1).max_hand_size
    };
    // Winter, then Reliquary Tower: no maximum, before and after delirium.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Winter, Misanthropic Guide");
    t.battlefield(P1, "Reliquary Tower");
    assert_eq!(max(&mut t), None);
    delirium_graveyard(&mut t, P0);
    assert_eq!(max(&mut t), None);
    // Reliquary Tower, then Winter: no maximum until delirium, then 7 - 4.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Reliquary Tower");
    t.battlefield(P0, "Winter, Misanthropic Guide");
    assert_eq!(max(&mut t), None);
    delirium_graveyard(&mut t, P0);
    assert_eq!(max(&mut t), Some(3));
}

#[test]
fn to_the_slaughter_a_creature_planeswalker_doesnt_count_twice() {
    cr!("701.21a", "207.2c", "608.2c");
    ruling!(
        "To the Slaughter",
        "If the delirium ability applies, that player must sacrifice a creature object and a planeswalker object if able. In the unusual case that the player controls a single permanent that's both a creature and a planeswalker, that player must sacrifice another creature or planeswalker (unless they control no other creatures and no other planeswalkers)."
    );
    supported("To the Slaughter");
    // During P1's turn Gideon Blackblade is a creature and a planeswalker.
    for other in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        delirium_graveyard(&mut t, P0);
        let gideon = t.battlefield(P1, "Gideon Blackblade");
        t.g.recompute();
        assert!(t.obj_now(gideon).is(CardType::Creature));
        let bears = other.then(|| t.battlefield(P1, "Grizzly Bears"));
        cast_new(&mut t, P0, "To the Slaughter", &[Entity::Player(P1)]);
        t.resolve_all();
        assert!(!t.on_battlefield(gideon), "{}", t.dump_log());
        if let Some(b) = bears {
            assert!(!t.on_battlefield(b));
        }
    }
}

#[test]
fn convert_to_slime_counts_the_permanents_it_destroyed_for_delirium() {
    cr!("608.2c", "207.2c", "701.8a");
    ruling!(
        "Convert to Slime",
        "If you destroy some of your own permanents with Convert to Slime, they'll go to your graveyard in time for their types to be counted for the delirium ability."
    );
    supported("Convert to Slime");
    let mut t = TestGame::new(2);
    // Only a land in the graveyard; P0's own artifact, creature and enchantment destroyed.
    t.graveyard(P0, "Forest");
    let stone = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ench = t.battlefield(P0, "Glorious Anthem");
    cast_new(
        &mut t,
        P0,
        "Convert to Slime",
        &[obj(stone), obj(bears), obj(ench)],
    );
    t.resolve_all();
    let ooze = with_subtype(&t, P0, "Ooze");
    assert_eq!(ooze.len(), 1, "{}", t.dump_log());
    // Mana values 2 + 2 + 3.
    assert_eq!(t.pt(ooze[0]), (7, 7));
}

#[test]
fn transforming_flourish_copies_resolve_opponent_first() {
    cr!("702.144a", "405.5", "101.4");
    ruling!(
        "Transforming Flourish",
        "If you cast a spell with demonstrate and both you and an opponent copy it, the opponent’s copy will resolve first, then your copy will resolve, and finally the original spell will resolve."
    );
    supported("Transforming Flourish");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Hill Giant");
    let y = t.battlefield(P1, "Craw Wurm");
    // The original targets the Giant; P0 copies it (new target: the Wurm); P1's copy
    // targets P0's Bears.
    t.answer_targets(P0, &[obj(x)]);
    t.answer_yes(P0, true);
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(obj(y))]);
    crate::r_s25_common::change_copy_targets(&mut t, P1, &[Some(obj(mine))]);
    cast_new(&mut t, P0, "Transforming Flourish", &[]);
    t.settle();
    let order: Vec<ObjectId> = {
        t.resolve_all();
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                mtg_engine::events::Event::ZoneChange { old, .. } if [mine, x, y].contains(old) => {
                    Some(*old)
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(order, vec![mine, y, x], "{}", t.dump_log());
}

#[test]
fn prison_barricade_kicked_attacks_until_it_leaves() {
    cr!("702.3b", "702.33d", "611.3a");
    ruling!(
        "Prison Barricade",
        "If the Prison Barricade is kicked, it enters with the ability to attack as if it did not have Defender and will keep this ability until it leaves the battlefield."
    );
    supported("Prison Barricade");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let b = t.hand(P0, "Prison Barricade");
    t.cast(P0, b).kicked(true).go();
    t.resolve_all();
    let b = t.g.current(b);
    t.g.objects[b.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, b));
    // On a later turn too.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, b));
    assert!(has(&mut t, b, KeywordKind::Defender));
}

#[test]
fn coastal_breach_cost_stays_once_determined() {
    cr!("601.2f", "601.2g", "605.1a", "702.125a");
    ruling!(
        "Coastal Breach",
        "Causing an opponent to lose the game after you’ve announced that you’re casting a spell with undaunted and determined its total cost won’t cause you to have to pay more mana."
    );
    supported("Coastal Breach");
    // Four players: Coastal Breach ({6}{U}, undaunted) costs {3}{U}. P0 has exactly four
    // mana sources, one of which (a custom land: "{T}: Add {C}. [P3] loses the game.")
    // makes P3 lose the game as P0 pays.
    let mut t = TestGame::new(4);
    t.lands(P0, "Island", 3);
    let spring = t.custom(
        P0,
        crate::r_s17_undaunted::doom_spring(P3),
        Zone::Battlefield,
    );
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cb = t.hand(P0, "Coastal Breach");
    let spell = t.cast(P0, cb).go();
    assert!(!t.g.player(P3).in_game());
    assert!(t.obj(spring).tapped);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P1));
}
