//! Rulings batch P121 — looking at and playing the top card of a library: a player
//! allowed to look at it may do so any time, but not at a new top card while casting a
//! spell (CR 401.5); a card played from the top follows the normal timing and cost rules
//! (CR 601.3, 305.2, 118.9a) and isn't in its owner's hand (CR 602.2); "once each turn"
//! permissions of a new object are new (CR 400.7).

use crate::r_p121_common::*;
use crate::r_s01_common::watch;
use crate::r_s02_common::can_play_land;
use crate::r_s08_common::{actions_of, legal_cast_methods};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::Game;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn methods(t: &mut TestGame, card: ObjectId) -> Vec<CastMethod> {
    legal_cast_methods(t, P0, card)
}

/// Whether P0 may look at the top card of their library now.
fn p0_sees_top(g: &Game) -> bool {
    g.library_top(P0).is_some_and(|top| can_look_at(g, P0, top))
}

#[test]
fn the_top_card_can_be_looked_at_any_time_without_priority() {
    cr!("401.5");
    ruling!(
        "Sphinx of Jwar Isle",
        "Essentially, Sphinx of Jwar Isle lets you play with the top card of your library revealed only to you. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand. You may look at the top card of your library whenever you want, even if you don’t have priority. This action doesn’t use the stack."
    );
    ruling!(
        "Experimental Frenzy",
        "Experimental Frenzy lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don’t have priority. This action doesn’t use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    ruling!(
        "Into the Pit",
        "While you control Into the Pit, you can look at the top card of your library whenever you want (with one restriction; see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    for name in ["Sphinx of Jwar Isle", "Experimental Frenzy", "Into the Pit"] {
        supported(name);
        let mut t = TestGame::new(2);
        let top = t.library_top(P0, "Grizzly Bears");
        assert!(!can_look_at(&t.g, P0, top));
        t.battlefield(P0, name);
        t.g.recompute();
        // During P1's turn, with P1 holding priority.
        t.set_step(P1, Step::DeclareAttackers);
        assert_eq!(t.g.turn.priority, Some(P1));
        assert!(can_look_at(&t.g, P0, top), "{name}");
        assert!(!can_look_at(&t.g, P1, top), "{name}");
        assert_eq!(t.stack_len(), 0);
        // Only the top card.
        let next = t.library_top(P0, "Hill Giant");
        assert!(can_look_at(&t.g, P0, next) && !can_look_at(&t.g, P0, top));
    }
}

#[test]
fn sphinx_of_jwar_isle_new_top_card_hidden_while_casting() {
    cr!("401.5", "601.2i");
    ruling!(
        "Sphinx of Jwar Isle",
        "If the top card of your library changes during the process of casting a spell or activating an ability, you can’t look at the new top card until the process of casting the spell or activating the ability ends (all targets are chosen, all costs are paid, and so on)."
    );
    supported("Garruk's Horde");
    // Garruk's Horde lets P0 cast the creature on top (Endless One, {X}); Sphinx lets
    // P0 look at the top card.
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let one = t.library_top(P0, "Endless One");
    t.battlefield(P0, "Sphinx of Jwar Isle");
    t.battlefield(P0, "Garruk's Horde");
    t.lands(P0, "Wastes", 2);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseX { .. }),
        p0_sees_top,
    );
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, one).go();
    assert_eq!(t.g.library_top(P0), Some(next));
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert!(p0_sees_top(&t.g));
}

#[test]
fn gwenom_look_permission_ends_with_the_turn() {
    cr!("401.5", "514.2");
    ruling!(
        "Gwenom, Remorseless",
        "After the resolution of the triggered ability, Gwenom lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand. Once the effect has ended in your cleanup step, however, you may not look at the top card anymore."
    );
    supported("Gwenom, Remorseless");
    let mut t = TestGame::new(2);
    let gwenom = t.battlefield(P0, "Gwenom, Remorseless");
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    attack_with(&mut t, &[(gwenom, Entity::Player(P1))]);
    t.resolve_all();
    t.g.recompute();
    assert!(can_look_at(&t.g, P0, top));
    t.advance_to(P0, Step::End);
    assert!(can_look_at(&t.g, P0, top));
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_look_at(&t.g, P0, top));
}

#[test]
fn gwenom_spells_cost_life_and_may_be_kicked() {
    cr!("118.9a", "118.8", "601.2f");
    ruling!(
        "Gwenom, Remorseless",
        "If you cast a spell for another cost \"rather than pay its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as a kicker cost. If the card has any mandatory additional costs, those must be paid to cast the card."
    );
    let mut t = TestGame::new(2);
    let gwenom = t.battlefield(P0, "Gwenom, Remorseless");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    attack_with(&mut t, &[(gwenom, Entity::Player(P1))]);
    t.resolve_all();
    // Snuff Out: only for Gwenom's cost, not its own alternative cost.
    let snuff = t.library_top(P0, "Snuff Out");
    assert_eq!(
        methods(&mut t, snuff),
        vec![CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST)]
    );
    // Burst Lightning kicked: 1 life and {4}.
    let burst = t.library_top(P0, "Burst Lightning");
    let m = methods(&mut t, burst);
    assert_eq!(m.len(), 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, burst)
        .method(m[0].clone())
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.life(P0), 19);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Bone Splinters (a sorcery) must have its creature sacrificed: Gwenom is the only
    // one.
    t.advance_to(P0, Step::PostcombatMain);
    let splinters = t.library_top(P0, "Bone Splinters");
    let m = methods(&mut t, splinters);
    assert_eq!(m.len(), 1);
    t.cast(P0, splinters).method(m[0].clone()).target(giant).go();
    assert!(t.in_graveyard(P0, "Gwenom, Remorseless"));
}

// --- once-each-turn permissions of a new object --------------------------------------------

#[test]
fn a_new_johann_grants_a_new_permission() {
    cr!("400.7", "601.3");
    ruling!(
        "Johann, Apprentice Sorcerer",
        "If Johann leaves the battlefield and returns to the battlefield in the same turn, or if Johann leaves and another Johann appears, the casting permission granted by the old one is different than the casting permission granted by the new one. On the turn in which the new Johann enters the battlefield, you may use its casting permission once, even if you used the old Johann's casting permission to cast an instant or sorcery spell from the top of your library in that same turn. Note that if you control multiple Johanns you won't have the opportunity to cast an instant or sorcery spell from the top of your library before the \"legend rule\" gets you down to one Johann."
    );
    supported("Johann, Apprentice Sorcerer");
    let mut t = TestGame::new(2);
    let johann = t.battlefield(P0, "Johann, Apprentice Sorcerer");
    t.lands(P0, "Mountain", 3);
    let b2 = t.library_top(P0, "Lightning Bolt");
    let b1 = t.library_top(P0, "Lightning Bolt");
    let m = methods(&mut t, b1);
    assert_eq!(m.len(), 1);
    t.cast(P0, b1).method(m[0].clone()).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(methods(&mut t, b2).is_empty(), "used this turn");
    exile_now(&mut t, johann);
    t.enter(P0, "Johann, Apprentice Sorcerer");
    let m = methods(&mut t, b2);
    assert_eq!(m.len(), 1);
    t.cast(P0, b2).method(m[0].clone()).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn a_new_one_with_the_multiverse_grants_a_new_free_cast() {
    cr!("400.7", "118.9");
    ruling!(
        "One with the Multiverse",
        "If One with the Multiverse leaves the battlefield and later returns to the battlefield (or you play a second one), it's considered a new object and you may cast a spell using its last ability regardless of whether or not you did so earlier this turn using the enchantment that was on the battlefield before."
    );
    supported("One with the Multiverse");
    let mut t = TestGame::new(2);
    let one = t.battlefield(P0, "One with the Multiverse");
    let b1 = t.hand(P0, "Lightning Bolt");
    let b2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1)
        .method(CastMethod::Free)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(!methods(&mut t, b2).contains(&CastMethod::Free));
    exile_now(&mut t, one);
    t.battlefield(P0, "One with the Multiverse");
    assert!(methods(&mut t, b2).contains(&CastMethod::Free));
    t.cast(P0, b2)
        .method(CastMethod::Free)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

// --- free casts and costs -------------------------------------------------------------------

#[test]
fn one_with_the_multiverse_free_prototype_either_way() {
    cr!("718.3", "118.9");
    ruling!(
        "One with the Multiverse",
        "While casting a card with the prototype ability using the last ability of One with the Multiverse, you can either choose to cast it normally or choose to cast it as a prototyped spell. In either case, you won't pay its mana cost."
    );
    supported("Phyrexian Fleshgorger");
    for proto in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "One with the Multiverse");
        let c = t.library_top(P0, "Phyrexian Fleshgorger");
        if proto {
            // The prototype permission without paying its mana cost.
            t.answer(P0, DecisionKind::Option, Answer::Index(1));
            t.cast(P0, c)
                .method(CastMethod::Keyword(
                    mtg_engine::keywords::KeywordKind::Prototype,
                ))
                .go();
        } else {
            t.cast(P0, c).method(CastMethod::Free).go();
        }
        t.resolve_all();
        let f = t.named_on_battlefield("Phyrexian Fleshgorger");
        assert_eq!(f.len(), 1);
        assert_eq!(t.pt(f[0]), if proto { (3, 3) } else { (7, 5) });
    }
}

#[test]
fn one_with_the_multiverse_free_cast_takes_additional_costs_only() {
    cr!("118.9", "118.9a", "118.8");
    ruling!(
        "One with the Multiverse",
        "While casting a spell without paying its mana cost using the last ability, you can't pay alternative costs, but you may choose to pay any optional additional costs the spell may have (such as a kicker cost). You still must pay any mandatory additional costs."
    );
    // Kicker: Burst Lightning cast free, kicked for {4}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "One with the Multiverse");
    t.lands(P0, "Mountain", 4);
    let burst = t.hand(P0, "Burst Lightning");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, burst)
        .method(CastMethod::Free)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Mandatory: Bone Splinters can't be cast free without a creature to sacrifice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "One with the Multiverse");
    t.battlefield(P1, "Hill Giant");
    let splinters = t.hand(P0, "Bone Splinters");
    assert!(!methods(&mut t, splinters).contains(&CastMethod::Free));
    t.battlefield(P0, "Grizzly Bears");
    assert!(methods(&mut t, splinters).contains(&CastMethod::Free));
    // Snuff Out: its alternative cost is a separate way of casting it, not combined with
    // the free cast (a free cast pays no life).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "One with the Multiverse");
    t.lands(P0, "Swamp", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let snuff = t.hand(P0, "Snuff Out");
    t.cast(P0, snuff).method(CastMethod::Free).target(giant).go();
    assert_eq!(t.life(P0), 20);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
}

#[test]
fn spells_from_the_top_follow_timing_and_costs() {
    cr!("601.3", "307.1", "118.9a", "601.2f");
    ruling!(
        "One with the Multiverse",
        "You must follow all normal timing rules for spells cast and lands played using the middle ability and must pay all costs for spells cast this way. To cast a spell this way, you may choose to pay any alternative costs or optional additional costs the spell may have (such as a kicker cost). You must pay any mandatory additional costs."
    );
    ruling!(
        "Experimental Frenzy",
        "You’ll still pay all costs for a spell you cast from your library, including additional costs. You may also pay alternative costs."
    );
    ruling!(
        "Into the Pit",
        "You pay all costs and follow all timing rules for spells cast from the top of your library this way."
    );
    supported("Divination");
    for perm in [
        "One with the Multiverse",
        "Experimental Frenzy",
        "Into the Pit",
    ] {
        // Alternative cost: Snuff Out's "pay 4 life" (Into the Pit adds a sacrifice).
        let mut t = TestGame::new(2);
        t.battlefield(P0, perm);
        t.lands(P0, "Swamp", 1);
        t.battlefield(P0, "Ornithopter");
        let snuff = t.library_top(P0, "Snuff Out");
        assert!(
            methods(&mut t, snuff)
                .iter()
                .any(|m| matches!(m, CastMethod::Alternative(_))),
            "{perm}"
        );
        // Timing: a sorcery on top can't be cast during P1's turn.
        let mut t = TestGame::new(2);
        t.battlefield(P0, perm);
        t.lands(P0, "Island", 3);
        t.battlefield(P0, "Ornithopter");
        let div = t.library_top(P0, "Divination");
        assert!(methods(&mut t, div).contains(&CastMethod::Normal), "{perm}");
        t.set_step(P1, Step::PrecombatMain);
        assert!(!methods(&mut t, div).contains(&CastMethod::Normal), "{perm}");
        // Costs: Divination can't be cast from the top without its mana; Bone Splinters
        // not without a creature to sacrifice.
        let mut t = TestGame::new(2);
        t.battlefield(P0, perm);
        t.lands(P0, "Swamp", 1);
        let div = t.library_top(P0, "Divination");
        assert!(!methods(&mut t, div).contains(&CastMethod::Normal), "{perm}");
        let splinters = t.library_top(P0, "Bone Splinters");
        assert!(
            !methods(&mut t, splinters).contains(&CastMethod::Normal),
            "{perm}"
        );
        t.battlefield(P0, "Grizzly Bears");
        assert!(
            methods(&mut t, splinters).contains(&CastMethod::Normal),
            "{perm}"
        );
    }
}

#[test]
fn into_the_pit_casts_a_morph_face_down_from_the_top() {
    cr!("702.37c", "601.3", "118.8");
    ruling!(
        "Into the Pit",
        "While you control Into the Pit, if the top card of your library has disguise or morph, you may cast it face down from the top of your library by paying {3} and sacrificing a nonland permanent."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Into the Pit");
    t.lands(P0, "Wastes", 3);
    let thopter = t.battlefield(P0, "Ornithopter");
    let hydra = t.library_top(P0, "Hooded Hydra");
    let fd = CastMethod::FaceDown(mtg_engine::keywords::KeywordKind::Morph);
    assert!(methods(&mut t, hydra).contains(&fd));
    t.answer_choose(P0, &[obj(thopter)]);
    t.cast(P0, hydra).method(fd).go();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    t.resolve_all();
    let fd = t
        .g
        .permanents()
        .find(|o| o.face_down)
        .map(|o| o.id)
        .expect("a face-down creature");
    assert_eq!(t.pt(fd), (2, 2));
}

// --- lands from the top, and the top card isn't in the hand ---------------------------------

#[test]
fn a_land_from_the_top_needs_a_land_play_and_sorcery_timing() {
    cr!("305.2", "305.1", "116.2a");
    ruling!(
        "Experimental Frenzy",
        "You can play a land card on top of your library only if you have available land plays remaining."
    );
    ruling!(
        "Isu the Abominable",
        "You must pay all costs and follow all timing rules for cards played from the top of your library this way. For example, you may play a snow land this way only while the stack is empty during one of your own main phases, and only if you haven't played a land yet this turn."
    );
    supported("Isu the Abominable");
    for (perm, land) in [
        ("Experimental Frenzy", "Forest"),
        ("Isu the Abominable", "Snow-Covered Forest"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, perm);
        let top = t.library_top(P0, land);
        assert!(can_play_land(&mut t, P0, top), "{perm}");
        // Not while the stack isn't empty, nor during P1's turn.
        cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        assert!(!can_play_land(&mut t, P0, top), "{perm}");
        t.resolve_all();
        t.set_step(P1, Step::PrecombatMain);
        assert!(!can_play_land(&mut t, P0, top), "{perm}");
        // After the turn's land play, not even in P0's main phase.
        let mut t = TestGame::new(2);
        t.battlefield(P0, perm);
        let top = t.library_top(P0, land);
        let hand = t.hand(P0, "Plains");
        if perm == "Experimental Frenzy" {
            // Frenzy forbids playing lands from the hand: use the top one instead.
            t.play_land(P0, top).expect("land from the top");
            t.settle();
            let next = t.library_top(P0, land);
            assert!(!can_play_land(&mut t, P0, next), "{perm}");
        } else {
            t.play_land(P0, hand).expect("land from the hand");
            t.settle();
            assert!(!can_play_land(&mut t, P0, top), "{perm}");
        }
    }
}

#[test]
fn isu_the_top_card_isnt_in_the_hand() {
    cr!("602.2", "702.29a");
    ruling!(
        "Isu the Abominable",
        "The top card of your library is still in your library and not in your hand. You can't discard cards from the top of your library or activate abilities of cards on top of your library that normally work in your hand."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isu the Abominable");
    t.lands(P0, "Swamp", 3);
    let horror = t.library_top(P0, "Horror of the Broken Lands");
    let horror = t.g.current(horror);
    let acts = actions_of(&mut t, P0);
    assert!(!acts.iter().any(|a| matches!(a,
        Action::Activate { source, .. } if *source == horror)));
    // The same card in the hand can be cycled.
    let in_hand = t.hand(P0, "Horror of the Broken Lands");
    let acts = actions_of(&mut t, P0);
    assert!(acts.iter().any(|a| matches!(a,
        Action::Activate { source, .. } if *source == in_hand)));
    // A discard effect doesn't reach it.
    exile_now(&mut t, in_hand);
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s29_common::cast_and_resolve(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    assert_eq!(t.g.library_top(P0), Some(horror));
}
