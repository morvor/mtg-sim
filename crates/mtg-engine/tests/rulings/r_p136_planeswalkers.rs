//! Rulings batch P136 (removal) — planeswalkers with removal abilities: delayed returns
//! (CR 603.7, 610.3), emblems (CR 114), effects that don't lock in what they affect
//! (CR 611.2c), choices made on resolution (CR 603.7c), Karn Liberated's restart (CR 727)
//! and Liliana, Dreadhorde General's leaves-the-battlefield triggers (CR 603.10a).

use crate::r_p108_common::end_step;
use crate::r_p136_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the planeswalker onto the battlefield for `p` with `loyalty` loyalty counters.
fn pw(t: &mut TestGame, p: PlayerId, name: &str, loyalty: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize]
        .counters
        .insert("loyalty".into(), loyalty);
    id
}

/// Activates the planeswalker's `index`th loyalty ability (allowing another activation
/// this turn afterwards) and resolves it.
fn loyalty(t: &mut TestGame, p: PlayerId, walker: ObjectId, index: usize, targets: &[Entity]) {
    t.activate(p, walker, index, targets).unwrap();
    t.resolve_all();
    let w = t.g.current(walker);
    t.g.objects[w.0 as usize].activations_this_turn.clear();
}

// ---------------------------------------------------------------------------
// Venser, the Sojourner
// ---------------------------------------------------------------------------

#[test]
fn venser_returns_the_permanent_even_without_venser() {
    cr!("603.7a", "610.3c");
    ruling!(
        "Venser, the Sojourner",
        "A permanent exiled by the first ability will return to the battlefield under your control at the beginning of the next end step even if you no longer control Venser at that time."
    );
    let mut t = TestGame::new(2);
    let venser = pw(&mut t, P0, "Venser, the Sojourner", 3);
    let bear = t.battlefield(P0, "Grizzly Bears");
    loyalty(&mut t, P0, venser, 0, &[o(bear)]);
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, venser);
    assert!(t.in_graveyard(P0, "Venser, the Sojourner"));
    end_step(&mut t, P0);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
}

#[test]
fn venser_exiling_a_token_it_doesnt_return() {
    cr!("111.7", "111.8", "704.5d");
    ruling!(
        "Venser, the Sojourner",
        "If the first ability exiles a token, that token will cease to exist. It won’t return to the battlefield."
    );
    let mut t = TestGame::new(2);
    let venser = pw(&mut t, P0, "Venser, the Sojourner", 3);
    let token = create_token(&mut t, P0, "Soldier");
    loyalty(&mut t, P0, venser, 0, &[o(token)]);
    assert!(!t.on_battlefield(token));
    end_step(&mut t, P0);
    assert!(t.named_on_battlefield("Soldier").is_empty());
}

#[test]
fn venser_minus_one_applies_to_every_creature_all_turn() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Venser, the Sojourner",
        "Venser’s second ability doesn’t lock in what it applies to."
    );
    let mut t = TestGame::new(2);
    let venser = pw(&mut t, P0, "Venser, the Sojourner", 3);
    let blocker = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Grizzly Bears");
    loyalty(&mut t, P0, venser, 1, &[]);
    // A creature that enters later, and one that loses all its abilities.
    let later = t.battlefield(P0, "Savannah Lions");
    let lost = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Island", 2);
    let frog = t.hand(P0, "Turn to Frog");
    t.cast_with(P0, frog, &[o(lost)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(lost), (1, 1));
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[
            (mine, Entity::Player(P1)),
            (later, Entity::Player(P1)),
            (lost, Entity::Player(P1)),
        ],
    );
    for a in [mine, later, lost] {
        assert!(!legal_blocks(&mut t, P1, &[(blocker, a)]));
    }
}

/// P0 controls `n` Venser emblems (from its third ability).
fn venser_emblems(t: &mut TestGame, n: usize) {
    let venser = pw(t, P0, "Venser, the Sojourner", 8 * n as u32 + 1);
    for _ in 0..n {
        loyalty(t, P0, venser, 2, &[]);
    }
}

#[test]
fn venser_emblem_triggers_on_casting_and_resolves_first() {
    cr!("114.4", "603.3", "603.2");
    ruling!(
        "Venser, the Sojourner",
        "Whenever you cast a spell, the emblem’s ability triggers and goes on the stack on top of it. It will resolve before the spell does."
    );
    ruling!(
        "Venser, the Sojourner",
        "If you control more than one such emblem, each one’s ability will trigger separately whenever you cast a spell."
    );
    let mut t = TestGame::new(2);
    venser_emblems(&mut t, 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(a)]);
    t.answer_targets(P0, &[o(b)]);
    let spell = cast(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(t.g.stack.len(), 3);
    assert_eq!(t.g.stack[0], spell);
    // The two triggers resolve first: both creatures are exiled while Divination is
    // still on the stack.
    t.resolve();
    t.resolve();
    assert!(t.g.stack == vec![spell]);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
}

#[test]
fn venser_emblem_is_colorless_so_protection_from_white_doesnt_stop_it() {
    cr!("114.4", "105.2c", "702.16b");
    ruling!(
        "Venser, the Sojourner",
        "Because emblems are colorless, you can target permanents with protection from white or from blue"
    );
    let mut t = TestGame::new(2);
    venser_emblems(&mut t, 1);
    let knight = t.battlefield(P1, "Black Knight");
    t.answer_targets(P0, &[o(knight)]);
    cast(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    assert!(t.in_exile("Black Knight"));
}

// ---------------------------------------------------------------------------
// Teferi, Hero of Dominaria
// ---------------------------------------------------------------------------

#[test]
fn teferi_chooses_the_lands_to_untap_at_the_end_step() {
    cr!("603.7c", "608.2h");
    ruling!(
        "Teferi, Hero of Dominaria",
        "You don't decide which two lands to untap until the next end step."
    );
    supported("Teferi, Hero of Dominaria");
    let mut t = TestGame::new(2);
    let teferi = pw(&mut t, P0, "Teferi, Hero of Dominaria", 4);
    loyalty(&mut t, P0, teferi, 0, &[]);
    // The lands are tapped (and one of them enters) only after the ability resolved.
    let mut lands = t.lands(P0, "Island", 2);
    lands.push(t.battlefield(P0, "Plains"));
    for l in &lands {
        t.g.tap(*l);
    }
    t.answer_choose(P0, &[o(lands[1]), o(lands[2])]);
    end_step(&mut t, P0);
    assert!(t.obj_now(lands[0]).tapped);
    assert!(!t.obj_now(lands[1]).tapped);
    assert!(!t.obj_now(lands[2]).tapped);
}

#[test]
fn teferi_emblem_target_is_chosen_after_the_draw() {
    cr!("603.3d", "121.1");
    ruling!(
        "Teferi, Hero of Dominaria",
        "You choose the target for the triggered ability of Teferi's emblem after you've seen the card you drew."
    );
    let mut t = TestGame::new(2);
    let teferi = pw(&mut t, P0, "Teferi, Hero of Dominaria", 9);
    loyalty(&mut t, P0, teferi, 2, &[]);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let drawn = t.library_top(P0, "Lightning Bolt");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.player(PlayerId(0)).hand.len(),
    );
    let before = t.hand_size(P0);
    t.answer_targets(P0, &[o(bear)]);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().clone(), vec![before + 1]);
    assert_eq!(t.zone(drawn), Zone::Hand(P0));
    assert!(t.in_exile("Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// Karn Liberated
// ---------------------------------------------------------------------------

fn karn_game(n: usize) -> (TestGame, ObjectId) {
    let mut t = TestGame::with_config(
        n,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let k = pw(&mut t, P0, "Karn Liberated", 30);
    (t, k)
}

fn karn_restart(t: &mut TestGame, k: ObjectId) {
    t.activate(P0, k, 2, &[]).unwrap();
    t.g.resolve_top();
}

/// Runs the new game until the first turn's upkeep, with priority.
fn to_first_upkeep(t: &mut TestGame) {
    let ok = t.g.run_until(100, |g| {
        g.turn.step == Step::Upkeep && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    assert_eq!(t.g.turn.number, 1);
}

#[test]
fn karn_keeps_only_permanent_cards_exiled_by_its_first_two_abilities() {
    cr!("607.2a", "727.5");
    ruling!(
        "Karn Liberated",
        "Only non-Aura permanent cards exiled by either of Karn's first two abilities will remain in exile when the game restarts."
    );
    supported("Karn Liberated");
    let (mut t, k) = karn_game(2);
    // +4: P1 exiles a card from their hand (each time their only card).
    t.hand(P1, "Hill Giant");
    loyalty(&mut t, P0, k, 0, &[Entity::Player(P1)]);
    t.hand(P1, "Lightning Bolt");
    loyalty(&mut t, P0, k, 0, &[Entity::Player(P1)]);
    // −3 on a permanent; and a card exiled by something else.
    let bear = t.battlefield(P1, "Grizzly Bears");
    loyalty(&mut t, P0, k, 1, &[o(bear)]);
    t.exile(P1, "Savannah Lions");
    assert!(t.in_exile("Hill Giant") && t.in_exile("Lightning Bolt"));
    karn_restart(&mut t, k);
    let mut names: Vec<String> = t
        .g
        .permanents()
        .map(|p| p.chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
    assert!(t.g.permanents().all(|p| p.controller == P0));
    assert!(t.g.exile.is_empty());
}

#[test]
fn karn_triggers_wait_for_the_first_upkeep_and_tapped_permanents_untap() {
    cr!("727.4", "502.3");
    ruling!(
        "Karn Liberated",
        "If this causes any triggered abilities to trigger, those abilities are put onto the stack at the beginning of the first upkeep step."
    );
    ruling!(
        "Karn Liberated",
        "Any permanents put onto the battlefield with Karn's ability that entered the battlefield tapped will untap during their controller's first untap step."
    );
    let (mut t, k) = karn_game(2);
    let visionary = t.battlefield(P1, "Elvish Visionary");
    let gate = t.battlefield(P1, "Selesnya Guildgate");
    loyalty(&mut t, P0, k, 1, &[o(visionary)]);
    loyalty(&mut t, P0, k, 1, &[o(gate)]);
    karn_restart(&mut t, k);
    // Before the first turn: both entered, the gate tapped; no trigger on the stack yet.
    let gate = t.named_on_battlefield("Selesnya Guildgate")[0];
    assert!(t.g.obj(gate).tapped);
    assert!(t.g.stack.is_empty());
    let hand = t.hand_size(P0);
    assert!(t.g.pending_triggers.len() == 1);
    to_first_upkeep(&mut t);
    t.settle();
    assert!(!t.g.obj(gate).tapped);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn karn_permanents_are_exiled_if_their_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Karn Liberated",
        "If you leave the game, the permanents you control from Karn's ability are exiled."
    );
    let (mut t, k) = karn_game(3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    loyalty(&mut t, P0, k, 1, &[o(bear)]);
    karn_restart(&mut t, k);
    let bear = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.g.obj(bear).controller, P0);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn karn_nothing_from_the_old_game_carries_over() {
    cr!("727.2", "727.1");
    ruling!(
        "Karn Liberated",
        "No actions taken in the game that was restarted apply to the new game."
    );
    ruling!(
        "Karn Liberated",
        "Players can't move cards between their deck and sideboard before the new game."
    );
    let (mut t, k) = karn_game(2);
    // Stigma Lasher deals damage to P0: P0 can't gain life for the rest of the game.
    let lasher = t.battlefield(P1, "Stigma Lasher");
    t.g.deal_damage(lasher, Entity::Player(P0), 2, false);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.gain_life(P0, 3), 0);
    t.g.add_to_sideboard(P0, vec![mtg_engine::card::card("Hill Giant")]);
    let from = t.asked().len();
    karn_restart(&mut t, k);
    // The new game: P0 can gain life again; the sideboard is untouched and nobody was
    // offered to swap cards with it.
    assert_eq!(t.g.gain_life(P0, 3), 3);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.g.player(P0).sideboard.len(), 1);
    assert!(asked_since(&t, from).is_empty());
}

// ---------------------------------------------------------------------------
// Liliana, Dreadhorde General
// ---------------------------------------------------------------------------

#[test]
fn liliana_dying_with_creatures_draws_for_each() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Liliana, Dreadhorde General",
        "If Liliana dies at the same time as one or more creatures you control, her first ability triggers for each of those creatures."
    );
    supported("Planar Cleansing");
    let mut t = TestGame::new(2);
    pw(&mut t, P0, "Liliana, Dreadhorde General", 6);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Gray Ogre");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    cast(&mut t, P1, "Planar Cleansing", &[]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Liliana, Dreadhorde General"));
    assert_eq!(triggers_on_stack(&t, "draw a card"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn liliana_as_a_creature_dying_triggers_her_ability() {
    cr!("603.10a", "700.4");
    ruling!(
        "Liliana, Dreadhorde General",
        "If Liliana somehow becomes a creature and dies, her first ability will trigger."
    );
    supported("Sarkhan the Masterless");
    let mut t = TestGame::new(2);
    let lili = pw(&mut t, P0, "Liliana, Dreadhorde General", 6);
    let sarkhan = pw(&mut t, P0, "Sarkhan the Masterless", 5);
    loyalty(&mut t, P0, sarkhan, 0, &[]);
    t.g.recompute();
    assert!(t.obj_now(lili).is(CardType::Creature));
    let hand = t.hand_size(P0);
    destroy(&mut t, lili);
    assert!(t.in_graveyard(P0, "Liliana, Dreadhorde General"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn liliana_minus_four_each_player_chooses_in_order_then_all_sacrifice() {
    cr!("101.4", "701.21a");
    ruling!(
        "Liliana, Dreadhorde General",
        "As Liliana's second loyalty ability resolves, first the player whose turn it is chooses two creatures they control, then each other player in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time. If any player can choose only one creature, that player does so."
    );
    let mut t = TestGame::new(3);
    let lili = pw(&mut t, P0, "Liliana, Dreadhorde General", 6);
    let mine: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant", "Gray Ogre"]
        .iter()
        .map(|n| t.battlefield(P0, n))
        .collect();
    let p1: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant", "Gray Ogre"]
        .iter()
        .map(|n| t.battlefield(P1, n))
        .collect();
    let p2 = t.battlefield(P2, "Savannah Lions");
    t.answer_choose(P0, &[o(mine[0]), o(mine[1])]);
    t.answer_choose(P1, &[o(p1[1]), o(p1[2])]);
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| {
            watch(
                &mut t,
                p,
                |d| matches!(d, Decision::ChooseEntities { .. }),
                |g| g.permanents().count(),
            )
        })
        .collect();
    let before = t.g.permanents().count();
    let from = t.asked().len();
    t.activate(P0, lili, 1, &[]).unwrap();
    t.resolve();
    let order: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order[..2], [P0, P1]);
    // Nothing was sacrificed until everyone had chosen.
    for s in &seen[..2] {
        assert_eq!(s.lock().unwrap().first().copied(), Some(before));
    }
    assert!(t.on_battlefield(mine[2]) && t.on_battlefield(p1[0]));
    for id in [mine[0], mine[1], p1[1], p1[2], p2] {
        assert!(!t.on_battlefield(id));
    }
    // Liliana's first ability triggered for P0's two creatures.
    assert_eq!(triggers_on_stack(&t, "draw a card"), 2);
}

// ---------------------------------------------------------------------------
// Barrin, Tolarian Archmage
// ---------------------------------------------------------------------------

#[test]
fn barrin_checks_the_whole_turn() {
    cr!("603.4");
    ruling!(
        "Barrin, Tolarian Archmage",
        "Barrin's last ability checks the entire turn, even before Barrin was on the battlefield."
    );
    supported("Barrin, Tolarian Archmage");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    bounce(&mut t, bear);
    t.settle();
    t.battlefield(P0, "Barrin, Tolarian Archmage");
    let hand = t.hand_size(P0);
    end_step(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn barrin_doesnt_trigger_without_a_returned_permanent() {
    cr!("603.4");
    ruling!(
        "Barrin, Tolarian Archmage",
        "If a permanent wasn't put into your hand from the battlefield before your end step begins, Barrin's ability doesn't trigger at all."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barrin, Tolarian Archmage");
    // A card put into P0's hand from elsewhere doesn't count.
    let gy = t.graveyard(P0, "Grizzly Bears");
    t.g.move_object(gy, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.advance_to(P0, Step::End);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
}

#[test]
fn barrin_counts_a_returned_token() {
    cr!("111.7", "603.4");
    ruling!(
        "Barrin, Tolarian Archmage",
        "If a token is returned to your hand, it's put there before it ceases to exist."
    );
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P0, "Soldier");
    t.answer_targets(P0, &[o(token)]);
    t.enter(P0, "Barrin, Tolarian Archmage");
    t.resolve_all();
    assert!(t.named_on_battlefield("Soldier").is_empty());
    let hand = t.hand_size(P0);
    end_step(&mut t, P0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// The candidates of each object choice `p` was asked since `from`.
fn offers(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<ObjectId>> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => {
                Some(candidates.iter().filter_map(|e| e.object()).collect())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn liliana_ultimate_opponents_choose_in_order_then_sacrifice_the_rest_at_once() {
    cr!("101.4", "101.4c", "110.4", "701.21a");
    ruling!(
        "Liliana, Dreadhorde General",
        "As Liliana's last ability resolves, the next opponent in turn order (or, if it's somehow an opponent's turn, that opponent) makes all of their choices for it, then each other opponent in turn order does the same, knowing the choices made before them. Then all the unchosen permanents are sacrificed at the same time."
    );
    ruling!(
        "Liliana, Dreadhorde General",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker. Supertypes, like legendary, aren't permanent types."
    );
    supported("Liliana, Dreadhorde General");
    let mut t = TestGame::new(3);
    let lili = pw(&mut t, P0, "Liliana, Dreadhorde General", 10);
    let mine = t.battlefield(P0, "Grizzly Bears");
    // P1: two lands and two legendary creatures; keeps a land and a creature.
    let f1 = t.battlefield(P1, "Forest");
    let f2 = t.battlefield(P1, "Forest");
    let l1 = t.battlefield(P1, "Isamaru, Hound of Konda");
    let l2 = t.battlefield(P1, "Thalia, Guardian of Thraben");
    // P2: an artifact and two creatures.
    let art = t.battlefield(P2, "Bonesplitter");
    let c1 = t.battlefield(P2, "Hill Giant");
    let c2 = t.battlefield(P2, "Gray Ogre");
    t.answer_choose(P1, &[o(l2)]);
    t.answer_choose(P1, &[o(f1)]);
    t.answer_choose(P2, &[o(art)]);
    t.answer_choose(P2, &[o(c2)]);
    let seen: Vec<_> = [P1, P2]
        .into_iter()
        .map(|p| {
            watch(
                &mut t,
                p,
                |d| matches!(d, Decision::ChooseEntities { .. }),
                |g| g.permanents().count(),
            )
        })
        .collect();
    let before = t.g.permanents().count();
    let from = t.asked().len();
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve();
    let order: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    // P1 made all of its choices before P2 made any.
    let last_p1 = order.iter().rposition(|p| *p == P1).unwrap();
    let first_p2 = order.iter().position(|p| *p == P2).unwrap();
    assert!(last_p1 < first_p2);
    assert!(!order.contains(&P0));
    for s in &seen {
        assert!(s.lock().unwrap().iter().all(|n| *n == before));
    }
    // Being legendary didn't let P1 keep a second creature.
    for id in [mine, f1, l2, art, c2] {
        assert!(t.on_battlefield(id));
    }
    for id in [f2, l1, c1] {
        assert!(!t.on_battlefield(id));
    }
}

#[test]
fn liliana_ultimate_a_permanent_counts_for_any_of_its_types() {
    cr!("101.4c");
    ruling!(
        "Liliana, Dreadhorde General",
        "While making choices for Liliana's last ability, if a permanent has more than one permanent type, it can count for any of them."
    );
    // An artifact creature as the artifact, another creature as the creature, an
    // enchantment creature as the enchantment.
    let mut t = TestGame::new(2);
    let lili = pw(&mut t, P0, "Liliana, Dreadhorde General", 10);
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ench_creature = t.battlefield(P1, "Boon Satyr");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_choose(P1, &[o(thopter)]);
    t.answer_choose(P1, &[o(bears)]);
    t.answer_choose(P1, &[o(ench_creature)]);
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve();
    for id in [thopter, bears, ench_creature] {
        assert!(t.on_battlefield(id));
    }
    assert!(!t.on_battlefield(anthem));
    // An enchantment creature as both the creature and the enchantment, though there's
    // another creature and another enchantment.
    let mut t = TestGame::new(2);
    let lili = pw(&mut t, P0, "Liliana, Dreadhorde General", 10);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ench_creature = t.battlefield(P1, "Boon Satyr");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_choose(P1, &[o(ench_creature)]);
    t.answer_choose(P1, &[o(ench_creature)]);
    let from = t.asked().len();
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve();
    let offered = offers(&t, P1, from);
    assert_eq!(offered.len(), 2);
    assert!(offered.iter().all(|c| c.contains(&ench_creature)));
    assert!(t.on_battlefield(ench_creature));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(anthem));
}
