//! Rulings batch P102 — "each player sacrifices a creature" (Fleshbag Marauder and
//! friends: APNAP choices, sacrificing the source itself, simultaneous sacrifice),
//! creatures that sacrifice themselves as a cost (targets are chosen before costs are
//! paid, CR 601.2c/601.2h), and Magus of the Balance.

use crate::r_p102_common::*;
use crate::r_p137_common::{choosers_since, is_choose};
use crate::r_s01_common::watch;
use crate::r_s02_common::{can_activate, can_attack, destroy, target_candidates};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s creature `name` enters (its ETB trigger goes on the stack); `choices` are queued
/// first (`(player, objects to sacrifice)`), then everything resolves.
fn etb_with_choices(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    choices: &[(PlayerId, Vec<ObjectId>)],
) -> ObjectId {
    supported(name);
    let id = t.enter(p, name);
    for (q, ids) in choices {
        choose(t, *q, ids);
    }
    t.resolve_all();
    id
}

#[test]
fn edict_creatures_sacrifice_themselves_when_needed_or_chosen() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Fleshbag Marauder",
        "When its ability resolves, you may sacrifice Fleshbag Marauder itself. If you control no other creatures, you'll have to sacrifice Fleshbag Marauder."
    );
    ruling!(
        "Merciless Executioner",
        "If you control no other creatures as Merciless Executioner's ability resolves, you'll have to sacrifice Merciless Executioner."
    );
    ruling!(
        "Accursed Marauder",
        "You may sacrifice Accursed Marauder to its own ability. If it is the only creature you control when that ability resolves, you must sacrifice it."
    );
    ruling!(
        "Slum Reaper",
        "Slum Reaper may be the creature its controller sacrifices for its own ability."
    );
    ruling!(
        "Abyssal Gorestalker",
        "You may sacrifice Abyssal Gorestalker to its own ability. If it is the only creature you control or one of two creatures you control, you must sacrifice it."
    );
    // (card, P0's other creatures, P0 chooses itself, it's sacrificed)
    let cases: &[(&str, usize, bool, bool)] = &[
        ("Fleshbag Marauder", 0, false, true),
        ("Fleshbag Marauder", 1, true, true),
        ("Fleshbag Marauder", 1, false, false),
        ("Merciless Executioner", 0, false, true),
        ("Accursed Marauder", 0, false, true),
        ("Accursed Marauder", 1, true, true),
        ("Slum Reaper", 1, true, true),
        ("Slum Reaper", 1, false, false),
        ("Abyssal Gorestalker", 0, false, true),
        ("Abyssal Gorestalker", 1, false, true),
        ("Abyssal Gorestalker", 2, true, true),
        ("Abyssal Gorestalker", 2, false, false),
    ];
    for &(name, others, pick_self, gone) in cases {
        let mut t = TestGame::new(2);
        let bears: Vec<ObjectId> = (0..others)
            .map(|_| t.battlefield(P0, "Grizzly Bears"))
            .collect();
        t.battlefield(P1, "Hill Giant");
        let n = if name == "Abyssal Gorestalker" { 2 } else { 1 };
        supported(name);
        let id = t.enter(P0, name);
        let mut pick: Vec<ObjectId> = if pick_self { vec![id] } else { vec![] };
        for b in &bears {
            if pick.len() < n {
                pick.push(*b);
            }
        }
        choose(&mut t, P0, &pick);
        t.resolve_all();
        assert_eq!(!t.on_battlefield(id), gone, "{name} with {others} others");
        assert_eq!(
            creatures(&t, P0).len(),
            (others + 1).saturating_sub(n),
            "{name} with {others} others"
        );
        assert!(t.in_graveyard(P1, "Hill Giant"), "{name}");
    }
}

#[test]
fn edicts_make_a_player_with_one_creature_sacrifice_it() {
    cr!("701.21a");
    ruling!(
        "Abyssal Gorestalker",
        "If a player controls only one creature as the ability resolves, they must sacrifice it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    etb_with_choices(&mut t, P0, "Abyssal Gorestalker", &[]);
    assert!(in_gy(&t, giant));
}

#[test]
fn slum_reaper_skips_players_without_creatures() {
    cr!("701.21a", "101.3");
    ruling!(
        "Slum Reaper",
        "If any player controls no creatures, that player simply doesn’t sacrifice anything. Each other player still sacrifices a creature."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    let reaper = etb_with_choices(&mut t, P0, "Slum Reaper", &[(P0, vec![bears])]);
    assert!(t.on_battlefield(reaper));
    assert!(in_gy(&t, bears));
    assert!(in_gy(&t, giant));
}

#[test]
fn slum_reaper_choices_are_made_in_apnap_order_then_sacrificed_together() {
    cr!("101.4", "701.21a", "608.2c");
    ruling!(
        "Slum Reaper",
        "As Slum Reaper’s ability resolves, first the player whose turn it is chooses a creature they control, then each other player in turn order does the same"
    );
    // It's P1's turn; P0 casts nothing, the Reaper enters under P2's control.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let bears = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P2, "Grizzly Bears"),
    ];
    let seen = watch(&mut t, P2, is_choose, |g| {
        g.battlefield
            .iter()
            .filter(|id| g.obj(**id).is(CardType::Creature))
            .count()
    });
    let reaper = t.enter(P2, "Slum Reaper");
    t.settle();
    let from = t.asked().len();
    choose(&mut t, P2, &[bears[2]]);
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P1, P2, P0]);
    // When P2 (after P1) chooses, P1's chosen creature is still on the battlefield.
    assert_eq!(seen.lock().unwrap().clone(), vec![4]);
    assert!(bears.iter().all(|b| in_gy(&t, *b)));
    assert!(t.on_battlefield(reaper));
}

/// The sacrifice-a-creature-as-a-cost reanimators: their targets ("target creature card
/// from your graveyard") are chosen before the cost is paid, so they can't target
/// themselves.
#[test]
fn self_sacrificing_reanimators_cant_target_themselves() {
    cr!("601.2c", "601.2h", "602.2b");
    ruling!(
        "Soulcoil Viper",
        "Because you choose the target of Soulcoil Viper's ability before you pay the cost of sacrificing it, Soulcoil Viper can't be the target of its own ability."
    );
    ruling!(
        "Bonecaller Cleric",
        "Bonecaller Cleric can't be the target of its own ability. This is because targets are chosen for activated abilities before costs"
    );
    ruling!(
        "Valgavoth's Faithful",
        "Since targets are chosen before costs are paid, Valgavoth's Faithful isn't a legal target for its own ability because it won't be in the graveyard at that time."
    );
    for name in [
        "Soulcoil Viper",
        "Bonecaller Cleric",
        "Valgavoth's Faithful",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        lots(&mut t, P0);
        // An empty graveyard: no target, no activation (so it can't sacrifice itself and
        // then return itself).
        assert!(!can_activate(&mut t, P0, id), "{name}");
        let bears = t.graveyard(P0, "Grizzly Bears");
        assert!(can_activate(&mut t, P0, id), "{name}");
        let from = t.asked().len();
        t.activate(P0, id, 0, &[o(bears)]).unwrap();
        let offered = target_candidates(&t, P0, from);
        assert_eq!(offered, vec![vec![o(bears)]], "{name}");
        t.resolve_all();
        assert!(in_gy(&t, id), "{name}");
        assert!(t.on_battlefield(bears), "{name}");
    }
}

#[test]
fn hells_caretaker_may_sacrifice_itself_but_not_target_itself() {
    cr!("601.2c", "601.2h", "602.5b");
    ruling!(
        "Hell's Caretaker",
        "Hell's Caretaker can be the creature sacrificed to activate its ability, but it can't be the target of its own ability."
    );
    supported("Hell's Caretaker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    let hc = t.battlefield(P0, "Hell's Caretaker");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    choose(&mut t, P0, &[hc]);
    t.activate(P0, hc, 0, &[o(bears)]).unwrap();
    assert_eq!(target_candidates(&t, P0, from), vec![vec![o(bears)]]);
    assert!(in_gy(&t, hc));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(in_gy(&t, hc));
}

#[test]
fn creatures_returned_during_upkeep_are_summoning_sick() {
    cr!("302.6", "503.1");
    ruling!(
        "Hell's Caretaker",
        "Because the turn has already begun, creatures put onto the battlefield during your upkeep won't be able to attack or pay a {T} cost this turn."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    let hc = t.battlefield(P0, "Hell's Caretaker");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    choose(&mut t, P0, &[fodder]);
    t.activate(P0, hc, 0, &[o(elves)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(elves));
    assert!(!can_activate(&mut t, P0, elves), "can't pay {{T}}");
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, elves));
    // Only during your upkeep: not in the main phase.
    t.graveyard(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P0, hc));
}

#[test]
fn brindle_boar_sacrificed_in_combat_deals_no_damage_or_dies_first() {
    cr!("506.4", "510.1", "704.5g", "117.3a");
    ruling!(
        "Brindle Boar",
        "If you sacrifice an attacking or blocking Brindle Boar during the declare blockers step, it won’t deal combat damage."
    );
    supported("Brindle Boar");
    // Sacrificed after blockers are declared: the blocking Hill Giant takes no damage.
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Brindle Boar");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(boar, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(giant, boar)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, boar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.obj_now(giant).damage, 0);
    // Waiting until the combat damage step: lethal damage destroys it before any player
    // gets priority.
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Brindle Boar");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(boar, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(giant, boar)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    // State-based actions are checked before P0 would get priority.
    t.settle();
    assert!(in_gy(&t, boar));
    assert!(!can_activate(&mut t, P0, boar));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn selfless_cathar_affects_only_creatures_controlled_on_resolution() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Selfless Cathar",
        "Only creatures you control when Selfless Cathar's ability resolves will be affected."
    );
    ruling!(
        "Selfless Cathar",
        "You can activate Selfless Cathar's ability even if you control no other creatures."
    );
    supported("Selfless Cathar");
    let mut t = TestGame::new(2);
    let cathar = t.battlefield(P0, "Selfless Cathar");
    lots(&mut t, P0);
    assert!(can_activate(&mut t, P0, cathar));
    t.activate(P0, cathar, 0, &[]).unwrap();
    // A creature that enters before it resolves is affected; one after it isn't.
    let before = t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(in_gy(&t, cathar));
    let after = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_p015_common::gain_control(&mut t, P0, theirs);
    assert_eq!(t.pt(before), (3, 3));
    assert_eq!(t.pt(after), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
}

/// Magus of the Balance under P0 with mana to activate it.
fn magus(t: &mut TestGame) -> ObjectId {
    supported("Magus of the Balance");
    let m = t.battlefield(P0, "Magus of the Balance");
    t.g.players[0].mana_pool.add_type(ManaType::W, 5);
    m
}

#[test]
fn magus_of_the_balance_levels_to_the_player_with_the_fewest() {
    cr!("608.2c", "701.21a", "701.9a");
    ruling!(
        "Magus of the Balance",
        "If a player has zero cards in hand, controls no lands, or controls no creatures, other players will have to discard their entire hand, sacrifice all of their lands, or sacrifice all of their creatures, as appropriate."
    );
    let mut t = TestGame::new(2);
    let m = magus(&mut t);
    t.lands(P0, "Plains", 2);
    t.hand(P0, "Shock");
    t.battlefield(P0, "Grizzly Bears");
    // P1: no lands, no hand, no creatures.
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert!(crate::r_p116_common::lands_controlled(&t, P0) == 0);
    assert_eq!(t.hand_size(P0), 0);
    assert!(creatures(&t, P0).is_empty());
}

#[test]
fn magus_of_the_balance_choices_in_apnap_order_then_simultaneous() {
    cr!("101.4", "608.2c", "608.2e", "603.3");
    ruling!(
        "Magus of the Balance",
        "When Magus of the Balance's ability resolves, first the player whose turn it is chooses an appropriate number of lands they control"
    );
    ruling!(
        "Magus of the Balance",
        "Players can't take actions once Magus of the Balance's ability begins to resolve until it's done resolving. If any abilities trigger during this process, they're put onto the stack after it's done resolving."
    );
    // P0 (active): 3 lands; P1: 1 land, Blood Artist and Grizzly Bears.
    let mut t = TestGame::new(2);
    let m = magus(&mut t);
    let p0_lands = t.lands(P0, "Plains", 3);
    t.lands(P1, "Island", 1);
    let p0_bears = t.battlefield(P0, "Grizzly Bears");
    let artist = t.battlefield(P1, "Blood Artist");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    // (The engine asks which permanents to sacrifice: "the rest".)
    choose(&mut t, P0, &[p0_lands[1], p0_lands[2]]);
    choose(&mut t, P1, &[p1_bears]);
    t.activate(P0, m, 0, &[]).unwrap();
    // Blood Artist's trigger for the Magus (sacrificed as a cost) resolves first.
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 19);
    let from = t.asked().len();
    t.resolve();
    // P0 chose first; nobody got priority during the resolution.
    assert_eq!(choosers_since(&t, from).first(), Some(&P0));
    let asked = t.asked();
    let last_choice = asked.iter().rposition(|(_, d)| is_choose(d)).unwrap();
    assert!(!asked[from..last_choice]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    assert_eq!(crate::r_p116_common::lands_controlled(&t, P0), 1);
    assert!(t.on_battlefield(p0_lands[0]));
    // P0 has 1 creature (the Magus was sacrificed as a cost), so P1 keeps Blood Artist
    // and sacrifices the Bears; Blood Artist's trigger waited until the ability was done.
    assert!(t.on_battlefield(p0_bears));
    assert!(in_gy(&t, p1_bears));
    assert!(t.on_battlefield(artist));
    assert_eq!(t.stack_len(), 1);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}
