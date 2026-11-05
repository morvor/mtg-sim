//! Rulings batch P051 — the other abilities of drain-life and draw-matters cards: costs
//! (CR 118, 601.2f, 602.2), affinity locked in before payment (CR 601.2f), cards played
//! with a permission follow the normal timing and cost rules (CR 601.3, 305.2), mana
//! spent to cast a spell (CR 601.2h), phasing (CR 702.26), a permanent that's exiled and
//! returned is a new object (CR 400.7), simultaneous sacrifices (CR 101.4), and token
//! doubling of created tokens only (CR 111.1, 707.10f).

use crate::r_p046_common::{cost_to_cast, fill_pool, tap_lands};
use crate::r_p051_common::*;
use crate::r_p057_common::pump;
use crate::r_p179_common::battlefield_back;
use crate::r_s01_common::*;
use crate::r_s05_common::{move_to, run_from};
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::player_control::can_see;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

fn loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), n);
}

/// Creates a predefined token (Food, Treasure, ...) for `p`.
fn token(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    run_from(
        t,
        p,
        None,
        Effect::CreateToken {
            spec: mtg_engine::tokens::predefined(name).expect("predefined token"),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &[],
    );
    *with_subtype(t, p, name).last().expect("the token")
}

#[test]
fn tezzeret_master_of_the_bridge_doesnt_have_affinity_himself() {
    cr!("702.41a", "601.2f");
    ruling!(
        "Tezzeret, Master of the Bridge",
        "Tezzeret himself doesn't have affinity for artifacts."
    );
    supported("Tezzeret, Master of the Bridge");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    let tez = t.hand(P0, "Tezzeret, Master of the Bridge");
    assert_eq!(cost_to_cast(&mut t, P0, tez, &[]), Some(6));
    // On the battlefield, it gives creature spells affinity.
    let wurm = t.hand(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(cost_to_cast(&mut t, P0, wurm, &[]), Some(3));
}

#[test]
fn affinity_is_locked_in_before_sacrificing_artifacts_for_mana() {
    cr!("601.2f", "601.2g", "601.2h", "702.41a");
    ruling!(
        "Ethersworn Sphinx",
        "The cost reduction is set before you pay any of the spell's costs. Specifically, you could lock in a discount for an artifact you control and then sacrifice that artifact to activate a mana ability."
    );
    ruling!(
        "Tezzeret, Master of the Bridge",
        "The cost reduction is set before you pay any of the spell's costs. Specifically, you could lock in a discount for an artifact you control and then sacrifice that artifact to activate a mana ability."
    );
    supported("Ethersworn Sphinx");
    // Ethersworn Sphinx ({7}{W}{U}, affinity) with three Lotus Petals and three lands:
    // {4}{W}{U}, paid with the Petals and the lands.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Lotus Petal");
    }
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let sphinx = t.hand(P0, "Ethersworn Sphinx");
    t.cast(P0, sphinx).go();
    assert!(t.named_on_battlefield("Lotus Petal").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ethersworn Sphinx").len(), 1);
    // Tezzeret's granted affinity: Craw Wurm ({4}{G}{G}) for three Petals.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tezzeret, Master of the Bridge");
    for _ in 0..3 {
        t.battlefield(P0, "Lotus Petal");
    }
    let wurm = t.hand(P0, "Craw Wurm");
    t.cast(P0, wurm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn tezzeret_agent_of_bolas_animated_equipment_keeps_its_types_and_falls_off() {
    cr!("205.1b", "301.5c", "704.5n");
    ruling!(
        "Tezzeret, Agent of Bolas",
        "The artifact targeted by the second ability will retain any types, subtypes, or supertypes it has. Notably, if an Equipment becomes an artifact creature, it usually can't be attached to another creature. If it was attached to a creature, it becomes unattached."
    );
    supported("Tezzeret, Agent of Bolas");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Agent of Bolas");
    let bears = t.battlefield(P0, BEARS);
    let sword = t.battlefield(P0, "Short Sword");
    t.g.attach(sword, obj(bears));
    t.g.recompute();
    t.activate(P0, tez, 1, &[obj(sword)]).unwrap();
    t.resolve_all();
    let s = t.obj_now(sword);
    assert!(s.is(CardType::Creature) && s.is(CardType::Artifact));
    assert!(s.chars.has_subtype("Equipment"));
    assert_eq!(t.pt(sword), (5, 5));
    assert_eq!(s.attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn bontu_counts_a_creature_that_died_even_if_it_left_the_graveyard() {
    cr!("508.1c", "700.4");
    ruling!(
        "Bontu the Glorified",
        "The creature that died doesn't have to still be in its owner's graveyard to satisfy Bontu's combat restriction."
    );
    supported("Bontu the Glorified");
    let mut t = TestGame::new(2);
    let bontu = t.battlefield(P0, "Bontu the Glorified");
    assert!(!t.g.can_attack(bontu));
    let bears = t.battlefield(P0, BEARS);
    t.g.destroy(bears, None);
    t.settle();
    let dead = t.g.current(bears);
    move_to(&mut t, dead, Zone::Exile);
    assert!(t.in_exile(BEARS));
    assert!(t.g.can_attack(bontu));
}

#[test]
fn maccready_gives_skulk_whatever_the_power_on_resolution() {
    cr!("603.2");
    ruling!(
        "MacCready, Lamplight Mayor",
        "The power of your attacking creature is checked only when MacCready’s first ability triggers. Once it triggers, the creature will gain skulk until end of turn when the ability resolves, regardless of whether or not its power is still 2 or less at that time."
    );
    supported("MacCready, Lamplight Mayor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "MacCready, Lamplight Mayor");
    let bears = t.battlefield(P0, BEARS);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(bears, pl(P1)), (wurm, pl(P1))]);
    assert_eq!(triggers_on_stack(&t, "skulk"), 1);
    pump(&mut t, bears, 3, 0);
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Skulk));
    assert!(!t.obj_now(wurm).has_keyword(KeywordKind::Skulk));
}

/// Lobelia enters with Forest on top of P1's library (exiled face down); P0 activates
/// the first mode (sacrificing an Ornithopter). Returns the exiled Forest.
fn lobelia_forest(t: &mut TestGame) -> ObjectId {
    supported("Lobelia, Defender of Bag End");
    let forest = t.library_top(P1, "Forest");
    let lobelia = t.enter(P0, "Lobelia, Defender of Bag End");
    t.resolve_all();
    let exiled = t.g.current(forest);
    assert_eq!(t.zone(exiled), Zone::Exile);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.g.objects[lobelia.0 as usize].summoning_sick = false;
    assert!(activate_text(t, P0, lobelia, "Choose one", &[]));
    t.resolve_all();
    exiled
}

#[test]
fn lobelia_lands_follow_the_normal_timing_and_land_play_rules() {
    cr!("305.2", "116.2a");
    ruling!(
        "Lobelia, Defender of Bag End",
        "To play a card using the permission granted by the first mode of Lobelia, Defender of Bag End's activated ability, you must still follow any timing restrictions and permissions, including those based on the card's type. For instance, you can play a land from exile this way only during your main phase while the stack is empty, and only if you have an available land play remaining for the turn."
    );
    // No land play left.
    let mut t = TestGame::new(2);
    let forest = lobelia_forest(&mut t);
    let land = t.hand(P0, "Plains");
    t.play_land(P0, land).unwrap();
    assert!(t.play_land(P0, forest).is_err());
    // The stack isn't empty.
    let mut t = TestGame::new(2);
    let forest = lobelia_forest(&mut t);
    let bolt = t.hand(P1, "Lightning Bolt");
    fill_pool(&mut t, P1, 1);
    t.cast(P1, bolt).target(pl(P1)).go();
    assert!(t.play_land(P0, forest).is_err());
    t.resolve_all();
    // Main phase, empty stack, a land play: yes.
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
}

#[test]
fn lobelias_face_down_cards_are_seen_only_by_you() {
    cr!("406.3");
    ruling!(
        "Lobelia, Defender of Bag End",
        "You may look at these face-down exiled cards any time you wish. No other player may look at the face-down cards you exiled with Lobelia, Defender of Bag End, even if another player takes control of it."
    );
    supported("Lobelia, Defender of Bag End");
    let mut t = TestGame::new(2);
    let top = t.library_top(P1, BEARS);
    let lobelia = t.enter(P0, "Lobelia, Defender of Bag End");
    t.resolve_all();
    let exiled = t.g.current(top);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(can_see(&t.g, P0, exiled));
    assert!(!can_see(&t.g, P1, exiled));
    crate::r_s06_common::give_control(&mut t, lobelia, P1);
    assert!(can_see(&t.g, P0, exiled));
    assert!(!can_see(&t.g, P1, exiled));
}

#[test]
fn cards_cast_with_a_permission_follow_timing_rules_and_costs() {
    cr!("601.3", "307.1", "601.2h");
    ruling!(
        "Vohar, Vodalian Desecrator",
        "You must pay all costs and follow all normal timing rules for cards cast with Vohar, Vodalian Desecrator's last ability."
    );
    supported("Vohar, Vodalian Desecrator");
    let mut t = TestGame::new(2);
    let vohar = t.battlefield(P0, "Vohar, Vodalian Desecrator");
    let div = t.graveyard(P0, "Divination");
    assert!(activate_text(&mut t, P0, vohar, "Sacrifice", &[obj(div)]));
    t.resolve_all();
    tap_lands(&mut t, P0);
    t.g.players[0].mana_pool = Default::default();
    // No mana: it can't be cast.
    assert!(t.cast(P0, div).try_go().is_err());
    // Not with a spell on the stack (it's a sorcery).
    let bolt = t.hand(P1, "Lightning Bolt");
    fill_pool(&mut t, P1, 1);
    t.cast(P1, bolt).target(pl(P1)).go();
    fill_pool(&mut t, P0, 3);
    assert!(t.cast(P0, div).try_go().is_err());
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.cast(P0, div).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);

    ruling!(
        "Quintorius Kand",
        "You must follow all normal timing rules for a card you play with Quintorius Kand's last ability and, if it's a spell, you must pay its costs to cast it."
    );
    supported("Quintorius Kand");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quintorius Kand");
    loyalty(&mut t, q, 6);
    let div = t.graveyard(P0, "Divination");
    t.activate(P0, q, 2, &[obj(div)]).unwrap();
    t.resolve_all();
    let div = t.g.current(div);
    assert_eq!(t.zone(div), Zone::Exile);
    // Only the {R} it added: not enough for {2}{U}.
    assert!(t.cast(P0, div).try_go().is_err());
    // The {R}, {U} and one more pay for it.
    t.g.players[0].mana_pool.add_type(ManaType::U, 1);
    t.g.players[0].mana_pool.add_type(ManaType::C, 1);
    t.cast(P0, div).go();
    t.resolve_all();
    assert!(t.in_exile("Divination") || t.in_graveyard(P0, "Divination"));
}

#[test]
fn tor_wauki_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a", "702.15b");
    ruling!(
        "Tor Wauki the Younger",
        "Tor Wauki the Younger's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Tor Wauki the Younger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tor Wauki the Younger");
    t.answer_targets(P0, &[pl(P1)]);
    let spell = cast_pool(&mut t, P0, "Divination", &[]);
    t.settle();
    fill_pool(&mut t, P1, 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Divination"));
    t.resolve_all();
    // 2 damage (Tor Wauki's own, not increased), with lifelink.
    assert_eq!(lives(&t, 2), vec![22, 18]);
}

/// Casts Vibrance paying from exactly the given mana (lands tapped). Returns P1's life
/// and P0's after everything resolves, and whether a land was searched for.
fn cast_vibrance(mana: &[ManaType]) -> (i32, i32, bool) {
    supported("Vibrance");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    tap_lands(&mut t, P0);
    for m in mana {
        t.g.players[0].mana_pool.add_type(*m, 1);
    }
    let v = t.hand(P0, "Vibrance");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose(P0, &[obj(forest)]);
    t.cast(P0, v).go();
    t.resolve_all();
    (t.life(P1), t.life(P0), t.in_hand(P0, "Forest"))
}

#[test]
fn vibrance_checks_the_mana_spent_on_its_whole_cost_once() {
    cr!("601.2h", "603.4");
    ruling!(
        "Vibrance",
        "Vibrance's first and second abilities care about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid mana symbols in its cost."
    );
    ruling!(
        "Vibrance",
        "Vibrance's first and second abilities check to see if at least two mana of the appropriate colors were spent to pay Vibrance's cost. It doesn't matter if more than two mana of that color was spent; the effect isn't multiplied."
    );
    use ManaType::*;
    // {R}{R} and {G}{G} among five mana: both, whichever symbols they paid.
    assert_eq!(cast_vibrance(&[R, R, G, G, C]), (17, 22, true));
    // Five red: 3 damage once; no search.
    assert_eq!(cast_vibrance(&[R, R, R, R, R]), (17, 20, false));
    // One of each: neither.
    assert_eq!(cast_vibrance(&[R, G, C, C, C]), (20, 20, false));
}

#[test]
fn creeping_inn_phased_out_doesnt_exist_and_remembers_its_cards() {
    cr!("702.26b", "702.26d", "607.2a");
    ruling!(
        "Hostile Hostel // Creeping Inn",
        "While Creeping Inn is phased out, it's treated as though it doesn't exist. It can't be the target of spells or abilities, its static abilities have no effect on the game, its triggered abilities can't trigger, it can't attack or block, and so on."
    );
    ruling!(
        "Hostile Hostel // Creeping Inn",
        "When Creeping Inn phases in, it still remembers what cards were previously exiled with it."
    );
    supported("Hostile Hostel // Creeping Inn");
    let mut t = TestGame::new(2);
    let inn = battlefield_back(&mut t, P0, "Hostile Hostel // Creeping Inn");
    let a = t.graveyard(P0, BEARS);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    attack_with(&mut t, &[(inn, pl(P1))]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
    t.set_step(P0, Step::End);
    assert!(activate_text(&mut t, P0, inn, "phases out", &[]));
    t.resolve_all();
    assert!(t.obj_now(inn).phased_out);
    // Phased out: Wrath of God doesn't destroy it and it can't be targeted.
    t.set_step(P1, Step::PrecombatMain);
    crate::r_p044_common::wrath(&mut t, P1);
    // Doom Blade ("Destroy target nonblack creature.") has no legal target.
    let blade = t.hand(P1, "Doom Blade");
    fill_pool(&mut t, P1, 2);
    t.answer_targets(P1, &[obj(inn)]);
    assert!(t.cast(P1, blade).try_go().is_err());
    t.clear_answers();
    // It phases in during P0's untap step, remembering the first card.
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(inn).phased_out);
    assert!(t.on_battlefield(inn));
    let b = t.graveyard(P0, BEARS);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(b)]);
    attack_with(&mut t, &[(inn, pl(P1))]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![23, 17]);
}

#[test]
fn ghost_council_returns_as_a_new_untapped_object() {
    cr!("400.7", "303.4c", "122.2");
    ruling!(
        "Ghost Council of Orzhova",
        "When Ghost Council of Orzhova is exiled, any Auras or Equipment on it fall off and any counters on it are removed. When it returns to the battlefield, it comes back untapped and is treated as a new version of the card."
    );
    supported("Ghost Council of Orzhova");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Ghost Council of Orzhova");
    let bears = t.battlefield(P0, BEARS);
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, obj(g));
    t.g.add_counters(obj(g), "+1/+1", 1, None);
    t.g.tap(g);
    t.g.recompute();
    assert_eq!(t.pt(g), (6, 7));
    t.answer_choose(P0, &[obj(bears)]);
    assert!(activate_text(&mut t, P0, g, "Exile", &[]));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Ghost Council of Orzhova");
    assert_eq!(back.len(), 1);
    let back = back[0];
    assert_ne!(back, g);
    assert!(!t.obj_now(back).tapped);
    assert_eq!(t.counters(back, "+1/+1"), 0);
    assert_eq!(t.pt(back), (4, 4));
}

#[test]
fn papalymo_opponents_who_lost_life_choose_in_turn_order_then_sacrifice_together() {
    cr!("101.4", "701.21a");
    ruling!(
        "Papalymo Totolymo",
        "While resolving Papalymo Totolymo's second ability, the next opponent in turn order who lost life this turn chooses a creature with the greatest power among creatures they control, then each other opponent who lost life this turn in turn order does the same. Then each of the chosen creatures are sacrificed simultaneously."
    );
    supported("Papalymo Totolymo");
    let mut t = TestGame::new(4);
    let pap = t.battlefield(P0, "Papalymo Totolymo");
    let b1 = [t.battlefield(P1, BEARS), t.battlefield(P1, BEARS)];
    t.battlefield(P1, "Savannah Lions");
    let b2 = t.battlefield(P2, "Craw Wurm");
    let b3 = [
        t.battlefield(P3, "Craw Wurm"),
        t.battlefield(P3, "Craw Wurm"),
    ];
    t.g.lose_life(P1, 1);
    t.g.lose_life(P3, 1);
    t.g.flush_events();
    // The battlefield size when each player chooses.
    let seen3 = watch(
        &mut t,
        P3,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    let before = t.g.battlefield.len();
    t.answer_choose(P1, &[obj(b1[1])]);
    t.answer_choose(P3, &[obj(b3[0])]);
    let from = t.asked().len();
    assert!(activate_text(&mut t, P0, pap, "Sacrifice", &[]));
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P1, P3]);
    // Nothing had been sacrificed yet as P3 chose (Papalymo itself was, as a cost).
    assert_eq!(*seen3.lock().unwrap(), vec![before - 1]);
    assert!(t.on_battlefield(b1[0]) && !t.on_battlefield(b1[1]));
    assert!(t.on_battlefield(b2));
    assert!(!t.on_battlefield(b3[0]) && t.on_battlefield(b3[1]));
}

#[test]
fn archon_of_cruelty_follows_its_instructions_in_order() {
    cr!("608.2c", "603.3", "104.3b", "800.4d");
    ruling!(
        "Archon of Cruelty",
        "You and the opponent you target take all the actions in the order listed. Notably, the creature or planeswalker they sacrifice won't be on the battlefield as they discard a card or lose life. If any abilities trigger, they will wait to be put on the stack until after the spell resolves. If any abilities controlled by that opponent trigger but losing 3 life causes them to lose the game, none of those abilities will be put on the stack."
    );
    supported("Archon of Cruelty");
    let mut t = TestGame::new(3);
    // P1's only creature is Platinum Emperion ("Your life total can't change."); P1 has
    // Faith of the Devoted ("Whenever you cycle or discard a card, you may pay {1}...").
    t.battlefield(P1, "Platinum Emperion");
    t.battlefield(P1, "Faith of the Devoted");
    t.hand(P1, BEARS);
    t.g.players[1].life = 3;
    // "Whenever you gain life, put a +1/+1 counter on this creature."
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    t.answer_targets(P0, &[pl(P1)]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Archon of Cruelty");
    t.settle();
    t.resolve();
    // The Emperion was gone as P1 lost 3 life: P1 lost the game, and Faith of the
    // Devoted's trigger was never put on the stack.
    assert!(t.has_lost(P1));
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 0);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 1);
    // P0's Pridemate trigger waited until the ability resolved.
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}

#[test]
fn joshua_discarding_no_cards_draws_none() {
    cr!("701.9a");
    ruling!(
        "Joshua, Phoenix's Dominant // Phoenix, Warden of Fire",
        "You can choose to discard no cards when Joshua's first ability resolves. If you do, you won't draw any cards."
    );
    supported("Joshua, Phoenix's Dominant // Phoenix, Warden of Fire");
    for n in [0usize, 2] {
        let mut t = TestGame::new(2);
        let cards: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, BEARS)).collect();
        let pick: Vec<Entity> = cards[..n].iter().map(|c| obj(*c)).collect();
        t.answer_choose(P0, &pick);
        t.enter(P0, "Joshua, Phoenix's Dominant // Phoenix, Warden of Fire");
        t.resolve_all();
        assert_eq!(t.graveyard_size(P0), n, "{n}");
        assert_eq!(drawn_this_turn(&t, P0), n as u32, "{n}");
        assert_eq!(t.hand_size(P0), 3, "{n}");
    }
}

#[test]
fn sling_gang_lieutenant_can_sacrifice_any_goblin_including_itself() {
    cr!("602.2b", "701.21a");
    ruling!(
        "Sling-Gang Lieutenant",
        "You can sacrifice any Goblin you control to activate Sling-Gang Lieutenant’s activated ability, not just the ones its triggered ability creates. You can even sacrifice Sling-Gang Lieutenant itself."
    );
    supported("Sling-Gang Lieutenant");
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Sling-Gang Lieutenant");
    let goblin = t.battlefield(P0, "Raging Goblin");
    for g in [goblin, sg] {
        t.answer_choose(P0, &[obj(g)]);
        assert!(activate_text(
            &mut t,
            P0,
            sg,
            "Sacrifice a Goblin",
            &[pl(P1)]
        ));
        t.resolve_all();
        assert!(!t.on_battlefield(g));
    }
    assert_eq!(lives(&t, 2), vec![22, 18]);
}

#[test]
fn treacherous_greed_sacrifices_a_creature_whose_damage_recipient_is_gone() {
    cr!("601.2b", "601.2h");
    ruling!(
        "Treacherous Greed",
        "You can sacrifice any creature you control that dealt damage this turn to pay Treacherous Greed's additional cost, even if the permanent it dealt damage to is no longer on the battlefield or the player it dealt damage to is no longer in the game."
    );
    supported("Treacherous Greed");
    for to_player in [false, true] {
        let mut t = TestGame::new(3);
        let mine = t.battlefield(P0, BEARS);
        let other = t.battlefield(P0, BEARS);
        if to_player {
            t.g.deal_damage(mine, pl(P2), 2, false);
            t.g.flush_events();
            t.take_action(P2, Action::Concede);
            assert!(t.has_lost(P2));
        } else {
            let victim = t.battlefield(P1, "Savannah Lions");
            t.g.deal_damage(mine, obj(victim), 2, false);
            t.g.flush_events();
            t.settle();
            assert!(t.in_graveyard(P1, "Savannah Lions"));
        }
        fill_pool(&mut t, P0, 5);
        let s = t.hand(P0, "Treacherous Greed");
        t.answer_choose(P0, &[obj(mine)]);
        t.cast(P0, s).go();
        assert!(!t.on_battlefield(mine), "{to_player}");
        assert!(t.on_battlefield(other), "{to_player}");
        t.resolve_all();
        assert_eq!(t.life(P0), 23, "{to_player}");
    }
}

#[test]
fn lathril_taps_ten_other_elves_even_summoning_sick_ones() {
    cr!("302.6", "602.5a", "118.3");
    ruling!(
        "Lathril, Blade of the Elves",
        "You can tap any ten untapped Elves you control, including ones you haven't controlled continuously since the beginning of your most recent turn, to pay that part of the cost of Lathril's activated ability. You must have controlled Lathril continuously since the beginning of your most recent turn, however. Lathril doesn't count as one of the ten."
    );
    supported("Lathril, Blade of the Elves");
    // (other Elves, Lathril summoning sick, activated?)
    for (elves, sick, ok) in [(10, false, true), (9, false, false), (10, true, false)] {
        let mut t = TestGame::new(2);
        let l = if sick {
            t.battlefield_sick(P0, "Lathril, Blade of the Elves")
        } else {
            t.battlefield(P0, "Lathril, Blade of the Elves")
        };
        for _ in 0..elves {
            t.battlefield_sick(P0, "Llanowar Elves");
        }
        let did = activate_text(&mut t, P0, l, "ten untapped Elves", &[]);
        assert_eq!(did, ok, "{elves} {sick}");
        t.resolve_all();
        let drained = if ok { vec![30, 10] } else { vec![20, 20] };
        assert_eq!(lives(&t, 2), drained, "{elves} {sick}");
    }
}

#[test]
fn cauldron_familiar_returns_only_from_the_graveyard() {
    cr!("113.6m");
    ruling!(
        "Cauldron Familiar",
        "You can't activate Cauldron Familiar's last ability unless it's in your graveyard."
    );
    supported("Cauldron Familiar");
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Cauldron Familiar");
    let food = token(&mut t, P0, "Food");
    t.answer_choose(P0, &[obj(food)]);
    assert!(!activate_text(&mut t, P0, cat, "Return", &[]));
    t.clear_answers();
    assert!(t.on_battlefield(food));
    move_to(&mut t, cat, Zone::Graveyard(P0));
    t.answer_choose(P0, &[obj(food)]);
    assert!(activate_text(&mut t, P0, cat, "Return", &[]));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Cauldron Familiar").len(), 1);
    assert!(!t.on_battlefield(food));
}

#[test]
fn thrull_parasite_cant_stop_a_loyalty_cost_already_paid() {
    cr!("602.2b", "606.4", "601.2h");
    ruling!(
        "Thrull Parasite",
        "You can't activate Thrull Parasite's last ability to stop a player from paying the cost of a spell or an ability that requires removing a counter from a permanent (such as a planeswalker's loyalty ability)."
    );
    supported("Thrull Parasite");
    let mut t = TestGame::new(2);
    let parasite = t.battlefield(P0, "Thrull Parasite");
    let ajani = t.battlefield(P1, "Ajani Vengeant");
    t.set_step(P1, Step::PrecombatMain);
    // "−2: Ajani deals 3 damage to any target and you gain 3 life."
    t.activate(P1, ajani, 1, &[pl(P0)]).unwrap();
    // The cost is already paid as P0 can respond.
    assert_eq!(t.counters(ajani, "loyalty"), 1);
    assert!(activate_text(
        &mut t,
        P0,
        parasite,
        "Remove a counter",
        &[obj(ajani)]
    ));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ajani Vengeant"));
    // The loyalty ability still resolved: 3 damage, 3 life; P0 paid 2 life.
    assert_eq!(lives(&t, 2), vec![15, 23]);
}

#[test]
fn vizkopa_confessor_paying_zero_life_reveals_nothing() {
    cr!("119.4b");
    ruling!(
        "Vizkopa Confessor",
        "You may pay 0 life when Vizkopa Confessor's \"enters\" ability resolves. If you do, no cards are revealed or exiled."
    );
    supported("Vizkopa Confessor");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, BEARS);
    }
    t.answer_targets(P0, &[pl(P1)]);
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    t.enter(P0, "Vizkopa Confessor");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 3);
    assert_eq!(t.life(P0), 20);
    assert!(!t.in_exile(BEARS));
}

#[test]
fn bard_doubles_created_tokens_only_and_they_enter_together() {
    cr!("111.1", "614.1a", "608.3f");
    ruling!(
        "Bard, King of Dale",
        "Bard's last ability only applies to tokens that are created. Copies of permanent spells that resolve become tokens on the battlefield, but those tokens are not created and will not be doubled by Bard's ability."
    );
    ruling!(
        "Bard, King of Dale",
        "For Bard's last ability, all of the tokens enter the battlefield simultaneously. They'll be created with the same name, color, type and subtype, abilities, power, toughness, and so on."
    );
    supported("Bard, King of Dale");
    supported("Double Major");
    supported("Woodland Champion");
    // A copy of a creature spell (Double Major): one token Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bard, King of Dale");
    let bears = cast_pool(&mut t, P0, BEARS, &[]);
    cast_pool(&mut t, P0, "Double Major", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(BEARS).len(), 2);
    assert_eq!(tokens(&t, P0).len(), 1);
    // A created token: two identical Spirits entering together (Woodland Champion:
    // "Whenever one or more tokens you control enter, put that many +1/+1 counters on
    // this creature." triggers once).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bard, King of Dale");
    let champion = t.battlefield(P0, "Woodland Champion");
    fill_pool(&mut t, P0, 10);
    let lc = t.hand(P0, "Lorehold Command");
    t.cast(P0, lc).modes(&[0, 1]).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "that many"), 1);
    t.resolve_all();
    let spirits = with_subtype(&t, P0, "Spirit");
    assert_eq!(spirits.len(), 2);
    let (a, b) = (t.obj_now(spirits[0]), t.obj_now(spirits[1]));
    assert_eq!(a.chars.name, b.chars.name);
    assert_eq!(a.chars.colors, b.chars.colors);
    assert_eq!(t.pt(spirits[0]), t.pt(spirits[1]));
    assert_eq!(t.counters(champion, "+1/+1"), 2);
}
