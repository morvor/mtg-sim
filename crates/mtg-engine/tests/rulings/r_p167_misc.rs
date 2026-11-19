//! Rulings batch P167 — fliers (Archon of Redemption's last known power, Watcher of the
//! Spheres' cost order, Rally of Wings' locked-in set), foretell given by Dream Devourer
//! and Cosmos Charger (CR 702.143), exile triggers (Rakshasa Vizier, Slagstone Refinery,
//! Stonebinder's Familiar), casting from a graveyard (Burning Vengeance, Secrets of the
//! Dead), explore triggers, and Food and Clue tokens.

use crate::r_p194_common::{assert_total_cost, mana};
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, can_play_land, destroy, target_candidates};
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s08_common::mana_value;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);
const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Fliers
// ---------------------------------------------------------------------------------------

#[test]
fn errant_and_giada_looks_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Errant and Giada",
        "Errant and Giada lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don’t have priority. This action doesn’t use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    t.battlefield(P0, "Errant and Giada");
    t.g.recompute();
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, top));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn celestial_enforcer_activation_cant_be_interrupted() {
    cr!("602.2", "602.5b", "117.1");
    ruling!(
        "Celestial Enforcer",
        "Once you begin activating Celestial Enforcer's ability, players can't take any other actions until you're done activating it. Notably, they can't try to remove your creatures with flying to make it illegal to activate the ability."
    );
    let mut t = TestGame::new(2);
    let ce = t.battlefield(P0, "Celestial Enforcer");
    t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let from = t.asked().len();
    t.activate(P0, ce, 0, &[obj(bears)]).unwrap();
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
    // Removing the flier afterwards doesn't stop it.
    let drake = t.named_on_battlefield("Wind Drake")[0];
    destroy(&mut t, drake);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    // Without a flier, it can't be activated.
    let mut t = TestGame::new(2);
    let ce = t.battlefield(P0, "Celestial Enforcer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(t.activate(P0, ce, 0, &[obj(bears)]).is_err());
}

#[test]
fn rally_of_wings_affects_the_fliers_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Rally of Wings",
        "Rally of Wings affects only creatures you control at the time it resolves. Creatures that gain flying later in the turn or that you begin to control later in the turn won't get +2/+2."
    );
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(drake);
    let rally = t.hand(P0, "Rally of Wings");
    give_mana_for(&mut t, P0, "Rally of Wings");
    t.cast(P0, rally).go();
    t.resolve_all();
    assert!(!t.obj_now(drake).tapped);
    assert_eq!(t.pt(drake), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
    // Later: a flier arriving, and the Bears gaining flying (Jump).
    let later = t.battlefield(P0, "Storm Crow");
    let jump = t.hand(P0, "Jump");
    mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, jump).target(obj(bears)).go();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(later), (1, 2));
}

#[test]
fn flying_restatements_table() {
    cr!("113.6", "115.1", "702.9a");
    ruling!(
        "Skycat Sovereign",
        "Skycat Sovereign's first ability applies only while it's on the battlefield."
    );
    ruling!(
        "Spire Mangler",
        "Spire Mangler can be the target of its own ability."
    );
    ruling!(
        "Flame Sweep",
        "The only creatures that avoid Flame Sweep’s damage are those you control with flying. The set of creatures dealt damage includes creatures you don’t control with flying and creatures you control without flying."
    );
    // Skycat Sovereign in a hand and in a graveyard, with two fliers on the battlefield:
    // 1/1; on the battlefield, 3/3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wind Drake");
    t.battlefield(P0, "Storm Crow");
    let in_hand = t.hand(P0, "Skycat Sovereign");
    let in_gy = t.graveyard(P0, "Skycat Sovereign");
    let on_bf = t.battlefield(P0, "Skycat Sovereign");
    assert_eq!(t.pt(in_hand), (1, 1));
    assert_eq!(t.pt(in_gy), (1, 1));
    assert_eq!(t.pt(on_bf), (3, 3));
    // Spire Mangler targets itself.
    let mut t = TestGame::new(2);
    let sm = t.enter(P0, "Spire Mangler");
    t.g.flush_events();
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(sm)]);
    t.settle();
    assert!(target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&obj(sm))));
    t.resolve_all();
    assert_eq!(t.pt(sm), (4, 1));
    // Flame Sweep.
    let mut t = TestGame::new(2);
    let my_flier = t.battlefield(P0, "Wind Drake");
    let my_walker = t.battlefield(P0, "Hill Giant");
    let their_flier = t.battlefield(P1, "Serra Angel");
    let sweep = t.hand(P0, "Flame Sweep");
    give_mana_for(&mut t, P0, "Flame Sweep");
    t.cast(P0, sweep).go();
    t.resolve_all();
    assert_eq!(t.obj_now(my_flier).damage, 0);
    assert_eq!(t.obj_now(my_walker).damage, 2);
    assert_eq!(t.obj_now(their_flier).damage, 2);
}

#[test]
fn isperia_names_the_card_as_it_resolves_with_no_window_to_play_it() {
    cr!("608.2", "117.1", "701.20a");
    ruling!(
        "Isperia the Inscrutable",
        "The card is named when the ability resolves. The named card can't be played after it's named but before the player's hand is revealed."
    );
    let mut t = TestGame::new(2);
    let isperia = t.battlefield(P0, "Isperia the Inscrutable");
    t.hand(P1, "Grizzly Bears");
    let drake = t.library_top(P0, "Wind Drake");
    attack_with(&mut t, &[(isperia, Entity::Player(P1))]);
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_choose(P0, &[obj(drake)]);
    let from = t.asked().len();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    let asked = &t.asked()[from..];
    let named = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::NameCard { .. }))
        .expect("a card is named as it resolves");
    // No one gets priority between naming the card and the end of the ability.
    let after = &asked[named..];
    let next_priority = after
        .iter()
        .position(|(_, d)| matches!(d, Decision::Priority { .. }))
        .unwrap_or(after.len());
    assert!(after[..next_priority]
        .iter()
        .all(|(p, d)| *p == P0 || !matches!(d, Decision::Priority { .. })));
    assert!(t.in_hand(P0, "Wind Drake"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn archon_of_redemption_uses_power_on_resolution_or_last_known() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Archon of Redemption",
        "The creature’s power is determined as the triggered ability resolves, so it will take into account any effects that modify it. If the creature has left the battlefield by the time the triggered ability resolves, its last existence on the battlefield is checked to determine its power."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Archon of Redemption");
        let drake = enter(&mut t, P0, "Wind Drake");
        assert_eq!(t.stack_len(), 1);
        // Giant Growth on the Drake: power 5.
        let gg = t.hand(P0, "Giant Growth");
        mana(&mut t, P0, ManaType::G, 1);
        t.cast(P0, gg).target(obj(drake)).go();
        t.resolve();
        if leaves {
            destroy(&mut t, drake);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.life(P0), 25, "{leaves}");
    }
}

#[test]
fn watcher_of_the_spheres_applies_increases_before_reductions() {
    cr!("601.2f", "202.3");
    ruling!(
        "Watcher of the Spheres",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Watcher of the Spheres). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    // Suntail Hawk ({W}, flying) with Sphere of Resistance ({1} more): {1}{W} - {1} = {W}.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Watcher of the Spheres");
        t.battlefield(P1, "Sphere of Resistance");
    };
    assert_total_cost(&setup, P0, "Suntail Hawk", &[(ManaType::W, 1)], 0);
    let mut t = TestGame::new(2);
    setup(&mut t);
    let hawk = t.hand(P0, "Suntail Hawk");
    mana(&mut t, P0, ManaType::W, 1);
    let spell = t.cast(P0, hawk).go();
    assert_eq!(mana_value(&t, spell), 1);
    // Serra Angel ({3}{W}{W}): {4}{W}{W} - {1}: the same total, mana value 5.
    assert_total_cost(&setup, P0, "Serra Angel", &[(ManaType::W, 2)], 3);
}

// ---------------------------------------------------------------------------------------
// Foretell: Dream Devourer, Cosmos Charger, Lifestream's Blessing
// ---------------------------------------------------------------------------------------

/// Foretells `card` from `p`'s hand with `generic` mana for it; the card in exile.
fn foretell_for(t: &mut TestGame, p: PlayerId, card: ObjectId, generic: u32) -> ObjectId {
    mana(t, p, ManaType::C, generic);
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(SpecialAction::Foretell { card }))
        .expect("foretell");
    t.g.flush_events();
    t.settle();
    let new = t.g.current(card);
    assert_eq!(t.zone(new), Zone::Exile);
    new
}

#[test]
fn dream_devourer_foretold_cards_keep_their_given_cost_without_it() {
    cr!("702.143a", "702.143d", "611.2c");
    ruling!(
        "Dream Devourer",
        "Cards that are foretold by virtue of Dream Devourer’s ability won’t continue to have the foretell ability once they leave your hand, but you can still cast them from exile on a later turn for the foretell costs they were given by Dream Devourer."
    );
    ruling!(
        "Dream Devourer",
        "It doesn’t matter if Dream Devourer is still on the battlefield or still under your control as you cast the spell from exile."
    );
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let dd = t.battlefield(P0, "Dream Devourer");
        let giant = t.hand(P0, "Hill Giant");
        let card = foretell_for(&mut t, P0, giant, 2);
        t.resolve_all();
        if gone {
            destroy(&mut t, dd);
        }
        assert!(!can_cast(&mut t, P0, card, FORETELL), "not this turn");
        t.advance_to(P0, Step::Upkeep);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        // Hill Giant ({3}{R}) reduced by {2}: {1}{R}.
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::C, 1);
        assert!(can_cast(&mut t, P0, card, FORETELL), "{gone}");
        t.cast(P0, card).method(FORETELL).go();
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    }
}

#[test]
fn dream_devourer_triggers_on_foretelling_not_on_casting() {
    cr!("702.143a", "603.2");
    ruling!(
        "Dream Devourer",
        "The last ability triggers when you exile a card from your hand using foretell, not when you cast a foretold card from exile."
    );
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Dream Devourer");
    let giant = t.hand(P0, "Hill Giant");
    let card = foretell_for(&mut t, P0, giant, 2);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(dd), (2, 3));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.pt(dd), (0, 3));
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, card).method(FORETELL).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger on casting it");
    t.resolve_all();
    assert_eq!(t.pt(dd), (0, 3));
}

#[test]
fn dream_devourer_modal_double_faced_cards() {
    cr!("702.143a", "712.12", "712.13", "305.9");
    ruling!(
        "Dream Devourer",
        "If you foretell a modal double-faced card, the foretell cost will be based on the mana cost of the face you cast from exile. For example, if you foretell Kolvori, God of Kinship/The Ringhart Crest, you can cast Kolvori by paying {G}{G} or The Ringhart Crest by paying {G} on a future turn. If you foretell a modal double-faced card whose front face is a nonland face but whose back face is a land face, you can’t play that card as a land."
    );
    // (face cast, mana)
    for (back, cost) in [(false, 2u32), (true, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dream Devourer");
        let k = t.hand(P0, "Kolvori, God of Kinship // The Ringhart Crest");
        let card = foretell_for(&mut t, P0, k, 2);
        t.resolve_all();
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        mana(&mut t, P0, ManaType::G, cost);
        // The face: the front (index 0) or the back (index 1).
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(if back { 1 } else { 0 }),
        );
        t.cast(P0, card).method(FORETELL).go();
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{back}");
        t.resolve_all();
        let name = if back {
            "The Ringhart Crest"
        } else {
            "Kolvori, God of Kinship"
        };
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{back}");
    }
    // A land back face can't be played.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dream Devourer");
    let m = t.hand(P0, "Kazandu Mammoth // Kazandu Valley");
    let card = foretell_for(&mut t, P0, m, 2);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, card));
    assert!(t.play_land(P0, card).is_err());
}

#[test]
fn cosmos_charger_reduces_foretelling_not_casting() {
    cr!("702.143a", "601.2f");
    ruling!(
        "Cosmos Charger",
        "The cost reduction refers to the cost you pay as you exile the card from your hand face down, not the cost you pay as you cast the spell from exile."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cosmos Charger");
    // Demon Bolt: foretell {E}... ({2}{R}; foretell {R}).
    let bolt = t.hand(P0, "Demon Bolt");
    let card = foretell_for(&mut t, P0, bolt, 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.advance_to(P1, Step::Upkeep);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Its foretell cost {R} isn't reduced.
    assert!(!can_cast(&mut t, P0, card, FORETELL));
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, card).method(FORETELL).target(obj(bears)).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn lifestreams_blessing_x_is_locked_in_as_it_is_cast() {
    cr!("601.2", "608.2h", "702.143a");
    ruling!(
        "Lifestream's Blessing",
        "The value of X is determined only once, as you cast Lifestream's Blessing. It doesn't matter what happens to your creatures after this; the value of X won't change."
    );
    // Cast from exile (foretold): X = 3 (Hill Giant), and the Giant dies before it
    // resolves: draw 3, gain 6.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let lb = t.hand(P0, "Lifestream's Blessing");
    let card = foretell_for(&mut t, P0, lb, 2);
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, card).method(FORETELL).go();
    destroy(&mut t, giant);
    t.battlefield(P0, "Craw Wurm");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.life(P0), 26);
}

// ---------------------------------------------------------------------------------------
// Exile triggers
// ---------------------------------------------------------------------------------------

#[test]
fn rakshasa_vizier_delve_is_one_event_and_instead_isnt_from_the_graveyard() {
    cr!("702.66a", "603.2c", "614.6");
    ruling!(
        "Rakshasa Vizier",
        "Exiling cards from your graveyard to pay the cost of a spell with delve is a single action. It will cause Rakshasa Vizier’s ability to trigger once, putting a number of +1/+1 counters on it equal to the number of cards you exiled."
    );
    ruling!(
        "Rakshasa Vizier",
        "If a card is exiled “instead” of being put into a graveyard, Rakshasa Vizier’s ability won’t trigger."
    );
    // Treasure Cruise ({7}{U}, delve) delving four cards.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Rakshasa Vizier");
    let gy: Vec<ObjectId> = (0..4).map(|_| t.graveyard(P0, "Shock")).collect();
    let cruise = t.hand(P0, "Treasure Cruise");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 3);
    let pick: Vec<Entity> = gy.iter().map(|g| obj(*g)).collect();
    t.answer_choose(P0, &pick);
    t.cast(P0, cruise).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 4);
    // Rest in Peace: a creature dying is exiled instead: no trigger.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Rakshasa Vizier");
    t.battlefield(P0, "Rest in Peace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(v, counters::PLUS1), 0);
}

#[test]
fn slagstone_refinery_triggers_for_each_artifact_leaving_with_it() {
    cr!("603.10a", "603.2");
    ruling!(
        "Slagstone Refinery",
        "If Slagstone Refinery is put into a graveyard from the battlefield or exiled from the battlefield at the same time that one or more other nontoken artifacts controlled by the same player are put into a graveyard from the battlefield and/or exiled from the battlefield, its triggered ability will trigger once for each of them plus once for itself."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slagstone Refinery");
    t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Ornithopter");
    // Shatterstorm: "Destroy all artifacts. They can't be regenerated."
    let storm = t.hand(P1, "Shatterstorm");
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Shatterstorm");
    t.cast(P1, storm).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Powerstone").len(), 3);
}

#[test]
fn stonebinders_familiar_triggers_once_for_one_event() {
    cr!("603.2c", "603.2h");
    ruling!(
        "Stonebinder's Familiar",
        "If multiple cards are put into exile as part of the same event on your turn, the triggered ability will trigger only once."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Stonebinder's Familiar");
    let gy: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P1, "Shock")).collect();
    crate::r_p146_common::exile_together(&mut t, &gy);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(f, counters::PLUS1), 1);
    // Another exile later this turn: no second trigger.
    let more = t.graveyard(P1, "Shock");
    crate::r_p146_common::exile_together(&mut t, &[more]);
    assert_eq!(t.stack_len(), 0);
}

// ---------------------------------------------------------------------------------------
// Casting from a graveyard
// ---------------------------------------------------------------------------------------

#[test]
fn burning_vengeance_and_secrets_of_the_dead_resolve_first_and_ignore_abilities() {
    cr!("603.3", "405.5", "702.34a", "602.1");
    ruling!(
        "Burning Vengeance",
        "Burning Vengeance’s triggered ability will resolve before the spell you cast from your graveyard."
    );
    ruling!(
        "Secrets of the Dead",
        "Secrets of the Dead’s triggered ability resolves before the spell you cast from your graveyard does."
    );
    ruling!(
        "Burning Vengeance",
        "Burning Vengeance doesn’t trigger when you activate an ability of a card in your graveyard, such as unearth or the ability of Reassembling Skeleton."
    );
    // Think Twice's flashback ({2}{U}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Burning Vengeance");
    t.battlefield(P0, "Secrets of the Dead");
    let tt = t.graveyard(P0, "Think Twice");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, tt).method(FLASHBACK).go();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    assert_eq!(t.g.stack.first(), Some(&spell));
    let hand = t.hand_size(P0);
    // The two triggers resolve first: 2 damage and a card, then Think Twice's card.
    t.resolve();
    t.resolve();
    assert_eq!(t.g.stack.as_slice(), &[spell]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Reassembling Skeleton's ability from the graveyard: no trigger.
    let skel = t.graveyard(P0, "Reassembling Skeleton");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, skel, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Explore
// ---------------------------------------------------------------------------------------

#[test]
fn wildgrowth_walker_gone_still_gains_life() {
    cr!("608.2b", "701.44a");
    ruling!(
        "Wildgrowth Walker",
        "If Wildgrowth Walker leaves the battlefield while its triggered ability is on the stack, you won’t put a +1/+1 counter on anything but you will gain 3 life."
    );
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let ww = t.battlefield(P0, "Wildgrowth Walker");
        t.library_top(P0, "Forest");
        // Merfolk Branchwalker: "When this creature enters, it explores."
        enter(&mut t, P0, "Merfolk Branchwalker");
        t.resolve();
        assert_eq!(t.stack_len(), 1, "the Walker's trigger");
        if gone {
            destroy(&mut t, ww);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 23, "{gone}");
        if !gone {
            assert_eq!(t.counters(ww, counters::PLUS1), 1);
        }
    }
}

#[test]
fn lurking_chupacabra_targets_after_the_explore() {
    cr!("603.3d", "701.44a");
    ruling!(
        "Lurking Chupacabra",
        "You don’t choose a target for Lurking Chupacabra’s ability until after your creature has finished exploring."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lurking Chupacabra");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Forest");
    enter(&mut t, P0, "Merfolk Branchwalker");
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    // The land went to the hand (explore done) before the target was chosen.
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.stack_len(), 1);
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Food and Clues
// ---------------------------------------------------------------------------------------

#[test]
fn academy_manufactor_clues_arent_investigating() {
    cr!("701.16a", "614.1a", "603.2");
    ruling!(
        "Academy Manufactor",
        "Creating a Clue token this way isn't the same thing as investigating. Notably, an ability that triggers whenever you investigate won't trigger because you created a Clue token due to this replacement effect. If the original token creation was because you investigated once, those abilities will still each trigger once, no matter how many Clue tokens you end up creating."
    );
    // Gilded Goose makes a Food: a Clue too, but no investigating (Erdwal Illuminator).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Academy Manufactor");
    t.battlefield(P0, "Erdwal Illuminator");
    enter(&mut t, P0, "Gilded Goose");
    t.resolve_all();
    for s in ["Clue", "Food", "Treasure"] {
        assert_eq!(tokens_with_subtype(&t, P0, s).len(), 1, "{s}");
    }
    // Thraben Inspector investigates once: Erdwal triggers once (one more investigation).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Academy Manufactor");
    t.battlefield(P0, "Erdwal Illuminator");
    enter(&mut t, P0, "Thraben Inspector");
    t.resolve();
    assert_eq!(tokens_with_subtype(&t, P0, "Clue").len(), 1);
    assert_eq!(triggers_on_stack(&t, "investigate"), 1);
    t.resolve_all();
    for s in ["Clue", "Food", "Treasure"] {
        assert_eq!(tokens_with_subtype(&t, P0, s).len(), 2, "{s}");
    }
}

#[test]
fn hobbits_sting_counts_a_food_creature_twice() {
    cr!("608.2h", "205.3g");
    ruling!(
        "Hobbit's Sting",
        "If a Food you control is also a creature, it will be counted twice."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gingerbrute");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let sting = t.hand(P0, "Hobbit's Sting");
    give_mana_for(&mut t, P0, "Hobbit's Sting");
    t.cast(P0, sting).target(obj(wurm)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
}
