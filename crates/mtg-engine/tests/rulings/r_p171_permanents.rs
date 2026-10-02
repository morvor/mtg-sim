//! Rulings batch P171 — triggered and static abilities of assorted permanents: tapped
//! creature checks (Dragonscale General, Flight-Deck Coordinator, Frontline War-Rager,
//! Supply Caravan), Dimir Spybug and Knowledge and Power (surveil and scry), Sanctum of
//! Tranquil Light, Keldon Warcaller, Dead of Winter, On Thin Ice, Ember Weaver and
//! Wandering Champion.

use crate::r_p171_common::*;
use crate::r_s02_common::can_activate;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s09_common::declare;
use crate::r_s11_common::triggered_from;
use crate::r_s19_common::add_lore;
use crate::r_s25_common::creature_tokens;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// A creature P0 controls, tapped.
fn tapped_creature(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.battlefield(P0, name);
    t.g.tap(id);
    id
}

// ---------------------------------------------------------------------------------
// Tapped creatures.
// ---------------------------------------------------------------------------------

#[test]
fn dragonscale_general_counts_as_it_resolves_and_bolsters_any_creature() {
    cr!("701.39a", "608.2h");
    ruling!(
        "Dragonscale General",
        "Count the number of tapped creatures you control as the ability resolves to determine the value of X."
    );
    ruling!(
        "Dragonscale General",
        "The creature you put the counters on doesn’t have to be one of the tapped creatures."
    );
    supported("Dragonscale General");
    // "At the beginning of your end step, bolster X, where X is the number of tapped
    // creatures you control."
    let mut t = TestGame::new(2);
    let general = t.battlefield(P0, "Dragonscale General");
    tapped_creature(&mut t, "Hill Giant");
    tapped_creature(&mut t, "Craw Wurm");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, general), 1);
    // Two tapped creatures as it triggered; a third is tapped before it resolves.
    t.g.tap(general);
    t.resolve_all();
    // The untapped Elves (least toughness) get X = 3 counters.
    assert!(!tapped(&t, elves));
    assert_eq!(t.counters(elves, "+1/+1"), 3);
}

#[test]
fn end_step_tapped_creature_checks_happen_as_the_end_step_starts() {
    cr!("603.4");
    ruling!(
        "Flight-Deck Coordinator",
        "Flight-Deck Coordinator’s ability will check as your end step starts to see if you control two or more tapped creatures. If you don’t, the ability won’t trigger at all."
    );
    ruling!(
        "Frontline War-Rager",
        "Frontline War-Rager’s ability will check as your end step starts to see if you control two or more tapped creatures. If you don’t, the ability won’t trigger at all."
    );
    for name in ["Flight-Deck Coordinator", "Frontline War-Rager"] {
        supported(name);
        // One tapped creature as the end step starts: no trigger, and tapping another
        // during the end step is too late.
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        tapped_creature(&mut t, "Hill Giant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_from(&t, perm), 0, "{name}");
        t.g.tap(bears);
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{name}");
        assert_eq!(t.counters(perm, "+1/+1"), 0, "{name}");
        // Two tapped creatures, but one untaps before it resolves: nothing happens.
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        let giant = tapped_creature(&mut t, "Hill Giant");
        tapped_creature(&mut t, "Grizzly Bears");
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_from(&t, perm), 1, "{name}");
        t.g.untap(giant);
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{name}");
        assert_eq!(t.counters(perm, "+1/+1"), 0, "{name}");
        // Two tapped creatures throughout: it does its thing.
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        tapped_creature(&mut t, "Hill Giant");
        tapped_creature(&mut t, "Grizzly Bears");
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve_all();
        if name == "Flight-Deck Coordinator" {
            assert_eq!(t.life(P0), 22);
        } else {
            assert_eq!(t.counters(perm, "+1/+1"), 1);
        }
    }
}

/// Supply Caravan enters under P0's control (tapped if `tapped`).
fn caravan_enters(t: &mut TestGame, tapped: bool) -> ObjectId {
    let id =
        t.g.create_card_object(card("Supply Caravan"), P0, Zone::Nowhere);
    let id =
        t.g.move_object_ev(MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(P0),
            etb: EtbInfo {
                controller: Some(P0),
                tapped,
                ..Default::default()
            },
            source: None,
        })
        .expect("Supply Caravan enters");
    t.g.flush_events();
    t.settle();
    id
}

fn warriors(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.subtypes.iter().any(|s| s == "Warrior"))
        .count()
}

#[test]
fn supply_caravan_checks_for_a_tapped_creature_as_it_enters() {
    cr!("603.4", "603.6a");
    ruling!(
        "Supply Caravan",
        "If an effect causes Supply Caravan to enter the battlefield tapped, it will satisfy its own triggered ability."
    );
    ruling!(
        "Supply Caravan",
        "If you don’t control a tapped creature as Supply Caravan enters the battlefield, its ability doesn’t trigger, even if you can tap a creature right away. If you control no tapped creatures as the ability resolves, nothing happens."
    );
    supported("Supply Caravan");
    // Entering tapped: it satisfies its own trigger.
    let mut t = TestGame::new(2);
    let c = caravan_enters(&mut t, true);
    assert_eq!(triggers_from(&t, c), 1);
    t.resolve_all();
    assert_eq!(warriors(&t), 1);
    // No tapped creature as it enters: no trigger, even if one is tapped right away.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = caravan_enters(&mut t, false);
    assert_eq!(triggers_from(&t, c), 0);
    t.g.tap(bears);
    t.resolve_all();
    assert_eq!(warriors(&t), 0);
    // A tapped creature as it enters, untapped before the ability resolves: nothing.
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, "Grizzly Bears");
    let c = caravan_enters(&mut t, false);
    assert_eq!(triggers_from(&t, c), 1);
    t.g.untap(bears);
    t.resolve_all();
    assert_eq!(warriors(&t), 0);
}

// ---------------------------------------------------------------------------------
// Surveil and scry.
// ---------------------------------------------------------------------------------

#[test]
fn dimir_spybug_gets_one_counter_per_surveil() {
    cr!("701.25a");
    ruling!(
        "Dimir Spybug",
        "You put only one +1/+1 counter on Dimir Spybug each time you surveil, no matter how many cards you looked at when you surveilled."
    );
    supported("Dimir Spybug");
    supported("Notion Rain");
    let mut t = TestGame::new(2);
    let bug = t.battlefield(P0, "Dimir Spybug");
    // Notion Rain: "Surveil 2, then draw two cards. You lose 2 life."
    cast_card(&mut t, P0, "Notion Rain");
    t.resolve_all();
    assert_eq!(t.counters(bug, "+1/+1"), 1);
}

#[test]
fn knowledge_and_power_triggers_once_per_scry_and_pays_on_resolution() {
    cr!("701.22a", "603.3d", "603.5", "118.12");
    ruling!(
        "Knowledge and Power",
        "Knowledge and Power's ability triggers only once each time you scry, no matter how many cards you look at."
    );
    ruling!(
        "Knowledge and Power",
        "You choose the target of the ability as it's put on the stack. You choose whether to pay {2} as it resolves. You can pay {2} only once each time you scry."
    );
    supported("Knowledge and Power");
    supported("Preordain");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let kp = t.battlefield(P0, "Knowledge and Power");
        let lands = t.lands(P0, "Wastes", 4);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        // Preordain: "Scry 2, then draw a card."
        cast_card(&mut t, P0, "Preordain");
        t.resolve();
        assert_eq!(triggers_from(&t, kp), 1);
        let trig = *t.stack.last().unwrap();
        assert_eq!(
            crate::r_s25_common::targets_of(&t, trig),
            vec![Entity::Player(P1)]
        );
        assert!(lands.iter().all(|l| !tapped(&t, *l)));
        let from = t.asked().len();
        t.answer_yes(P0, pay);
        t.answer_yes(P0, pay);
        t.resolve_all();
        let yes_no = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .count();
        assert_eq!(yes_no, 1);
        assert_eq!(t.life(P1), if pay { 18 } else { 20 });
        let spent = lands.iter().filter(|l| tapped(&t, **l)).count();
        assert_eq!(spent, if pay { 2 } else { 0 });
    }
}

// ---------------------------------------------------------------------------------
// Sanctum of Tranquil Light, Keldon Warcaller, Dead of Winter.
// ---------------------------------------------------------------------------------

#[test]
fn sanctum_of_tranquil_light_reduces_only_generic_mana() {
    cr!("601.2f", "602.2b");
    ruling!(
        "Sanctum of Tranquil Light",
        "The cost reduction applies only to generic mana in the cost of the activated ability. It can't reduce the {W} requirement."
    );
    supported("Sanctum of Tranquil Light");
    // Seven Shrines: {5}{W} reduced by up to 7 is {W}.
    for (land, ok) in [("Plains", true), ("Wastes", false)] {
        let mut t = TestGame::new(2);
        let sanctum = t.battlefield(P0, "Sanctum of Tranquil Light");
        for shrine in [
            "Sanctum of Calm Waters",
            "Sanctum of Fruitful Harvest",
            "Sanctum of Shattered Heights",
            "Sanctum of Stone Fangs",
            "Honden of Cleansing Fire",
            "Honden of Seeing Winds",
        ] {
            t.battlefield(P0, shrine);
        }
        t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, land, 1);
        assert_eq!(can_activate(&mut t, P0, sanctum), ok, "{land}");
    }
}

#[test]
fn keldon_warcaller_chapter_resolves_before_blockers() {
    cr!("714.2b", "508.2");
    ruling!(
        "Keldon Warcaller",
        "The target Saga’s appropriate chapter ability triggers and resolves before blockers are declared."
    );
    supported("Keldon Warcaller");
    supported("History of Benalia");
    let mut t = TestGame::new(2);
    let caller = t.battlefield(P0, "Keldon Warcaller");
    let saga = t.battlefield(P0, "History of Benalia");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0), 1);
    t.answer_targets(P0, &[obj(saga)]);
    declare(&mut t, P0, &[(caller, Entity::Player(P1))]);
    t.resolve_all();
    // Still in the declare attackers step: chapter II already created the second Knight.
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.counters(saga, "lore"), 2);
    assert_eq!(creature_tokens(&t, P0), 2);
}

#[test]
fn dead_of_winter_locks_in_its_creatures_and_x() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Dead of Winter",
        "Dead of Winter affects only creatures that aren’t snow creatures at the time it resolves. Creatures that enter the battlefield later in the turn won’t get -X/-X."
    );
    ruling!(
        "Dead of Winter",
        "The value of X is determined only as Dead of Winter begins to resolve. It won’t change later in the turn if the number of snow permanents you control changes."
    );
    supported("Dead of Winter");
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Swamp", 2);
    let giant = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Dead of Winter");
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 2));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Snow-Covered Swamp", 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(giant), (4, 2));
}

// ---------------------------------------------------------------------------------
// On Thin Ice: "Enchant snow land you control. When this Aura enters, exile target
// creature an opponent controls until this Aura leaves the battlefield."
// ---------------------------------------------------------------------------------

#[test]
fn on_thin_ice_exiling_drops_auras_unattaches_equipment_and_loses_counters() {
    cr!("704.5m", "704.5n", "122.2", "400.7");
    ruling!(
        "On Thin Ice",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    supported("On Thin Ice");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Snow-Covered Plains");
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = attach_new(&mut t, P1, "Pacifism", obj(giant));
    let gear = attach_new(&mut t, P1, "Bonesplitter", obj(giant));
    t.g.add_counters(obj(giant), types::counters::PLUS1, 2, None);
    t.g.recompute();
    cast_targeting(&mut t, P0, "On Thin Ice", &[obj(land), obj(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(!t.on_battlefield(aura));
    assert!(t.on_battlefield(gear));
    assert_eq!(attached_to(&t, gear), None);
    let exiled = t.g.current(giant);
    assert_eq!(t.g.obj(exiled).counter("+1/+1"), 0);
}

#[test]
fn on_thin_ice_leaving_first_means_nothing_is_exiled() {
    cr!("610.3b");
    ruling!(
        "On Thin Ice",
        "If On Thin Ice leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Snow-Covered Plains");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_targeting(&mut t, P0, "On Thin Ice", &[obj(land), obj(giant)]);
    t.resolve();
    let ice = t.named_on_battlefield("On Thin Ice")[0];
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, ice, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------------
// Ember Weaver: "As long as you control a red permanent, this creature gets +1/+0 and
// has first strike."
// ---------------------------------------------------------------------------------

#[test]
fn ember_weaver_first_strike_matters_only_as_combat_damage_begins() {
    cr!("510.4");
    ruling!(
        "Ember Weaver",
        "Whether Ember Weaver has first strike matters only at the point that the normal combat damage step would begin"
    );
    supported("Ember Weaver");
    // With first strike as combat damage begins: it deals damage in the first-strike step,
    // and not again after losing first strike.
    let mut t = TestGame::new(2);
    let weaver = t.battlefield(P0, "Ember Weaver");
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(weaver, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(t.life(P1), 17);
    move_to(&mut t, goblin, Zone::Graveyard(P0));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // Without it at that point (White Knight creates the first-strike step): it deals
    // damage in the normal step, even after gaining first strike.
    let mut t = TestGame::new(2);
    let weaver = t.battlefield(P0, "Ember Weaver");
    let knight = t.battlefield(P0, "White Knight");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (weaver, Entity::Player(P1)),
            (knight, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(t.life(P1), 18);
    t.battlefield(P0, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(weaver), (3, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

// ---------------------------------------------------------------------------------
// Wandering Champion: "Whenever this creature deals combat damage to a player, if you
// control a blue or red permanent, you may discard a card. If you do, draw a card."
// ---------------------------------------------------------------------------------

#[test]
fn wandering_champion_checks_right_after_damage_and_again_on_resolution() {
    cr!("603.4", "603.10");
    ruling!(
        "Wandering Champion",
        "If the only blue or red permanent you control is a creature that’s dealt lethal damage at the same time as Wandering Champion deals combat damage, Wandering Champion’s ability triggers."
    );
    ruling!(
        "Wandering Champion",
        "Wandering Champion’s ability doesn’t trigger if you don’t control a blue or red permanent immediately after it deals combat damage to a player."
    );
    ruling!(
        "Wandering Champion",
        "If you don’t control a blue or red permanent as Wandering Champion’s ability resolves, you can’t discard and draw a card. The blue or red permanent you control as the ability resolves doesn’t have to be one you controlled as the ability triggered."
    );
    supported("Wandering Champion");
    // The Coral Merfolk (blue) dies in the same combat damage: it triggers, but with no
    // blue or red permanent on resolution, nothing happens.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Wandering Champion");
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let giant = t.battlefield(P1, "Hill Giant");
    t.hand(P0, "Lightning Bolt");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (champ, Entity::Player(P1)),
            (merfolk, Entity::Player(P1)),
        ]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, merfolk)]),
    );
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(t.in_graveyard(P0, "Coral Merfolk"));
    assert_eq!(triggered_from(&t, champ), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // No blue or red permanent after combat damage: no trigger.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Wandering Champion");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(champ, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 17);
    assert_eq!(triggered_from(&t, champ), 0);
    // A different blue permanent on resolution: discard and draw.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Wandering Champion");
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(champ, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggered_from(&t, champ), 1);
    move_to(&mut t, merfolk, Zone::Graveyard(P0));
    t.battlefield(P0, "Coral Merfolk");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 1);
}

// ---------------------------------------------------------------------------------
// Nightscape Familiar: "{1}{B}: Regenerate this creature."
// ---------------------------------------------------------------------------------

#[test]
fn nightscape_familiar_regeneration_shield_replaces_the_next_destruction() {
    cr!("701.19a", "614.8");
    ruling!(
        "Nightscape Familiar",
        "Activating the ability creates a replacement effect that acts like a shield, replacing the next time Nightscape Familiar would be destroyed that turn. This shield works against effects that try to destroy Nightscape Familiar or lethal damage that would be dealt to Nightscape Familiar."
    );
    ruling!(
        "Nightscape Familiar",
        "You can activate the regeneration ability even if Nightscape Familiar isn’t at risk of being destroyed."
    );
    supported("Murder");
    let mut t = TestGame::new(2);
    let fam = t.battlefield(P0, "Nightscape Familiar");
    let regen = |t: &mut TestGame| {
        t.activate(P0, fam, 0, &[]).expect("regenerate");
        t.resolve_all();
    };
    t.lands(P0, "Swamp", 4);
    // Nothing threatens it: the ability can still be activated, and the shield lasts.
    assert!(can_activate(&mut t, P0, fam));
    regen(&mut t);
    // Lethal damage: regenerated (tapped, damage removed).
    cast_targeting(&mut t, P1, "Lightning Bolt", &[obj(fam)]);
    t.resolve_all();
    assert!(t.on_battlefield(fam));
    assert!(tapped(&t, fam));
    assert_eq!(damage_on(&t, fam), 0);
    // A destroy effect: a new shield regenerates it.
    regen(&mut t);
    cast_targeting(&mut t, P1, "Murder", &[obj(fam)]);
    t.resolve_all();
    assert!(t.on_battlefield(fam));
    // The shield was used: the next destruction isn't replaced.
    cast_targeting(&mut t, P1, "Murder", &[obj(fam)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Nightscape Familiar"));
}
