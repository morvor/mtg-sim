//! Item `rulings-residue-misc-2`, pass B — the remaining rulings: what's counted or chosen
//! as an ability resolves (Garruk, the Veil-Cursed; Sigurd's lore counter; Fasting's
//! skipped draw step), then restatements of the cards' own text, a compact case each.

use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Garruk, the Veil-Cursed --------------------------------------------------------------------

/// P0's Garruk, the Veil-Cursed (transformed) with `loyalty` loyalty counters.
fn veil_cursed(t: &mut TestGame, loyalty: u32) -> ObjectId {
    let g = t.battlefield(P0, "Garruk Relentless // Garruk, the Veil-Cursed");
    t.g.objects[g.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), loyalty);
    assert!(mtg_engine::dfc::transform(&mut t.g, g));
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    let g = t.g.current(g);
    assert_eq!(t.obj_now(g).chars.name, "Garruk, the Veil-Cursed");
    g
}

#[test]
fn garruk_veil_cursed_overrun_counts_and_affects_only_at_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Garruk Relentless // Garruk, the Veil-Cursed",
        "Only creatures you control when the third ability of Garruk, the Veil-Cursed resolves will receive the bonus. Creatures that enter or that you gain control of later in the turn won't be affected."
    );
    ruling!(
        "Garruk Relentless // Garruk, the Veil-Cursed",
        "The number of creature cards in your graveyard is counted when the third ability of Garruk, the Veil-Cursed resolves. Once the ability resolves, the bonus doesn't change if that number changes later in the turn."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let g = veil_cursed(&mut t, 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.activate(P0, g, 2, &[]).expect("-3");
    // One more creature card before it resolves: X = 2.
    t.graveyard(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    // A third card later, and a creature arriving later: no change, no bonus.
    t.graveyard(P0, "Llanowar Elves");
    let late = t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(late), (2, 1));
    assert!(!t.obj_now(late).has_keyword(KeywordKind::Trample));
}

#[test]
fn garruk_veil_cursed_sacrifice_doesnt_target_but_is_required() {
    cr!("115.1", "701.21a", "608.2d");
    ruling!(
        "Garruk Relentless // Garruk, the Veil-Cursed",
        "The second ability of Garruk, the Veil-Cursed doesn't target a creature. However, when that ability resolves, you must sacrifice a creature if you control one."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let g = veil_cursed(&mut t, 3);
    t.battlefield(P0, "Grizzly Bears");
    let wurm = t.library_top(P0, "Craw Wurm");
    let from = t.asked().len();
    // P0 declines the sacrifice: it happens anyway (the only creature).
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[obj(wurm)]);
    t.activate(P0, g, 1, &[]).expect("-1");
    t.resolve_all();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Craw Wurm"));
}

// --- Sigurd, Jarl of Ravensthorpe ---------------------------------------------------------------

#[test]
fn sigurd_lore_counter_past_the_final_chapter_triggers_nothing() {
    cr!("714.2b", "714.2c", "714.4");
    ruling!(
        "Sigurd, Jarl of Ravensthorpe",
        "Putting a lore counter on a Saga will usually cause the next chapter ability to trigger. If it already has a number of lore counters on it equal to its final chapter number however, no abilities trigger. In particular, this doesn’t cause the final chapter ability to trigger again."
    );
    supported("History of Benalia");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let sig = t.battlefield(P0, "Sigurd, Jarl of Ravensthorpe");
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    attack_with(&mut t, &[(sig, pl(P1))]);
    // The third lore counter: chapter III triggers and waits.
    t.g.add_counters(obj(saga), counters::LORE, 1, Some(sig));
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_of(&t, "History of Benalia").len(), 1);
    // Sigurd's boast puts a fourth one on: no chapter ability triggers again.
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    activate_containing(&mut t, P0, sig, "lore counter").expect("boast");
    t.resolve();
    assert_eq!(t.counters(saga, counters::LORE), 4, "{}", t.dump_log());
    assert_eq!(triggers_of(&t, "History of Benalia").len(), 1);
}

// --- Fasting ----------------------------------------------------------------------------------

#[test]
fn fasting_skip_is_chosen_as_the_draw_step_begins_after_the_upkeep_trigger() {
    cr!("614.10", "614.1b", "504.1");
    ruling!(
        "Fasting",
        "You choose whether or not to skip your draw step as that step begins. This ability is now separate from the beginning-of-upkeep triggered ability."
    );
    supported("Fasting");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fasting");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| {
            let f = g
                .battlefield
                .iter()
                .copied()
                .find(|o| g.obj(*o).chars.name == "Fasting");
            (f.map(|f| g.obj(f).counter("hunger")), g.turn.step)
        },
    );
    t.answer_yes(P0, true);
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    // Asked once the upkeep trigger had put its counter on, as the draw step would begin.
    let s = seen.lock().unwrap().clone();
    assert_eq!(s.len(), 1, "{s:?}");
    assert_eq!(s[0].0, Some(1));
    assert_eq!(s[0].1, Step::Draw);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 22);
    assert!(t.on_battlefield(f));
}

// --- Tolarian Contempt ------------------------------------------------------------------------

#[test]
fn tolarian_contempt_owner_orders_several_cards_and_others_cant_see_them() {
    cr!("401.4", "401.2");
    ruling!(
        "Tolarian Contempt",
        "However, if a player ends up putting multiple cards on top of their library or on the bottom of their library, either because a creature they owned but didn't control was rejected along with one they controlled or a rejected creature they owned was comprised of multiple cards (e.g. a mutated or melded permanent), other players don't get to know the relative order of those cards."
    );
    // P1 owns a Hill Giant P1 controls and a Grizzly Bears P2 controls; both rejected to
    // the bottom of P1's library.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tolarian Contempt");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, bears, P2);
    for c in [giant, bears] {
        t.g.add_counters(obj(t.g.current(c)), "rejection", 1, None);
    }
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let lib = &t.g.player(P1).library;
    let bottom: Vec<ObjectId> = lib[..2].to_vec();
    let names: Vec<String> = bottom
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert!(
        names.contains(&"Hill Giant".to_string()) && names.contains(&"Grizzly Bears".to_string()),
        "{names:?}"
    );
    // Their owner arranged them; the other players can't look at them.
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Order { .. })));
    for c in bottom {
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, c));
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P2, c));
    }
}

// --- Restatements -------------------------------------------------------------------------------

#[test]
fn restatements_of_the_cards_text() {
    cr!("608.2c", "111.4", "509.1a", "701.20a");
    // Arid Archway: alone, it returns itself; with another land, P0 may still choose it.
    ruling!(
        "Arid Archway",
        "Arid Archway will return itself to its owner’s hand if you control no other lands when its enters-the-battlefield ability resolves (or if you choose to return it to its owner’s hand despite controlling other lands). It’s almost never a good idea to play it on your first turn."
    );
    for other in [false, true] {
        let mut t = TestGame::new(2);
        if other {
            t.battlefield(P0, "Forest");
        }
        let aa = t.hand(P0, "Arid Archway");
        t.answer_choose_moved(P0, &[obj(aa)]);
        t.play_land(P0, aa).unwrap();
        t.resolve_all();
        assert!(t.in_hand(P0, "Arid Archway"), "other land: {other}");
    }

    // Mounted Archers: two activations, three blockers.
    ruling!(
        "Mounted Archers",
        "You can use the ability multiple times so it can block more than two creatures."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let atk: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let ma = t.battlefield(P1, "Mounted Archers");
    t.lands(P1, "Plains", 2);
    for _ in 0..2 {
        activate_containing(&mut t, P1, ma, "block an additional").expect("{W}");
        t.resolve_all();
    }
    attack_with(
        &mut t,
        &atk.iter().map(|a| (*a, pl(P1))).collect::<Vec<_>>(),
    );
    assert!(legal_blocks(
        &mut t,
        P1,
        &atk.iter().map(|a| (ma, *a)).collect::<Vec<_>>()
    ));

    // Neriv: two Elf tokens and two Goblin tokens are two names: two cards exiled.
    ruling!(
        "Neriv, Crackling Vanguard",
        "To determine the number of differently named tokens you control, count each token you control once, but only if its English name isn’t the same as another token you’ve already counted this way. For example, if you control two tokens named “Elf Token” and one token named “Goblin Token,” you exile two cards from the top of your library."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let n = t.enter(P0, "Neriv, Crackling Vanguard");
    t.resolve_all();
    create_token(&mut t, P0, "Elf");
    create_token(&mut t, P0, "Elf");
    t.g.objects[n.0 as usize].summoning_sick = false;
    let lib = t.library_size(P0);
    attack_with(&mut t, &[(n, pl(P1))]);
    t.resolve_all();
    assert_eq!(lib - t.library_size(P0), 2, "{}", t.dump_log());

    // Angelic Voices: white-green and red artifact creatures are fine; green isn't.
    ruling!(
        "Angelic Voices",
        "You get the bonus if all of your creatures are white or artifact even if they also have other colors or qualities. So, if your only creature is a white and green creature, or a red artifact creature, you would get the bonus."
    );
    for (c, bonus) in [
        ("Watchwolf", true),
        ("Kavaron Harrier", true),
        ("Grizzly Bears", false),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Angelic Voices");
        let id = t.battlefield(P0, c);
        let base = base_pt(c);
        let want = if bonus {
            (base.0 + 1, base.1 + 1)
        } else {
            base
        };
        assert_eq!(t.pt(id), want, "{c}");
    }

    // Sphinx Ambassador: the card is chosen first, then the player names a card; a
    // noncreature card can be chosen but never put onto the battlefield.
    ruling!(
        "Sphinx Ambassador",
        "Once you’ve chosen a card during your search, you must clearly indicate which card you’ve chosen (preferably without showing the front of that card to the player!). Then the player names a card."
    );
    ruling!(
        "Sphinx Ambassador",
        "You may search for any card, not just a creature card. Of course, if you choose a noncreature card, you won’t be able to put it onto the battlefield no matter what card the player names."
    );
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx Ambassador");
    let ring = t.library_top(P1, "Sol Ring");
    t.answer_choose(P0, &[obj(ring)]);
    t.answer(P1, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.attack(&[(sphinx, pl(P1))], &[]);
    t.resolve_all();
    let order: Vec<&str> = asked_since(&t, from)
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { .. } if *p == P0 => Some("search"),
            Decision::NameCard { .. } if *p == P1 => Some("name"),
            _ => None,
        })
        .collect();
    assert_eq!(order.first(), Some(&"search"), "{order:?}");
    assert!(order.contains(&"name"));
    assert!(t.named_on_battlefield("Sol Ring").is_empty());

    // Draugr Necromancer: an animated land is exiled with an ice counter, but it can't be
    // played from exile.
    ruling!(
        "Draugr Necromancer",
        "Similarly, if a nontoken land an opponent controls that’s become a creature dies, that card will be exiled with an ice counter, but Draugr Necromancer doesn’t allow you to play lands from exile."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Draugr Necromancer");
    let factory = t.battlefield(P1, "Mishra's Factory");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, factory, "becomes").expect("animate");
    t.resolve_all();
    destroy(&mut t, factory);
    let card = t.g.current(factory);
    assert_eq!(t.zone(card), Zone::Exile, "{}", t.dump_log());
    assert_eq!(t.counters(card, "ice"), 1);
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, card));

    // Specter's Shriek with an empty hand; Lobotomy picks one card.
    ruling!(
        "Specter's Shriek",
        "If you have no cards in hand, you won't have to exile anything if you have the player exile a nonblack card from their hand."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let theirs = t.hand(P1, "Grizzly Bears");
    let shriek = t.hand(P0, "Specter's Shriek");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(theirs)]);
    t.cast(P0, shriek).target(pl(P1)).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
    ruling!(
        "Lobotomy",
        "You only get to pick one card from the player’s hand."
    );
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let lob = t.hand(P0, "Lobotomy");
    let from = t.asked().len();
    t.cast(P0, lob).target(pl(P1)).go();
    t.resolve_all();
    let picks: Vec<u32> = asked_since(&t, from)
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { max, .. } if *p == P0 => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(picks.first(), Some(&1), "{picks:?}");

    // Ferropede: its controller chooses which kind of counter.
    ruling!(
        "Ferropede",
        "If there’s more than one type of counter on the targeted permanent, Ferropede’s controller chooses which counter to remove from it."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let fp = t.battlefield(P0, "Ferropede");
    let bears = t.battlefield(P1, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    t.g.add_counters(obj(bears), "shield", 1, None);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    attack_with(&mut t, &[(fp, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    let asked: Vec<PlayerId> = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P0], "{}", t.dump_log());
    let left = t.counters(bears, counters::PLUS1) + t.counters(bears, "shield");
    assert_eq!(left, 1);

    // Gollum with one card in the library: look at it, put it back, carry on.
    ruling!(
        "Gollum, Scheming Guide",
        "If you only have one card in your library as Gollum, Scheming Guide's ability resolves, you'll look at that card, put it back on top of your library, and then continue with the ability as normal."
    );
    let mut t = TestGame::new(2);
    let gollum = t.battlefield(P0, "Gollum, Scheming Guide");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.g.players[0].library.clear();
    t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // land
    t.answer(P1, DecisionKind::Option, Answer::Index(1)); // guesses wrong
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(gollum, pl(P1))]),
    );
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.g.is_attacking(gollum));

    // Rapid Augmenter: Triskelion (base power 1) enters with counters and still gains haste.
    ruling!(
        "Rapid Augmenter",
        "Normally, a creature's base power is the power printed on the card or, for a token, the power set by the effect that created it. If an effect modifies a creature's power without setting it, that is not included when determining its base power."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rapid Augmenter");
    let tri = t.enter(P0, "Triskelion");
    t.resolve_all();
    assert_eq!(t.pt(tri), (4, 4));
    assert!(t.obj_now(tri).has_keyword(KeywordKind::Haste));

    // Volrath's Laboratory: a Bird token without flying, named "Bird Token".
    ruling!(
        "Volrath's Laboratory",
        "The token does not have any built-in abilities. For example, choosing creature type Bird does not give the token Flying."
    );
    ruling!(
        "Volrath's Laboratory",
        "The token has the same name as the creature type plus the word \"Token.\" For example, if the chosen creature type is Bird, the token's name is \"Bird Token.\""
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // blue
    choose_creature_type(&mut t, P0, "Bird");
    let lab = t.enter(P0, "Volrath's Laboratory");
    t.lands(P0, "Wastes", 5);
    t.activate(P0, lab, 0, &[]).expect("{5}, {T}");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Bird Token");
    assert!(o.chars.has_subtype("Bird"));
    assert!(!o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(toks[0]), (2, 2));
}
