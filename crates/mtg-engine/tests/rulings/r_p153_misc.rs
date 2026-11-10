//! Rulings batch P153 — "scales with mana value": the remaining one-off rulings of the
//! batch: cost reductions that count exiled cards (CR 601.2f), a loyalty ability that
//! isn't a mana ability (CR 606.3, 605.1), choices made as an ability resolves (CR 608.2c),
//! the order of a spell's instructions (CR 608.2c, 704.3), damage staying marked (CR
//! 120.6), excess damage (CR 120.10), linked abilities (CR 607), last known information
//! (CR 113.7a, 608.2h), and "can't gain life" against a life total being set (CR 119.7).

use crate::r_p153_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported, tokens, triggers_on_stack, watch};
use crate::r_s02_common::{can_activate, can_play_land, destroy};
use crate::r_s04_common::{next_upkeep, on_stack};
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};
use crate::r_s07_common::resolved;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn kenriths_royal_funeral_stops_counting_a_card_that_left_exile() {
    cr!("601.2f", "607.2a");
    ruling!(
        "The Kenriths' Royal Funeral",
        "Any of the exiled cards that somehow leave exile will no longer count toward the discount offered by the last ability."
    );
    supported("The Kenriths' Royal Funeral");
    let mut t = TestGame::new(2);
    let bosh = t.graveyard(P0, "Bosh, Iron Golem");
    let mik = t.graveyard(P0, "Mikaeus, the Lunarch");
    t.answer_targets(P0, &[obj(bosh), obj(mik)]);
    enter(&mut t, P0, "The Kenriths' Royal Funeral");
    t.resolve_all();
    assert_eq!(t.zone(bosh), Zone::Exile);
    assert_eq!(t.zone(mik), Zone::Exile);
    assert_eq!((t.hand_size(P0), t.life(P0)), (8, 12));
    // Orah ({2}{W}{B}) costs {2} less: {W}{B}.
    let orah = t.hand(P0, "Orah, Skyclave Hierophant");
    mana(&mut t, P0, W, 1);
    mana(&mut t, P0, B, 1);
    t.cast(P0, orah).go();
    t.resolve();
    assert!(t.on_battlefield(orah));
    // Mikaeus leaves exile: only Bosh counts, {1}{W}{B}.
    move_to(&mut t, mik, Zone::Hand(P0));
    let orah2 = t.hand(P0, "Orah, Skyclave Hierophant");
    mana(&mut t, P0, W, 1);
    mana(&mut t, P0, B, 1);
    assert!(t.cast(P0, orah2).try_go().is_err());
    mana(&mut t, P0, C, 1);
    t.cast(P0, orah2).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
}

#[test]
fn stronghold_arena_reveals_the_card_only_after_you_decide_and_then_you_must_take_it() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Stronghold Arena",
        "As Stronghold Arena's last ability resolves, you don't get to see what the card of your library is before deciding whether to reveal it. If you choose to reveal it, you must put it into your hand and will lose life."
    );
    supported("Stronghold Arena");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stronghold Arena");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    // When asked whether to reveal, the top card is still an unrevealed card of the
    // library.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| {
            let top = *g.player(P0).library.last().unwrap();
            let revealed = g.reveals.revealed.iter().any(|r| r.card == top);
            (g.obj(top).zone, revealed)
        },
    );
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![(Zone::Library(P0), false)]);
    assert_eq!(t.zone(giant), Zone::Hand(P0));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn narsets_first_ability_is_a_loyalty_ability_not_a_mana_ability() {
    cr!("606.3", "605.1a", "606.2");
    ruling!(
        "Narset of the Ancient Way",
        "Because it's a loyalty ability, Narset's first ability isn't a mana ability. It can be activated only any time you could cast a sorcery. It uses the stack and can be responded to."
    );
    supported("Narset of the Ancient Way");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset of the Ancient Way");
    // Not on an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P0, narset));
    // Not with a spell on the stack.
    t.set_step(P0, Step::PrecombatMain);
    let bolt = cast_blaze(&mut t, P0, 0, Entity::Player(P1));
    assert!(!can_activate(&mut t, P0, narset));
    t.resolve();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // In a main phase with an empty stack: it uses the stack, and can be responded to.
    assert!(can_activate(&mut t, P0, narset));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    activate_containing(&mut t, P0, narset, "gain 2 life").unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    assert_eq!(t.life(P0), 20);
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(narset).go();
    t.resolve(); // Shock
    assert_eq!(t.counters(narset, "loyalty"), 5 - 2);
    t.resolve(); // the loyalty ability
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
}

#[test]
fn deaths_oasis_may_return_one_of_the_two_cards_it_just_milled() {
    cr!("608.2c", "115.1");
    ruling!(
        "Death's Oasis",
        "Because the triggered ability doesn't target the creature card in your graveyard, you may choose one of the two cards you put there from your library."
    );
    supported("Death's Oasis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Death's Oasis");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lib = stack_library(&mut t, P0, &["Memnite", "Forest"]);
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[obj(lib[0])]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(
        crate::r_s02_common::target_candidates(&t, P0, from),
        Vec::<Vec<Entity>>::new()
    );
    assert_eq!(t.zone(lib[0]), Zone::Hand(P0), "{}", t.dump_log());
    assert_eq!(t.zone(lib[1]), Zone::Graveyard(P0));
}

#[test]
fn captain_rex_nebulas_vehicle_is_sacrificed_before_its_crash_land_damage() {
    cr!("608.2c", "113.7a", "611.2a");
    ruling!(
        "Captain Rex Nebula",
        "the Crash Land ability now causes the Vehicle to be sacrificed before it deals damage"
    );
    supported("Captain Rex Nebula");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Captain Rex Nebula");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let crew = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // "Becomes a Vehicle artifact": no longer a creature until crewed.
    let o = t.obj_now(bears);
    assert!(o.is(CardType::Artifact) && o.chars.has_subtype("Vehicle"));
    assert!(!o.is(CardType::Creature));
    assert!(crate::r_s04_common::crew(&mut t, P0, bears, &[crew]));
    t.resolve_all();
    assert!(t.obj_now(bears).is(CardType::Creature));
    assert_eq!(t.pt(bears), (2, 2));
    // The Bears deal 2 combat damage to P1: the die shows 2 (its mana value), so it is
    // sacrificed, then deals 2 damage to any target (the Bears as they last existed).
    t.g.dice.loaded.push_back(2);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(on_stack(&t, "roll a six-sided die"), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 16);
    let log = t.dump_log();
    let sac = log.find("Battlefield -> Graveyard").expect("sacrificed");
    let dmg = log.rfind("deals 2 damage").expect("damage dealt");
    assert!(sac < dmg, "{log}");
    // The granted abilities last until end of turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Captain Rex Nebula");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(bears).chars.has_subtype("Vehicle"));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).chars.has_subtype("Vehicle"));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn ride_the_avalanche_has_no_target_and_its_delayed_trigger_chooses_one() {
    cr!("603.7a", "603.3d", "115.1");
    ruling!(
        "Ride the Avalanche",
        "Casting Ride the Avalanche doesn't require a target. As you put the delayed triggered ability on the stack after casting your next spell, you choose the target creature, if any."
    );
    supported("Ride the Avalanche");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let ride = cast_simple(&mut t, P0, "Ride the Avalanche");
    assert!(t.g.spell_body(ride).targets.is_empty());
    assert!(crate::r_s02_common::target_candidates(&t, P0, from).is_empty());
    t.resolve();
    // Hill Giant (mana value 4) is cast: now the target is chosen.
    let giant = cast_simple(&mut t, P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.counters(bears, "+1/+1"), 4);
}

#[test]
fn command_the_dreadhorde_damages_you_before_the_permanents_enter() {
    cr!("608.2c", "704.3", "704.5a");
    ruling!(
        "Command the Dreadhorde",
        "Any abilities they have that trigger when they enter the battlefield won’t resolve if you lose the game, but their static abilities (such as that of Platinum Angel) may help you survive with 0 or less life."
    );
    supported("Command the Dreadhorde");
    supported("Platinum Angel");
    // Platinum Angel (7) among the returned cards: P0 goes to -1 life and doesn't lose.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 6;
    let angel = t.graveyard(P0, "Platinum Angel");
    let ctd = t.hand(P0, "Command the Dreadhorde");
    mana(&mut t, P0, B, 2);
    mana(&mut t, P0, C, 4);
    t.cast(P0, ctd).targets(&[obj(angel)]).go();
    t.resolve();
    assert_eq!(t.life(P0), -1);
    assert!(t.on_battlefield(angel));
    assert!(!t.has_lost(P0));
    // Wall of Omens (2) and Hill Giant (4) at 5 life: P0 loses before the Wall's enters
    // trigger resolves.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    let wall = t.graveyard(P0, "Wall of Omens");
    let giant = t.graveyard(P0, "Hill Giant");
    let ctd = t.hand(P0, "Command the Dreadhorde");
    mana(&mut t, P0, B, 2);
    mana(&mut t, P0, C, 4);
    t.cast(P0, ctd).targets(&[obj(wall), obj(giant)]).go();
    t.g.settle();
    t.g.resolve_top();
    assert!(t.on_battlefield(wall) && t.on_battlefield(giant));
    assert_eq!(t.life(P0), -1);
    t.settle();
    assert!(t.has_lost(P0), "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn havi_dies_when_it_loses_indestructible_with_lethal_damage_marked() {
    cr!("120.6", "704.5g", "702.12b");
    ruling!(
        "Havi, the All-Father",
        "If Havi loses indestructible (probably because one or more historic cards in your graveyard left your graveyard) after having been dealt lethal damage earlier in the turn, it will die."
    );
    let mut t = TestGame::new(2);
    let havi = t.battlefield(P0, "Havi, the All-Father");
    let historic: Vec<ObjectId> = (0..4).map(|_| t.graveyard(P0, "Sol Ring")).collect();
    assert!(t.obj_now(havi).has_keyword(KeywordKind::Indestructible));
    cast_blaze(&mut t, P0, 6, obj(havi));
    t.resolve_all();
    assert!(t.on_battlefield(havi));
    assert_eq!(t.obj_now(havi).damage, 6);
    move_to(&mut t, historic[0], Zone::Exile);
    assert!(!t.on_battlefield(havi), "{}", t.dump_log());
    assert_eq!(t.zone(havi), Zone::Graveyard(P0));
}

/// Megatron, Destructive Force (converted) under P0's control, attacking P1 with its
/// trigger's sacrifice of `artifact` and damage to `target` queued: the game right after
/// the reflexive trigger resolved.
fn megatron_attacks(t: &mut TestGame, artifact: ObjectId, target: ObjectId) -> ObjectId {
    let mega = t.battlefield(P0, "Megatron, Tyrant // Megatron, Destructive Force");
    assert!(mtg_engine::dfc::transform(&mut t.g, mega));
    t.g.recompute();
    let mega = t.g.current(mega);
    assert_eq!(t.obj_now(mega).chars.name, "Megatron, Destructive Force");
    assert!(t.obj_now(mega).is(CardType::Creature), "living metal on your turn");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(artifact)]);
    t.answer_targets(P0, &[obj(target)]);
    attack_with(t, &[(mega, Entity::Player(P1))]);
    t.resolve_all();
    mega
}

#[test]
fn megatrons_excess_damage_is_what_exceeds_lethal_ignoring_indestructible() {
    cr!("120.4a", "702.2b", "702.12b");
    ruling!(
        "Megatron, Tyrant // Megatron, Destructive Force",
        "This is usually the creature's toughness minus the amount of damage that it already has marked on it, but ignoring any replacement or prevention effects that will modify this damage. Also ignore whether the creature has an ability such as indestructible that will result in it not being destroyed by this damage."
    );
    ruling!(
        "Megatron, Tyrant // Megatron, Destructive Force",
        "If Megatron has deathtouch as the reflexive triggered ability resolves, 1 damage is lethal damage, and any amount beyond that is excess damage."
    );
    supported("Darksteel Myr");
    supported("Basilisk Collar");
    // Thran Dynamo (4) is sacrificed; Hill Giant (3/3) has 1 damage: 2 is lethal, 2 is
    // excess, dealt to P1 instead; Megatron converts.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P0, "Thran Dynamo");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, jug, 1, giant);
    let from = t.asked().len();
    let mega = megatron_attacks(&mut t, jug, giant);
    assert_eq!(
        t.zone(giant),
        Zone::Graveyard(P1),
        "{}\n{:?}",
        t.dump_log(),
        crate::r_s01_common::asked_since(&t, from)
    );
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(mega).chars.name, "Megatron, Tyrant");
    // Darksteel Myr (0/1, indestructible): 1 is lethal, 3 is excess.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P0, "Thran Dynamo");
    let myr = t.battlefield(P1, "Darksteel Myr");
    megatron_attacks(&mut t, jug, myr);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.obj_now(myr).damage, 1);
    assert_eq!(t.life(P1), 17);
    // With deathtouch (Basilisk Collar): 1 is lethal to Hill Giant, 3 is excess.
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P0, "Thran Dynamo");
    let giant = t.battlefield(P1, "Hill Giant");
    let mega = t.battlefield(P0, "Megatron, Tyrant // Megatron, Destructive Force");
    assert!(mtg_engine::dfc::transform(&mut t.g, mega));
    t.g.recompute();
    let mega = t.g.current(mega);
    attach_new(&mut t, P0, "Basilisk Collar", mega);
    assert!(t.obj_now(mega).has_keyword(KeywordKind::Deathtouch));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(jug)]);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(mega, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P1), "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
}

#[test]
fn each_creation_of_avacyn_refers_to_its_own_exiled_card() {
    cr!("607.2a", "714.2b");
    ruling!(
        "The Creation of Avacyn",
        "Each The Creation of Avacyn you control has its own set of face-down exiled cards."
    );
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    // Saga A exiles Grizzly Bears, Saga B Hill Giant (chapter I).
    t.answer_choose(P0, &[obj(bears)]);
    let a = enter(&mut t, P0, "The Creation of Avacyn");
    t.resolve_all();
    t.answer_choose(P0, &[obj(giant)]);
    let b = enter(&mut t, P0, "The Creation of Avacyn");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    // Chapter II on P0's next turn: 2 + 4 life, each Saga for its own card only.
    next_upkeep(&mut t, P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(a, "lore"), 2);
    assert_eq!(t.counters(b, "lore"), 2);
    assert_eq!(t.life(P0), 14, "{}", t.dump_log());
    // Chapter III: each puts its own card onto the battlefield (yes), and is sacrificed.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
}

#[test]
fn earthquake_dragons_last_ability_works_only_from_the_graveyard() {
    cr!("113.6b");
    ruling!(
        "Earthquake Dragon",
        "Earthquake Dragon's last ability can be activated only if it is in your graveyard."
    );
    supported("Earthquake Dragon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    mana(&mut t, P0, G, 1);
    mana(&mut t, P0, C, 2);
    let on = t.battlefield(P0, "Earthquake Dragon");
    assert!(!can_activate(&mut t, P0, on));
    let in_hand = t.hand(P0, "Earthquake Dragon");
    assert!(!can_activate(&mut t, P0, in_hand));
    let in_gy = t.graveyard(P0, "Earthquake Dragon");
    assert!(can_activate(&mut t, P0, in_gy));
    activate_containing(&mut t, P0, in_gy, "Return ~").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(in_gy), Zone::Hand(P0));
}

#[test]
fn edgar_need_only_be_controlled_as_the_graveyard_cast_begins() {
    cr!("601.3", "608.2");
    ruling!(
        "Edgar, Master Machinist",
        "If, for example, you sacrifice Edgar to pay an additional cost of the spell or Edgar is destroyed in response, the spell will still resolve normally."
    );
    supported("Edgar, Master Machinist");
    let mut t = TestGame::new(2);
    let edgar = t.battlefield(P0, "Edgar, Master Machinist");
    let ring = t.graveyard(P0, "Sol Ring");
    mana(&mut t, P0, C, 1);
    let spell = t.cast(P0, ring).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    destroy(&mut t, edgar);
    t.resolve();
    assert!(t.on_battlefield(ring), "{}", t.dump_log());
    // Without Edgar it couldn't have been cast at all.
    let mut t = TestGame::new(2);
    let ring = t.graveyard(P0, "Sol Ring");
    mana(&mut t, P0, C, 1);
    assert!(t.cast(P0, ring).try_go().is_err());
}

#[test]
fn nylas_exiled_card_stays_exiled_if_nyla_leaves_or_returns_as_a_new_object() {
    cr!("400.7", "607.2a");
    ruling!(
        "Nyla, Shirshu Sleuth",
        "If Nyla returns to the battlefield as a new object (such as if you airbend it and cast it again), it doesn't maintain connection to cards exiled by Nyla previously, and they cannot be targeted to return by the current Nyla."
    );
    supported("Nyla, Shirshu Sleuth");
    // Memnite (mana value 0): no life lost, no Clues, so the end step trigger returns it.
    let mut t = TestGame::new(2);
    let memnite = t.graveyard(P0, "Memnite");
    t.answer_targets(P0, &[obj(memnite)]);
    let nyla = enter(&mut t, P0, "Nyla, Shirshu Sleuth");
    t.resolve_all();
    assert_eq!(t.zone(memnite), Zone::Exile);
    assert!(tokens(&t, P0).is_empty());
    // Nyla leaves and comes back as a new object before the end step: no connection.
    move_to(&mut t, nyla, Zone::Hand(P0));
    let nyla2 = enter(&mut t, P0, "Nyla, Shirshu Sleuth");
    t.resolve_all();
    assert!(t.on_battlefield(nyla2));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "return target card"), 0, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.zone(memnite), Zone::Exile);
    // Staying on the battlefield, Nyla returns it.
    let mut t = TestGame::new(2);
    let memnite = t.graveyard(P0, "Memnite");
    t.answer_targets(P0, &[obj(memnite)]);
    enter(&mut t, P0, "Nyla, Shirshu Sleuth");
    t.resolve_all();
    t.answer_targets(P0, &[obj(t.g.current(memnite))]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(memnite), Zone::Hand(P0));
}

#[test]
fn saruman_has_ward_2_when_it_is_an_orc() {
    cr!("702.21a", "613.1d", "613.1f");
    ruling!(
        "Saruman, the White Hand",
        "If Saruman, the White Hand is a Goblin or an Orc, it will have ward {2}."
    );
    supported("Maskwood Nexus");
    let mut t = TestGame::new(2);
    let saruman = t.battlefield(P0, "Saruman, the White Hand");
    assert!(!t.obj_now(saruman).has_keyword(KeywordKind::Ward));
    // Maskwood Nexus: creatures you control are every creature type.
    let nexus = t.battlefield(P0, "Maskwood Nexus");
    assert!(t.obj_now(saruman).chars.has_subtype("Orc"));
    assert!(t.obj_now(saruman).has_keyword(KeywordKind::Ward));
    // P1's Shock at Saruman: countered unless P1 pays {2}.
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    let shock = t.cast(P1, shock).target(saruman).go();
    t.settle();
    assert_eq!(trigger_sources(&t, "Ward"), vec![saruman]);
    t.resolve_all();
    assert_eq!(t.zone(shock), Zone::Graveyard(P1));
    assert_eq!(t.obj_now(saruman).damage, 0);
    destroy(&mut t, nexus);
    assert!(!t.obj_now(saruman).has_keyword(KeywordKind::Ward));
}

#[test]
fn ulamog_entering_from_exile_counts_itself() {
    cr!("614.12", "614.1c", "400.7");
    ruling!(
        "Ulamog, the Defiler",
        "If Ulamog is entering the battlefield directly from exile, it will see itself when determining which card has the greatest mana value among cards in exile. If that's Ulamog, which seems likely, it will enter with ten +1/+1 counters on it."
    );
    supported("Ulamog, the Defiler");
    let mut t = TestGame::new(2);
    t.exile(P1, "Grizzly Bears");
    let ulamog = t.exile(P0, "Ulamog, the Defiler");
    move_to(&mut t, ulamog, Zone::Battlefield);
    assert!(t.on_battlefield(ulamog));
    assert_eq!(t.counters(ulamog, "+1/+1"), 10);
    // From the hand (cast), only Grizzly Bears is in exile: two counters.
    let mut t = TestGame::new(2);
    t.exile(P1, "Grizzly Bears");
    let ulamog = t.hand(P0, "Ulamog, the Defiler");
    mana(&mut t, P0, C, 10);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, ulamog).go();
    t.resolve_all();
    assert!(t.on_battlefield(ulamog));
    assert_eq!(t.counters(ulamog, "+1/+1"), 2);
}

#[test]
fn spark_of_creativity_lets_you_play_an_exiled_land_if_you_have_a_land_play() {
    cr!("305.1", "305.2");
    ruling!(
        "Spark of Creativity",
        "If a land card is exiled this way, you can play that land card until end of turn if you have any available land plays."
    );
    for played_already in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if played_already {
            let f = t.hand(P0, "Forest");
            t.play_land(P0, f).unwrap();
        }
        let forest = t.library_top(P0, "Forest");
        let spark = t.hand(P0, "Spark of Creativity");
        mana(&mut t, P0, R, 1);
        t.answer_yes(P0, false);
        t.cast(P0, spark).target(bears).go();
        t.resolve();
        assert_eq!(t.zone(forest), Zone::Exile);
        assert_eq!(t.obj_now(bears).damage, 0);
        let forest = t.g.current(forest);
        assert_eq!(
            can_play_land(&mut t, P0, forest),
            !played_already,
            "played already: {played_already}"
        );
        if !played_already {
            t.play_land(P0, forest).unwrap();
            assert!(t.on_battlefield(forest));
        }
    }
}

#[test]
fn cerebral_eruption_returns_to_hand_when_a_land_is_revealed() {
    cr!("608.2c", "608.2n");
    ruling!(
        "Cerebral Eruption",
        "If a land card is revealed this way, Cerebral Eruption is returned to its owner’s hand from the stack. It finishes resolving, but it isn’t put into the graveyard."
    );
    let mut t = TestGame::new(2);
    t.library_top(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ce = t.hand(P0, "Cerebral Eruption");
    mana(&mut t, P0, R, 2);
    mana(&mut t, P0, C, 2);
    let spell = t.cast(P0, ce).target(Entity::Player(P1)).go();
    t.resolve();
    assert!(resolved(&t, spell));
    assert_eq!(t.zone(spell), Zone::Hand(P0));
    assert!(!t.in_graveyard(P0, "Cerebral Eruption"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn scrap_trawler_can_target_the_card_of_a_copy_with_greater_mana_value() {
    cr!("608.2h", "707.2", "603.10a");
    ruling!(
        "Scrap Trawler",
        "If an artifact is a copy of another artifact with greater mana value, such as Sculpting Steel copying an artifact with mana value 4, Scrap Trawler's ability can target that artifact card in your graveyard when that artifact is put into your graveyard."
    );
    supported("Scrap Trawler");
    supported("Sculpting Steel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scrap Trawler");
    t.battlefield(P0, "Juggernaut");
    let steel = t.hand(P0, "Sculpting Steel");
    mana(&mut t, P0, C, 3);
    t.answer_yes(P0, true);
    let juggernaut = t.g.find_in_zone(Zone::Battlefield, "Juggernaut")[0];
    t.answer_choose(P0, &[obj(juggernaut)]);
    let steel = t.cast(P0, steel).go();
    t.resolve();
    assert_eq!(t.obj_now(steel).chars.name, "Juggernaut");
    assert_eq!(mv(&t, steel), 4);
    // It dies as a mana value 4 artifact; in the graveyard it's a Sculpting Steel (3),
    // a legal target for its own trigger.
    let from = t.asked().len();
    destroy(&mut t, steel);
    let card = t.g.current(steel);
    assert_eq!(t.g.obj(card).chars.name, "Sculpting Steel");
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands, vec![vec![obj(card)]], "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Hand(P0));
}

#[test]
fn lord_of_pain_stops_a_life_total_being_set_higher() {
    cr!("119.7");
    ruling!(
        "The Lord of Pain",
        "If an effect says to set an opponent's life total to a number that's higher than their current life total, that player's life total won't change."
    );
    supported("The Lord of Pain");
    supported("Magister Sphinx");
    for (life, expected) in [(5, 5), (15, 10)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Lord of Pain");
        t.g.players[1].life = life;
        t.set_step(P1, Step::PrecombatMain);
        t.answer_targets(P1, &[Entity::Player(P1)]);
        enter(&mut t, P1, "Magister Sphinx");
        t.resolve_all();
        assert_eq!(t.life(P1), expected, "from {life}");
    }
}

#[test]
fn pact_weapon_keeps_you_alive_while_attached_to_a_creature_an_opponent_controls() {
    cr!("704.5a", "301.5d"); // an Equipment stays attached when the creature changes control
    ruling!(
        "Pact Weapon",
        "If an opponent gains control of the creature that Pact Weapon is attached to but doesn't also gain control of Pact Weapon, you still control Pact Weapon and still don't lose the game for having 0 or less life as long as it is attached."
    );
    supported("Pact Weapon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let weapon = attach_new(&mut t, P0, "Pact Weapon", bears);
    t.g.players[0].life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
    give_control(&mut t, bears, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(weapon).controller, P0);
    assert_eq!(attached_to(&t, weapon), Some(obj(bears)));
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    destroy(&mut t, bears);
    assert!(t.has_lost(P0));
}

#[test]
fn arni_metalbrow_uses_the_attackers_last_known_mana_value() {
    cr!("608.2h");
    ruling!(
        "Arni Metalbrow",
        "If the attacking creature or creature that entered the battlefield attacking isn't on the battlefield as the triggered ability resolves, use its mana value from when it was last on the battlefield."
    );
    supported("Arni Metalbrow");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arni Metalbrow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let memnite = t.hand(P0, "Memnite");
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(memnite)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "pay {1}{R}"), 1, "{}", t.dump_log());
    let from = t.asked().len();
    destroy(&mut t, bears);
    t.resolve_all();
    // Mana value less than 2 (the Bears as they last existed): Memnite, not Hill Giant.
    assert!(
        t.on_battlefield(memnite),
        "{}\n{:?}",
        t.dump_log(),
        crate::r_s01_common::asked_since(&t, from)
    );
    assert!(t.obj_now(memnite).tapped);
    let cands = entity_candidates(&t, P0, from);
    assert!(
        cands.iter().any(|c| c.contains(&obj(memnite)) && !c.contains(&obj(giant))),
        "{cands:?}"
    );
}
