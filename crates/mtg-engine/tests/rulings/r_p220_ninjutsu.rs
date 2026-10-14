//! Rulings batch P220 — ninjutsu (CR 702.49): Yuffie, Materia Hunter, Kaito, Bane of
//! Nightmares, Orochi
//! Soul-Reaver, Sakashima's Student, Higure, the Still Wind, Thousand-Faced Shadow.

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s12_common::attack_target;
use mtg_engine::decision::Decision;
use mtg_engine::kw::ninjutsu::revealed_cards;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` activates the ninjutsu ability of the card `ninja` in their hand, returning
/// `returned`.
fn ninjutsu(t: &mut TestGame, p: PlayerId, ninja: ObjectId, returned: ObjectId) {
    t.g.recompute();
    // The ninjutsu ability among the card's activated abilities (a planeswalker's loyalty
    // abilities come first).
    let index = t
        .obj(ninja)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with("Ninjutsu"))
        .expect("a ninjutsu ability");
    t.answer_choose(p, &[Entity::Object(returned)]);
    t.activate(p, ninja, index, &[]).expect("ninjutsu");
}

/// P0's Grizzly Bears attack `at`, unblocked, and the game is in the declare blockers
/// step; `ninja` is in P0's hand with plenty of mana. Returns (bears, ninja card).
fn unblocked_bears(t: &mut TestGame, ninja: &str, at: Entity) -> (ObjectId, ObjectId) {
    supported(ninja);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 4);
    let n = t.hand(P0, ninja);
    to_blockers(t, &[(bears, at)], &[]);
    (bears, n)
}

#[test]
fn a_ninja_put_onto_the_battlefield_attacking_wasnt_declared_as_an_attacker() {
    cr!("702.49c", "508.3a", "508.4");
    ruling!(
        "Yuffie, Materia Hunter",
        "Although Yuffie enters attacking when its ninjutsu ability resolves, it was never declared as an attacking creature (for purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    ruling!(
        "Kaito, Bane of Nightmares",
        "Although Kaito enters attacking when his ninjutsu ability resolves, he was never declared as an attacking creature (for purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    // Hellrider: "Whenever a creature you control attacks, this creature deals 1 damage to
    // the player or planeswalker it's attacking."
    supported("Hellrider");
    for name in ["Yuffie, Materia Hunter", "Kaito, Bane of Nightmares"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hellrider");
        let (bears, ninja) = unblocked_bears(&mut t, name, Entity::Player(P1));
        // The Bears' attack triggered Hellrider.
        assert_eq!(t.life(P1), 19);
        ninjutsu(&mut t, P0, ninja, bears);
        t.resolve_all();
        let n = t.g.current(ninja);
        assert!(t.g.is_attacking(n), "{name}");
        // The Ninja entering attacking didn't.
        assert_eq!(t.life(P1), 19, "{name}");
        assert_eq!(t.stack_len(), 0, "{name}");
    }
}

#[test]
fn the_ninja_is_revealed_and_enters_only_if_still_in_hand() {
    cr!("702.49a", "702.49b");
    ruling!(
        "Yuffie, Materia Hunter",
        "As you activate Yuffie's ninjutsu ability, you reveal Yuffie from your hand and return the attacking creature. Yuffie isn't put onto the battlefield until the ability resolves. If Yuffie leaves your hand before then, it won't enter at all."
    );
    ruling!(
        "Kaito, Bane of Nightmares",
        "As you activate Kaito's ninjutsu ability, you reveal Kaito from your hand and return the attacking creature. Kaito isn't put onto the battlefield until the ability resolves. If Kaito leaves your hand before then, he won't enter at all."
    );
    ruling!(
        "Orochi Soul-Reaver",
        "As you activate a ninjutsu ability, you reveal the Ninja card in your hand and return the attacking creature to its owner’s hand. The Ninja card stays revealed and isn’t put onto the battlefield until the ability resolves. If it leaves your hand before then, it won’t enter the battlefield at all."
    );
    for name in [
        "Yuffie, Materia Hunter",
        "Kaito, Bane of Nightmares",
        "Orochi Soul-Reaver",
    ] {
        for discard in [true, false] {
            let mut t = TestGame::new(2);
            let (bears, ninja) = unblocked_bears(&mut t, name, Entity::Player(P1));
            ninjutsu(&mut t, P0, ninja, bears);
            // The Bears were returned as a cost; the Ninja is revealed, still in hand.
            assert!(t.in_hand(P0, "Grizzly Bears"), "{name}");
            assert!(t.in_hand(P0, name), "{name}");
            assert_eq!(revealed_cards(&t.g), vec![ninja], "{name}");
            if discard {
                t.g.discard(P0, ninja, None);
            }
            t.resolve_all();
            assert_eq!(
                t.named_on_battlefield(name).len(),
                usize::from(!discard),
                "{name}, discarded: {discard}"
            );
        }
    }
}

#[test]
fn ninjutsu_in_the_first_strike_step_deals_damage_in_the_regular_step() {
    cr!("702.49a", "510.4");
    ruling!(
        "Yuffie, Materia Hunter",
        "If at least one creature in combat has first strike or double strike, you can activate Yuffie's ninjutsu ability during the first-strike combat damage step. In that case, Yuffie will deal combat damage during the regular combat damage step, even if it has first strike."
    );
    ruling!(
        "Kaito, Bane of Nightmares",
        "If at least one creature in combat has first strike or double strike, you can activate Kaito's ninjutsu ability during the first-strike combat damage step. In that case, Kaito will deal combat damage during the regular combat damage step, even if he has first strike."
    );
    supported("Knighthood");
    // Knighthood: "Creatures you control have first strike." Both Ninjas have power 3.
    for name in ["Yuffie, Materia Hunter", "Kaito, Bane of Nightmares"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Knighthood");
        let (bears, ninja) = unblocked_bears(&mut t, name, Entity::Player(P1));
        t.advance_to(P0, Step::FirstStrikeDamage);
        assert_eq!(t.life(P1), 18);
        ninjutsu(&mut t, P0, ninja, bears);
        t.resolve_all();
        let n = t.g.current(ninja);
        assert!(
            t.obj(n).has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike),
            "{name}"
        );
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 15, "{name}");
    }
}

#[test]
fn ninjutsu_after_combat_damage_deals_no_combat_damage() {
    cr!("702.49a", "510.2", "511.1");
    ruling!(
        "Yuffie, Materia Hunter",
        "The ninjutsu ability can be activated during the declare blockers step, combat damage step, or end of combat step. If you wait until after the declare blockers step, because all combat damage is dealt at once, Yuffie won't normally deal combat damage."
    );
    ruling!(
        "Kaito, Bane of Nightmares",
        "The ninjutsu ability can be activated during the declare blockers step, combat damage step, or end of combat step. If you wait until after the declare blockers step, because all combat damage is dealt at once, Kaito won't normally deal combat damage."
    );
    ruling!(
        "Orochi Soul-Reaver",
        "The ninjutsu ability can be activated during the declare blockers step, combat damage step, or end of combat step. In most cases (see below), if you wait until the combat damage step or end of combat step, it will be after combat damage has been dealt, so the Ninja won’t deal combat damage."
    );
    for (name, step) in [
        ("Yuffie, Materia Hunter", Step::EndOfCombat),
        ("Kaito, Bane of Nightmares", Step::CombatDamage),
        ("Orochi Soul-Reaver", Step::CombatDamage),
    ] {
        let mut t = TestGame::new(2);
        let (bears, ninja) = unblocked_bears(&mut t, name, Entity::Player(P1));
        t.advance_to(P0, step);
        assert_eq!(t.life(P1), 18);
        ninjutsu(&mut t, P0, ninja, bears);
        t.resolve_all();
        let n = t.g.current(ninja);
        assert!(t.g.is_attacking(n), "{name}");
        t.advance_to(P0, Step::PostcombatMain);
        t.resolve_all();
        assert_eq!(t.life(P1), 18, "{name}");
        // Orochi's "deal combat damage to a player" trigger didn't happen: no Treasure.
        assert!(with_subtype(&t, P0, "Treasure").is_empty(), "{name}");
    }
}

#[test]
fn the_ninja_attacks_what_the_returned_creature_was_attacking() {
    cr!("702.49c", "508.4");
    ruling!(
        "Yuffie, Materia Hunter",
        "Yuffie enters attacking the same player, planeswalker, or battle the returned creature was attacking. This is a rule specific to ninjutsu; in other cases, when a creature is put onto the battlefield attacking, that creature's controller chooses which player, planeswalker, or battle it's attacking."
    );
    ruling!(
        "Kaito, Bane of Nightmares",
        "Kaito enters attacking the same player, planeswalker, or battle the returned creature was attacking. This is a rule specific to ninjutsu; in other cases, when a creature is put onto the battlefield attacking, that creature's controller chooses which player, planeswalker, or battle it's attacking."
    );
    ruling!(
        "Orochi Soul-Reaver",
        "The creature with ninjutsu enters the battlefield attacking the same player, planeswalker, or battle that the returned creature was attacking. This is a rule specific to ninjutsu; in other cases, when a creature is put onto the battlefield attacking, that creature’s controller chooses which player, planeswalker, or battle it’s attacking."
    );
    for name in [
        "Yuffie, Materia Hunter",
        "Kaito, Bane of Nightmares",
        "Orochi Soul-Reaver",
    ] {
        let mut t = TestGame::new(3);
        let jace = t.battlefield(P2, "Jace Beleren");
        let (bears, ninja) = unblocked_bears(&mut t, name, Entity::Object(jace));
        let from = t.asked().len();
        ninjutsu(&mut t, P0, ninja, bears);
        t.resolve_all();
        assert_eq!(attack_target(&t, ninja), Some(Entity::Object(jace)), "{name}");
        // No choice of what it attacks was asked.
        assert!(!asked_since(&t, from).iter().any(|(_, d)| matches!(
            d,
            Decision::ChooseEntities { prompt, .. } if prompt.contains("attacking")
        )));
    }
}

#[test]
fn sakashimas_student_cant_copy_the_creature_returned_for_ninjutsu() {
    cr!("702.49a", "707.9", "614.1c");
    ruling!(
        "Sakashima's Student",
        "Because you return the unblocked attacking creature to its owner's hand while activating the ninjutsu ability, you can't have Sakashima's Student enter as a copy of that creature."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let (bears, student) = unblocked_bears(&mut t, "Sakashima's Student", Entity::Player(P1));
    ninjutsu(&mut t, P0, student, bears);
    let from = t.asked().len();
    // It enters as a copy of the only creature it can copy: the Giant.
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    let returned = t.g.current(bears);
    let choices: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(choices.contains(&Entity::Object(giant)));
    assert!(!choices.contains(&Entity::Object(bears)));
    assert!(!choices.contains(&Entity::Object(returned)));
    let s = t.g.current(student);
    assert!(t.on_battlefield(s) && t.g.is_attacking(s));
    assert_eq!(t.obj(s).chars.name, "Hill Giant");
}

#[test]
fn higures_searched_ninja_comes_too_late_to_deal_damage_but_can_swap_for_higure() {
    cr!("702.49a", "510.2", "506.4");
    ruling!(
        "Higure, the Still Wind",
        "If Higure deals regular combat damage and you use its ability to search for another Ninja, that card won't be in your hand in time to activate its ninjutsu ability to put it onto the battlefield and deal damage. However, the card you searched for could still be swapped for an unblocked creature, including Higure, as long as you activate its ninjutsu ability before the end of combat."
    );
    supported("Higure, the Still Wind");
    supported("Ninja of the Deep Hours");
    let mut t = TestGame::new(2);
    let higure = t.battlefield(P0, "Higure, the Still Wind");
    let ninja = t.library_top(P0, "Ninja of the Deep Hours");
    t.lands(P0, "Island", 2);
    to_blockers(&mut t, &[(higure, Entity::Player(P1))], &[]);
    // Higure deals 3 combat damage; its trigger finds Ninja of the Deep Hours.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ninja)]);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Ninja of the Deep Hours"));
    // Still in combat: the Ninja's ninjutsu returns Higure (unblocked).
    let card = t.g.current(ninja);
    ninjutsu(&mut t, P0, card, higure);
    t.resolve_all();
    assert!(t.in_hand(P0, "Higure, the Still Wind"));
    let n = t.g.current(card);
    assert!(t.on_battlefield(n) && t.g.is_attacking(n));
    t.advance_to(P0, Step::PostcombatMain);
    // No combat damage from the Ninja.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn thousand_faced_shadows_token_can_attack_a_different_player() {
    cr!("702.49c", "508.4", "707.2");
    ruling!(
        "Thousand-Faced Shadow",
        "You choose which opponent or opposing planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Thousand-Faced Shadow is attacking."
    );
    supported("Thousand-Faced Shadow");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 4);
    let shadow = t.hand(P0, "Thousand-Faced Shadow");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[],
    );
    ninjutsu(&mut t, P0, shadow, bears);
    // The Shadow enters from hand attacking P1: its trigger copies the Hill Giant, and
    // the token attacks P2.
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve_all();
    let s = t.g.current(shadow);
    assert_eq!(attack_target(&t, s), Some(Entity::Player(P1)));
    let tok: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.obj(tok[0]).chars.name, "Hill Giant");
    assert!(t.obj(tok[0]).tapped);
    assert_eq!(attack_target(&t, tok[0]), Some(Entity::Player(P2)));
}

#[test]
fn kaito_is_a_hexproof_ninja_creature_only_during_your_turn() {
    cr!("611.3a", "613.1d", "306.5b");
    supported("Kaito, Bane of Nightmares");
    // "During your turn, as long as Kaito has one or more loyalty counters on him, he's a
    // 3/4 Ninja creature and has hexproof."
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kaito, Bane of Nightmares");
    assert!(t.counters(k, "loyalty") > 0);
    let o = t.obj(k);
    assert!(o.is_creature() && o.chars.has_subtype("Ninja"));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Hexproof));
    assert_eq!(t.pt(k), (3, 4));
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    let o = t.obj(k);
    assert!(!o.is_creature());
    assert!(!o.has_keyword(mtg_engine::keywords::KeywordKind::Hexproof));
}


