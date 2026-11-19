//! Rulings batch P164 — blocker synergies: "can block an additional creature" (Vanguard's
//! Shield, High Ground, Entourage of Trest, Foriysian Interceptor, Fibrous Entangler),
//! "whenever ~ blocks (a creature)" (Noble Stand, Brimaz, Guardian of the Gateless, Ezuri's
//! Archers, Frostweb Spider, Pride Guardian, Shielded Aether Thief), tokens created
//! attacking or blocking (Brimaz) or created in combat (Jedit Ojanen, Elder Gargaroth),
//! Gomazoa, Shape of the Wiitigo and Shield Dancer.

use crate::r_p164_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn blocked(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat.as_ref().unwrap().is_blocked(t.g.current(id))
}

/// The attackers `blocker` is blocking now.
fn blocking(t: &TestGame, blocker: ObjectId) -> Vec<ObjectId> {
    blocks_now(t)
        .into_iter()
        .filter(|(b, _)| *b == blocker)
        .map(|(_, a)| a)
        .collect()
}

#[test]
fn high_ground_and_vanguards_shield_are_cumulative() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Vanguard's Shield",
        "Abilities that allow a creature to block an additional creature are cumulative."
    );
    ruling!(
        "High Ground",
        "High Ground’s effect is cumulative. If you have a creature that can already block an additional creature, now it can block three creatures."
    );
    supported("Vanguard's Shield");
    supported("High Ground");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    let shield = t.battlefield(P1, "Vanguard's Shield");
    t.g.objects[shield.0 as usize].attached_to = Some(o(wall));
    t.g.dirty = true;
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))]);
    // The Shield alone: two.
    assert!(legal_blocks(&mut t, P1, &[(wall, a), (wall, b)]));
    assert!(!legal_blocks(&mut t, P1, &[(wall, a), (wall, b), (wall, c)]));
    // With High Ground too: three.
    t.battlefield(P1, "High Ground");
    assert!(legal_blocks(&mut t, P1, &[(wall, a), (wall, b), (wall, c)]));
    // High Ground alone lets the Foriysian Interceptor (already two) block three.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Hill Giant");
    let fi = t.battlefield(P1, "Foriysian Interceptor");
    t.battlefield(P1, "High Ground");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(fi, a), (fi, b), (fi, c)]));
}

#[test]
fn high_ground_allows_overlapping_blocks_and_the_defender_divides_damage() {
    cr!("509.1b", "509.1c", "510.1d");
    ruling!(
        "High Ground",
        "you can have X block A and B, Y block B and C, and Z block just C, among many other possible options. The defending player chooses how each blocking creature’s combat damage will be divided among the creatures it’s blocking."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Wall of Stone");
    let y = t.battlefield(P1, "Craw Wurm");
    let z = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "High Ground");
    let blocks = [(x, a), (x, b), (y, b), (y, c), (z, c)];
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))]);
    assert!(legal_blocks(&mut t, P1, &blocks));
    // Without High Ground, it isn't legal.
    // The Craw Wurm (6 power) blocking the Elves and the Hill Giant: P1 puts 1 on the
    // Elves and 5 on the Giant.
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    t.answer(P1, DecisionKind::Damage, Answer::Numbers(vec![1, 5]));
    t.advance_to(P0, Step::EndOfCombat);
    let dividers: Vec<(PlayerId, ObjectId)> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::AssignCombatDamage {
                creature,
                recipients,
                ..
            } if recipients.len() > 1 => Some((*p, *creature)),
            _ => None,
        })
        .collect();
    assert!(dividers.contains(&(P1, y)), "{dividers:?}");
    // The blocking creatures' damage is divided by P1 alone.
    assert!(dividers
        .iter()
        .filter(|(_, cr)| [x, y, z].contains(cr))
        .all(|(p, _)| *p == P1));
    assert!(!t.on_battlefield(b) && !t.on_battlefield(c));
    let mut t2 = TestGame::new(2);
    let a = t2.battlefield(P0, "Grizzly Bears");
    let b = t2.battlefield(P0, "Llanowar Elves");
    let x = t2.battlefield(P1, "Wall of Stone");
    attack_with(&mut t2, &[(a, pl(P1)), (b, pl(P1))]);
    assert!(!legal_blocks(&mut t2, P1, &[(x, a), (x, b)]));
}

#[test]
fn entourage_blocks_two_divides_damage_and_the_block_survives_losing_the_monarchy() {
    cr!("509.1b", "510.1c", "510.1d", "725.2");
    ruling!(
        "Entourage of Trest",
        "Entourage of Trest checks if you're the monarch only as blockers are declared. If it blocks two attacking creatures, another player becoming the monarch won't change or undo that block."
    );
    ruling!(
        "Entourage of Trest",
        "If a creature blocks two creatures, its combat damage is assigned to those creatures and vice versa using the same rules as if one creature were blocked by two creatures."
    );
    supported("Entourage of Trest");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let ent = t.battlefield(P1, "Entourage of Trest");
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(ent, a), (ent, b)]));
    // Not the monarch: one only.
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    assert!(!legal_blocks(&mut t, P1, &[(ent, a), (ent, b)]));
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ent, a), (ent, b)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocking(&t, ent).len(), 2);
    // P0 becomes the monarch: the block stands.
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.settle();
    assert_eq!(blocking(&t, ent).len(), 2);
    assert!(blocked(&t, a) && blocked(&t, b));
    // The Entourage's 4 damage is divided as P1 chooses (2 and 2); both attackers deal
    // their damage to it.
    t.answer(P1, DecisionKind::Damage, Answer::Numbers(vec![2, 2]));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.obj_now(ent).damage, 3);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn foriysian_interceptor_that_cant_block_blocks_nothing() {
    cr!("509.1a", "509.1b");
    ruling!(
        "Foriysian Interceptor",
        "If an effect says Foriysian Interceptor can't block, then it can't block any creatures."
    );
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let fi = t.battlefield(P1, "Foriysian Interceptor");
    let pacifism = t.battlefield(P0, "Pacifism");
    t.g.objects[pacifism.0 as usize].attached_to = Some(o(fi));
    t.g.dirty = true;
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(fi, a)]));
    assert!(!legal_blocks(&mut t, P1, &[(fi, a), (fi, b)]));
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn fibrous_entangler_needs_no_blocker_when_none_can_block_for_free() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Tangleclaw Werewolf // Fibrous Entangler",
        "If each creature that could block Fibrous Entangler is tapped or is affected by a spell or ability that says it can't block, then none of them block. If there's a cost associated with having each of those creatures block, its controller isn't forced to pay the cost, so they don't have to block in that case either."
    );
    supported("War Cadence");
    let entangler = |t: &mut TestGame| {
        let ent = t.battlefield(P0, "Tangleclaw Werewolf // Fibrous Entangler");
        assert!(mtg_engine::dfc::transform(&mut t.g, ent));
        t.g.recompute();
        t.g.current(ent)
    };
    // Tapped or can't block: no block is required.
    let mut t = TestGame::new(2);
    let ent = entangler(&mut t);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    let pacified = t.battlefield(P1, "Hill Giant");
    let pacifism = t.battlefield(P0, "Pacifism");
    t.g.objects[pacifism.0 as usize].attached_to = Some(o(pacified));
    t.g.dirty = true;
    attack_with(&mut t, &[(ent, pl(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    // A creature that could block for free must.
    t.battlefield(P1, "Llanowar Elves");
    assert!(!legal_blocks(&mut t, P1, &[]));
    // War Cadence ("creatures can't block unless their controller pays {X} for each
    // blocking creature they control"): P1 doesn't have to pay, so needn't block.
    let mut t = TestGame::new(2);
    let ent = entangler(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    attack_with(&mut t, &[(ent, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    let cadence = t.battlefield(P0, "War Cadence");
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, cadence, 0, &[]).unwrap();
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(blocks_now(&t).is_empty());
    assert!(!blocked(&t, ent));
    assert!(t.on_battlefield(bears));
}

#[test]
fn shape_of_the_wiitigo_counters_stay_after_it_leaves() {
    cr!("122.2", "603.2");
    ruling!(
        "Shape of the Wiitigo",
        "If Shape of the Wiitigo leaves the battlefield but the creature doesn't, the +1/+1 counters stay on the creature."
    );
    supported("Shape of the Wiitigo");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 6);
    let shape = t.hand(P0, "Shape of the Wiitigo");
    t.cast(P0, shape).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 6);
    let shape = t.named_on_battlefield("Shape of the Wiitigo")[0];
    bounce(&mut t, shape);
    t.settle();
    assert_eq!(t.counters(bears, counters::PLUS1), 6);
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn noble_stand_triggers_once_per_blocking_creature() {
    cr!("509.1i", "603.2");
    ruling!(
        "Noble Stand",
        "Triggers once per creature you have that blocks. Does not trigger multiple times when a single creature blocks more than one attacker."
    );
    supported("Noble Stand");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Hill Giant");
    let stand = t.battlefield(P1, "Noble Stand");
    let wall = t.battlefield(P1, "Wall of Stone");
    let other = t.battlefield(P1, "Craw Wurm");
    valor_made_real(&mut t, wall);
    to_blockers(
        &mut t,
        &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))],
        &[(wall, a), (wall, b), (other, c)],
    );
    assert_eq!(triggers_of(&t, stand), 2);
    t.resolve_all();
    assert_eq!(gains(&t, P1), vec![2, 2]);
}

#[test]
fn brimaz_tokens_were_never_declared_and_may_attack_another_player() {
    cr!("508.4", "509.4", "508.3a", "509.1i");
    ruling!(
        "Brimaz, King of Oreskos",
        "Although the tokens enter the battlefield attacking or blocking, they were never declared as attacking or blocking creatures. Abilities that trigger whenever a creature attacks or blocks won't trigger."
    );
    ruling!(
        "Brimaz, King of Oreskos",
        "you declare which player, planeswalker, or battle the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Brimaz is attacking."
    );
    supported("Brimaz, King of Oreskos");
    supported("Hellrider");
    // Attacking in a three-player game: Brimaz attacks P1, the token attacks P2.
    // Hellrider ("Whenever a creature you control attacks, ...") triggers only for Brimaz.
    let mut t = TestGame::new(3);
    let brimaz = t.battlefield(P0, "Brimaz, King of Oreskos");
    let rider = t.battlefield(P0, "Hellrider");
    attack_with(&mut t, &[(brimaz, pl(P1))]);
    assert_eq!(triggers_of(&t, rider), 1);
    t.answer_choose(P0, &[pl(P2)]);
    t.resolve_all();
    let cat = tokens(&t, P0)[0];
    let combat = t.g.combat.as_ref().unwrap();
    assert!(t.g.is_attacking(cat));
    assert_eq!(combat.defending_player_of(&t.g, cat), Some(P2));
    assert_eq!(combat.defending_player_of(&t.g, brimaz), Some(P1));
    assert_eq!(triggers_of(&t, rider), 1);
    // Blocking: Noble Stand triggers for Brimaz only, not the token.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let brimaz = t.battlefield(P1, "Brimaz, King of Oreskos");
    let stand = t.battlefield(P1, "Noble Stand");
    to_blockers(&mut t, &[(giant, pl(P1))], &[(brimaz, giant)]);
    t.resolve_all();
    let cat = tokens(&t, P1)[0];
    assert_eq!(blocking(&t, cat), vec![giant]);
    assert_eq!(triggers_of(&t, stand), 1);
    assert_eq!(gains(&t, P1), vec![2]);
}

#[test]
fn per_creature_block_triggers_trigger_for_each_creature_blocked() {
    cr!("509.1i", "603.2");
    ruling!(
        "Brimaz, King of Oreskos",
        "If Brimaz somehow blocks two or more creatures (note it can't naturally do this), its last ability will trigger that many times."
    );
    ruling!(
        "Ezuri's Archers",
        "If Ezuri’s Archers somehow blocks multiple creatures with flying (perhaps because it’s equipped by Echo Circlet), its ability triggers that many times."
    );
    ruling!(
        "Frostweb Spider",
        "If Frostweb Spider somehow blocks more than one creature with flying during the same combat, its ability will trigger multiple times."
    );
    supported("Ezuri's Archers");
    supported("Frostweb Spider");
    // Brimaz: one token blocking each.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let brimaz = t.battlefield(P1, "Brimaz, King of Oreskos");
    valor_made_real(&mut t, brimaz);
    to_blockers(&mut t, &[(a, pl(P1)), (b, pl(P1))], &[(brimaz, a), (brimaz, b)]);
    assert_eq!(triggers_of(&t, brimaz), 2);
    t.resolve_all();
    let cats = tokens(&t, P1);
    assert_eq!(cats.len(), 2);
    let mut blocked_by_cats: Vec<ObjectId> = cats.iter().flat_map(|c| blocking(&t, *c)).collect();
    blocked_by_cats.sort();
    let mut ab = vec![a, b];
    ab.sort();
    assert_eq!(blocked_by_cats, ab);
    // Ezuri's Archers: +3/+0 per flier.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Birds of Paradise");
    let b = t.battlefield(P0, "Birds of Paradise");
    let archers = t.battlefield(P1, "Ezuri's Archers");
    valor_made_real(&mut t, archers);
    to_blockers(&mut t, &[(a, pl(P1)), (b, pl(P1))], &[(archers, a), (archers, b)]);
    assert_eq!(triggers_of(&t, archers), 2);
    t.resolve_all();
    assert_eq!(t.pt(archers), (7, 2));
    // Frostweb Spider: two counters at end of combat.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Birds of Paradise");
    let b = t.battlefield(P0, "Birds of Paradise");
    let spider = t.battlefield(P1, "Frostweb Spider");
    valor_made_real(&mut t, spider);
    to_blockers(&mut t, &[(a, pl(P1)), (b, pl(P1))], &[(spider, a), (spider, b)]);
    assert_eq!(triggers_of(&t, spider), 2);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(spider, counters::PLUS1), 2);
}

#[test]
fn frostweb_spider_dies_before_it_gets_the_counter() {
    cr!("510.2", "704.5g", "603.7");
    ruling!(
        "Frostweb Spider",
        "If Frostweb Spider blocks a creature with flying that’s just powerful enough to kill it, it will die before it can get the +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let spider = t.battlefield(P1, "Frostweb Spider");
    to_blockers(&mut t, &[(angel, pl(P1))], &[(spider, angel)]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(spider));
    assert!(t.in_graveyard(P1, "Frostweb Spider"));
}

#[test]
fn once_per_block_triggers_trigger_once_however_many_are_blocked() {
    cr!("509.1i", "603.2");
    ruling!(
        "Guardian of the Gateless",
        "Guardian of the Gateless's last ability triggers only once when it's declared as a blocker no matter how many creatures it's blocking."
    );
    ruling!(
        "Pride Guardian",
        "If Pride Guardian gains the ability to block additional creatures and does so, you’ll still gain only 3 life."
    );
    ruling!(
        "Shielded Aether Thief",
        "If Shielded Aether Thief somehow blocks more than one creature at once, you get only one {E}."
    );
    supported("Guardian of the Gateless");
    supported("Pride Guardian");
    supported("Shielded Aether Thief");
    let two_attackers_blocked_by = |name: &str, extra: bool| {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Llanowar Elves");
        let c = t.battlefield(P0, "Hill Giant");
        let blocker = t.battlefield(P1, name);
        if extra {
            valor_made_real(&mut t, blocker);
        }
        to_blockers(
            &mut t,
            &[(a, pl(P1)), (b, pl(P1)), (c, pl(P1))],
            &[(blocker, a), (blocker, b), (blocker, c)],
        );
        assert_eq!(triggers_of(&t, blocker), 1, "{name}");
        t.resolve_all();
        (t, blocker)
    };
    // Guardian of the Gateless: one trigger, +1/+1 for each of the three.
    let (t, g) = two_attackers_blocked_by("Guardian of the Gateless", false);
    assert_eq!(t.pt(g), (6, 6));
    let (t, _) = two_attackers_blocked_by("Pride Guardian", true);
    assert_eq!(gains(&t, P1), vec![3]);
    let (t, _) = two_attackers_blocked_by("Shielded Aether Thief", true);
    assert_eq!(energy_of(&t, P1), 1);
}

#[test]
fn tokens_created_in_combat_are_too_late_to_attack_or_block() {
    cr!("508.1a", "509.1a", "506.3");
    ruling!(
        "Jedit Ojanen of Efrava",
        "If Jedit Ojanen of Efrava attacks or blocks and a token is created, it will be too late for that token to be declared as an attacker (if it somehow gets haste) or a blocker during that combat phase."
    );
    ruling!(
        "Elder Gargaroth",
        "If you create a Beast token, that token can't attack or block during the same combat, even if it gains haste."
    );
    supported("Jedit Ojanen of Efrava");
    // Attacking: the token isn't attacking, and no second declaration is asked for.
    for name in ["Jedit Ojanen of Efrava", "Elder Gargaroth"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        attack_with(&mut t, &[(c, pl(P1))]);
        t.resolve_all();
        let token = tokens(&t, P0)[0];
        // It gains haste (Fervor-like): still too late.
        t.g.objects[token.0 as usize].summoning_sick = false;
        let before = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::DeclareAttackers { .. }))
            .count();
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(attackers_now(&t), vec![c], "{name}");
        let after = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::DeclareAttackers { .. }))
            .count();
        assert_eq!(before, after, "{name}");
    }
    // Blocking: the token doesn't block.
    for name in ["Jedit Ojanen of Efrava", "Elder Gargaroth"] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Llanowar Elves");
        let c = t.battlefield(P1, name);
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
        to_blockers(&mut t, &[(a, pl(P1)), (b, pl(P1))], &[(c, a)]);
        t.resolve_all();
        let token = tokens(&t, P1)[0];
        t.advance_to(P0, Step::EndOfCombat);
        assert!(blocking(&t, token).is_empty(), "{name}");
        assert!(t.on_battlefield(b));
        assert_eq!(t.life(P1), 19, "{name}: the Elves were unblocked");
    }
}

#[test]
fn gomazoa_can_be_activated_without_blocking() {
    cr!("602.2", "602.5");
    ruling!(
        "Gomazoa",
        "Gomazoa’s ability can be activated any time its controller has priority. It doesn’t need to be blocking any creatures."
    );
    supported("Gomazoa");
    let mut t = TestGame::new(2);
    let goma = t.battlefield(P0, "Gomazoa");
    let lib = t.library_size(P0);
    t.activate(P0, goma, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(goma));
    assert_eq!(t.library_size(P0), lib + 1);
    // On the opponent's turn, outside combat, too.
    let mut t = TestGame::new(2);
    let goma = t.battlefield(P1, "Gomazoa");
    t.advance_to(P0, Step::Upkeep);
    t.activate(P1, goma, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(goma));
}

#[test]
fn shield_dancer_used_twice_reflects_the_next_two_attacks_once_each() {
    cr!("615.1", "616.1", "510.2", "506.1");
    ruling!(
        "Shield Dancer",
        "If the ability is used twice, it will cause the next two attacks this turn (which only happens if something gives a player two attacks) from the creature to “reflect” back to itself. It will not result in double damage back from a single attack."
    );
    supported("Shield Dancer");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    // Siege Mastodon, 3/5: survives one reflected attack.
    let mastodon = t.battlefield(P0, "Siege Mastodon");
    let dancer = t.battlefield(P1, "Shield Dancer");
    t.lands(P1, "Plains", 6);
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(mastodon, pl(P1))]);
    for _ in 0..2 {
        t.activate(P1, dancer, 0, &[o(mastodon)]).unwrap();
        t.resolve_all();
    }
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(dancer, mastodon)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    // The 3 damage is dealt to itself once (3, not 6); the Dancer is unharmed.
    assert_eq!(damage_from(&t, mastodon), vec![(o(mastodon), 3)]);
    assert_eq!(t.obj_now(dancer).damage, 0);
    // (plus the Dancer's own 1 combat damage)
    assert_eq!(t.obj_now(mastodon).damage, 4);
    // A second attack this turn (Relentless Assault): the second shield reflects it too.
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 4);
    let assault = t.hand(P0, "Relentless Assault");
    t.cast(P0, assault).go();
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(mastodon, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(dancer, mastodon)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(
        damage_from(&t, mastodon),
        vec![(o(mastodon), 3), (o(mastodon), 3)]
    );
    assert_eq!(t.obj_now(dancer).damage, 0);
    assert!(!t.on_battlefield(mastodon));
}
