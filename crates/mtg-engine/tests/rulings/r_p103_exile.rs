//! Rulings batch P103 — cards exiled from libraries and what's done with them: Jester's
//! Scepter, Grimoire Thief and Mindreaver (names, CR 201.2a, 709.3; looking at face-down
//! exiled cards, CR 406.3; linked abilities and new objects, CR 607.2a, 400.7), Memory
//! Vessel and Precognition Field (playing cards from exile and from the top of a library,
//! CR 401.5, 601.3, 118.9), Sanity Grinding (chroma), Thieves' Guild Enforcer's
//! deathtouch (CR 702.2b), Szadek's replacement (CR 616.1), and Chains of Mephistopheles
//! replacing a draw paid as a cost (CR 614.1a, 601.2h).

use crate::r_p103_common::*;
use crate::r_s01_common::{custom_card, stack_library, supported};
use crate::r_s02_common::{can_activate, can_cast};
use crate::r_s06_common::give_control;
use crate::r_s07_common::{cast_methods, resolved};
use crate::r_s25_common::cast_new;
use mtg_engine::ability::{AbilityKind, CostPart, Effect, PlayerRef, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

// --- Jester's Scepter -------------------------------------------------------------------

/// Jester's Scepter enters under P0's control, exiling the top five cards of P1's library
/// (`top`, top first, then fillers).
fn scepter(t: &mut TestGame, top: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    supported("Jester's Scepter");
    let cards = stack_library(t, P1, top);
    t.answer_targets(P0, &[pl(P1)]);
    let s = t.enter(P0, "Jester's Scepter");
    t.g.flush_events();
    t.resolve_all();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(t.obj_now(*c).face_down);
    }
    (s, cards)
}

#[test]
fn jesters_scepter_you_choose_the_exiled_card_to_pay_with() {
    cr!("602.2b", "406.3", "201.2a");
    ruling!(
        "Jester's Scepter",
        "Because the cards that are exiled are not hidden to you, you can choose which one to put into its owner’s graveyard when you activate the second ability."
    );
    let mut t = TestGame::new(2);
    let (s, cards) = scepter(
        &mut t,
        &["Island", "Shock", "Grizzly Bears", "Forest", "Opt"],
    );
    // P0 may look at them; P1 may not.
    assert!(cards.iter().all(|c| can_look_at(&t.g, P0, t.g.current(*c))));
    assert!(!can_look_at(&t.g, P1, t.g.current(cards[1])));
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(t.g.current(cards[1]))]);
    t.activate(P0, s, 0, &[obj(shock)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(cards[1]), Zone::Graveyard(P1));
    assert_eq!(t.zone(shock), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn jesters_scepter_counters_a_split_half_with_the_split_card() {
    cr!("709.3b", "709.4", "201.2a");
    ruling!(
        "Jester's Scepter",
        "If half of a split card is cast (for example, Hit), and you put one of those split cards into its owner’s graveyard with Jester’s Scepter (for example, Hit/Run), the spell is countered."
    );
    let mut t = TestGame::new(2);
    let (s, cards) = scepter(
        &mut t,
        &["Fire // Ice", "Island", "Island", "Island", "Island"],
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1 casts Ice ("Tap target permanent. Draw a card.").
    t.lands(P1, "Island", 2);
    let fi = t.hand(P1, "Fire // Ice");
    t.g.turn.priority = Some(P1);
    let ice = t
        .cast(P1, fi)
        .method(CastMethod::Half(1))
        .target(obj(bears))
        .go();
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(t.g.current(cards[0]))]);
    t.activate(P0, s, 0, &[obj(ice)]).unwrap();
    t.resolve_all();
    assert!(!resolved(&t, ice));
    assert!(!t.obj_now(bears).tapped);
}

// --- Grimoire Thief ---------------------------------------------------------------------

/// Grimoire Thief, controlled by P0, exiles the top three cards of P1's library (`top`).
fn thief(t: &mut TestGame, top: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    supported("Grimoire Thief");
    let thief = t.battlefield(P0, "Grimoire Thief");
    let cards = stack_library(t, P1, top);
    t.answer_targets(P0, &[pl(P1)]);
    t.g.tap(thief);
    t.g.flush_events();
    t.resolve_all();
    let cards: Vec<ObjectId> = cards.iter().map(|c| t.g.current(*c)).collect();
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(t.obj(*c).face_down);
    }
    (thief, cards)
}

#[test]
fn grimoire_thief_counters_both_halves_of_a_split_card_name() {
    cr!("709.3b", "709.4", "201.2a");
    ruling!(
        "Grimoire Thief",
        "If a split card is turned face up, the ability counters all spells with the same name as either side of that split card."
    );
    let mut t = TestGame::new(2);
    let (thief, _) = thief(&mut t, &["Fire // Ice", "Island", "Island"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Island", 1);
    t.lands(P1, "Mountain", 3);
    let a = t.hand(P1, "Fire // Ice");
    let b = t.hand(P1, "Fire // Ice");
    t.g.turn.priority = Some(P1);
    let ice = t
        .cast(P1, a)
        .method(CastMethod::Half(1))
        .target(obj(bears))
        .go();
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![2]));
    let fire = t
        .cast(P1, b)
        .method(CastMethod::Half(0))
        .target(pl(P0))
        .go();
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.lands(P0, "Island", 1);
    t.activate(P0, thief, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!resolved(&t, ice) && !resolved(&t, fire));
    assert!(resolved(&t, shock));
    assert_eq!(t.life(P0), 18, "only Shock");
}

#[test]
fn grimoire_thiefs_new_controller_may_look_and_lookers_keep_looking() {
    cr!("406.3", "607.2a");
    ruling!(
        "Grimoire Thief",
        "Once a player gains control of Grimoire Thief, that player can look at all cards it exiled, even if those cards were exiled while someone else controlled Grimoire Thief."
    );
    ruling!(
        "Grimoire Thief",
        "Once a player is allowed to look at the exiled face-down cards, that player may continue to do so even after Grimoire Thief leaves the battlefield or another player gains control of it."
    );
    let mut t = TestGame::new(3);
    let (thief, cards) = thief(&mut t, &["Shock", "Island", "Opt"]);
    assert!(cards.iter().all(|c| can_look_at(&t.g, P0, *c)));
    assert!(!can_look_at(&t.g, P2, cards[0]));
    // P2 gains control: P2 may look at them too; P0 still may.
    give_control(&mut t, thief, P2);
    assert!(cards.iter().all(|c| can_look_at(&t.g, P2, *c)));
    assert!(cards.iter().all(|c| can_look_at(&t.g, P0, *c)));
    // The Thief leaves the battlefield: both still may; P1 never could.
    crate::r_p189_common::bounce(&mut t, thief);
    t.settle();
    assert!(cards.iter().all(|c| can_look_at(&t.g, P0, *c)));
    assert!(cards.iter().all(|c| can_look_at(&t.g, P2, *c)));
    assert!(!can_look_at(&t.g, P1, cards[0]));
}

// --- Mindreaver -------------------------------------------------------------------------

#[test]
fn a_new_mindreaver_doesnt_know_the_old_ones_cards() {
    cr!("400.7", "607.2a");
    ruling!(
        "Mindreaver",
        "If Mindreaver leaves the battlefield, and later another Mindreaver enters the battlefield, it is a new object (even if the two were represented by the same card). The last ability of the second Mindreaver doesn’t refer to any cards exiled with the first one."
    );
    supported("Mindreaver");
    for again in [false, true] {
        let mut t = TestGame::new(2);
        let mr = t.battlefield(P0, "Mindreaver");
        stack_library(&mut t, P1, &["Shock", "Island", "Island"]);
        cast_new(&mut t, P0, "Giant Growth", &[obj(mr)]);
        t.answer_targets(P0, &[pl(P1)]);
        t.resolve_all();
        assert!(t.in_exile("Shock"));
        let mr = if again {
            // The same card leaves and comes back: a new object.
            crate::r_p189_common::bounce(&mut t, mr);
            let card = t.g.current(mr);
            t.lands(P0, "Island", 2);
            t.cast(P0, card).go();
            t.resolve_all();
            t.named_on_battlefield("Mindreaver")[0]
        } else {
            mr
        };
        let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
        t.lands(P0, "Island", 2);
        let r = t.activate(P0, mr, 0, &[obj(shock)]);
        assert_eq!(r.is_ok(), !again);
        t.resolve_all();
        assert_eq!(t.life(P0), if again { 18 } else { 20 });
    }
}

// --- Memory Vessel ----------------------------------------------------------------------

#[test]
fn memory_vessel_plays_only_cards_exiled_from_libraries() {
    cr!("601.3", "101.2");
    ruling!(
        "Memory Vessel",
        "players can play only cards they exiled from their library during the effect of the activated ability, not the Memory Vessel that was exiled to activate the ability."
    );
    supported("Memory Vessel");
    let mut t = TestGame::new(2);
    let vessel = t.battlefield(P0, "Memory Vessel");
    let bears = t.library_top(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    t.activate(P0, vessel, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(vessel), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, vessel, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, in_hand, CastMethod::Normal));
}

// --- Precognition Field -----------------------------------------------------------------

#[test]
fn precognition_field_lets_you_look_at_the_top_card_any_time() {
    cr!("401.2", "401.5");
    ruling!(
        "Precognition Field",
        "Precognition Field lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don’t have priority."
    );
    supported("Precognition Field");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Precognition Field");
    t.settle();
    // On P1's turn, with P1 to act.
    t.set_step(P1, Step::Upkeep);
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
}

#[test]
fn precognition_field_exiles_whatever_is_on_top_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Precognition Field",
        "The card you’ll exile from the top of your library is determined as Precognition Field’s last ability resolves. This might not be the card that was on top of your library when you activated that ability."
    );
    let mut t = TestGame::new(2);
    let field = t.battlefield(P0, "Precognition Field");
    let first = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, field, 0, &[]).unwrap();
    let second = t.library_top(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.zone(second), Zone::Exile);
    assert_eq!(t.zone(first), Zone::Library(P0));
}

#[test]
fn precognition_fields_top_card_isnt_in_your_hand() {
    cr!("401.2", "702.29a", "602.2");
    ruling!(
        "Precognition Field",
        "The top card of your library isn’t in your hand, so you can’t cycle it, discard it, or activate any of its activated abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Precognition Field");
    // Censor: an instant with cycling {U}.
    let censor = t.library_top(P0, "Censor");
    t.lands(P0, "Island", 3);
    assert!(!can_activate(&mut t, P0, censor));
    // It can be cast (it's an instant) with a spell to counter.
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    assert!(can_cast(&mut t, P0, censor, CastMethod::Normal));
    t.resolve_all();
    // A discard can't take it either.
    t.answer_choose(P0, &[obj(censor)]);
    t.hand(P0, "Island");
    let from = t.asked().len();
    t.g.turn.priority = Some(P0);
    cast_new(&mut t, P0, "Mind Rot", &[pl(P0)]);
    t.resolve();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(offered.iter().all(|c| !c.contains(&obj(censor))));
    assert_eq!(t.zone(censor), Zone::Library(P0));
}

#[test]
fn precognition_field_casts_pay_all_costs_or_alternative_ones() {
    cr!("601.2b", "601.2f", "118.9");
    ruling!(
        "Precognition Field",
        "You’ll still pay all costs for that spell, including additional costs. You may also pay alternative costs such as that granted by Jodah, Archmage Eternal."
    );
    // Kicked Burst Lightning from the top: {R} + {4}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Precognition Field");
    let burst = t.library_top(P0, "Burst Lightning");
    let lands = t.lands(P0, "Mountain", 5);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, burst).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    // Jodah's alternative cost is offered for a spell cast from the top.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Precognition Field");
    t.battlefield(P0, "Jodah, Archmage Eternal");
    let shock = t.library_top(P0, "Shock");
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, land, 1);
    }
    let methods = cast_methods(&mut t, P0, shock);
    assert!(methods.contains(&CastMethod::Normal));
    assert!(methods
        .iter()
        .any(|m| matches!(m, CastMethod::Alternative(_))));
}

// --- Sanity Grinding --------------------------------------------------------------------

#[test]
fn sanity_grinding_with_a_short_library_reveals_and_reorders_it_all() {
    cr!("609.3", "701.20a");
    ruling!(
        "Sanity Grinding",
        "If there are fewer than ten cards in your library, reveal them all. For each blue mana symbol in those cards’ mana costs, the targeted opponent puts the top card of their library into their graveyard. Then you reorder your entire library as you choose."
    );
    supported("Sanity Grinding");
    let mut t = TestGame::new(2);
    library_of(&mut t, P0, "Island", 0);
    stack_library(&mut t, P0, &["Counterspell", "Island", "Opt"]);
    // Opt, Island, Counterspell from the top afterwards.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 1, 0]));
    cast_new(&mut t, P0, "Sanity Grinding", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.library_size(P0), 3);
    assert_eq!(library_names(&t, P0), vec!["Opt", "Island", "Counterspell"]);
}

// --- Thieves' Guild Enforcer ------------------------------------------------------------

#[test]
fn thieves_guild_enforcer_deathtouch_gained_after_damage_doesnt_apply() {
    cr!("702.2b", "704.5h", "510.2");
    ruling!(
        "Thieves' Guild Enforcer",
        "If Thieves' Guild Enforcer gains deathtouch only after dealing damage (most likely because another creature was dealt lethal damage at the same time that Thieves' Guild Enforcer dealt damage), the damage it dealt won't be from a source with deathtouch."
    );
    supported("Thieves' Guild Enforcer");
    let mut t = TestGame::new(2);
    let enforcer = t.battlefield(P0, "Thieves' Guild Enforcer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..7 {
        t.graveyard(P1, "Island");
    }
    // P1 blocks the Enforcer with a 0/8 wall and the Bears with a 1/1: the 1/1 dies,
    // giving P1 an eighth card in the graveyard.
    let wall = t.battlefield(P1, "Wall of Stone");
    let goblin = t.battlefield(P1, "Raging Goblin");
    crate::r_s01_common::attack_with(&mut t, &[(enforcer, pl(P1)), (bears, pl(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(wall, enforcer), (goblin, bears)]);
    assert_eq!(t.zone(goblin), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P1), 8);
    // The Enforcer has deathtouch now, but the wall took non-deathtouch damage.
    assert!(t
        .obj_now(enforcer)
        .has_keyword(mtg_engine::keywords::KeywordKind::Deathtouch));
    assert!(t.on_battlefield(wall));
}

// --- Szadek, Lord of Secrets ------------------------------------------------------------

#[test]
fn szadek_and_a_prevention_shield_the_player_chooses_the_order() {
    cr!("616.1", "616.1e", "616.1f", "615.7");
    ruling!(
        "Szadek, Lord of Secrets",
        "If another effect would prevent Szadek's combat damage from being dealt to the defending player or replace it with something else, that player chooses which effect applies first. For example, the player can choose to first have Mending Hands prevent 4 of the damage and then apply Szadek's ability so that it gets only one counter."
    );
    supported("Szadek, Lord of Secrets");
    let mut t = TestGame::new(2);
    let szadek = t.battlefield(P0, "Szadek, Lord of Secrets");
    // Mending Hands: "Prevent the next 4 damage that would be dealt to any target this
    // turn."
    cast_new(&mut t, P1, "Mending Hands", &[pl(P1)]);
    t.resolve_all();
    let from = t.asked().len();
    // P1 applies the prevention first.
    prefer_replacement(&mut t, P1, "Mending Hands");
    t.attack(&[(szadek, pl(P1))], &[]);
    t.resolve_all();
    let asked: Vec<(PlayerId, Decision)> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
        .cloned()
        .collect();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, P1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(szadek, "+1/+1"), 1);
    assert_eq!(
        t.graveyard_size(P1),
        1 + 1,
        "Mending Hands and one milled card"
    );
    // P1 applies Szadek's replacement first instead: all five damage becomes counters and
    // milled cards, and the shield is left for later damage.
    let mut t = TestGame::new(2);
    let szadek = t.battlefield(P0, "Szadek, Lord of Secrets");
    cast_new(&mut t, P1, "Mending Hands", &[pl(P1)]);
    t.resolve_all();
    prefer_replacement(&mut t, P1, "Szadek");
    t.attack(&[(szadek, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(szadek, "+1/+1"), 5);
    assert_eq!(
        t.graveyard_size(P1),
        1 + 5,
        "Mending Hands and five milled cards"
    );
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "the shield prevents the Shock's damage");
}

// --- Chains of Mephistopheles -----------------------------------------------------------

/// A custom artifact with "Draw a card: You gain 1 life." (no real card has a draw as a
/// cost; Chains of Mephistopheles's ruling is about one).
fn drawing_cost_artifact() -> mtg_engine::card::CardDef {
    let mut def = custom_card("Drawer", "Artifact", "{1}", None, "{1}: You gain 1 life.");
    let face = &mut def.faces[0];
    for a in face.chars.abilities.iter_mut() {
        let a = Arc::make_mut(a);
        if let AbilityKind::Activated(act) = &mut a.kind {
            act.cost.mana = None;
            act.cost.parts = vec![CostPart::Effect(Box::new(Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(1),
            }))];
        }
    }
    def
}

#[test]
fn chains_of_mephistopheles_replaces_a_draw_paid_as_a_cost() {
    cr!("614.1a", "602.2b", "601.2h");
    ruling!(
        "Chains of Mephistopheles",
        "Cards which are drawn as a cost are affected by this card because replacement effects can alter the payment of costs."
    );
    supported("Chains of Mephistopheles");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Chains of Mephistopheles");
    let drawer = t.custom(P0, drawing_cost_artifact(), Zone::Battlefield);
    let bears = t.hand(P0, "Grizzly Bears");
    let top = t.library_top(P0, "Island");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, drawer, 0, &[]).unwrap();
    // The draw was replaced: discard the Bears, then draw the Island.
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(top), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // With an empty hand, the player mills a card instead.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Chains of Mephistopheles");
    let drawer = t.custom(P0, drawing_cost_artifact(), Zone::Battlefield);
    let top = t.library_top(P0, "Island");
    t.activate(P0, drawer, 0, &[]).unwrap();
    assert_eq!(t.zone(top), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 0);
}
