//! Rulings batch P184 — Goblin typal cards: lords (CR 613.4c), generic-only cost
//! reductions (CR 601.2f), dies triggers looking back (CR 603.10a), counts made as an
//! ability resolves (CR 608.2h), intervening "if" clauses (CR 603.4), tokens put onto the
//! battlefield attacking (CR 508.4), "[subtype] card" (CR 205.3), and sacrifice costs.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s03_common::choice_candidates;
use crate::r_s04_common::ability_targets;
use crate::r_s12_common::attack_target;
use crate::r_s25_common::cast_new;
use mtg_engine::events::MoveCause;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn two_goblin_kings_pump_each_other() {
    cr!("613.4c", "611.3a");
    ruling!(
        "Goblin King",
        "Goblin King now has the Goblin creature type and its ability has been reworded to affect *other* Goblins. This means that if two Goblin Kings are on the battlefield, each gives the other a bonus."
    );
    supported("Goblin King");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin King");
    assert_eq!(t.pt(a), (2, 2));
    let b = t.battlefield(P1, "Goblin King");
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
}

#[test]
fn goblin_warchief_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Goblin Warchief",
        "Goblin Warchief’s effect reduces only generic mana in the cost of Goblin spells you cast. For example, it doesn’t reduce the cost of Skirk Prospector below {R}."
    );
    supported("Goblin Warchief");
    supported("Skirk Prospector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Warchief");
    let prospector = t.hand(P0, "Skirk Prospector");
    assert!(!can_cast(&mut t, P0, prospector, CastMethod::Normal));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, prospector, CastMethod::Normal));
    // Goblin King ({1}{R}{R}) costs {R}{R}.
    let king = t.hand(P0, "Goblin King");
    assert!(!can_cast(&mut t, P0, king, CastMethod::Normal));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, king, CastMethod::Normal));
}

#[test]
fn goblin_dies_triggers_see_goblins_dying_with_the_source() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Boggart Cursecrafter",
        "If Boggart Cursecrafter dies at the same time as one or more other Goblins you control, its last ability will trigger for each of those other Goblins."
    );
    ruling!(
        "Boggart Mischief",
        "If Boggart Mischief is put into a graveyard from the battlefield at the same time as one or more Goblin creatures you control die, its last ability will trigger for each of those Goblin creatures."
    );
    supported("Boggart Cursecrafter");
    supported("Boggart Mischief");
    supported("Wrath of God");
    supported("Akroma's Vengeance");
    // Cursecrafter: dies with two other Goblins to Wrath of God.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boggart Cursecrafter");
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Mischief: an enchantment destroyed by Akroma's Vengeance with two Goblins.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boggart Mischief");
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    cast_new(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Boggart Mischief"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn goblin_matron_finds_only_cards_with_the_goblin_subtype() {
    cr!("205.3", "701.23a");
    ruling!(
        "Goblin Matron",
        "If an effect refers to a “[subtype] spell” or “[subtype] card,” it refers only to a spell or card that has that subtype. For example, Goblin War Party is a card that references and creates Goblins, but it isn't a Goblin card."
    );
    supported("Goblin Matron");
    let mut t = TestGame::new(2);
    let party = t.library_top(P0, "Goblin War Party");
    let king = t.library_top(P0, "Goblin King");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.enter(P0, "Goblin Matron");
    t.g.flush_events();
    t.resolve_all();
    let offered: Vec<Entity> = choice_candidates(&t, from, "").concat();
    assert!(offered.contains(&Entity::Object(king)), "{offered:?}");
    assert!(!offered.contains(&Entity::Object(party)));
    assert!(t.in_hand(P0, "Goblin King"));
    assert!(!t.in_hand(P0, "Goblin War Party"));
}

#[test]
fn goblin_pyromancer_destroys_goblins_only_if_it_is_still_there() {
    cr!("113.6", "603.2", "513.1");
    ruling!(
        "Goblin Pyromancer",
        "It only destroys goblins if it is on the battlefield at the end of turn."
    );
    supported("Goblin Pyromancer");
    for stays in [true, false] {
        let mut t = TestGame::new(2);
        let pyro = t.battlefield(P0, "Goblin Pyromancer");
        let goblin = t.battlefield(P0, "Raging Goblin");
        if !stays {
            destroy(&mut t, pyro);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(goblin), !stays);
    }
}

#[test]
fn goblin_piledriver_counts_other_attacking_goblins_as_it_resolves() {
    cr!("608.2h", "508.1m");
    ruling!(
        "Goblin Piledriver",
        "The number of Goblins is counted when this ability resolves."
    );
    supported("Goblin Piledriver");
    let mut t = TestGame::new(2);
    let pile = t.battlefield(P0, "Goblin Piledriver");
    let a = t.battlefield(P0, "Raging Goblin");
    let b = t.battlefield(P0, "Raging Goblin");
    attack_with(
        &mut t,
        &[
            (pile, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(pile), (3, 2));
}

#[test]
fn volley_veteran_counts_goblins_as_it_resolves_including_itself() {
    cr!("608.2h", "603.3");
    ruling!(
        "Volley Veteran",
        "The number of Goblins you control is counted only as Volley Veteran's ability resolves. If Volley Veteran is still on the battlefield, it will count itself."
    );
    supported("Volley Veteran");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.enter(P0, "Volley Veteran");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    // The Veteran and one Raging Goblin.
    assert_eq!(t.obj_now(wurm).damage, 2);
}

#[test]
fn mad_auntie_can_target_another_mad_auntie_but_not_itself() {
    cr!("115.1b", "115.5");
    ruling!(
        "Mad Auntie",
        "The second ability can target any Goblin permanent except the Mad Auntie whose ability is being activated. It can target a different Mad Auntie."
    );
    supported("Mad Auntie");
    let mut t = TestGame::new(2);
    let auntie = t.battlefield(P0, "Mad Auntie");
    let other = t.battlefield(P0, "Mad Auntie");
    let theirs = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cands = ability_targets(&mut t, auntie, 0);
    assert!(cands.contains(&Entity::Object(other)));
    assert!(cands.contains(&Entity::Object(theirs)));
    assert!(!cands.contains(&Entity::Object(auntie)));
    assert!(!cands.contains(&Entity::Object(bears)));
}

#[test]
fn goblin_warrens_can_sacrifice_its_own_tokens() {
    cr!("118.3", "602.2b");
    ruling!(
        "Goblin Warrens",
        "The token Goblins can be sacrificed to the Warrens to generate new Goblins."
    );
    supported("Goblin Warrens");
    let mut t = TestGame::new(2);
    let warrens = t.battlefield(P0, "Goblin Warrens");
    let a = t.battlefield(P0, "Raging Goblin");
    let b = t.battlefield(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 6);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, warrens, 0, &[]).unwrap();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 3);
    t.answer_choose(P0, &[Entity::Object(toks[0]), Entity::Object(toks[1])]);
    t.activate(P0, warrens, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 4);
    assert!(!t.on_battlefield(toks[0]));
    assert!(!t.on_battlefield(toks[1]));
}

#[test]
fn stenchskipper_checks_for_goblins_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Stenchskipper",
        "This ability checks whether you control any Goblins twice each turn: once when the ability would trigger, and once when it resolves. If you control any Goblins at either time, you won’t sacrifice Stenchskipper. (If you control any Goblins at the time the ability would trigger, it doesn’t trigger at all.)"
    );
    supported("Stenchskipper");
    // A Goblin at the beginning of the end step: no trigger.
    let mut t = TestGame::new(2);
    let skipper = t.battlefield(P0, "Stenchskipper");
    t.battlefield(P0, "Raging Goblin");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(skipper));

    // No Goblin then, but one when it resolves: not sacrificed.
    let mut t = TestGame::new(2);
    let skipper = t.battlefield(P0, "Stenchskipper");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Raging Goblin");
    t.resolve_all();
    assert!(t.on_battlefield(skipper));
    // No Goblin at either time: sacrificed.
    let mut t = TestGame::new(2);
    let skipper = t.battlefield(P0, "Stenchskipper");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(skipper));
}

#[test]
fn general_kreats_token_enters_attacking_without_being_declared() {
    cr!("508.4", "508.3a", "506.3a");
    ruling!(
        "General Kreat, the Boltbringer",
        "Although the token created by General Kreat's first ability enters attacking, it was never declared as an attacker. Abilities that trigger whenever a creature attacks won't trigger. If there are any costs to have a creature attack, those costs won't apply to the token."
    );
    ruling!(
        "General Kreat, the Boltbringer",
        "You choose which player, planeswalker, or battle the token is attacking. It doesn't need to be the same player, planeswalker, or battle that any of the other Goblins are attacking."
    );
    supported("General Kreat, the Boltbringer");
    supported("Druids' Repository");
    let mut t = TestGame::new(2);
    let kreat = t.battlefield(P0, "General Kreat, the Boltbringer");
    let repo = t.battlefield(P0, "Druids' Repository");
    let liliana = t.battlefield(P1, "Liliana of the Veil");
    t.answer_choose(P0, &[Entity::Object(liliana)]);
    attack_with(&mut t, &[(kreat, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(attack_target(&t, toks[0]), Some(Entity::Object(liliana)));
    assert!(t.obj_now(toks[0]).tapped);
    // Only Kreat's attack put a charge counter on the Repository.
    assert_eq!(t.counters(repo, counters::CHARGE), 1);
}

#[test]
fn general_kreat_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "603.10a");
    ruling!(
        "General Kreat, the Boltbringer",
        "If General Kreat enters at the same time as one or more other creatures you control, its last ability will trigger for each of those creatures."
    );
    supported("General Kreat, the Boltbringer");
    let mut t = TestGame::new(2);
    let ids = [
        t.graveyard(P0, "General Kreat, the Boltbringer"),
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Raging Goblin"),
    ];
    let moves = ids
        .iter()
        .map(|id| MoveEv {
            obj: *id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(P0),
            etb: EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
