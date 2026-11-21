//! Rulings batch P215 — flashback (CR 702.34): additional costs, mana value with {X},
//! targets and timing, cost reductions of flashback costs, and tapping untapped
//! creatures (even summoning-sick ones) to pay costs.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::{priority_asked_since, respond};
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s25_common::spell_copies;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn flashback_spells_still_pay_their_additional_costs() {
    cr!("702.34a", "601.2b", "601.2h", "118.8");
    ruling!(
        "Corpse Cobble",
        "If you cast Corpse Cobble using flashback, you still pay its additional cost of sacrificing any number of creatures."
    );
    ruling!(
        "Electric Revelation",
        "If you cast Electric Revelation using flashback, you must still pay its additional cost of discarding a card."
    );
    ruling!(
        "Rite of Oblivion",
        "If you cast Rite of Oblivion using flashback, you must still pay its additional cost of sacrificing a nonland permanent."
    );
    ruling!(
        "Gaze of Justice",
        "The additional cost must be paid if Gaze of Justice is cast with flashback."
    );
    // Corpse Cobble ({3}{U}{B}): sacrificing the Hill Giant makes a 3/3 Zombie.
    supported("Corpse Cobble");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let cc = t.graveyard(P0, "Corpse Cobble");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, cc).method(FLASHBACK).go();
    assert!(!t.on_battlefield(giant));
    t.resolve_all();
    let z = tokens_with_subtype(&t, P0, "Zombie");
    assert_eq!(z.len(), 1);
    assert_eq!(t.pt(z[0]), (3, 3));
    assert!(t.in_exile("Corpse Cobble"));
    // Electric Revelation ({3}{R}): no card to discard, no flashback.
    supported("Electric Revelation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let er = t.graveyard(P0, "Electric Revelation");
    assert!(!can_cast(&mut t, P0, er, FLASHBACK));
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, er, FLASHBACK));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, er).method(FLASHBACK).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Rite of Oblivion ({2}{W}{B}): sacrifice the Ornithopter, exile P1's Bears.
    supported("Rite of Oblivion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 1);
    let rite = t.graveyard(P0, "Rite of Oblivion");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, rite, FLASHBACK));
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.cast(P0, rite).method(FLASHBACK).target(theirs).go();
    assert!(!t.on_battlefield(thopter));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Gaze of Justice ({5}{W}): tap three untapped white creatures.
    supported("Gaze of Justice");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let gaze = t.graveyard(P0, "Gaze of Justice");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let w: Vec<_> = (0..2)
        .map(|_| t.battlefield(P0, "Savannah Lions"))
        .collect();
    assert!(!can_cast(&mut t, P0, gaze, FLASHBACK));
    let third = t.battlefield(P0, "Savannah Lions");
    t.cast(P0, gaze).method(FLASHBACK).target(theirs).go();
    assert!(w.iter().chain([&third]).all(|c| tapped(&t, *c)));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn conflagrate_mana_value_includes_x_when_cast_with_flashback() {
    cr!("202.3e", "702.34a", "107.3a");
    ruling!(
        "Conflagrate",
        "If a spell has {X} in its mana cost, include the value chosen for that X when determining the mana value of that spell, even if it was cast with flashback and no mana was spent on X."
    );
    supported("Conflagrate");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.graveyard(P0, "Conflagrate");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    let spell = t.cast(P0, c).method(FLASHBACK).x(2).target(P1).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    // {X}{X}{R} with X = 2.
    assert_eq!(mana_value(&t, spell), 5);
    assert_eq!(t.hand_size(P0), 0);
}

fn flashback_first(_g: &Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::Priority { actions } => actions
            .iter()
            .find(|a| matches!(a, Action::Cast { method, .. } if *method == FLASHBACK))
            .cloned()
            .map(Answer::Action),
        _ => None,
    }
}

#[test]
fn a_resolved_flashback_spell_can_be_flashed_back_before_others_can_purify_the_grave() {
    cr!("702.34a", "117.3b", "608.2n");
    ruling!(
        "Purify the Grave",
        "If it's your turn, and you cast a spell with flashback from your hand, you'll have priority to cast that spell again using flashback before any other player has priority to cast Purify the Grave in order to remove that card from your graveyard."
    );
    supported("Purify the Grave");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.lands(P1, "Plains", 1);
    t.hand(P1, "Purify the Grave");
    let tt = t.hand(P0, "Think Twice");
    t.cast(P0, tt).go();
    let from = t.asked().len();
    respond(&mut t, P0, flashback_first);
    let ok = t.g.run_until(1000, |g| {
        g.stack.iter().any(|id| {
            g.obj(*id)
                .stack
                .as_ref()
                .is_some_and(|si| si.cast.method == FLASHBACK)
        })
    });
    assert!(ok);
    // P0 and P1 passed, Think Twice resolved, then P0 got priority first.
    assert_eq!(priority_asked_since(&t, from), vec![P0, P1, P0]);
}

#[test]
fn a_spell_cast_with_flashback_is_on_the_stack_not_in_the_graveyard() {
    cr!("601.2a", "702.34a", "115.1");
    ruling!(
        "Purify the Grave",
        "Purify the Grave can't be used in response to a spell being cast with flashback or a spell that requires cards to exiled from a graveyard as an additional cost to try and counter the spell or prevent it from being cast."
    );
    ruling!(
        "Memory's Journey",
        "If you cast Memory's Journey with flashback, it won't be in the graveyard when you choose targets. It can't target itself."
    );
    supported("Memory's Journey");
    // Purify the Grave can't target a card being flashed back.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let tt = t.graveyard(P0, "Think Twice");
    let other = t.graveyard(P0, "Grizzly Bears");
    t.cast(P0, tt).method(FLASHBACK).go();
    assert_eq!(t.zone(t.g.current(tt)), Zone::Stack);
    t.lands(P1, "Plains", 1);
    let purify = t.hand(P1, "Purify the Grave");
    let from = t.asked().len();
    t.cast(P1, purify).target(other).go();
    let c = target_candidates(&t, P1, from);
    assert!(!c[0].contains(&Entity::Object(t.g.current(tt))));
    assert!(c[0].contains(&Entity::Object(other)));
    // Memory's Journey can't target itself.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let mj = t.graveyard(P0, "Memory's Journey");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.cast(P0, mj)
        .method(FLASHBACK)
        .targets(&[Entity::Object(bears)])
        .go();
    let all: Vec<Entity> = target_candidates(&t, P0, from).concat();
    assert!(all.contains(&Entity::Object(bears)));
    assert!(!all.contains(&Entity::Object(mj)));
    assert!(!all.contains(&Entity::Object(t.g.current(mj))));
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn seize_the_storm_elemental_counts_cards_as_they_change() {
    cr!("704.3", "604.3", "611.3a");
    ruling!(
        "Seize the Storm",
        "If you cast Seize the Storm with no instant or sorcery cards in your graveyard and no cards with flashback in exile, the Elemental will start as a 0/0 creature. However, state-based actions won't be checked until after Seize the Storm has finished resolving. The Elemental will then be a 1/1 creature and survive because Seize the Storm will either be in your graveyard or in exile."
    );
    ruling!(
        "Seize the Storm",
        "The power and toughness of the Elemental token will change as the number of instant and sorcery cards in your graveyard and the number of cards with flashback you own in exile change."
    );
    ruling!(
        "Seize the Storm",
        "While a spell is on the stack, it is neither in your graveyard nor in exile, so it won't contribute to the Elemental's power and toughness. For example, with Seize the Storm in your graveyard and no cards in exile, the 1/1 Elemental in the above scenario will become 0/0 and die if you cast Seize the Storm from your graveyard using flashback."
    );
    supported("Seize the Storm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 12);
    let sts = t.hand(P0, "Seize the Storm");
    t.cast(P0, sts).go();
    t.resolve_all();
    let first = tokens_with_subtype(&t, P0, "Elemental");
    assert_eq!(first.len(), 1);
    assert_eq!(t.pt(first[0]), (1, 1));
    // More instants and sorceries in the graveyard: bigger.
    let s = t.graveyard(P0, "Shock");
    assert_eq!(t.pt(first[0]), (2, 2));
    move_to(&mut t, s, Zone::Exile);
    assert_eq!(t.pt(first[0]), (1, 1));
    // Flashing it back: on the stack it isn't counted; the first token dies.
    let sts = t.g.current(sts);
    t.cast(P0, sts).method(FLASHBACK).go();
    t.settle();
    assert!(!t.on_battlefield(first[0]));
    t.resolve_all();
    // In exile with flashback: the new token is 1/1.
    let second = tokens_with_subtype(&t, P0, "Elemental");
    assert_eq!(second.len(), 1);
    assert_eq!(t.pt(second[0]), (1, 1));
}

#[test]
fn galvanic_iteration_after_another_is_copied_and_copies_twice() {
    cr!("707.10", "603.7c", "702.34a");
    ruling!(
        "Galvanic Iteration",
        "If you resolve one Galvanic Iteration after another (or cast the same one with flashback), the second Galvanic Iteration will be copied. That copy and spell each resolve, creating two delayed triggered abilities. The next spell you cast after that will be copied twice. If that spell is also a Galvanic Iteration, the following spell will be copied three times, and so on."
    );
    supported("Galvanic Iteration");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 3);
    let gi = t.hand(P0, "Galvanic Iteration");
    t.cast(P0, gi).go();
    t.resolve_all();
    // The same one, with flashback: copied.
    let gi = t.g.current(gi);
    t.cast(P0, gi).method(FLASHBACK).go();
    t.settle();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    // Two delayed triggers: Shock is copied twice.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.settle();
    t.resolve_all();
    // Shock and its two copies.
    assert_eq!(t.life(P1), 14);
}

/// P0's commander `name` on the battlefield under P1's control.
fn stolen_commander(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P0, name);
    t.g.objects[c.0 as usize].is_commander = true;
    t.g.players[P0.idx()].commander_names.push(name.into());
    crate::r_s06_common::give_control(t, c, P1);
    assert_eq!(t.obj_now(c).controller, P1);
    c
}

#[test]
fn visions_count_a_commander_another_player_controls_and_reduce_only_generic_mana() {
    cr!("903.3", "601.2f", "118.7", "702.34a");
    ruling!(
        "Visions of Dominance",
        "If your commander is under the control of another player as you cast Visions of Dominance using its flashback ability, that commander's mana value can be used to determine the value of X."
    );
    ruling!(
        "Visions of Dread",
        "If your commander is under the control of another player as you cast Visions of Dread using its flashback ability, that commander's mana value can be used to determine the value of X."
    );
    ruling!(
        "Visions of Dominance",
        "The cost-reduction effect that applies if you cast Visions of Dominance using flashback applies only to the generic portion of the cost to cast it, although it can apply to any additional costs there may be to cast the spell. It can't reduce any colored mana required to cast the spell."
    );
    ruling!(
        "Visions of Dread",
        "The cost-reduction effect that applies if you cast Visions of Dread using flashback applies only to the generic portion of the cost to cast it, although it can apply to any additional costs there may be to cast the spell. It can't reduce any colored mana required to cast the spell."
    );
    for (card, color) in [
        ("Visions of Dominance", "Forest"),
        ("Visions of Dread", "Swamp"),
    ] {
        supported(card);
        // A mana value 6 commander (Craw Wurm) that P1 controls: {8}{C}{C} -> {2}{C}{C}.
        let mut t = TestGame::new(2);
        stolen_commander(&mut t, "Craw Wurm");
        t.graveyard(P1, "Grizzly Bears");
        let v = t.graveyard(P0, card);
        t.lands(P0, color, 2);
        t.lands(P0, "Wastes", 1);
        assert!(!can_cast(&mut t, P0, v, FLASHBACK), "{card}");
        t.lands(P0, "Wastes", 1);
        assert!(can_cast(&mut t, P0, v, FLASHBACK), "{card}");
        // A mana value 11 commander: {C}{C} still has to be paid.
        let mut t = TestGame::new(2);
        stolen_commander(&mut t, "Darksteel Colossus");
        t.graveyard(P1, "Grizzly Bears");
        let v = t.graveyard(P0, card);
        t.lands(P0, color, 1);
        t.lands(P0, "Wastes", 4);
        assert!(!can_cast(&mut t, P0, v, FLASHBACK), "{card}");
        t.lands(P0, color, 1);
        assert!(can_cast(&mut t, P0, v, FLASHBACK), "{card}");
    }
}

/// The number of cards P0 was offered to put into their hand from decision `from` on.
fn looked_at(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn memory_deluge_looks_at_the_mana_actually_spent() {
    cr!("601.2h", "702.34a");
    ruling!(
        "Memory Deluge",
        "Memory Deluge cares about the mana you actually paid to cast it, not its mana cost. If you cast Memory Deluge for {2}{U}{U}, you will look at four cards. If you cast it with flashback for {5}{U}{U}, you will look at seven cards. Any effects that increase or decrease the cost to cast it will also be taken into account."
    );
    supported("Memory Deluge");
    for (flashback, n) in [(false, 4), (true, 7)] {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Island"; 10]);
        t.lands(P0, "Island", 7);
        let md = if flashback {
            t.graveyard(P0, "Memory Deluge")
        } else {
            t.hand(P0, "Memory Deluge")
        };
        let m = if flashback {
            FLASHBACK
        } else {
            CastMethod::Normal
        };
        t.cast(P0, md).method(m).go();
        let from = t.asked().len();
        t.resolve_all();
        assert_eq!(looked_at(&t, from), n);
    }
}

#[test]
fn past_in_flames_affects_only_cards_in_the_graveyard_as_it_resolves() {
    cr!("611.2c", "702.34a");
    ruling!(
        "Past in Flames",
        "Past in Flames affects only cards in your graveyard at the time it resolves. Instant and sorcery cards put into your graveyard later in the turn won't gain flashback."
    );
    supported("Past in Flames");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let shock = t.graveyard(P0, "Shock");
    let pif = t.hand(P0, "Past in Flames");
    t.cast(P0, pif).go();
    t.resolve_all();
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, shock, FLASHBACK));
    assert!(!can_cast(&mut t, P0, bolt, FLASHBACK));
}

#[test]
fn smiting_helix_is_black_even_cast_for_its_flashback_cost() {
    cr!("105.2", "202.2", "702.34a");
    ruling!(
        "Smiting Helix",
        "Smiting Helix is always a black spell. It’s not red and white if you cast it for its flashback cost."
    );
    supported("Smiting Helix");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    let sh = t.graveyard(P0, "Smiting Helix");
    let spell = t.cast(P0, sh).method(FLASHBACK).target(P1).go();
    let colors = t.obj_now(spell).chars.colors;
    assert!(colors.contains(Color::Black));
    assert!(!colors.contains(Color::Red) && !colors.contains(Color::White));
}

#[test]
fn tapping_creatures_for_a_cost_ignores_summoning_sickness() {
    cr!("302.6", "702.34a", "118.9");
    ruling!(
        "Group Project",
        "You can tap any untapped creatures you control to pay for Group Project's flashback cost, even ones you haven't controlled continuously since the beginning of your most recent turn."
    );
    ruling!(
        "Battle Screech",
        "You can tap any untapped white creatures you control to pay for Battle Screech’s flashback cost, even ones you haven’t controlled continuously since the beginning of your most recent turn."
    );
    ruling!(
        "Sephara, Sky's Blade",
        "To pay Sephara's alternative cost, you may tap any untapped creatures you control with flying, including ones you haven't controlled continuously since the beginning of your most recent turn."
    );
    // (card, creature, how many)
    for (card, creature) in [
        ("Group Project", "Grizzly Bears"),
        ("Battle Screech", "Savannah Lions"),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        let sick: Vec<_> = (0..3).map(|_| t.battlefield_sick(P0, creature)).collect();
        assert!(can_cast(&mut t, P0, c, FLASHBACK), "{card}");
        t.cast(P0, c).method(FLASHBACK).go();
        assert!(sick.iter().all(|s| tapped(&t, *s)), "{card}");
    }
    supported("Sephara, Sky's Blade");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let s = t.hand(P0, "Sephara, Sky's Blade");
    let birds: Vec<_> = (0..4)
        .map(|_| t.battlefield_sick(P0, "Storm Crow"))
        .collect();
    let alt = legal_cast_methods(&mut t, P0, s)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("alternative cost");
    t.cast(P0, s).method(alt).go();
    assert!(birds.iter().all(|b| tapped(&t, *b)));
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(s)));
}

#[test]
fn dread_return_targets_before_its_flashback_sacrifice() {
    cr!("601.2c", "601.2h", "702.34a");
    ruling!(
        "Dread Return",
        "You choose the target for Dread Return before paying any of its costs, so it's not possible to cast it using flashback and bring back one of the creatures you sacrifice."
    );
    supported("Dread Return");
    let mut t = TestGame::new(2);
    let dr = t.graveyard(P0, "Dread Return");
    let mine: Vec<_> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    // No creature card in the graveyard yet: no legal target, no casting.
    assert!(!can_cast(&mut t, P0, dr, FLASHBACK));
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    let e: Vec<Entity> = mine.iter().map(|m| Entity::Object(*m)).collect();
    t.answer_choose(P0, &e);
    t.cast(P0, dr).method(FLASHBACK).target(giant).go();
    let c = target_candidates(&t, P0, from);
    assert_eq!(c[0], vec![Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(giant)));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
}
