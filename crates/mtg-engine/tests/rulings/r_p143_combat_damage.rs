//! Rulings batch P143 — "whenever this creature deals combat damage to a player" card
//! advantage: Lone Revenant's intervening "if", looking at / revealing "that many" cards,
//! Garruk, Savage Herald's abilities, and casting exiled cards with normal timing.

use crate::r_p107_common::mana;
use crate::r_p108_common::put_counters;
use crate::r_s02_common::can_cast;
use crate::r_s09_common::declare;
use crate::r_p143_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 attacks P1 with `attackers` (P1 blocks with `blocks`) and combat finishes.
fn fight(t: &mut TestGame, attackers: &[ObjectId], at: Entity, blocks: &[(ObjectId, ObjectId)]) {
    let decl: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, at)).collect();
    declare(t, P0, &decl);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    t.advance_to(P0, Step::EndOfCombat);
}

const TOP: &[&str] = &[
    "Island",
    "Grizzly Bears",
    "Swamp",
    "Memnite",
    "Plains",
    "Hill Giant",
    "Mountain",
    "Forest",
];

#[test]
fn lone_revenant_needs_to_be_alone_when_damage_is_dealt() {
    cr!("603.4", "510.2");
    supported("Lone Revenant");
    ruling!(
        "Lone Revenant",
        "Check whether Lone Revenant’s ability triggers after combat damage is dealt but before any creatures that are destroyed due to combat damage leave the battlefield."
    );
    let mut t = TestGame::new(2);
    let rev = t.battlefield(P0, "Lone Revenant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let lib = set_library(&mut t, P0, TOP);
    fight(&mut t, &[rev, bears], Entity::Player(P1), &[(giant, bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 16);
    assert_eq!(library_top_first(&t, P0), lib);
    assert_eq!(t.hand_size(P0), 0);

    ruling!(
        "Lone Revenant",
        "If you control Lone Revenant and any other creature (including another Lone Revenant), its ability won’t trigger."
    );
    for other in ["Grizzly Bears", "Lone Revenant"] {
        let mut t = TestGame::new(2);
        let rev = t.battlefield(P0, "Lone Revenant");
        t.battlefield(P0, other);
        let lib = set_library(&mut t, P0, TOP);
        fight(&mut t, &[rev], Entity::Player(P1), &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), 16);
        assert_eq!(library_top_first(&t, P0), lib, "{other}");
    }
}

#[test]
fn lone_revenant_resolves_with_no_creatures_and_must_take_one() {
    cr!("603.4", "608.2c");
    ruling!(
        "Lone Revenant",
        "If you control no creatures when Lone Revenant’s ability resolves, you’ll still look at the top four cards and put one into your hand."
    );
    ruling!(
        "Lone Revenant",
        "You must put one of the cards into your hand, even if you’d rather put all four on the bottom of your library."
    );
    let mut t = TestGame::new(2);
    let rev = t.battlefield(P0, "Lone Revenant");
    let lib = set_library(&mut t, P0, TOP);
    declare(&mut t, P0, &[(rev, Entity::Player(P1))]);
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, rev);
    let from = n_asked(&t);
    t.resolve_all();
    let looks = entity_choices(&t, P0, from);
    assert_eq!(looks.len(), 1);
    assert_eq!((looks[0].0, looks[0].1, looks[0].2.len()), (1, 1, 4));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(library_top_first(&t, P0)[0], lib[4]);
}

#[test]
fn garruks_harbinger_looks_at_all_the_damage_dealt() {
    cr!("120.4", "510.2");
    supported("Garruk's Harbinger");
    ruling!(
        "Garruk's Harbinger",
        "If Garruk's Harbinger deals damage to a planeswalker greater than their loyalty or to a player greater than their life total, you look at a number of cards equal to that full amount of damage dealt"
    );
    // A player at 2 life.
    let mut t = TestGame::new(3);
    let h = t.battlefield(P0, "Garruk's Harbinger");
    t.g.players[1].life = 2;
    let lib = set_library(&mut t, P0, TOP);
    fight(&mut t, &[h], Entity::Player(P1), &[]);
    t.resolve_all();
    assert_eq!(library_top_first(&t, P0)[0], lib[4]);
    // A planeswalker with 1 loyalty.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Garruk's Harbinger");
    let w = t.battlefield(P1, "Narset, Parter of Veils");
    t.g.obj_mut(w).counters.insert(counters::LOYALTY.into(), 1);
    let lib = set_library(&mut t, P0, TOP);
    fight(&mut t, &[h], obj(w), &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(w));
    assert_eq!(library_top_first(&t, P0)[0], lib[4]);
}

#[test]
fn gishath_reveals_what_there_is_without_losing() {
    cr!("121.4", "701.20a");
    supported("Gishath, Sun's Avatar");
    ruling!(
        "Gishath, Sun's Avatar",
        "If you have fewer cards in your library than the amount of damage Gishath deals, you reveal the cards you have. Because you're not drawing cards, you don't lose the game."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gishath, Sun's Avatar");
    // Seven damage, two cards: both are revealed and the Dinosaur is put onto the
    // battlefield.
    let lib = set_library(&mut t, P0, &["Island", "Ancient Brontodon"]);
    t.answer_choose(P0, &[obj(lib[1])]);
    fight(&mut t, &[g], Entity::Player(P1), &[]);
    t.resolve_all();
    t.settle();
    assert_eq!(t.life(P1), 13);
    assert!(t.on_battlefield(lib[1]));
    assert_eq!(library_top_first(&t, P0), vec![t.g.current(lib[0])]);
    assert!(!t.has_lost(P0));
}

#[test]
fn ojer_kaslem_may_put_any_combination() {
    cr!("608.2c");
    supported("Ojer Kaslem, Deepest Growth // Temple of Cultivation");
    ruling!(
        "Ojer Kaslem, Deepest Growth // Temple of Cultivation",
        "While resolving Ojer Kaslem, Deepest Growth's middle ability, you could put no cards, a creature card, a land card, or a creature card and a land card onto the battlefield."
    );
    for pick in [&[][..], &[1][..], &[0][..], &[0, 1][..]] {
        let mut t = TestGame::new(2);
        let o = t.battlefield(P0, "Ojer Kaslem, Deepest Growth");
        let lib = set_library(&mut t, P0, &["Island", "Grizzly Bears", "Memnite", "Swamp", "Hill Giant", "Forest"]);
        let chosen: Vec<Entity> = pick.iter().map(|i| obj(lib[*i])).collect();
        t.answer_choose(P0, &chosen);
        fight(&mut t, &[o], Entity::Player(P1), &[]);
        t.resolve_all();
        let on: Vec<bool> = lib.iter().map(|c| t.on_battlefield(*c)).collect();
        for (i, c) in on.iter().enumerate() {
            assert_eq!(*c, pick.contains(&i), "{pick:?}: card {i}");
        }
    }
}

#[test]
fn dream_pillager_cards_follow_normal_timing_and_costs() {
    cr!("601.3", "307.1", "302.1");
    supported("Dream Pillager");
    ruling!(
        "Dream Pillager",
        "Casting a card this way follows the normal rules for casting the card. You must pay its costs, and you must follow all applicable timing rules."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dream Pillager");
    let lib = set_library(&mut t, P0, &["Grizzly Bears", "Lightning Bolt", "Island", "Island", "Island"]);
    declare(&mut t, P0, &[(d, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(is_in(&t, lib[0], Zone::Exile));
    // Without mana, nothing can be cast.
    assert!(!can_cast(&mut t, P0, lib[1], CastMethod::Normal));
    // In combat: the instant can be cast (with mana), the creature can't.
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    assert!(can_cast(&mut t, P0, lib[1], CastMethod::Normal));
    // In the postcombat main phase with an empty stack, with mana.
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Normal));
}

#[test]
fn garruk_minus_two_with_an_illegal_target() {
    cr!("608.2b", "701.14a");
    supported("Garruk, Savage Herald");
    ruling!(
        "Garruk, Savage Herald",
        "If either creature is an illegal target as Garruk's second ability tries to resolve, the creature you control won't deal damage to any creature."
    );
    for remove_mine in [false, true] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Garruk, Savage Herald");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let other = t.battlefield(P1, "Grizzly Bears");
        crate::r_p107_common::act(&mut t, P0, g, "deals damage equal", &[obj(giant), obj(bears)])
            .unwrap();
        // The other one becomes illegal: it leaves (or bounces).
        let gone = if remove_mine { giant } else { bears };
        crate::r_s02_common::destroy(&mut t, gone);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), remove_mine);
        if !remove_mine {
            assert_eq!(t.obj_now(giant).damage, 0);
        }
        assert!(t.on_battlefield(other));
    }
}

/// Activates Garruk's −7 (with 7 loyalty) and resolves it.
fn garruk_ultimate(t: &mut TestGame) {
    let g = t.battlefield(P0, "Garruk, Savage Herald");
    put_counters(t, g, counters::LOYALTY, 2);
    crate::r_p107_common::act(t, P0, g, "Until end of turn", &[]).unwrap();
    t.resolve_all();
}

#[test]
fn garruk_ultimate_assigns_all_damage_one_way() {
    cr!("510.1c", "510.1a");
    ruling!(
        "Garruk, Savage Herald",
        "When assigning combat damage, you choose for each creature whether you want to assign its damage to the creature blocking it, or if you want to assign it to the player or planeswalker the creature is attacking. You can't split the damage assignment between them."
    );
    for unblocked in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let blocker = t.battlefield(P1, "Grizzly Bears");
        garruk_ultimate(&mut t);
        t.answer_yes(P0, unblocked);
        fight(&mut t, &[giant], Entity::Player(P1), &[(blocker, giant)]);
        if unblocked {
            assert_eq!(t.life(P1), 17);
            assert!(t.on_battlefield(blocker));
        } else {
            assert_eq!(t.life(P1), 20);
            assert!(!t.on_battlefield(blocker));
        }
    }

    ruling!(
        "Garruk, Savage Herald",
        "Garruk's last ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't gain the ability."
    );
    let mut t = TestGame::new(2);
    garruk_ultimate(&mut t);
    let late = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let from = n_asked(&t);
    fight(&mut t, &[late], Entity::Player(P1), &[(blocker, late)]);
    assert!(!asked_of(&t, P0, from)
        .iter()
        .any(|d| matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(blocker));
}

#[test]
fn doomskar_warrior_triggers_for_each_player_and_battle_dealt_damage() {
    cr!("510.2", "603.2c", "614.9");
    supported("Doomskar Warrior");
    supported("Treacherous Link");
    ruling!(
        "Doomskar Warrior",
        "In some unusual situations (mostly involving damage redirection), Doomskar Warrior may deal combat damage to multiple players and/or battles at the same time. If this happens, its ability will trigger for each of those players and/or battles."
    );
    // Doomskar Warrior (4/3 trample) attacks P0's Siege, protected by P1. P1 blocks with
    // Grizzly Bears enchanted with Treacherous Link ("All damage that would be dealt to
    // enchanted creature is dealt to its controller instead."): 2 damage is assigned to
    // the Bears (dealt to P1 instead) and 2 tramples over to the battle.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Doomskar Warrior");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    t.g.dirty = true;
    let bears = t.battlefield(P1, "Grizzly Bears");
    let link = t.battlefield(P1, "Treacherous Link");
    assert!(t.g.attach(link, obj(bears)));
    t.g.recompute();
    let defense = t.counters(battle, counters::DEFENSE);
    let lib = set_library(&mut t, P0, &["Island", "Swamp", "Plains", "Forest", "Wastes"]);
    declare(&mut t, P0, &[(w, obj(battle))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, w)]));
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[]);
    let from = n_asked(&t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(battle, counters::DEFENSE), defense - 2);
    // Two triggers, each looking at two cards (put on the bottom in a random order).
    let looks: Vec<usize> = entity_choices(&t, P0, from)
        .iter()
        .filter(|(_, _, c)| c.iter().all(|e| lib.iter().any(|l| *e == obj(*l))))
        .map(|(_, _, c)| c.len())
        .collect();
    assert_eq!(looks, vec![2, 2]);
    assert_eq!(library_top_first(&t, P0)[0], lib[4]);
}
