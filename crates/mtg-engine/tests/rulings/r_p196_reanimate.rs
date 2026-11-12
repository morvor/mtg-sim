//! Rulings batch P196 — creatures returned to the battlefield with characteristics or
//! abilities that last indefinitely (CR 611.2): delayed exile triggers and "exile it
//! instead" replacements (603.7, 614), new objects (400.7), returned creatures' colors and
//! types (105.3, 205.1b), triggers that look back in time (603.10), and abilities that
//! don't target (115.10).

use crate::r_p196_common::*;
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

/// P0 activates Whip of Erebos returning Hill Giant from P0's graveyard; the Giant is
/// returned.
fn whip_giant(t: &mut TestGame) -> ObjectId {
    let whip = t.battlefield(P0, "Whip of Erebos");
    t.graveyard(P0, "Hill Giant");
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Hill Giant")[0];
    act(t, P0, whip, 0, &[obj(card)]);
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Haste));
    giant
}

#[test]
fn whip_of_erebos_delayed_trigger_countered_leaves_the_replacement() {
    cr!("603.7", "603.7b", "614.1a", "701.6a");
    ruling!(
        "Whip of Erebos",
        "At the beginning of the next end step, the creature returned to the battlefield with Whip of Erebos is exiled. This is a delayed triggered ability. If the ability is countered, the creature will stay on the battlefield and the delayed trigger won't trigger again. However, the replacement effect will still exile the creature when it eventually leaves the battlefield."
    );
    supported("Stifle");
    // Not countered: exiled at the end step.
    let mut t = TestGame::new(2);
    let giant = whip_giant(&mut t);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.in_exile("Hill Giant"));
    // Countered with Stifle: it stays, and isn't exiled at later end steps.
    let mut t = TestGame::new(2);
    let giant = whip_giant(&mut t);
    t.advance_to(P0, Step::End);
    t.settle();
    let trig = top_of_stack(&t);
    cast_at(&mut t, P0, "Stifle", &[obj(trig)]);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    pass_turn(&mut t);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // When it dies, it's exiled instead.
    kill(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn whip_of_erebos_creature_already_being_exiled_returns_as_a_new_object() {
    cr!("614.1a", "614.6", "400.7", "610.3");
    ruling!(
        "Whip of Erebos",
        "If a creature returned to the battlefield with Whip of Erebos would leave the battlefield for any reason, it's exiled instead. However, if that creature is already being exiled, then the replacement effect won't apply. If the spell or ability that exiles it later returns it to the battlefield (as Chained to the Rocks might, for example), the creature card will return to the battlefield as a new object with no relation to its previous existence. The effects from Whip of Erebos will no longer apply to it."
    );
    supported("Banisher Priest");
    // Bounced: exiled instead.
    let mut t = TestGame::new(2);
    let giant = whip_giant(&mut t);
    crate::r_p189_common::bounce(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_hand(P0, "Hill Giant"));
    // Banisher Priest exiles it until the Priest leaves; then it's back as a new object.
    let mut t = TestGame::new(2);
    let giant = whip_giant(&mut t);
    t.answer_targets(P1, &[obj(giant)]);
    let priest = t.enter(P1, "Banisher Priest");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    kill(&mut t, priest);
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    let back = back[0];
    assert_ne!(back, giant);
    assert!(!t.obj_now(back).has_keyword(KeywordKind::Haste));
    // Not exiled at the end step, and it dies normally.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
    kill(&mut t, back);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

/// P0's Kheru Lich Lord upkeep trigger is on the stack (P0 has mana to pay).
fn kheru_upkeep(t: &mut TestGame) -> ObjectId {
    let kheru = t.battlefield(P0, "Kheru Lich Lord");
    to_upkeep(t, P0);
    assert_eq!(t.stack_len(), 1);
    lots(t, P0);
    kheru
}

#[test]
fn kheru_lich_lord_returns_a_card_put_there_in_response_even_itself() {
    cr!("115.10", "608.2c", "603.3");
    ruling!(
        "Kheru Lich Lord",
        "Because the ability doesn’t target any creature card, any creature card (including Kheru Lich Lord itself) put into the graveyard in response to that ability may be returned to the battlefield."
    );
    supported("Kheru Lich Lord");
    let mut t = TestGame::new(2);
    let kheru = kheru_upkeep(&mut t);
    assert_eq!(t.graveyard_size(P0), 0);
    kill(&mut t, kheru);
    assert!(t.in_graveyard(P0, "Kheru Lich Lord"));
    t.answer_yes(P0, true);
    t.resolve();
    let back = t.named_on_battlefield("Kheru Lich Lord");
    assert_eq!(back.len(), 1);
    let o = t.obj_now(back[0]);
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Haste));
}

#[test]
fn kheru_lich_lord_replacement_doesnt_follow_a_new_object() {
    cr!("614.1a", "614.6", "400.7");
    ruling!(
        "Kheru Lich Lord",
        "If a creature card returned to the battlefield with Kheru Lich Lord would leave the battlefield for any reason, it’s exiled instead. However, if that creature is already being exiled, then the replacement effect won’t apply. If the spell or ability that exiles it later returns it to the battlefield (as Suspension Field might, for example), the creature card will return to the battlefield as a new object with no relation to its previous existence. The effects from Kheru Lich Lord will no longer apply to it."
    );
    supported("Suspension Field");
    let setup = |t: &mut TestGame| {
        t.graveyard(P0, "Hill Giant");
        kheru_upkeep(t);
        t.answer_yes(P0, true);
        t.resolve();
        let giant = t.named_on_battlefield("Hill Giant")[0];
        assert!(t.obj_now(giant).has_keyword(KeywordKind::Trample));
        giant
    };
    // Dies: exiled instead.
    let mut t = TestGame::new(2);
    let giant = setup(&mut t);
    kill(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    // Suspension Field exiles it; when the Field leaves it's back with nothing from Kheru.
    let mut t = TestGame::new(2);
    let giant = setup(&mut t);
    t.answer_targets(P1, &[obj(giant)]);
    t.answer_yes(P1, true);
    let field = t.enter(P1, "Suspension Field");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    kill(&mut t, field);
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant")[0];
    let o = t.obj_now(back);
    assert!(!o.has_keyword(KeywordKind::Flying) && !o.has_keyword(KeywordKind::Trample));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
    kill(&mut t, back);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn grave_betrayal_returns_each_creature_with_its_own_delayed_trigger() {
    cr!("603.7", "603.3b", "405.5");
    ruling!(
        "Grave Betrayal",
        "Each creature you don't control that dies causes a delayed triggered ability to trigger at the beginning of the next end step. If there are multiple such abilities, you can put them on the stack in any order. The creatures will return to the battlefield one at a time as each ability resolves. The last ability you put on the stack will be the first one to resolve."
    );
    supported("Grave Betrayal");
    let mut firsts = vec![];
    for order in [[0usize, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grave Betrayal");
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        kill(&mut t, giant);
        kill(&mut t, bears);
        t.advance_to(P0, Step::End);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        t.settle();
        assert_eq!(t.stack_len(), 2);
        t.resolve();
        let back: Vec<String> =
            t.g.permanents()
                .filter(|o| o.controller == P0 && o.is(CardType::Creature))
                .map(|o| o.chars.name.to_string())
                .collect();
        assert_eq!(back.len(), 1, "one at a time");
        firsts.push(back[0].clone());
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    }
    assert_ne!(firsts[0], firsts[1], "{firsts:?}");
}

#[test]
fn grave_betrayal_adds_black_and_zombie_and_a_colorless_one_is_just_black() {
    cr!("105.2c", "105.3", "205.1b");
    ruling!(
        "Grave Betrayal",
        "Grave Betrayal doesn't overwrite any previous colors or types. Rather, it adds another color and another creature type."
    );
    ruling!(
        "Grave Betrayal",
        "If the creature is normally colorless, it will simply be black. It won't be both black and colorless."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grave Betrayal");
    let giant = t.battlefield(P1, "Hill Giant");
    let thopter = t.battlefield(P1, "Ornithopter");
    kill(&mut t, giant);
    kill(&mut t, thopter);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    let c = &t.obj_now(giant).chars;
    assert_eq!(
        c.colors,
        [Color::Red, Color::Black].into_iter().collect::<ColorSet>()
    );
    assert!(c.has_subtype("Giant") && c.has_subtype("Zombie"));
    let thopter = t.named_on_battlefield("Ornithopter")[0];
    let c = &t.obj_now(thopter).chars;
    assert_eq!(c.colors, [Color::Black].into_iter().collect::<ColorSet>());
    assert!(c.has_subtype("Thopter") && c.has_subtype("Zombie"));
    assert!(c.is(CardType::Artifact));
}

#[test]
fn grave_betrayal_creature_dying_in_the_end_step_waits_for_the_next() {
    cr!("513.2");
    ruling!(
        "Grave Betrayal",
        "If a creature you don't control dies during the end step, that creature won't return to the battlefield until the beginning of the next end step."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grave Betrayal");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.settle();
    kill(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    assert_eq!(ctl(&t, back[0]), P0);
}

#[test]
fn grave_betrayal_card_that_left_the_graveyard_doesnt_return() {
    cr!("400.7", "603.7", "603.7c");
    ruling!(
        "Grave Betrayal",
        "If the creature card leaves the graveyard before the delayed triggered ability resolves, it won't return to the battlefield. This is also true if the card leaves the graveyard and returns to the graveyard before that ability resolves."
    );
    for back_to_graveyard in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grave Betrayal");
        let giant = t.battlefield(P1, "Hill Giant");
        kill(&mut t, giant);
        let card = t.g.find_in_zone(Zone::Graveyard(P1), "Hill Giant")[0];
        t.g.move_object(card, Zone::Hand(P1), events::MoveCause::Effect, None);
        if back_to_graveyard {
            let card = t.g.find_in_zone(Zone::Hand(P1), "Hill Giant")[0];
            t.g.move_object(card, Zone::Graveyard(P1), events::MoveCause::Effect, None);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert!(
            t.named_on_battlefield("Hill Giant").is_empty(),
            "{back_to_graveyard}"
        );
    }
}

#[test]
fn black_zombie_reanimators_make_a_colorless_creature_just_black() {
    cr!("105.2c", "105.3");
    ruling!(
        "Necromantic Selection",
        "If the creature is normally colorless, it will simply become black. It can't be both black and colorless."
    );
    ruling!(
        "Rise from the Grave",
        "If the creature is normally colorless, it will simply become black. It can’t be both black and colorless."
    );
    supported("Necromantic Selection");
    supported("Rise from the Grave");
    // Necromantic Selection: destroy all creatures, return the Ornithopter.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose_moved(P0, &[obj(thopter)]);
    cast_at(&mut t, P0, "Necromantic Selection", &[]);
    t.resolve_all();
    let back = t.named_on_battlefield("Ornithopter");
    assert_eq!(back.len(), 1);
    assert_eq!(ctl(&t, back[0]), P0);
    assert_eq!(
        t.obj_now(back[0]).chars.colors,
        [Color::Black].into_iter().collect::<ColorSet>()
    );
    // Rise from the Grave.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Ornithopter");
    cast_at(&mut t, P0, "Rise from the Grave", &[obj(card)]);
    t.resolve_all();
    let back = t.named_on_battlefield("Ornithopter")[0];
    let c = &t.obj_now(back).chars;
    assert_eq!(c.colors, [Color::Black].into_iter().collect::<ColorSet>());
    assert!(c.has_subtype("Zombie"));
}

#[test]
fn postmortem_lunge_x_cards_and_the_end_step_exile() {
    cr!("107.3g", "202.3e", "603.7c");
    ruling!(
        "Postmortem Lunge",
        "If the creature card has {X} in its mana cost, that X is 0. Remember that the value of X chosen for Postmortem Lunge is independent of the value of X on other cards."
    );
    ruling!(
        "Postmortem Lunge",
        "If the creature is no longer on the battlefield at the beginning of the next end step, it won't be exiled."
    );
    supported("Postmortem Lunge");
    supported("Endless One");
    // Endless One ({X}) has mana value 0 in the graveyard: X = 0 targets it, X = 1
    // doesn't. It enters with no counters (Lunge's X isn't its X) and dies as a 0/0.
    let mut t = TestGame::new(2);
    let one = t.graveyard(P0, "Endless One");
    lots(&mut t, P0);
    let c = t.hand(P0, "Postmortem Lunge");
    assert!(t.cast(P0, c).x(1).target(one).try_go().is_err());
    t.clear_answers();
    let c = t.g.current(c);
    t.cast(P0, c).x(0).target(one).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    assert_ne!(t.g.find_in_zone(Zone::Graveyard(P0), "Endless One")[0], one);
    // Primordial Hydra ({X}{G}{G}, "enters with X +1/+1 counters") has mana value 2 there:
    // Lunge's X is 2, the Hydra's X is 0, so it enters with no counters and dies.
    supported("Primordial Hydra");
    let mut t = TestGame::new(2);
    let hydra = t.graveyard(P0, "Primordial Hydra");
    lots(&mut t, P0);
    let c = t.hand(P0, "Postmortem Lunge");
    t.cast(P0, c).x(2).target(hydra).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Primordial Hydra").is_empty());
    let now = t.g.find_in_zone(Zone::Graveyard(P0), "Primordial Hydra");
    assert_eq!(now.len(), 1);
    assert_ne!(now[0], hydra, "it was on the battlefield");
    // The Hill Giant dies before the end step: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Hill Giant");
    lots(&mut t, P0);
    let c = t.hand(P0, "Postmortem Lunge");
    t.cast(P0, c).x(4).target(card).go();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Haste));
    kill(&mut t, giant);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(!t.in_exile("Hill Giant"));
}

#[test]
fn archangel_elspeth_ultimate_treats_x_as_zero() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Archangel Elspeth",
        "For the last ability, if a card in your graveyard has {X} in its mana cost, X is 0."
    );
    supported("Chalice of the Void");
    let mut t = TestGame::new(2);
    let el = t.battlefield(P0, "Archangel Elspeth");
    t.g.add_counters(obj(el), "loyalty", 4, None);
    t.graveyard(P0, "Chalice of the Void");
    t.graveyard(P0, "Juggernaut");
    act(&mut t, P0, el, 2, &[]);
    assert_eq!(t.named_on_battlefield("Chalice of the Void").len(), 1);
    assert!(t.in_graveyard(P0, "Juggernaut"));
}

#[test]
fn ashiok_minus_x_finds_cards_from_either_exile_ability_of_this_ashiok() {
    cr!("607.2a", "400.7");
    ruling!(
        "Ashiok, Nightmare Weaver",
        "Ashiok's second ability refers to any creature card exiled because of Ashiok's first or third ability."
    );
    ruling!(
        "Ashiok, Nightmare Weaver",
        "If Ashiok leaves the battlefield, and later another Ashiok enters the battlefield, it is a new object (even if the two were represented by the same card). Creature cards exiled by the original Ashiok can't be put onto the battlefield by the second ability of the new Ashiok."
    );
    // +2 exiles Grizzly Bears from the library; −10 exiles Runeclaw Bear from P1's hand.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Weaver");
    crate::r_s01_common::stack_library(&mut t, P1, &["Grizzly Bears", "Island", "Island"]);
    act(&mut t, P0, a, 0, &[pl(P1)]);
    t.hand(P1, "Runeclaw Bear");
    t.exile(P1, "Elvish Visionary");
    t.g.add_counters(obj(a), "loyalty", 10, None);
    pass_turn(&mut t);
    pass_turn(&mut t);
    act(&mut t, P0, a, 2, &[]);
    assert!(t.in_exile("Runeclaw Bear"));
    pass_turn(&mut t);
    pass_turn(&mut t);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    act(&mut t, P0, a, 1, &[]);
    let cands = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("a choice on resolution");
    let names: Vec<String> = cands
        .iter()
        .filter_map(|e| e.object())
        .map(|o| t.g.obj(o).chars.name.to_string())
        .collect();
    assert_eq!(names.len(), 2, "{names:?}");
    assert!(names.contains(&"Grizzly Bears".to_string()));
    assert!(names.contains(&"Runeclaw Bear".to_string()));
    // The card the −10 exiled can be put onto the battlefield.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Weaver");
    t.hand(P1, "Runeclaw Bear");
    t.g.add_counters(obj(a), "loyalty", 10, None);
    act(&mut t, P0, a, 2, &[]);
    let runeclaw = t.g.find_in_zone(Zone::Exile, "Runeclaw Bear")[0];
    pass_turn(&mut t);
    pass_turn(&mut t);
    t.answer_choose(P0, &[obj(runeclaw)]);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    act(&mut t, P0, a, 1, &[]);
    let back = t.named_on_battlefield("Runeclaw Bear");
    assert_eq!(back.len(), 1);
    assert_eq!(ctl(&t, back[0]), P0);
    assert!(t.obj_now(back[0]).chars.has_subtype("Nightmare"));
    // A new Ashiok can't find what the old one exiled: the card isn't offered, so the
    // answer naming it isn't used.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Weaver");
    crate::r_s01_common::stack_library(&mut t, P1, &["Grizzly Bears", "Island", "Island"]);
    act(&mut t, P0, a, 0, &[pl(P1)]);
    let exiled = t.g.find_in_zone(Zone::Exile, "Grizzly Bears")[0];
    crate::r_p189_common::bounce(&mut t, a);
    let card = t.g.find_in_zone(Zone::Hand(P0), "Ashiok, Nightmare Weaver")[0];
    t.g.move_object(card, Zone::Battlefield, events::MoveCause::Effect, Some(P0));
    t.g.recompute();
    let a2 = t.named_on_battlefield("Ashiok, Nightmare Weaver")[0];
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(exiled)]);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    act(&mut t, P0, a2, 1, &[]);
    let offered = t.asked()[from..].iter().any(|(_, d)| {
        matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&obj(exiled)))
    });
    assert!(!offered);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn liliana_emblem_haste_lasts() {
    cr!("611.2a", "114.4");
    ruling!(
        "Liliana, Waker of the Dead",
        "Creatures put onto the battlefield with Liliana's emblem gain haste indefinitely."
    );
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, Waker of the Dead");
    t.g.add_counters(obj(l), "loyalty", 3, None);
    act(&mut t, P0, l, 2, &[]);
    let card = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(card)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(ctl(&t, giant), P0);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Haste));
}

#[test]
fn dranas_returned_creature_cant_join_the_attack() {
    cr!("508.1");
    ruling!(
        "Drana, the Last Bloodchief",
        "Because all attackers are declared at the same time, the returned creature can't attack during the same combat it returns, even if it has haste."
    );
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, the Last Bloodchief");
    let card = t.graveyard(P0, "Raging Goblin");
    t.answer_choose(P1, &[obj(card)]);
    crate::r_s01_common::attack_with(&mut t, &[(drana, pl(P1))]);
    t.resolve_all();
    let goblin = t.named_on_battlefield("Raging Goblin")[0];
    assert!(t.obj_now(goblin).has_keyword(KeywordKind::Haste));
    assert!(!crate::r_s10_common::attacking(&t, goblin));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn drana_attacking_a_planeswalker_its_controller_chooses() {
    cr!("506.2", "508.5");
    ruling!(
        "Drana, the Last Bloodchief",
        "If Drana attacks a planeswalker, that planeswalker's controller is the defending player."
    );
    let mut t = TestGame::new(3);
    let drana = t.battlefield(P0, "Drana, the Last Bloodchief");
    let pw = t.battlefield(P2, "Archangel Elspeth");
    t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    crate::r_s01_common::attack_with(&mut t, &[(drana, obj(pw))]);
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P2]);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

/// Dread Slaver for P0 and the given creature; the Slaver deals 1 damage to it.
fn slaver_hits(t: &mut TestGame, p: PlayerId, name: &str) -> (ObjectId, ObjectId) {
    let slaver = t.battlefield(P0, "Dread Slaver");
    let c = t.battlefield(p, name);
    crate::r_s06_common::damage(t, slaver, 1, c);
    (slaver, c)
}

#[test]
fn dread_slaver_checks_damage_earlier_in_the_turn_whoever_controlled_it() {
    cr!("603.10a", "603.2");
    ruling!(
        "Dread Slaver",
        "Each time a creature dies, check whether Dread Slaver had dealt any damage to it at any time during that turn. If so, Dread Slaver's ability will trigger. It doesn't matter who controlled the creature or whose graveyard it was put into."
    );
    supported("Dread Slaver");
    for p in [P0, P1] {
        let mut t = TestGame::new(2);
        let (_, giant) = slaver_hits(&mut t, p, "Hill Giant");
        let bears = t.battlefield(p, "Grizzly Bears");
        kill(&mut t, giant);
        kill(&mut t, bears);
        t.resolve_all();
        let back = t.named_on_battlefield("Hill Giant");
        assert_eq!(back.len(), 1, "{p}");
        assert_eq!(ctl(&t, back[0]), P0);
        let c = &t.obj_now(back[0]).chars;
        assert!(c.has_subtype("Zombie") && c.colors.contains(Color::Black));
        assert!(c.colors.contains(Color::Red));
        // The Bears weren't dealt damage by the Slaver.
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    }
    // Damage on an earlier turn doesn't count.
    let mut t = TestGame::new(2);
    let (_, giant) = slaver_hits(&mut t, P1, "Hill Giant");
    pass_turn(&mut t);
    kill(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn dread_slaver_returns_itself_after_damaging_itself() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Dread Slaver",
        "Dread Slaver will return if it somehow dealt damage to itself and then dies."
    );
    let mut t = TestGame::new(2);
    let slaver = t.battlefield(P0, "Dread Slaver");
    crate::r_s06_common::damage(&mut t, slaver, 5, slaver);
    t.resolve_all();
    let back = t.named_on_battlefield("Dread Slaver");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], slaver);
    assert_eq!(ctl(&t, back[0]), P0);
}

#[test]
fn dread_slaver_triggers_when_it_and_its_blocker_die_together() {
    cr!("603.10a", "510.2", "704.5g");
    ruling!(
        "Dread Slaver",
        "If Dread Slaver and a creature it was blocking or blocked by both die in combat, Dread Slaver's ability will trigger."
    );
    let mut t = TestGame::new(2);
    let slaver = t.battlefield(P0, "Dread Slaver");
    let brute =
        crate::r_s01_common::custom_card("Test Brute", "Creature — Ogre", "{4}", Some((5, 3)), "");
    let brute = t.custom(P1, brute, Zone::Battlefield);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(brute, pl(P0))], &[(slaver, brute)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dread Slaver"));
    let back = t.named_on_battlefield("Test Brute");
    assert_eq!(back.len(), 1);
    assert_eq!(ctl(&t, back[0]), P0);
}

#[test]
fn ghouls_night_out_noncreature_permanent_isnt_a_zombie_but_has_decayed() {
    cr!("205.1b", "205.3d");
    ruling!(
        "Ghouls' Night Out",
        "If any creature cards put onto the battlefield this way become permanents that aren't creatures because of another effect, those permanents won't be Zombies, but they will gain decayed."
    );
    supported("Ghouls' Night Out");
    supported("Erebos, God of the Dead");
    // Erebos isn't a creature with too little devotion to black.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Erebos, God of the Dead");
    t.graveyard(P1, "Hill Giant");
    cast_at(&mut t, P0, "Ghouls' Night Out", &[]);
    t.resolve_all();
    let erebos = t.named_on_battlefield("Erebos, God of the Dead")[0];
    let o = t.obj_now(erebos);
    assert!(!o.is(CardType::Creature));
    assert!(!o.chars.has_subtype("Zombie"));
    assert!(o.has_keyword(KeywordKind::Decayed));
    let giant = t.named_on_battlefield("Hill Giant")[0];
    let o = t.obj_now(giant);
    assert!(o.chars.has_subtype("Zombie") && o.has_keyword(KeywordKind::Decayed));
}

#[test]
fn excava_returned_permanent_is_one_one_unless_another_effect_sets_it() {
    cr!("613.4b", "613.7", "702.122a", "702.184a");
    ruling!(
        "Excava, the Risen Past",
        "If another effect causes one of the returned permanents to become a creature and sets its base power and toughness as it does so, that creature will have that power and toughness; it won't be 1/1. However, a Vehicle returned this way will remain a 1/1 if you crew it, and a Spacecraft returned this way will remain a 1/1 even if enough charge counters are placed on it for it to become a creature."
    );
    supported("Excava, the Risen Past");
    supported("Tezzeret, Betrayer of Flesh");
    supported("Smuggler's Copter");
    supported("Exploration Broodship");
    let excava = |t: &mut TestGame, name: &str| -> ObjectId {
        let ex = t.battlefield(P0, "Excava, the Risen Past");
        let card = t.graveyard(P0, name);
        t.answer_targets(P0, &[obj(card)]);
        crate::r_s01_common::attack_with(t, &[(ex, pl(P1))]);
        t.resolve_all();
        let back = t.named_on_battlefield(name)[0];
        assert_eq!(t.pt(back), (1, 1), "{name}");
        assert!(t.obj_now(back).chars.has_subtype("Spirit"));
        back
    };
    // Sol Ring, then Tezzeret's −2 ("becomes an artifact creature ... base 4/4").
    let mut t = TestGame::new(2);
    let ring = excava(&mut t, "Sol Ring");
    let tez = t.battlefield(P0, "Tezzeret, Betrayer of Flesh");
    t.set_step(P0, Step::PostcombatMain);
    act(&mut t, P0, tez, 1, &[obj(ring)]);
    assert_eq!(t.pt(ring), (4, 4));
    // Smuggler's Copter crewed: still 1/1.
    let mut t = TestGame::new(2);
    let copter = excava(&mut t, "Smuggler's Copter");
    t.set_step(P0, Step::PostcombatMain);
    let crew = t.battlefield(P0, "Hill Giant");
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[crew]));
    t.resolve_all();
    assert_eq!(t.pt(copter), (1, 1));
    // Exploration Broodship with 8 charge counters: still 1/1.
    let mut t = TestGame::new(2);
    let ship = excava(&mut t, "Exploration Broodship");
    crate::r_s16_common::add_charge(&mut t, ship, 8);
    assert!(t.obj_now(ship).is(CardType::Creature));
    assert_eq!(t.pt(ship), (1, 1));
}

#[test]
fn xu_ifit_returned_permanent_keeps_abilities_gained_later() {
    cr!("613.1f", "613.7");
    ruling!(
        "Xu-Ifit, Osteoharmonist",
        "If a permanent returned to the battlefield with Xu-Ifit’s ability gains an ability after being returned to the battlefield this way, it will keep that ability."
    );
    supported("Xu-Ifit, Osteoharmonist");
    let mut t = TestGame::new(2);
    let xu = t.battlefield(P0, "Xu-Ifit, Osteoharmonist");
    let card = t.graveyard(P0, "Serra Angel");
    act(&mut t, P0, xu, 0, &[obj(card)]);
    let angel = t.named_on_battlefield("Serra Angel")[0];
    assert!(t.obj_now(angel).chars.has_subtype("Skeleton"));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
    attach_new(&mut t, P0, "Rancor", angel);
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn trench_gorger_searches_even_if_it_left() {
    cr!("113.7a", "608.2c");
    ruling!(
        "Trench Gorger",
        "If Trench Gorger leaves the battlefield before its triggered ability resolves, you may still search your library."
    );
    supported("Trench Gorger");
    let mut t = TestGame::new(2);
    let lands = crate::r_s01_common::stack_library(&mut t, P0, &["Forest", "Island"]);
    let gorger = t.enter(P0, "Trench Gorger");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, gorger);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lands[0]), obj(lands[1])]);
    t.resolve_all();
    assert!(t.in_exile("Forest") && t.in_exile("Island"));
}

#[test]
fn olivia_discard_still_possible_after_the_creature_left() {
    cr!("608.2c", "400.7");
    ruling!(
        "Olivia, Mobilized for War",
        "If the entering creature leaves the battlefield before the triggered ability of Olivia, Mobilized for War resolves, you may still discard a card. The rest of the ability has no effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Olivia, Mobilized for War");
    t.hand(P0, "Island");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_p189_common::bounce(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Island"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let card = t.g.find_in_zone(Zone::Hand(P0), "Grizzly Bears")[0];
    assert_eq!(t.g.obj(card).counter("+1/+1"), 0);
}

#[test]
fn cyclopean_giant_exiles_the_card_that_was_copying_it() {
    cr!("707.2", "400.7e", "603.10a");
    ruling!(
        "Cyclopean Giant",
        "If a Vesuvan Shapeshifter that’s copying Cyclopean Giant is put into a graveyard, the Shapeshifter will be exiled when the ability resolves even though it’s not a Cyclopean Giant anymore."
    );
    supported("Cyclopean Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Cyclopean Giant");
    let land = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    let shifter = t.enter(P0, "Vesuvan Shapeshifter");
    t.settle();
    assert_eq!(t.obj_now(shifter).chars.name, "Cyclopean Giant");
    t.clear_answers();
    t.answer_targets(P0, &[obj(land)]);
    kill(&mut t, shifter);
    t.resolve_all();
    assert!(t.obj_now(land).chars.has_subtype("Swamp"));
    assert!(t.in_exile("Vesuvan Shapeshifter"));
    assert!(!t.in_graveyard(P0, "Vesuvan Shapeshifter"));
}

/// Ascent of the Worthy enters for P0 choosing `chosen` for chapter I.
fn ascent(t: &mut TestGame, chosen: ObjectId) {
    t.answer_choose(P0, &[obj(chosen)]);
    t.enter(P0, "Ascent of the Worthy");
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn ascent_of_the_worthy_redirect_follows_the_creature_not_its_controller() {
    cr!("614.1a", "614.9");
    ruling!(
        "Ascent of the Worthy",
        "If another player gains control of the chosen creature after the chapter I/II ability resolves but before your next turn, damage that would be dealt to creatures you control will still be dealt to that creature instead."
    );
    ruling!(
        "Ascent of the Worthy",
        "If the chosen creature isn’t on the battlefield at the time damage would be dealt to creatures you control, that damage is dealt to whichever creature(s) it was supposed to be dealt to. It won’t be redirected."
    );
    supported("Ascent of the Worthy");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    ascent(&mut t, wall);
    give_control(&mut t, wall, P1);
    let src = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 2, bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(wall).damage, 2);
    // The chosen creature is gone: the damage isn't redirected.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    ascent(&mut t, wall);
    kill(&mut t, wall);
    let src = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 1, bears);
    assert_eq!(t.obj_now(bears).damage, 1);
}
