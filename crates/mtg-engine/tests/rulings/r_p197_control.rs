//! Rulings batch P197 — control-changing effects with no duration (CR 611.2a, 613.1b):
//! they outlast the turn and their source, end when the controller leaves the game, and
//! the stolen permanent leaves with its owner (CR 800.4a); exchanges (CR 701.12); choices
//! and votes made while a spell resolves (CR 608.2, 701.38).

use crate::r_p197_common::*;
use crate::r_s02_common::can_attack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 takes P1's `victim` with `card` (cast, activated or triggered as the card does it,
/// everything resolving). Returns the permanent the effect came from, if any.
fn steal(t: &mut TestGame, card: &str, victim: ObjectId) -> Option<ObjectId> {
    supported(card);
    t.set_step(P0, Step::PrecombatMain);
    let src = match card {
        "Lullmage's Domination" | "Entrancing Melody" => {
            lots(t, P0);
            let x = i64::from(t.obj_now(victim).chars.mana_value());
            let c = t.hand(P0, card);
            t.cast(P0, c).x(x).target(victim).go();
            None
        }
        "Mass Manipulation" => {
            lots(t, P0);
            let c = t.hand(P0, card);
            t.cast(P0, c).x(1).target(victim).go();
            None
        }
        "Exert Influence" => {
            // Two colors spent ({U}{R} and three generic): power 2 or less.
            mana(t, P0, ManaType::U, 1);
            mana(t, P0, ManaType::R, 1);
            mana(t, P0, ManaType::C, 3);
            let c = t.hand(P0, card);
            t.cast(P0, c).target(victim).go();
            None
        }
        "Kefnet's Last Word" | "Tempted by the Oriq" => {
            cast_at(t, P0, card, &[obj(victim)]);
            None
        }
        "Rags // Riches" => {
            lots(t, P0);
            let c = t.graveyard(P0, card);
            t.answer_choose(P1, &[obj(victim)]);
            t.cast(P0, c)
                .method(CastMethod::Keyword(KeywordKind::Aftermath))
                .go();
            None
        }
        "Beguiler of Wills" => {
            let b = t.battlefield(P0, card);
            t.battlefield(P0, "Hill Giant");
            act_only(t, P0, b, 0, &[obj(victim)]);
            Some(b)
        }
        "Dack Fayden" => {
            let d = walker(t, P0, card, 3);
            act_only(t, P0, d, 1, &[obj(victim)]);
            Some(d)
        }
        "Jace, Ingenious Mind-Mage" => {
            let j = walker(t, P0, card, 10);
            t.answer_targets(P0, &[obj(victim)]);
            act_only(t, P0, j, 2, &[]);
            Some(j)
        }
        "Eyes Everywhere" => {
            let e = t.battlefield(P0, card);
            act_only(t, P0, e, 0, &[obj(victim)]);
            Some(e)
        }
        "Karrthus, Tyrant of Jund" => {
            let k = t.enter(P0, card);
            t.g.flush_events();
            t.settle();
            Some(k)
        }
        "Kiora Bests the Sea God" => {
            let s = blank_saga(t, P0, card);
            t.g.objects[s.0 as usize].counters.insert("lore".into(), 2);
            t.answer_targets(P0, &[obj(victim)]);
            add_lore(t, s, 1);
            Some(s)
        }
        "Emrakul, the World Anew" => {
            lots(t, P0);
            t.g.players[0].mana_pool.add_type(ManaType::C, 12);
            let c = t.hand(P0, card);
            t.cast(P0, c).target(pl(P1)).go();
            t.settle();
            t.resolve_all();
            Some(t.named_on_battlefield(card)[0])
        }
        "Confiscation Coup" => {
            lots(t, P0);
            t.answer_yes(P0, true);
            cast_at(t, P0, card, &[obj(victim)]);
            None
        }
        "Souvenir Snatcher" => {
            let host = t.battlefield(P0, "Hill Giant");
            lots(t, P0);
            let c = t.hand(P0, card);
            t.cast(P0, c)
                .method(CastMethod::Keyword(KeywordKind::Mutate))
                .target(host)
                .go();
            t.answer(P0, DecisionKind::Option, Answer::Index(0));
            t.answer_targets(P0, &[obj(victim)]);
            t.resolve();
            t.settle();
            Some(t.g.current(host))
        }
        other => panic!("no steal for {other}"),
    };
    t.resolve_all();
    assert_eq!(ctl(t, victim), P0, "{card} took it");
    src
}

/// Two full turns pass (each ends with a cleanup step).
fn two_turns(t: &mut TestGame) {
    pass_turn(t);
    pass_turn(t);
}

#[test]
fn control_changes_with_no_duration_outlast_the_turn() {
    cr!("611.2a", "613.1b", "514.2", "400.7", "800.4a");
    ruling!(
        "Lullmage's Domination",
        "Once Lullmage's Domination resolves, the control effect lasts until the creature leaves the battlefield or you lose the game. It doesn't expire at the end of turn."
    );
    ruling!(
        "Rags // Riches",
        "Riches's effect lasts indefinitely. It doesn't wear off during the cleanup step."
    );
    ruling!(
        "Tempted by the Oriq",
        "Tempted by the Oriq’s control-changing effect lasts indefinitely."
    );
    ruling!(
        "Entrancing Melody",
        "The control-change effect of Entrancing Melody lasts indefinitely. It doesn't wear off during the cleanup step."
    );
    ruling!(
        "Kefnet's Last Word",
        "The control-change effect of Kefnet’s Last Word lasts indefinitely. It doesn’t wear off during the cleanup step."
    );
    ruling!(
        "Mass Manipulation",
        "The control-change effect of Mass Manipulation lasts indefinitely. It doesn’t wear off during the cleanup step."
    );
    for card in [
        "Lullmage's Domination",
        "Rags // Riches",
        "Tempted by the Oriq",
        "Entrancing Melody",
        "Kefnet's Last Word",
        "Mass Manipulation",
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        steal(&mut t, card, bears);
        two_turns(&mut t);
        assert_eq!(ctl(&t, bears), P0, "{card}");
    }
    // Lullmage's Domination: it ends when the creature leaves the battlefield (the owner
    // gets the new object), or when you lose the game (three players).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    steal(&mut t, "Lullmage's Domination", bears);
    bounce(&mut t, bears);
    let card = t.g.find_in_zone(Zone::Hand(P1), "Grizzly Bears")[0];
    t.g.move_object(
        card,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
    let again = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(ctl(&t, again), P1);
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    steal(&mut t, "Lullmage's Domination", bears);
    concede(&mut t, P0);
    // Back under its owner's control on the battlefield (not exiled as an object P0 still
    // controlled as they left).
    assert!(t.on_battlefield(bears));
    assert_eq!(ctl(&t, bears), P1);
}

#[test]
fn control_that_lasts_ignores_the_creatures_power_later() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Beguiler of Wills",
        "The control-changing effect lasts indefinitely, even if the power of the creature becomes greater than the number of creatures you control after the ability resolves."
    );
    ruling!(
        "Exert Influence",
        "Once you’ve gained control of a creature with Exert Influence, it doesn’t matter what happens to its power. Raising it won’t cause the control-changing effect to end."
    );
    for card in ["Beguiler of Wills", "Exert Influence"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        steal(&mut t, card, bears);
        t.g.add_counters(obj(bears), "+1/+1", 5, None);
        t.g.recompute();
        assert_eq!(t.pt(bears).0, 7);
        two_turns(&mut t);
        assert_eq!(ctl(&t, bears), P0, "{card}");
    }
}

#[test]
fn role_reversal_lasts_after_the_permanents_stop_sharing_a_type() {
    cr!("611.2a", "701.12a", "613.1b");
    ruling!(
        "Role Reversal",
        "Role Reversal’s effect lasts indefinitely. It doesn’t wear off during the cleanup step, and it doesn’t expire if the permanents stop sharing a permanent type after Role Reversal has resolved."
    );
    supported("Role Reversal");
    supported("Mutavault");
    // P0's Mutavault, animated until end of turn, and P1's Bears share "creature".
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let vault = t.battlefield(P0, "Mutavault");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, vault, 1, &[]);
    assert!(t.obj_now(vault).is(mtg_engine::types::CardType::Creature));
    cast_multi(&mut t, P0, "Role Reversal", &[obj(vault), obj(bears)]);
    t.resolve_all();
    assert_eq!((ctl(&t, vault), ctl(&t, bears)), (P1, P0));
    two_turns(&mut t);
    // The Mutavault is just a land again; the exchange still holds.
    assert!(!t.obj_now(vault).is(mtg_engine::types::CardType::Creature));
    assert_eq!((ctl(&t, vault), ctl(&t, bears)), (P1, P0));
}

#[test]
fn control_outlasts_the_source_leaving_or_changing_hands() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Emrakul, the World Anew",
        "The control-change effect of Emrakul's first ability lasts indefinitely. It doesn't wear off during the cleanup step or when Emrakul leaves the battlefield."
    );
    ruling!(
        "Jace, Ingenious Mind-Mage",
        "The control-change effect of Jace’s last ability lasts indefinitely. It doesn’t wear off during the cleanup step or when you lose control of Jace."
    );
    ruling!(
        "Karrthus, Tyrant of Jund",
        "The control-change effect of Karrthus's triggered ability lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if Karrthus leaves the battlefield. In a multiplayer game, it does expire if you leave the game."
    );
    ruling!(
        "Souvenir Snatcher",
        "The control-change effect of Souvenir Snatcher lasts indefinitely. It doesn’t wear off during the cleanup step, and it doesn’t expire if Souvenir Snatcher leaves the battlefield. In a multiplayer game, it does expire if you leave the game. If the artifact’s owner leaves the game, that artifact leaves with them."
    );
    ruling!(
        "Dack Fayden",
        "The effect of Dack's second ability and the effect of the emblem's ability last indefinitely. You won't lose control of the permanents if Dack leaves the battlefield."
    );
    ruling!(
        "Eyes Everywhere",
        "The effect of Eyes Everywhere’s last ability lasts indefinitely. It doesn’t wear off during the cleanup step, and it doesn’t expire if Eyes Everywhere leaves the battlefield."
    );
    ruling!(
        "Kiora Bests the Sea God",
        "The control-change effect of the third chapter ability lasts indefinitely. It doesn’t wear off during the cleanup step. In a multiplayer game, it does expire if you leave the game."
    );
    for (card, victim) in [
        ("Emrakul, the World Anew", "Grizzly Bears"),
        ("Jace, Ingenious Mind-Mage", "Grizzly Bears"),
        ("Karrthus, Tyrant of Jund", "Shivan Dragon"),
        ("Souvenir Snatcher", "Sol Ring"),
        ("Dack Fayden", "Sol Ring"),
        ("Eyes Everywhere", "Grizzly Bears"),
        ("Kiora Bests the Sea God", "Grizzly Bears"),
    ] {
        let mut t = TestGame::new(2);
        let v = t.battlefield(P1, victim);
        let src = steal(&mut t, card, v);
        two_turns(&mut t);
        assert_eq!(ctl(&t, v), P0, "{card}: after the cleanup steps");
        let Some(src) = src else { continue };
        match card {
            // Jace changes hands instead.
            "Jace, Ingenious Mind-Mage" => {
                give_control(&mut t, src, P1);
                assert_eq!(ctl(&t, src), P1);
            }
            // Emrakul's leaves trigger will sacrifice the creatures P0 controls; until it
            // resolves, P0 still controls them.
            "Emrakul, the World Anew" => {
                exile(&mut t, src);
                assert_eq!(t.stack_len(), 1);
                assert_eq!(ctl(&t, v), P0, "{card}: Emrakul left");
                t.resolve_all();
                assert!(
                    t.in_graveyard(P1, victim),
                    "sacrificed, to its owner's graveyard"
                );
                continue;
            }
            // The Saga is gone already (sacrificed after chapter III).
            "Kiora Bests the Sea God" => assert!(!t.on_battlefield(src)),
            _ => exile(&mut t, src),
        }
        t.resolve_all();
        pass_turn(&mut t);
        assert_eq!(ctl(&t, v), P0, "{card}: after its source left");
    }
}

#[test]
fn a_stolen_permanent_leaves_with_its_owner_and_returns_if_you_leave() {
    cr!("800.4a", "611.2a", "613.1b");
    ruling!(
        "Dack Fayden",
        "If you gain control of a permanent another player owns, and that player leaves the game, the permanent will also leave the game."
    );
    ruling!(
        "Dack Fayden",
        "If you gain control of a permanent, and you leave the game, the control-changing effect will end. Unless there's another control-changing effect affecting that permanent, it will return to its owner's control."
    );
    ruling!(
        "Confiscation Coup",
        "In a multiplayer game, if a permanent's owner leaves the game after you've gained control of it, the permanent leaves with that player. If you leave the game before that player, the control-change effect ends."
    );
    ruling!(
        "Eyes Everywhere",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the effect giving you control of the target nonland permanent ends."
    );
    // (The Karrthus, Souvenir Snatcher and Kiora rulings cited above say the same.)
    for (card, victim) in [
        ("Dack Fayden", "Sol Ring"),
        ("Confiscation Coup", "Grizzly Bears"),
        ("Eyes Everywhere", "Grizzly Bears"),
        ("Karrthus, Tyrant of Jund", "Shivan Dragon"),
        ("Souvenir Snatcher", "Sol Ring"),
        ("Kiora Bests the Sea God", "Grizzly Bears"),
    ] {
        // The owner leaves: the permanent leaves the game with them.
        let mut t = TestGame::new(3);
        let v = t.battlefield(P1, victim);
        steal(&mut t, card, v);
        concede(&mut t, P1);
        assert!(!in_game(&t, v), "{card}: left with its owner");
        // You leave: it returns to its owner.
        let mut t = TestGame::new(3);
        let v = t.battlefield(P1, victim);
        steal(&mut t, card, v);
        concede(&mut t, P0);
        assert!(t.on_battlefield(v), "{card}");
        assert_eq!(ctl(&t, v), P1, "{card}: back to its owner");
    }
    // Another control-changing effect still applies: P2 had taken the Sol Ring first.
    let mut t = TestGame::new(3);
    let ring = t.battlefield(P1, "Sol Ring");
    give_control(&mut t, ring, P2);
    // Dack's -2 targets it from P2.
    let d = walker(&mut t, P0, "Dack Fayden", 3);
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, d, 1, &[obj(ring)]);
    assert_eq!(ctl(&t, ring), P0);
    concede(&mut t, P0);
    assert_eq!(ctl(&t, ring), P2);
}

#[test]
fn liliana_emblem_creatures_are_exiled_if_you_leave() {
    cr!("800.4a", "110.2");
    ruling!(
        "Liliana, Waker of the Dead",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any permanents you control from Liliana's emblem are exiled."
    );
    supported("Liliana, Waker of the Dead");
    let setup = |t: &mut TestGame| -> (ObjectId, ObjectId) {
        let l = walker(t, P0, "Liliana, Waker of the Dead", 7);
        t.set_step(P0, Step::PrecombatMain);
        act(t, P0, l, 2, &[]);
        let theirs = t.graveyard(P1, "Grizzly Bears");
        let mine = t.graveyard(P0, "Hill Giant");
        // Two combats: one creature card from each graveyard.
        t.answer_targets(P0, &[obj(theirs)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        pass_turn(t);
        pass_turn(t);
        pass_turn(t);
        t.answer_targets(P0, &[obj(mine)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let theirs = t.g.current(theirs);
        let mine = t.g.current(mine);
        assert_eq!((ctl(t, theirs), ctl(t, mine)), (P0, P0));
        assert!(t.on_battlefield(theirs) && t.on_battlefield(mine));
        (theirs, mine)
    };
    // P0 leaves: the creature P1 owns is exiled (P0 still controls it: it was put onto
    // the battlefield under P0's control, no control-changing effect to end).
    let mut t = TestGame::new(3);
    let (theirs, mine) = setup(&mut t);
    concede(&mut t, P0);
    assert_eq!(t.zone(theirs), Zone::Exile);
    assert!(!in_game(&t, mine));
    // P1 leaves: their card leaves the game with them.
    let mut t = TestGame::new(3);
    let (theirs, mine) = setup(&mut t);
    concede(&mut t, P1);
    assert!(!in_game(&t, theirs));
    assert!(t.on_battlefield(mine));
}

#[test]
fn admiral_beckett_brass_control_outlasts_the_admiral() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Admiral Beckett Brass",
        "The control-change effect of Admiral Beckett Brass's triggered ability lasts indefinitely. It doesn't wear off during the cleanup step or when you lose control of Admiral Beckett Brass."
    );
    supported("Admiral Beckett Brass");
    let mut t = TestGame::new(2);
    let admiral = t.battlefield(P0, "Admiral Beckett Brass");
    let ps: Vec<ObjectId> = (0..3)
        .map(|_| vanilla(&mut t, P0, "Test Pirate", "Creature — Human Pirate", (1, 1)))
        .collect();
    let ring = t.battlefield(P1, "Sol Ring");
    t.advance_to(P0, Step::BeginningOfCombat);
    let decl: Vec<(ObjectId, Entity)> = ps.iter().map(|p| (*p, pl(P1))).collect();
    t.attack(&decl, &[]);
    assert_eq!(t.life(P1), 14);
    t.answer_targets(P0, &[obj(ring)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(ctl(&t, ring), P0);
    give_control(&mut t, admiral, P1);
    two_turns(&mut t);
    assert_eq!(ctl(&t, ring), P0);
}

#[test]
fn admiral_beckett_brass_counts_a_teammates_pirates_in_two_headed_giant() {
    cr!("810.2", "810.7", "805.10b", "810.9");
    ruling!(
        "Admiral Beckett Brass",
        "In a Two-Headed Giant game, combat damage dealt by Pirates your teammate controls will count when checking for Admiral Beckett Brass's ability."
    );
    // Two Pirates of P0's and one of P1's (P0's teammate) attack the other team: all three
    // at P2, or the teammate's at P3 (then no single player was dealt combat damage by
    // three Pirates, CR 810.9: damage is dealt to each player individually).
    for split in [false, true] {
        let mut t = crate::r_p050_common::two_headed_giant();
        t.battlefield(P0, "Admiral Beckett Brass");
        let ps = [
            vanilla(&mut t, P0, "Test Pirate", "Creature — Human Pirate", (1, 1)),
            vanilla(&mut t, P0, "Test Pirate", "Creature — Human Pirate", (1, 1)),
            vanilla(&mut t, P1, "Test Pirate", "Creature — Human Pirate", (1, 1)),
        ];
        let ring = t.battlefield(P2, "Sol Ring");
        t.advance_to(P0, Step::BeginningOfCombat);
        let life = t.life(P2);
        let decl: Vec<(ObjectId, Entity)> = ps
            .iter()
            .enumerate()
            .map(|(i, p)| (*p, pl(if split && i == 2 { P3 } else { P2 })))
            .collect();
        t.attack(&decl, &[]);
        // P0's Pirates are 2/2 with the Admiral; the teammate's isn't boosted. (P2 and P3
        // share their team's life total.)
        assert_eq!(t.life(P2), life - 5, "the three Pirates hit P2's team");
        t.answer_targets(P0, &[obj(ring)]);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(ctl(&t, ring), if split { P2 } else { P0 }, "split {split}");
    }
}

#[test]
fn captivating_vampire_control_and_vampire_type_have_no_duration() {
    cr!("611.2a", "613.1b", "613.1d", "613.7");
    ruling!(
        "Captivating Vampire",
        "The effect of Captivating Vampire’s activated ability has no duration. You retain control of the affected creature until the game ends, the creature leaves the battlefield, or a later effect causes someone else to gain control of it. It doesn’t matter whether Captivating Vampire remains on the battlefield. Similarly, the affected creature remains a Vampire in addition to its other types until the game ends, the creature leaves the battlefield, or a later effect changes its types or subtypes."
    );
    supported("Captivating Vampire");
    let mut t = TestGame::new(2);
    let cv = t.battlefield(P0, "Captivating Vampire");
    let vs = vampires(&mut t, P0, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mut tapped = vs.clone();
    tapped.push(cv);
    t.answer_choose(P0, &tapped.iter().map(|v| obj(*v)).collect::<Vec<_>>());
    act(&mut t, P0, cv, 0, &[obj(bears)]);
    assert_eq!(ctl(&t, bears), P0);
    kill(&mut t, cv);
    two_turns(&mut t);
    assert_eq!(ctl(&t, bears), P0);
    assert!(t.obj_now(bears).chars.has_subtype("Vampire"));
    assert!(t.obj_now(bears).chars.has_subtype("Bear"));
    // A later effect gives it to someone else; it's still a Vampire.
    give_control(&mut t, bears, P1);
    assert_eq!(ctl(&t, bears), P1);
    assert!(t.obj_now(bears).chars.has_subtype("Vampire"));
    // A later effect that sets its creature types (Turn to Frog, CR 613.7 timestamps)
    // overrides it: a Frog, not a Vampire.
    cast_at(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    let c = &t.obj_now(bears).chars;
    assert!(c.has_subtype("Frog") && !c.has_subtype("Vampire") && !c.has_subtype("Bear"));
}

#[test]
fn tapping_creatures_for_a_cost_without_the_tap_symbol_ignores_summoning_sickness() {
    cr!("302.6", "602.5a");
    ruling!(
        "Captivating Vampire",
        "Since Captivating Vampire’s activated ability doesn’t have a tap symbol in its cost, you can tap a Vampire (including Captivating Vampire itself) that hasn’t been under your control since your most recent turn began to pay the cost."
    );
    ruling!(
        "Gilt-Leaf Archdruid",
        "Since the activated ability doesn’t have a tap symbol in its cost, you can tap creatures (including Gilt-Leaf Archdruid itself) that haven’t been under your control since your most recent turn began to pay the cost."
    );
    supported("Gilt-Leaf Archdruid");
    let sick = |t: &mut TestGame, ids: &[ObjectId]| {
        for id in ids {
            t.g.obj_mut(*id).summoning_sick = true;
        }
    };
    // Captivating Vampire and four Vampires, all summoning sick.
    let mut t = TestGame::new(2);
    let cv = t.battlefield_sick(P0, "Captivating Vampire");
    let mut vs = vampires(&mut t, P0, 4);
    vs.push(cv);
    sick(&mut t, &vs);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &vs.iter().map(|v| obj(*v)).collect::<Vec<_>>());
    act(&mut t, P0, cv, 0, &[obj(bears)]);
    assert!(vs.iter().all(|v| t.obj_now(*v).tapped));
    assert_eq!(ctl(&t, bears), P0);
    // Gilt-Leaf Archdruid and six Druids, all summoning sick.
    let mut t = TestGame::new(2);
    let ga = t.battlefield_sick(P0, "Gilt-Leaf Archdruid");
    let mut ds: Vec<ObjectId> = (0..6)
        .map(|_| vanilla(&mut t, P0, "Test Druid", "Creature — Elf Druid", (1, 1)))
        .collect();
    ds.push(ga);
    sick(&mut t, &ds);
    let land = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &ds.iter().map(|v| obj(*v)).collect::<Vec<_>>());
    act(&mut t, P0, ga, 0, &[pl(P1)]);
    assert!(ds.iter().all(|v| t.obj_now(*v).tapped));
    assert_eq!(ctl(&t, land), P0);
}

#[test]
fn agent_of_treachery_counts_a_permanent_controlled_until_end_of_turn() {
    cr!("514.2", "513.1", "603.4");
    ruling!(
        "Agent of Treachery",
        "If you gain control of a permanent \"until end of turn,\" that effect will wear off during the cleanup step, after the end step. You'll control that permanent while considering Agent of Treachery's last ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agent of Treachery");
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, a, P0);
    give_control(&mut t, b, P0);
    t.set_step(P0, Step::PrecombatMain);
    give_control_until_eot(&mut t, c, P0);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(ctl(&t, c), P0);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, c), P1);
}

#[test]
fn olivia_voldaren_ability_does_nothing_if_you_lost_control_of_her() {
    cr!("611.2b", "608.2b");
    ruling!(
        "Olivia Voldaren",
        "If you activate Olivia Voldaren’s last ability, and before that ability resolves you lose control of Olivia Voldaren, the ability will resolve with no effect. You won’t gain control of the targeted Vampire."
    );
    supported("Olivia Voldaren");
    let mut t = TestGame::new(2);
    let olivia = t.battlefield(P0, "Olivia Voldaren");
    let v = vanilla(&mut t, P1, "Test Vampire", "Creature — Vampire", (1, 1));
    act_only(&mut t, P0, olivia, 1, &[obj(v)]);
    give_control(&mut t, olivia, P1);
    t.resolve_all();
    // The target is still legal: the ability resolves, with no effect.
    assert!(ability_resolved_from(&t, olivia));
    assert_eq!(ctl(&t, v), P1);
    // Keeping control: it works.
    let mut t = TestGame::new(2);
    let olivia = t.battlefield(P0, "Olivia Voldaren");
    let v = vanilla(&mut t, P1, "Test Vampire", "Creature — Vampire", (1, 1));
    act(&mut t, P0, olivia, 1, &[obj(v)]);
    assert_eq!(ctl(&t, v), P0);
}

#[test]
fn confiscation_coup_can_pay_zero_energy_for_a_zero_mana_value_permanent() {
    cr!("107.14", "118.5");
    ruling!(
        "Confiscation Coup",
        "If the target artifact or creature has a mana value of 0, you may choose to pay zero energy and gain control of it."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    steal(&mut t, "Confiscation Coup", thopter);
    assert_eq!(crate::r_s28_common::energy(&t, P0), 4);
}

#[test]
fn rags_riches_each_opponent_chooses_then_you_take_them_all_untargeted() {
    cr!("101.4", "608.2c", "115.1", "702.11b");
    ruling!(
        "Rags // Riches",
        "In a multiplayer game, each opponent in turn order chooses a creature they control, if they control any creatures. After each opponent has done so, you gain control of each chosen creature simultaneously."
    );
    ruling!(
        "Rags // Riches",
        "Once Riches begins to resolve, no player may take other actions until it's done. Notably, opponents can't try to remove a creature after choosing it but before you gain control of it."
    );
    ruling!(
        "Rags // Riches",
        "Riches's effect doesn't target. Creatures with hexproof may be lured away this way."
    );
    supported("Rags // Riches");
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    // P1's creature has hexproof; P2 controls none; P3 has a Bears.
    let hex = creature_with(
        &mut t,
        P1,
        "Test Hexproof",
        "Creature — Elf",
        (1, 1),
        "Hexproof",
    );
    assert!(t.obj_now(hex).has_keyword(KeywordKind::Hexproof));
    let bears = t.battlefield(P3, "Grizzly Bears");
    lots(&mut t, P0);
    let c = t.graveyard(P0, "Rags // Riches");
    t.answer_choose(P1, &[obj(hex)]);
    t.answer_choose(P3, &[obj(bears)]);
    let from = t.asked().len();
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Aftermath))
        .go();
    // Nothing is chosen while it's on the stack.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })));
    let before = t.asked().len();
    t.resolve();
    // P1 then P3 chose (P2 has no creature), all within the resolution, and P0 got both.
    let choosers: Vec<PlayerId> = t.asked()[before..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P1, P3]);
    assert!(t.asked()[before..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    assert_eq!((ctl(&t, hex), ctl(&t, bears)), (P0, P0));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn rags_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c", "613.1d");
    ruling!(
        "Rags // Riches",
        "Rags affects only creatures on the battlefield at the time it resolves. It won't affect creatures that enter the battlefield or noncreature permanents that become creatures later in the turn."
    );
    supported("Mutavault");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let vault = t.battlefield(P0, "Mutavault");
    lots(&mut t, P0);
    let c = t.hand(P0, "Rags // Riches");
    t.cast(P0, c).method(CastMethod::Half(0)).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, vault, 1, &[]);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(vault), (2, 2));
    assert!(t.on_battlefield(bears) && t.on_battlefield(vault));
}

#[test]
fn beguiler_of_wills_needs_a_legal_target_and_checks_it_again() {
    cr!("115.1", "602.2b", "608.2b");
    ruling!(
        "Beguiler of Wills",
        "In order to activate the ability, you must be able to target a creature with power less than or equal to the number of creatures you control. The comparison is made again when the ability resolves. If the power is greater than the number of creatures you control at that time, the target is illegal and the ability doesn’t resolve."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Beguiler of Wills");
    let other = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Two creatures: the Bears (power 2) can be targeted, the Giant (power 3) can't.
    let from = t.asked().len();
    act_only(&mut t, P0, b, 0, &[obj(bears)]);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(bears)));
    assert!(!offered[0].contains(&obj(giant)));
    // The Elves die in response: the ability doesn't resolve.
    kill(&mut t, other);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    assert!(!ability_resolved_from(&t, b));
}

#[test]
fn chromeshell_crab_swapped_creatures_cant_attack_that_turn() {
    cr!("302.6", "701.12a");
    ruling!(
        "Chromeshell Crab",
        "The creatures can’t attack until you start a turn with them under your control. So you can’t swap and then attack."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let crab = t.battlefield(P0, "Chromeshell Crab");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, crab));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    assert!(mtg_engine::facedown::turn_face_up_by(&mut t.g, crab, P0));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(ctl(&t, giant), P0);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, giant));
    assert!(can_attack(&mut t, crab));
    // Next turn it can.
    pass_turn(&mut t);
    pass_turn(&mut t);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, giant));
}

#[test]
fn avarice_totem_activated_twice_swaps_any_permanent() {
    cr!("701.12a", "405.5", "608.2");
    ruling!(
        "Avarice Totem",
        "It is possible to activate this ability in response to itself and generate some odd combinations."
    );
    supported("Avarice Totem");
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Avarice Totem");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    act_only(&mut t, P0, totem, 0, &[obj(mine)]);
    act_only(&mut t, P0, totem, 0, &[obj(theirs)]);
    // The second resolves first: the Totem for the Bears.
    t.resolve();
    assert_eq!((ctl(&t, totem), ctl(&t, theirs)), (P1, P0));
    // In between, the opponent controls the Totem and could activate it.
    lots(&mut t, P1);
    assert!(crate::r_s02_common::can_activate(&mut t, P1, totem));
    t.g.players[1].mana_pool.mana.clear();
    // Then the first: the Totem (now P1's) for the Giant.
    t.resolve();
    assert_eq!(ctl(&t, totem), P0);
    assert_eq!(ctl(&t, mine), P1);
    assert_eq!(ctl(&t, theirs), P0);
}

#[test]
fn juxtapose_exchanges_permanents_with_shroud() {
    cr!("701.12a", "702.18a", "115.1");
    ruling!(
        "Juxtapose",
        "No permanents are targeted by this effect, so permanents with Shroud may be exchanged this way."
    );
    supported("Juxtapose");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Argothian Enchantress");
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Shroud));
    cast_at(&mut t, P0, "Juxtapose", &[pl(P1)]);
    t.resolve_all();
    assert_eq!((ctl(&t, mine), ctl(&t, theirs)), (P1, P0));
}

#[test]
fn dacks_emblem_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5", "114.4", "611.2a");
    ruling!(
        "Dack Fayden",
        "The ability of Dack's emblem will resolve before the spell that caused it to trigger."
    );
    // (Its other half, the -2's effect outlasting Dack, is tested above.)
    ruling!(
        "Dack Fayden",
        "The effect of Dack's second ability and the effect of the emblem's ability last indefinitely."
    );
    let mut t = TestGame::new(2);
    let d = walker(&mut t, P0, "Dack Fayden", 6);
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, d, 2, &[]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let c = t.hand(P0, "Giant Growth");
    t.cast(P0, c).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(t.stack_len(), 1, "Giant Growth is still on the stack");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // The emblem's control change lasts indefinitely (Dack himself is gone: he used his
    // last loyalty on the emblem); Giant Growth's +3/+3 wore off.
    assert!(!t.on_battlefield(d));
    two_turns(&mut t);
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn votes_happen_only_as_the_spell_resolves() {
    cr!("701.38a", "608.2c");
    ruling!(
        "Expropriate",
        "No player votes until Expropriate resolves. Any responses to the spell must be made without knowing the outcome of the vote."
    );
    ruling!(
        "Expropriate",
        "Players can't do anything between voting and finishing the resolution of the spell or ability that included the vote."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.g.players[0].mana_pool.add_type(ManaType::U, 4);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    let c = t.hand(P0, "Expropriate");
    t.cast(P0, c).go();
    let votes = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .count()
    };
    assert_eq!(votes(&t, from), 0, "no votes while it's on the stack");
    // P1 responds without knowing the vote: Giant Growth on their Bears.
    lots(&mut t, P1);
    cast_at(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve();
    assert_eq!(votes(&t, from), 0);
    let before = t.asked().len();
    t.resolve();
    assert_eq!(votes(&t, before), 2);
    // Voting, choosing and the control change all happened in that one resolution, with
    // no priority in between.
    assert!(t.asked()[before..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    assert_eq!(ctl(&t, bears), P0);
    assert!(t.in_exile("Expropriate"));
}

#[test]
fn drana_defending_player_chooses_while_the_ability_resolves() {
    cr!("608.2c", "608.2", "603.3");
    ruling!(
        "Drana, the Last Bloodchief",
        "The defending player chooses a card and you put it onto the battlefield all while Drana's ability is resolving. No player may take actions in between those events."
    );
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, the Last Bloodchief");
    let card = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.answer_choose(P1, &[obj(card)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(drana, pl(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let before = t.asked().len();
    t.resolve();
    let asked = &t.asked()[before..];
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
    assert!(asked
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    let bears = t.g.current(card);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(t.obj_now(bears).chars.has_subtype("Vampire"));
}

#[test]
fn elrond_a_player_with_several_votes_may_choose_the_same_creature() {
    cr!("701.38d", "701.38a");
    ruling!(
        "Elrond of the White Council",
        "If you have multiple votes, you can vote for fellowship multiple times. If you do, you can choose the same creature each time."
    );
    supported("Elrond of the White Council");
    supported("Brago's Representative");
    // P1 has two votes (Brago's Representative), both fellowship, choosing the Bears twice.
    let mut t = TestGame::new(2);
    let brago = t.battlefield(P1, "Brago's Representative");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P1, &[obj(bears)]);
    t.answer_choose(P1, &[obj(bears)]);
    let from = t.asked().len();
    let elrond = t.enter(P0, "Elrond of the White Council");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let chooses: Vec<Decision> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. }))
        .map(|(_, d)| d.clone())
        .collect();
    assert_eq!(chooses.len(), 2, "one choice per fellowship vote");
    // The second choice still offers the Bears and P1 chose them again: only the Bears
    // changed hands (Brago's Representative wasn't taken in their place).
    assert!(matches!(
        &chooses[1],
        Decision::ChooseEntities { candidates, .. } if candidates.contains(&obj(bears))
    ));
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, brago), P1);
    // One aid vote.
    assert_eq!(t.counters(elrond, "+1/+1"), 1);
}
