//! Rulings residue `rulings-residue-hate-attack-block` (P079): Mirror Match's tokens enter
//! the battlefield blocking the creatures they copy (CR 509.4): never declared as blockers,
//! so no block restriction, requirement, cost or "whenever ~ blocks" trigger applies
//! (CR 509.4, 509.4b), and they copy only copiable values (CR 707.2, 111.3).

use crate::r_p079_common::cast_card;
use crate::r_residue_hate_attack_block_common::*;
use crate::r_s01_common::{supported, tokens};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks P1 with `attackers` and no blocks are declared; P1 casts Mirror Match in the
/// declare blockers step and it resolves. P1's tokens.
fn mirror_match(t: &mut TestGame, attackers: &[ObjectId]) -> Vec<ObjectId> {
    supported("Mirror Match");
    blocks(t, attackers, &[]);
    cast_card(t, P1, "Mirror Match", &[]).expect("Mirror Match");
    t.resolve_all();
    tokens(t, P1)
}

/// The token among `toks` blocking `attacker`.
fn blockers_of(t: &TestGame, toks: &[ObjectId], attacker: ObjectId) -> Vec<ObjectId> {
    toks.iter()
        .copied()
        .filter(|x| blocking(t, *x) == vec![attacker])
        .collect()
}

#[test]
fn tokens_block_attackers_that_cant_be_blocked_or_that_they_couldnt_block() {
    cr!("509.4", "509.4b");
    ruling!(
        "Mirror Match",
        "A token created by Mirror Match will be blocking the creature it’s copying even if that creature can’t be blocked or has an ability that would stop the token creature from blocking it (such as menace or protection). This is also true even if the token creature has an ability that would stop it from being declared as a blocker."
    );
    // Phantom Warrior can't be blocked, Boggart Brute has menace, Beloved Chaplain has
    // protection from creatures, and Hulking Goblin can't block.
    let names = [
        "Phantom Warrior",
        "Boggart Brute",
        "Beloved Chaplain",
        "Hulking Goblin",
    ];
    let mut t = TestGame::new(2);
    let attackers: Vec<ObjectId> = names.iter().map(|n| t.battlefield(P0, n)).collect();
    let toks = mirror_match(&mut t, &attackers);
    assert_eq!(toks.len(), 4, "{}", t.dump_log());
    for (a, name) in attackers.iter().zip(names) {
        let b = blockers_of(&t, &toks, *a);
        assert_eq!(b.len(), 1, "{name}");
        assert_eq!(t.obj_now(b[0]).chars.name.as_str(), name);
    }
}

#[test]
fn tokens_were_never_declared_blockers_no_block_triggers_or_costs() {
    cr!("509.4", "509.3a", "509.1d");
    ruling!(
        "Mirror Match",
        "Although the tokens enter the battlefield blocking, they were never declared as blockers. Abilities that trigger whenever a creature blocks won’t trigger. If there are any costs to have a creature block, those costs won’t apply to the tokens."
    );
    supported("Giant Badger");
    // P0 made blocking cost {3} for each blocker with War Cadence; P1 has no mana left
    // after Mirror Match. The copy of Giant Badger ("Whenever this creature blocks, it
    // gets +2/+2") blocks without paying and doesn't trigger.
    let mut t = TestGame::new(2);
    let cadence = t.battlefield(P0, "War Cadence");
    let badger = t.battlefield(P0, "Giant Badger");
    t.set_step(P0, Step::BeginningOfCombat);
    t.lands(P0, "Mountain", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, cadence, 0, &[]).unwrap();
    t.resolve_all();
    let toks = mirror_match(&mut t, &[badger]);
    assert_eq!(toks.len(), 1);
    assert_eq!(blocking(&t, toks[0]), vec![badger]);
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn tokens_copy_only_the_printed_values() {
    cr!("707.2", "707.3");
    ruling!(
        "Mirror Match",
        "Each token copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or is a token; see below). It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    // A tapped attacking Hill Giant with a +1/+1 counter, Holy Strength (+1/+2) and Giant
    // Growth (+3/+3).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.obj_mut(giant)
        .counters
        .insert(mtg_engine::types::counters::PLUS1.into(), 1);
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", giant);
    blocks(&mut t, &[giant], &[]);
    cast_card(&mut t, P0, "Giant Growth", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (8, 9));
    assert!(tapped(&t, giant));
    cast_card(&mut t, P1, "Mirror Match", &[]).unwrap();
    t.resolve_all();
    let toks = tokens(&t, P1);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert_eq!(t.pt(tok), (3, 3));
    assert!(!tapped(&t, tok));
    assert_eq!(t.counters(tok, mtg_engine::types::counters::PLUS1), 0);
    assert!(t.g.attachments_of(obj(t.g.current(tok))).is_empty());
}

#[test]
fn doubled_tokens_all_block_and_are_all_exiled() {
    cr!("509.4", "614.1a", "603.7c");
    ruling!(
        "Mirror Match",
        "If Mirror Match creates more than one token for any given attacking creature (due to an effect such as the one Doubling Season creates), each of those tokens will enter the battlefield blocking the creature it’s a copy of. All of those tokens will be exiled by the delayed triggered ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Doubling Season");
    let thopter = t.battlefield(P0, "Ornithopter");
    let toks = mirror_match(&mut t, &[thopter]);
    assert_eq!(toks.len(), 2, "{}", t.dump_log());
    for tok in &toks {
        assert_eq!(blocking(&t, *tok), vec![thopter]);
    }
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert!(tokens(&t, P1).is_empty(), "{}", t.dump_log());
}

#[test]
fn a_token_attacker_is_copied_as_its_creating_effect_made_it() {
    cr!("707.2", "111.3");
    ruling!(
        "Mirror Match",
        "If the copied creature is a token, the token created by Mirror Match copies the original characteristics of that token as stated by the effect that put the token onto the battlefield."
    );
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Raise the Alarm", &[]).unwrap();
    t.resolve_all();
    let soldiers = tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    let s = soldiers[0];
    // (Not summoning sick: as if created last turn.)
    t.g.obj_mut(s).summoning_sick = false;
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", s);
    let toks = mirror_match(&mut t, &[s]);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert_eq!(t.pt(s), (2, 3));
    assert_eq!(t.pt(tok), (1, 1));
    assert!(t.obj_now(tok).chars.subtypes.contains(&"Soldier".into()));
    assert!(t
        .obj_now(tok)
        .chars
        .colors
        .contains(mtg_engine::types::Color::White));
}
