//! Rulings batch P025 — copies of cards exiled with a permanent's linked ability (CR
//! 607.2a, 607.2n, 607.3): which exiled cards can be copied, that becoming a copy isn't
//! entering the battlefield (CR 603.6a), that the copy doesn't depend on the card staying
//! in exile (CR 707.2b), copy effects and linked abilities (CR 607.5), and playing the
//! cards Tibalt, Cosmic Impostor exiles (CR 406.3, 601.3).

use crate::r_p017_common::choice_candidates;
use crate::r_p025_common::*;
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Exiles `p`'s new sideboard card `name` with cards named `with` (as their "before you
/// shuffle your deck to start the game" ability would, CR 607.2n).
fn exiled_before_the_game(t: &mut TestGame, p: PlayerId, with: &str, name: &str) -> ObjectId {
    let side = t.g.add_to_sideboard(p, vec![card(name)]);
    let e =
        t.g.move_object(side[0], Zone::Exile, MoveCause::Effect, Some(p))
            .expect("exiled");
    t.g.named_exiles.push((p, with.into(), e));
    e
}

// --- Volatile Chimera ----------------------------------------------------------------------

#[test]
fn volatile_chimera_becoming_a_copy_doesnt_trigger_enters_abilities() {
    cr!("603.6a", "607.2n", "707.2");
    ruling!(
        "Volatile Chimera",
        "No enters-the-battlefield abilities of the creature card Volatile Chimera is copying will trigger. Volatile Chimera was already on the battlefield."
    );
    supported("Volatile Chimera");
    let mut t = TestGame::new(2);
    // Elvish Visionary: "When this creature enters, draw a card."
    exiled_before_the_game(&mut t, P0, "Volatile Chimera", "Elvish Visionary");
    let chimera = t.battlefield(P0, "Volatile Chimera");
    lands_for(&mut t, P0, "{1}{R}");
    let hand = t.hand_size(P0);
    t.activate(P0, chimera, 0, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, chimera), "Elvish Visionary");
    assert_eq!(abilities_with(&t, chimera, "draw a card"), 1);
    assert!(triggers_on_stack(&t).is_empty());
    assert_eq!(t.hand_size(P0), hand);
}

// --- Assimilation Aegis --------------------------------------------------------------------

/// `p`'s Assimilation Aegis enters with a Panharmonicon out, so its enters ability
/// triggers twice: the instances exile `first` and `second`. Returns the Aegis.
fn aegis_exiling_two(t: &mut TestGame, p: PlayerId, first: ObjectId, second: ObjectId) -> ObjectId {
    supported("Assimilation Aegis");
    t.battlefield(p, "Panharmonicon");
    t.answer_targets(p, &[obj(first)]);
    t.answer_targets(p, &[obj(second)]);
    let aegis = t.enter(p, "Assimilation Aegis");
    t.settle();
    assert_eq!(triggers_on_stack(t).len(), 2);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(first)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(second)), Zone::Exile);
    aegis
}

#[test]
fn assimilation_aegis_chooses_among_its_exiled_creature_cards_each_time() {
    cr!("607.2a", "607.3", "603.2d", "707.2");
    ruling!(
        "Assimilation Aegis",
        "If you somehow exile multiple creature cards with the same Assimilation Aegis (probably because its first ability was copied or something caused it to trigger more than once), you choose one of the exiled creature cards each time its second triggered ability resolves. You don't have to choose the same one each time."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let aegis = aegis_exiling_two(&mut t, P0, giant, wurm);
    let (giant_card, wurm_card) = (t.g.current(giant), t.g.current(wurm));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    // Attached to the Bears: the Hill Giant is chosen.
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant_card)]);
    assert!(t.g.attach(aegis, obj(bears)));
    t.resolve_all();
    let offered = choice_candidates(&t, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(giant_card)) && offered[0].contains(&obj(wurm_card)));
    assert_eq!(name_of(&t, bears), "Hill Giant");
    // Moved to the Wall: this time the Craw Wurm.
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(wurm_card)]);
    assert!(t.g.attach(aegis, obj(wall)));
    t.resolve_all();
    let offered = choice_candidates(&t, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(giant_card)) && offered[0].contains(&obj(wurm_card)));
    assert_eq!(name_of(&t, wall), "Craw Wurm");
    assert_eq!(t.pt(wall), (6, 4));
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
}

#[test]
fn assimilation_aegis_cant_copy_an_exiled_noncreature_card() {
    cr!("607.2a", "707.2", "603.2d");
    ruling!(
        "Assimilation Aegis",
        "If you somehow exile a noncreature card with Assimilation Aegis's first ability (possibly because it was a copy of a creature while on the battlefield), you can't choose to have the creature Assimilation Aegis is attached to become a copy of that card when Assimilation Aegis's second ability resolves."
    );
    // The Aegis exiles a Forest that was a creature on the battlefield and a Hill Giant:
    // only the Giant card can be copied.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    animate_land(&mut t, land);
    let giant = t.battlefield(P1, "Hill Giant");
    let aegis = aegis_exiling_two(&mut t, P0, land, giant);
    let land_card = t.g.current(land);
    assert!(!t.obj_now(land_card).is(CardType::Creature));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(land_card)]);
    assert!(t.g.attach(aegis, obj(bears)));
    t.resolve_all();
    for c in choice_candidates(&t, from) {
        assert!(!c.contains(&obj(land_card)), "{c:?}");
    }
    assert_eq!(name_of(&t, bears), "Hill Giant");
    assert!(is_creature(&t, bears));
    // With only the Forest card exiled, the creature stays as it was.
    let mut t = TestGame::new(2);
    supported("Assimilation Aegis");
    let land = t.battlefield(P1, "Forest");
    animate_land(&mut t, land);
    t.answer_targets(P0, &[obj(land)]);
    let aegis = t.enter(P0, "Assimilation Aegis");
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(land)), Zone::Exile);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(aegis, obj(bears)));
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
    assert!(is_creature(&t, bears));
    assert_eq!(t.pt(bears), (2, 2));
}

// --- Olag, Ludevic's Hubris ------------------------------------------------------------------

#[test]
fn olag_stays_a_copy_after_the_card_leaves_exile() {
    cr!("707.2b", "712.20", "607.2a");
    ruling!(
        "Ludevic, Necrogenius // Olag, Ludevic's Hubris",
        "Olag remains a copy of the creature card you choose even if that card later leaves exile."
    );
    supported("Ludevic, Necrogenius // Olag, Ludevic's Hubris");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ludevic = t.battlefield(P0, "Ludevic, Necrogenius // Olag, Ludevic's Hubris");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{2}{U}{U}{B}{B}");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(giant), obj(bears)]);
    t.answer_choose_moved(P0, &[obj(giant)]);
    t.activate(P0, ludevic, 0, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, ludevic), "Olag, Ludevic's Hubris");
    assert!(has_subtype(&t, ludevic, "Giant"));
    assert_eq!(t.pt(ludevic), (6, 6));
    // The Hill Giant card leaves exile.
    let card = t.g.current(giant);
    assert_eq!(t.zone(card), Zone::Exile);
    t.g.move_object(card, Zone::Graveyard(P0), MoveCause::Effect, Some(P1));
    t.g.recompute();
    assert!(!t.in_exile("Hill Giant"));
    let o = t.obj_now(ludevic);
    assert_eq!(o.chars.name, "Olag, Ludevic's Hubris");
    assert!(has_subtype(&t, ludevic, "Giant") && has_subtype(&t, ludevic, "Zombie"));
    assert!(o.chars.colors.contains(Color::Red));
    assert_eq!(t.pt(ludevic), (6, 6));
}

// --- Tibalt, Cosmic Impostor -----------------------------------------------------------------

/// `p` casts Tibalt, Cosmic Impostor (the back face of Valki); returns the planeswalker.
fn tibalt(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Valki, God of Lies // Tibalt, Cosmic Impostor");
    lands_for(t, p, "{5}{B}{R}");
    let c = t.hand(p, "Valki, God of Lies // Tibalt, Cosmic Impostor");
    t.cast(p, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
    t.named_on_battlefield("Tibalt, Cosmic Impostor")[0]
}

#[test]
fn tibalts_loyalty_abilities_exile_cards_face_up() {
    cr!("406.3", "606.3");
    ruling!(
        "Valki, God of Lies // Tibalt, Cosmic Impostor",
        "The cards exiled by Tibalt's loyalty abilities are all exiled face up."
    );
    let face_up = |t: &TestGame, ids: &[ObjectId]| {
        for id in ids {
            let c = t.g.current(*id);
            assert_eq!(t.zone(c), Zone::Exile, "{}", name_of(t, c));
            assert!(!t.obj_now(c).face_down, "{}", name_of(t, c));
        }
    };
    // +2: the top card of each player's library.
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t, P0);
    let tops = [
        t.library_top(P0, "Grizzly Bears"),
        t.library_top(P1, "Divination"),
    ];
    activate_containing(&mut t, P0, tib, "Exile the top card").expect("+2");
    t.resolve_all();
    face_up(&t, &tops);
    // −3: target artifact or creature.
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, tib, "Exile target artifact").expect("-3");
    t.resolve_all();
    face_up(&t, &[giant]);
    // −8: all graveyards.
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t, P0);
    t.g.add_counters(obj(tib), counters::LOYALTY, 3, None);
    let gy = [
        t.graveyard(P0, "Lightning Bolt"),
        t.graveyard(P1, "Hill Giant"),
    ];
    activate_containing(&mut t, P0, tib, "Exile all graveyards").expect("-8");
    t.resolve_all();
    face_up(&t, &gy);
}

#[test]
fn casting_a_card_exiled_with_tibalt_follows_the_normal_timing_and_cost_rules() {
    cr!("117.1a", "601.3", "601.2h");
    ruling!(
        "Valki, God of Lies // Tibalt, Cosmic Impostor",
        "Playing the cards exiled with Tibalt follows the normal rules for playing those cards. You must pay their costs, if any, and you must follow all applicable timing rules."
    );
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t, P0);
    // P1's Divination (a sorcery, {2}{U}) is exiled.
    let div = t.library_top(P1, "Divination");
    t.library_top(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, tib, "Exile the top card").expect("+2");
    t.resolve_all();
    let div = t.g.current(div);
    assert_eq!(t.zone(div), Zone::Exile);
    // Its mana cost must be paid: two lands aren't enough.
    t.lands(P0, "Mountain", 2);
    assert!(t.cast(P0, div).try_go().is_err());
    assert_eq!(t.zone(div), Zone::Exile);
    // Not while a spell is on the stack.
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert_eq!(t.stack_len(), 1);
    assert!(t.cast(P0, div).try_go().is_err());
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Not during combat, nor on an opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, div).try_go().is_err());
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, div).try_go().is_err());
    assert_eq!(t.zone(div), Zone::Exile);
    // In P0's main phase with an empty stack, paying {2}{U} with Mountains.
    t.set_step(P0, Step::PostcombatMain);
    let hand = t.hand_size(P0);
    t.cast(P0, div).try_go().expect("cast from exile");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Divination"));
    assert_eq!(t.hand_size(P0), hand + 2);
}

// --- Spirit of Resilience ----------------------------------------------------------------------

#[test]
fn spirit_of_resilience_copies_only_the_printed_card() {
    cr!("707.2", "400.7", "603.10a");
    ruling!(
        "Spirit of Resilience",
        "Spirit of Resilience copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Spirit of Resilience");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Spirit of Resilience");
    // P0's Clone copying a Serra Angel dies: in the graveyard it's a Clone card.
    let clone = cloned_angel(&mut t, P0);
    t.g.destroy(clone, None);
    t.settle();
    let card = t.g.current(clone);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // It leaves the graveyard: the Spirit gets a +1/+1 counter and becomes a copy of the
    // Clone card, a 0/0 with no abilities that matter here, not a Serra Angel.
    let moved =
        t.g.move_object(card, Zone::Exile, MoveCause::Effect, Some(P1))
            .expect("exiled");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(moved)]);
    t.settle();
    t.resolve_all();
    let o = t.obj_now(spirit);
    assert_eq!(o.chars.name, "Clone");
    assert!(!o.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(t.counters(spirit, counters::PLUS1), 1);
    assert_eq!(t.pt(spirit), (1, 1));
}

// --- Caller of the Untamed -------------------------------------------------------------------

/// Two Callers of the Untamed in P0's deck each exile a creature card P0 drafted before
/// the game (CR 607.2n); a Caller copies either. (Its ruling's "You'll choose which one to
/// copy at the time you choose the value of X" isn't cited: with two such cards of the
/// same mana value the engine has the card chosen as the ability resolves, see the
/// interpretation `x-defined-by-target`.)
#[test]
fn each_caller_of_the_untamed_in_the_deck_exiles_a_card_before_the_game() {
    cr!("607.2n", "607.3", "107.3c");
    interp!("x-defined-by-target");
    supported("Caller of the Untamed");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Caller of the Untamed");
    t.library_top(P0, "Caller of the Untamed");
    let side = t.g.add_to_sideboard(
        P0,
        vec![card("Hill Giant"), card("Grizzly Bears"), card("Craw Wurm")],
    );
    for c in [side[0], side[1]] {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(c)]);
    }
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
    assert!(!t.in_exile("Craw Wurm"));
    // X = 4 copies the Hill Giant, X = 2 the Grizzly Bears.
    for (x, name) in [(4, "Hill Giant"), (2, "Grizzly Bears")] {
        let caller = t.battlefield(P0, "Caller of the Untamed");
        t.lands(P0, "Forest", x as usize);
        t.answer(P0, DecisionKind::X, Answer::Number(x));
        let before = t.g.battlefield.clone();
        t.activate(P0, caller, 0, &[]).expect("activation");
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 1, "{name}");
        assert_eq!(name_of(&t, toks[0]), name);
    }
    // X = 6 (the Craw Wurm wasn't exiled) can't be announced.
    let caller = t.battlefield(P0, "Caller of the Untamed");
    t.lands(P0, "Forest", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(6));
    let before = t.g.battlefield.clone();
    let _ = t.activate(P0, caller, 0, &[]);
    t.resolve_all();
    assert!(new_tokens(&t, P0, &before)
        .iter()
        .all(|x| name_of(&t, *x) != "Craw Wurm"));
}
