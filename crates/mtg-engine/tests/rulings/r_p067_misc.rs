//! Rulings batch P067 — the rest: statics that apply only on the battlefield (CR 112.6),
//! mana abilities (CR 605), exiling a spell isn't countering it (CR 701.6), Auras put onto
//! the battlefield without being cast (CR 303.4f), copy effects as a permanent enters (CR
//! 707.5), searches (CR 701.23), controlled searches (CR 723), and others.

use crate::r_p067_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Casts the card `name` for `p` with enough mana of `ty` in its pool, with `targets`.
fn cast_named(t: &mut TestGame, p: PlayerId, name: &str, ty: ManaType, n: u32, targets: &[Entity]) -> ObjectId {
    mana(t, p, ty, n);
    let c = t.hand(p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

#[test]
fn chimil_stops_counters_only_on_the_battlefield_and_can_be_countered_as_a_spell() {
    cr!("113.6", "701.6a");
    ruling!(
        "Chimil, the Inner Sun",
        "Chimil, the Inner Sun's first ability applies only while it's on the battlefield. While it's a spell, it can be countered."
    );
    supported("Chimil, the Inner Sun");
    // As a spell: countered.
    let mut t = TestGame::new(2);
    let chimil = cast_named(&mut t, P0, "Chimil, the Inner Sun", ManaType::C, 6, &[]);
    cast_named(&mut t, P1, "Counterspell", ManaType::U, 2, &[o(chimil)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Chimil, the Inner Sun"));
    // On the battlefield: P0's Shock can't be countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chimil, the Inner Sun");
    let shock = cast_named(&mut t, P0, "Shock", ManaType::R, 1, &[Entity::Player(P1)]);
    cast_named(&mut t, P1, "Counterspell", ManaType::U, 2, &[o(shock)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn codie_is_a_mana_ability_and_the_spell_it_helps_cast_is_the_next_spell() {
    cr!("605.1a", "605.3a", "601.2g", "603.7c");
    ruling!(
        "Codie, Vociferous Codex",
        "Codie’s activated ability is a mana ability. It can’t be countered or responded to, and you can activate it while casting a spell. If you do, that spell is “the next spell” you cast this turn."
    );
    supported("Codie, Vociferous Codex");
    // Activated directly: no stack object; the mana is there at once.
    let mut t = TestGame::new(2);
    let codie = t.battlefield(P0, "Codie, Vociferous Codex");
    t.lands(P0, "Wastes", 4);
    let r = t.activate(P0, codie, 0, &[]).unwrap();
    assert!(r.is_none());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 5);
    // Activated while casting Divination (only Codie makes blue): Divination is the next
    // spell, and the delayed trigger finds Lightning Bolt (mana value less than 3).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Codie, Vociferous Codex");
    t.lands(P0, "Wastes", 4);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Forest");
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4, "{}", t.dump_log());
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert!(exiled(&t, bolt), "{}", t.dump_log());
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_free(&mut t, P0, bolt);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

/// Collector's Cage enters (hideaway 5, Lightning Bolt hidden) with Hill Giant, Grizzly
/// Bears and Llanowar Elves (powers 3, 2, 1); its second ability puts the counter on
/// `target` (by index). Returns P1's life after.
fn cage(target: usize) -> i32 {
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose_moved(P0, &[o(bolt)]);
    let cage = t.enter(P0, "Collector's Cage");
    t.resolve_all();
    assert!(exiled(&t, bolt));
    let cs = [
        t.battlefield(P0, "Hill Giant"),
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Llanowar Elves"),
    ];
    t.lands(P0, "Wastes", 1);
    t.g.obj_mut(t.g.current(cage)).summoning_sick = false;
    t.activate(P0, cage, 0, &[o(cs[target])]).unwrap();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.counters(cs[target], "+1/+1"), 1);
    t.life(P1)
}

#[test]
fn collectors_cage_counts_different_powers_as_its_ability_resolves() {
    cr!("608.2c", "702.75a");
    ruling!(
        "Collector's Cage",
        "Collector's Cage's last ability looks at the creatures you control as it resolves to determine if you can play the exiled card. For example, if you control a 4/5 creature, a 1/1 creature, and a 2/1 creature, there are three different powers among creatures you control."
    );
    // Powers 3, 2, 1 before; the counter on Llanowar Elves makes them 3, 2, 2: no.
    assert_eq!(cage(2), 20);
    // On Hill Giant: 4, 2, 1: yes.
    assert_eq!(cage(0), 17);
}

#[test]
fn etherium_horn_sorcerer_bounces_only_from_the_battlefield() {
    cr!("602.2", "113.6", "608.2b");
    ruling!(
        "Etherium-Horn Sorcerer",
        "Etherium-Horn Sorcerer's first ability can only be activated when it is on the battlefield. If Etherium-Horn Sorcerer isn't on the battlefield when the ability resolves, the ability won't do anything."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Etherium-Horn Sorcerer");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.g.recompute();
    let uid = t
        .obj(in_hand)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .unwrap()
        .uid;
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, in_hand, uid).is_err());
    let e = t.battlefield(P0, "Etherium-Horn Sorcerer");
    t.activate(P0, e, 0, &[]).unwrap();
    destroy(&mut t, e);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Etherium-Horn Sorcerer"));
    assert!(!t.in_hand(P0, "Etherium-Horn Sorcerer") || t.zone(in_hand) == Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 1, "only the first one");
}

#[test]
fn evolving_door_counts_the_colors_but_not_which() {
    cr!("105.2", "701.23a");
    ruling!(
        "Evolving Door",
        "Evolving Door's ability instructs you to count the number of colors, but it doesn't track what those colors were. For example, if you sacrifice a creature that is green and no other colors, you'll search for any creature card that is exactly two colors, even if the card you search for isn't green."
    );
    supported("Evolving Door");
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, "Evolving Door");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Wastes", 1);
    let daxos = t.library_top(P0, "Daxos of Meletis");
    let bears = t.library_top(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[o(elves)]);
    t.answer_choose(P0, &[o(daxos)]);
    t.answer_yes(P0, true);
    t.activate(P0, door, 0, &[]).unwrap();
    // Daxos ({1}{W}{U}) is cast as the ability resolves.
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::U, 1);
    t.lands(P0, "Wastes", 1);
    t.resolve_all();
    assert!(!was_offered(&t, from, bears));
    assert!(t.on_battlefield(daxos), "{}", t.dump_log());
}

#[test]
fn panoptic_mirror_exiling_is_optional_and_needs_mana_value_x() {
    cr!("608.2d", "107.3a");
    ruling!(
        "Panoptic Mirror",
        "Exiling a card when the activated ability resolves is optional. If you choose to exile a card, you can exile only a card with mana value equal to X."
    );
    for pick in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Panoptic Mirror");
        let bolt = t.hand(P0, "Lightning Bolt");
        let div = t.hand(P0, "Divination");
        t.lands(P0, "Wastes", 1);
        let from = t.asked().len();
        t.answer_choose(P0, if pick { vec![o(bolt)] } else { vec![] }.as_slice());
        t.answer_yes(P0, pick);
        activate_x(&mut t, P0, m, 0, 1, &[]);
        t.resolve_all();
        assert!(!was_offered(&t, from, div));
        assert_eq!(t.zone(div), Zone::Hand(P0));
        assert_eq!(exiled(&t, bolt), pick, "{}", t.dump_log());
    }
}

#[test]
fn exiling_an_uncounterable_spell_works() {
    cr!("701.6a", "608.2b", "701.65a");
    ruling!(
        "Shell of the Last Kappa",
        "Exiling a spell prevents that spell from resolving, but it doesn’t technically “counter” anything. This means that Shell of the Last Kappa can exile spells which “can’t be countered.”"
    );
    ruling!(
        "Aang, Swift Savior // Aang and La, Ocean's Fury",
        "If a spell is exiled this way, it's removed from the stack and thus will not resolve. The spell isn't countered; it is exiled. This works against a spell that can't be countered."
    );
    supported("Shell of the Last Kappa");
    supported("Aang, Swift Savior // Aang and La, Ocean's Fury");
    // P1's spells can't be countered (Chimil); Lava Axe targets P0.
    for exiler in ["Shell of the Last Kappa", "Aang"] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Chimil, the Inner Sun");
        t.set_step(P1, Step::PrecombatMain);
        let axe = cast_named(&mut t, P1, "Lava Axe", ManaType::R, 5, &[Entity::Player(P0)]);
        if exiler == "Aang" {
            t.answer_targets(P0, &[o(axe)]);
            t.enter(P0, "Aang, Swift Savior // Aang and La, Ocean's Fury");
        } else {
            let shell = t.battlefield(P0, "Shell of the Last Kappa");
            t.lands(P0, "Wastes", 3);
            t.activate(P0, shell, 0, &[o(axe)]).unwrap();
        }
        t.resolve();
        assert!(exiled(&t, axe), "{exiler}: {}", t.dump_log());
        assert!(!t.g.turn_events.iter().any(|e| matches!(e, mtg_engine::events::Event::Countered { .. })), "{exiler}");
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{exiler}");
    }
}

#[test]
fn extraordinary_journey_draws_for_any_creature_cast_from_exile() {
    cr!("603.2", "603.4");
    ruling!(
        "Extraordinary Journey",
        "Extraordinary Journey's second ability applies to all creatures that enter the battlefield from exile or were cast from exile, not just creatures cast with Extraordinary Journey's first ability."
    );
    supported("Extraordinary Journey");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Extraordinary Journey");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Release to the Wind", &[o(bears)]);
    t.resolve_all();
    assert!(exiled(&t, bears));
    let hand = t.hand_size(P0);
    cast_free(&mut t, P0, bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
}

#[test]
fn jace_architect_of_thought_searches_each_library_separately() {
    cr!("701.23a", "701.23b");
    ruling!(
        "Jace, Architect of Thought",
        "For each library, the search is complete only when you explicitly say it is. For example, you can look through one player's library, set that library down, look at another player's library, choose a nonland card in the first library, then choose a nonland card in the second library. Don't reveal any cards from those libraries to any other player until you exile them."
    );
    supported("Jace, Architect of Thought");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, Architect of Thought");
    t.g.obj_mut(j).counters.insert("loyalty".into(), 8);
    let mine = t.library_top(P0, "Divination");
    t.library_top(P0, "Forest");
    let theirs = t.library_top(P1, "Lightning Bolt");
    let theirs_land = t.library_top(P1, "Mountain");
    let from = t.asked().len();
    t.answer_choose(P0, &[o(mine)]);
    t.answer_choose(P0, &[o(theirs)]);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.answer_choose_moved(P0, &[]);
    t.activate(P0, j, 2, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, mine) && exiled(&t, theirs), "{:?} {}", &t.asked()[from..], t.dump_log());
    // One choice per library, each among that library's nonland cards.
    let choices = offered_since(&t, from);
    let in_lib = |e: &Entity, p: PlayerId| matches!(e, Entity::Object(x) if t.obj(*x).zone == Zone::Library(p) || t.obj(*x).owner == p);
    assert!(choices.iter().any(|c| c.contains(&o(mine)) && c.iter().all(|e| in_lib(e, P0))));
    assert!(choices.iter().any(|c| c.contains(&o(theirs)) && c.iter().all(|e| in_lib(e, P1))));
    assert!(!was_offered(&t, from, theirs_land));
    // No card was revealed.
    assert!(crate::r_s11_common::revealed_this_turn(&t).is_empty());
}

#[test]
fn hostage_taker_cant_target_itself() {
    cr!("115.1", "603.3d");
    ruling!(
        "Hostage Taker",
        "Hostage Taker has received errata to prevent it from targeting itself. The correct Oracle wording appears above."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let h = t.enter(P0, "Hostage Taker");
    t.settle();
    let h = t.g.current(h);
    assert_eq!(t.stack_len(), 0, "no other creature or artifact: no target");
    assert!(crate::r_s02_common::target_candidates(&t, P0, from)
        .iter()
        .all(|c| !c.contains(&o(h))));
    assert!(t.on_battlefield(h));
    // With another creature, only that one can be targeted.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[o(bears)]);
    let h2 = t.enter(P0, "Hostage Taker");
    t.resolve_all();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered.is_empty());
    assert!(offered.iter().all(|c| !c.contains(&o(t.g.current(h2)))));
}

#[test]
fn hostage_taker_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3c", "608.2b");
    ruling!(
        "Hostage Taker",
        "If Hostage Taker leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let h = t.enter(P0, "Hostage Taker");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, h);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn hostage_taker_card_returns_immediately_when_it_leaves() {
    cr!("610.3", "610.3c", "704.3");
    ruling!(
        "Hostage Taker",
        "If it's still in exile, the exiled card returns to the battlefield immediately after Hostage Taker leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let h = t.enter(P0, "Hostage Taker");
    t.resolve_all();
    assert!(exiled(&t, bears));
    // Hostage Taker leaves (without SBAs or a trigger): the Bears are already back.
    let h = t.g.current(h);
    t.g.move_object(h, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    // The move's events are processed (no state-based actions are checked, nothing is
    // put on the stack).
    t.g.flush_events();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn ashiok_minus_three_returned_token_cant_be_exiled_from_hand() {
    cr!("111.8", "704.5d", "608.2c");
    ruling!(
        "Ashiok, Nightmare Muse",
        "If Ashiok's second ability returns a token to its owner's hand, that player can't exile that token from their hand. They must exile a card from their hand if possible."
    );
    supported("Ashiok, Nightmare Muse");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Muse");
    let tok = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let forest = t.hand(P1, "Forest");
    let from = t.asked().len();
    t.answer_choose_moved(P1, &[o(tok)]);
    t.activate(P0, a, 1, &[o(tok)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert!(!was_offered(&t, from, tok));
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn living_lore_has_0_power_and_toughness_off_the_battlefield() {
    cr!("604.3", "607.2a", "400.7");
    ruling!(
        "Living Lore",
        "If Living Lore leaves the battlefield, the ability that defines its power and toughness will lose track of the exiled card. Its power and toughness are 0 in other zones."
    );
    supported("Living Lore");
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    t.answer_choose_moved(P0, &[o(div)]);
    let l = t.enter(P0, "Living Lore");
    assert_eq!(t.pt(l), (3, 3));
    let card = move_to(&mut t, l, Zone::Hand(P0));
    assert_eq!(t.pt(card), (0, 0));
    assert!(exiled(&t, div));
    // Returning, it exiles another card (Shock) and has nothing to do with Divination.
    t.graveyard(P0, "Shock");
    t.answer_choose_moved(P0, &[]);
    let back = t.g.move_object(card, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0)).unwrap();
    t.settle();
    assert_eq!(t.pt(back), (1, 1), "{}", t.dump_log());
}

#[test]
fn living_lore_cant_exile_the_spell_putting_it_onto_the_battlefield() {
    cr!("608.2n", "614.12");
    ruling!(
        "Living Lore",
        "If an instant or sorcery spell puts Living Lore onto the battlefield, that spell is still on the stack while you choose a card to exile. You can't exile it with Living Lore."
    );
    let mut t = TestGame::new(2);
    let lore = t.graveyard(P0, "Living Lore");
    let shock = t.graveyard(P0, "Shock");
    let from = t.asked().len();
    let z = cast_named(&mut t, P0, "Zombify", ManaType::B, 4, &[o(lore)]);
    t.resolve_all();
    assert!(!was_offered(&t, from, z));
    assert!(t.in_graveyard(P0, "Zombify"));
    assert!(exiled(&t, shock));
    assert_eq!(t.pt(lore), (1, 1), "{}", t.dump_log());
}

/// Sakashima's Protege enters for P0 (not cast), copying `choice` if offered.
fn protege_enters(t: &mut TestGame, choice: Option<ObjectId>) -> (ObjectId, usize) {
    let from = t.asked().len();
    if let Some(c) = choice {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(c)]);
    } else {
        t.answer_yes(P0, false);
    }
    let p = t.enter(P0, "Sakashima's Protege");
    t.settle();
    (p, from)
}

#[test]
fn sakashimas_protege_copying_a_planeswalker_gets_its_printed_loyalty() {
    cr!("306.5b", "707.2");
    ruling!(
        "Sakashima's Protege",
        "If Sakashima's Protege enters the battlefield as a copy of a planeswalker, it will enter with a number of loyalty counters on it equal to the loyalty printed in the lower right corner of the planeswalker. It won't copy the number of counters currently on that planeswalker."
    );
    supported("Sakashima's Protege");
    let mut t = TestGame::new(2);
    let ashiok = t.enter(P1, "Ashiok, Nightmare Muse");
    t.g.obj_mut(ashiok).counters.insert("loyalty".into(), 7);
    let (p, _) = protege_enters(&mut t, Some(ashiok));
    assert_eq!(t.obj_now(p).chars.name, "Ashiok, Nightmare Muse", "{}", t.dump_log());
    assert_eq!(t.counters(p, "loyalty"), 5);
}

#[test]
fn sakashimas_protege_cant_copy_a_permanent_entering_with_it() {
    cr!("707.5", "614.12");
    ruling!(
        "Sakashima's Protege",
        "If Sakashima's Protege somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature."
    );
    interp!("ruling-contradicting-oracle-gives-way");
    let mut t = TestGame::new(2);
    let earlier = t.enter(P1, "Hill Giant");
    // A noncreature permanent that entered this turn can be chosen too ("any permanent";
    // the ruling's "You may choose only a creature" is a Clone template's).
    let anthem = t.enter(P1, "Glorious Anthem");
    let old = t.battlefield(P1, "Serra Angel");
    t.g.obj_mut(old).entered_turn = 0;
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(earlier)]);
    let ids = crate::r_s24_common::enter_together(&mut t, &[(P0, "Sakashima's Protege"), (P0, "Grizzly Bears")]);
    t.settle();
    let offered = offered_since(&t, from);
    assert!(!offered.is_empty(), "{}", t.dump_log());
    assert!(offered.iter().all(|c| !c.contains(&o(t.g.current(ids[1])))));
    assert!(offered.iter().any(|c| c.contains(&o(earlier)) && c.contains(&o(anthem))));
    assert!(offered.iter().all(|c| !c.contains(&o(old))));
    assert_eq!(t.obj_now(ids[0]).chars.name, "Hill Giant");
}

#[test]
fn sakashimas_protege_copying_a_token_or_a_copy() {
    cr!("707.2", "707.3", "111.1");
    ruling!(
        "Sakashima's Protege",
        "If the chosen permanent is a token, Sakashima's Protege copies the original characteristics of that token as stated by the effect that put that token onto the battlefield. Sakashima's Protege doesn't become a token in this case."
    );
    ruling!(
        "Sakashima's Protege",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is another Sakashima's Protege), then Sakashima's Protege enters the battlefield as whatever the chosen permanent copied."
    );
    // A Soldier token from Raise the Alarm, pumped by Giant Growth.
    let mut t = TestGame::new(2);
    cast_named(&mut t, P1, "Raise the Alarm", ManaType::W, 2, &[]);
    t.resolve_all();
    let tok = crate::r_s01_common::tokens(&t, P1)[0];
    cast_named(&mut t, P1, "Giant Growth", ManaType::G, 1, &[o(tok)]);
    t.resolve_all();
    assert_eq!(t.pt(tok), (4, 4));
    let (p, _) = protege_enters(&mut t, Some(tok));
    assert_eq!(t.obj_now(p).chars.name, t.obj_now(tok).chars.name);
    assert_eq!(t.pt(p), (1, 1));
    assert!(!t.obj_now(p).is_token());
    // A Clone that entered as a copy of Grizzly Bears (which didn't enter this turn).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(bears).entered_turn = 0;
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[o(bears)]);
    let clone = t.enter(P1, "Clone");
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    let (p, from) = protege_enters(&mut t, Some(clone));
    assert!(!was_offered(&t, from, bears));
    assert_eq!(t.obj_now(p).chars.name, "Grizzly Bears", "{}", t.dump_log());
    assert_eq!(t.pt(p), (2, 2));
}

#[test]
fn an_aura_put_onto_the_battlefield_without_being_cast_doesnt_target() {
    cr!("303.4f", "702.11b");
    for name in [
        "Open the Vaults",
        "Eiganjo Dynastorian // Replenish",
        "Genesis Ultimatum",
        "Ajani, Outland Chaperone",
        "Aurora Awakener",
        "Illuna, Apex of Wishes",
    ] {
        supported(name);
    }
    ruling!(
        "Open the Vaults",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    ruling!(
        "Eiganjo Dynastorian // Replenish",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    ruling!(
        "Genesis Ultimatum",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    ruling!(
        "Ajani, Outland Chaperone",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    ruling!(
        "Aurora Awakener",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    ruling!(
        "Illuna, Apex of Wishes",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in its current zone."
    );
    for name in [
        "Open the Vaults",
        "Replenish",
        "Genesis Ultimatum",
        "Ajani, Outland Chaperone",
        "Aurora Awakener",
        "Illuna, Apex of Wishes",
    ] {
        for hexproof in [true, false] {
            // With P1's hexproof creature, P0 attaches Pacifism to it; with no creature
            // at all, Pacifism stays where it was (Open the Vaults returns it from a
            // graveyard; Replenish's prepared creature would be one to enchant).
            if !hexproof && name != "Open the Vaults" {
                continue;
            }
            let mut t = TestGame::new(2);
            let bogle = hexproof.then(|| t.battlefield(P1, "Slippery Bogle"));
            let from_gy = matches!(name, "Open the Vaults" | "Replenish");
            let pac = if from_gy {
                t.graveyard(P0, "Pacifism")
            } else {
                t.g.players[0].library.clear();
                let l = stack_library(&mut t, P0, &["Pacifism", "Hill Giant", "Hill Giant", "Hill Giant", "Hill Giant", "Hill Giant"]);
                l[0]
            };
            if let Some(b) = bogle {
                crate::r_s22_common::choose_named_when_offered(&mut t, P0, "Slippery Bogle");
                let _ = b;
            }
            match name {
                "Open the Vaults" => {
                    cast_named(&mut t, P0, name, ManaType::W, 6, &[]);
                }
                "Replenish" => {
                    let d = t.battlefield(P0, "Eiganjo Dynastorian // Replenish");
                    mtg_engine::designations::become_prepared(&mut t.g, d);
                    let copy = t.obj_now(d).prepared.unwrap();
                    mana(&mut t, P0, ManaType::W, 4);
                    t.cast(P0, copy).go();
                }
                "Genesis Ultimatum" => {
                    t.answer_choose_moved(P0, &[o(pac)]);
                    mana(&mut t, P0, ManaType::U, 3);
                    mana(&mut t, P0, ManaType::R, 2);
                    cast_named(&mut t, P0, name, ManaType::G, 2, &[]);
                }
                "Ajani, Outland Chaperone" => {
                    let a = t.battlefield(P0, name);
                    t.g.obj_mut(a).counters.insert("loyalty".into(), 8);
                    t.g.players[0].life = 3;
                    t.answer_choose_moved(P0, &[o(pac)]);
                    t.activate(P0, a, 2, &[]).unwrap();
                }
                "Aurora Awakener" => {
                    // One color among permanents P0 controls (its own green): X is 1.
                    t.answer_choose_moved(P0, &[o(pac)]);
                    t.enter(P0, name);
                }
                _ => {
                    // Illuna mutates onto a non-Human creature P0 owns.
                    let host = t.battlefield(P0, "Llanowar Elves");
                    let ill = t.hand(P0, name);
                    mana(&mut t, P0, ManaType::U, 2);
                    mana(&mut t, P0, ManaType::R, 1);
                    mana(&mut t, P0, ManaType::C, 3);
                    t.answer(P0, DecisionKind::Option, Answer::Index(0));
                    t.answer(P0, DecisionKind::Option, Answer::Index(0));
                    t.cast(P0, ill)
                        .method(mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Mutate))
                        .target(host)
                        .go();
                }
            }
            t.resolve_all();
            match bogle {
                Some(b) => {
                    assert!(t.on_battlefield(pac), "{name}: {}", t.dump_log());
                    assert_eq!(t.obj_now(pac).attached_to, Some(o(b)), "{name}");
                    assert_eq!(t.obj_now(pac).controller, P0, "{name}");
                }
                None => assert_eq!(t.zone(pac), Zone::Graveyard(P0), "{name}: {}", t.dump_log()),
            }
        }
    }
}

#[test]
fn two_shared_fates_replace_a_draw_only_once() {
    cr!("616.1", "614.5");
    ruling!(
        "Shared Fate",
        "If more than one Shared Fate is on the battlefield, you choose which one replaces each card draw, but you can replace a draw only once."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shared Fate");
    t.battlefield(P0, "Shared Fate");
    let l1 = t.library_size(P1);
    let from = t.asked().len();
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.library_size(P1), l1 - 1);
    assert_eq!(t.g.exile.len(), 1);
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. })), "{:?}", &t.asked()[from..]);
}

#[test]
fn grima_exiling_a_whole_library_without_a_find_puts_it_back() {
    cr!("701.24a", "608.2c");
    ruling!(
        "Gríma, Saruman's Footman",
        "If that player exiles their entire library without exiling an instant or sorcery card, they will randomize the order of the exiled cards, and the cards then become that player's library, ending the effect. They will not continue to exile and randomize their library forever."
    );
    supported("Gríma, Saruman's Footman");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gríma, Saruman's Footman");
    t.g.players[1].library.clear();
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Forest", "Hill Giant"]);
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 3, "{}", t.dump_log());
    for c in lib {
        assert_eq!(t.zone(c), Zone::Library(P1));
    }
    assert!(t.g.exile.is_empty());
}

#[test]
fn zoetic_glyph_sets_base_power_and_toughness_in_timestamp_order() {
    cr!("613.4b", "613.7");
    ruling!(
        "Zoetic Glyph",
        "If the artifact was already a creature, its base power and toughness will become 5/4. This overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Zoetic Glyph becomes attached to the artifact will overwrite this effect."
    );
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(orni), (0, 2));
    cast_named(&mut t, P0, "Turn to Frog", ManaType::U, 2, &[o(orni)]);
    t.resolve_all();
    assert_eq!(t.pt(orni), (1, 1));
    cast_named(&mut t, P0, "Zoetic Glyph", ManaType::U, 3, &[o(orni)]);
    t.resolve_all();
    assert_eq!(t.pt(orni), (5, 4), "{}", t.dump_log());
    assert!(t.obj_now(orni).chars.has_subtype("Golem"));
    cast_named(&mut t, P0, "Turn to Frog", ManaType::U, 2, &[o(orni)]);
    t.resolve_all();
    assert_eq!(t.pt(orni), (1, 1));
}

#[test]
fn knowledge_pools_each_track_their_own_cards() {
    cr!("607.2a", "603.3b", "608.2b");
    ruling!(
        "Knowledge Pool",
        "If multiple Knowledge Pools are on the battlefield, keep track of which cards are exiled by each of them."
    );
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    let shock = t.library_top(P1, "Shock");
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    assert!(exiled(&t, bolt) && exiled(&t, shock));
    // Two triggers; the first to resolve exiles Divination and lets P0 cast one of its
    // own Pool's cards; the other does nothing.
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let div = cast_named(&mut t, P0, "Divination", ManaType::U, 3, &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, ""), 2);
    t.resolve();
    assert!(exiled(&t, div));
    let offered = offered_since(&t, from);
    let all: Vec<Entity> = offered.concat();
    let has = |id: ObjectId| all.iter().any(|e| matches!(e, Entity::Object(x) if t.g.current(*x) == t.g.current(id)));
    assert!(has(shock) != has(bolt), "only one Pool's cards: {offered:?}");
    let n = offered.len();
    t.resolve_all();
    assert_eq!(offered_since(&t, from).len(), n, "the other trigger offers nothing");
    assert_eq!(t.hand_size(P0), 0);
    // Pools of two players: the active player's trigger goes on the stack first, so the
    // other player's resolves first, and its cards are the ones offered.
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    let shock = t.library_top(P1, "Shock");
    t.enter(P1, "Knowledge Pool");
    t.resolve_all();
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_named(&mut t, P0, "Divination", ManaType::U, 3, &[]);
    t.resolve_all();
    assert!(was_offered(&t, from, shock), "{}", t.dump_log());
    assert!(!was_offered(&t, from, bolt));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn opposition_agent_decides_what_is_found_in_every_zone_and_it_is_all_exiled() {
    cr!("723.2", "723.5", "701.23a");
    ruling!(
        "Opposition Agent",
        "If a spell or ability instructs an opponent you control to search multiple zones including the library, you decide what cards they find in all zones. They'll exile all cards they find in any of the zones."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    t.set_step(P1, Step::PrecombatMain);
    let gy = t.graveyard(P1, "Grizzly Bears");
    t.library_top(P1, "Llanowar Elves");
    t.answer_choose(P0, &[o(gy)]);
    let finale = t.hand(P1, "Finale of Devastation");
    mana(&mut t, P1, ManaType::G, 2);
    mana(&mut t, P1, ManaType::C, 2);
    t.cast(P1, finale).x(2).go();
    t.resolve_all();
    assert!(exiled(&t, gy), "{:?}\n{}", t.asked(), t.dump_log());
    assert!(crate::r_s01_common::creatures(&t, P1).is_empty());
    // P0 may play it.
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::B, 2);
    assert!(castable(&mut t, P0, gy));
}

#[test]
fn with_two_opposition_agents_the_latest_ones_controller_chooses_whose_effect_applies() {
    cr!("723.2", "723.5", "616.1");
    ruling!(
        "Opposition Agent",
        "If more than one player controls an Opposition Agent, and another player searches their library, the controller of the Opposition Agent that most recently entered the battlefield controls that player during the search."
    );
    interp!("default-first-listed-choice");
    for generous in [false, true] {
        let mut t = TestGame::new(3);
        t.enter(P0, "Opposition Agent");
        t.enter(P2, "Opposition Agent");
        t.set_step(P1, Step::PrecombatMain);
        let bolt = t.library_top(P1, "Lightning Bolt");
        t.library_top(P1, "Forest");
        let from = t.asked().len();
        // P2 controls P1 during the search: P2 makes P1 find Lightning Bolt.
        t.answer_choose(P2, &[o(bolt)]);
        if generous {
            t.answer(P2, DecisionKind::Option, Answer::Index(1));
        }
        let tutor = t.hand(P1, "Demonic Tutor");
        mana(&mut t, P1, ManaType::B, 2);
        t.cast(P1, tutor).go();
        t.resolve_all();
        assert!(exiled(&t, bolt), "{}", t.dump_log());
        let asked: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. } | Decision::ChooseOption { .. }))
            .map(|(p, _)| *p)
            .collect();
        // The search's choice is P1's (made by P2's agent, which found the Lightning Bolt
        // rather than the default first card, the Forest on top); then P2 chooses.
        assert_eq!(asked, vec![P1, P2], "{generous}");
        let (gets, not) = if generous { (P0, P2) } else { (P2, P0) };
        mana(&mut t, gets, ManaType::R, 1);
        mana(&mut t, not, ManaType::R, 1);
        assert!(castable(&mut t, gets, bolt), "{generous}");
        assert!(!castable(&mut t, not, bolt), "{generous}");
    }
}
