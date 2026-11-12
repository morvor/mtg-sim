//! Rulings batch P157 — cards an opponent gets to take or choose from (Allure of the
//! Unknown, Dubious Challenge, Fervent Mastery), putting cards onto the battlefield
//! without casting them (See the Unwritten, Court of Bounty), choices players make in turn
//! order (CR 101.4: Regna's Sanction, Mythos of Snapdax, Court of Ambition), and other
//! gifts to opponents (Clackbridge Troll, Sibilant Spirit, Hunted Dragon, Benthic
//! Explorers).

use crate::r_p157_common::*;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The players asked a non-targeted choice since `from`, in order (consecutive repeats
/// merged).
fn choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    let mut out: Vec<PlayerId> = vec![];
    for (p, d) in &t.asked()[from..] {
        if matches!(d, Decision::ChooseEntities { .. }) && out.last() != Some(p) {
            out.push(*p);
        }
    }
    out
}

#[test]
fn allure_of_the_unknown_six_lands_nothing_exiled() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Allure of the Unknown",
        "If all six of the revealed cards are land cards, your opponent doesn’t exile anything. You put all six cards into your hand."
    );
    supported("Allure of the Unknown");
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.library_top(P0, "Forest");
    }
    let hand = t.hand_size(P0);
    cast_at(&mut t, P0, "Allure of the Unknown", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 6);
    assert!(t.g.exile.is_empty());
}

#[test]
fn allure_of_the_unknown_opponent_casts_now_ignoring_timing() {
    cr!("608.2g", "601.2", "118.9");
    ruling!(
        "Allure of the Unknown",
        "If your opponent wishes to cast the card, they do so as part of the resolution of Allure of the Unknown. They can’t wait to cast it later. Timing permissions based on a card’s type are ignored."
    );
    // P1 casts Grizzly Bears (a creature) during P0's turn, as Allure resolves.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Forest");
    }
    let bears = t.library_top(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Allure of the Unknown", &[]);
    t.answer_choose_moved(P1, &[obj(bears)]);
    t.answer_yes(P1, true);
    t.resolve();
    assert!(t.g.stack.iter().any(|s| t.g.obj(*s).chars.name == "Grizzly Bears"));
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(ctl(&t, b[0]), P1);
    // Declined: it stays in exile and can't be cast later.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Forest");
    }
    let bears = t.library_top(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Allure of the Unknown", &[]);
    t.answer_choose_moved(P1, &[obj(bears)]);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let exiled = t.g.exile[0];
    assert!(legal_cast_methods(&mut t, P1, exiled).is_empty());
}

#[test]
fn allure_of_the_unknown_without_paying_additional_costs_still_apply() {
    cr!("118.9a", "118.9b", "118.9d", "601.2b");
    ruling!(
        "Allure of the Unknown",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Thrill of Possibility, those must be paid to cast the card."
    );
    // Thrill of Possibility: P1 must discard a card to cast it.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Forest");
    }
    let thrill = t.library_top(P0, "Thrill of Possibility");
    let island = t.hand(P1, "Island");
    cast_at(&mut t, P0, "Allure of the Unknown", &[]);
    t.answer_choose_moved(P1, &[obj(thrill)]);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(island)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Island"), "the additional cost was paid");
    assert!(t.in_graveyard(P0, "Thrill of Possibility"));
    assert_eq!(t.hand_size(P1), 2);
    // Gift of Growth: P1 may pay its kicker.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Forest");
    }
    let gift = t.library_top(P0, "Gift of Growth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Allure of the Unknown", &[]);
    lots(&mut t, P1);
    t.answer_choose_moved(P1, &[obj(gift)]);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P1, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6), "kicked");
    // Fervent Mastery: its alternative cost isn't paid, so the opponent doesn't discard.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Forest");
    }
    let fm = t.library_top(P0, "Fervent Mastery");
    cast_at(&mut t, P0, "Allure of the Unknown", &[]);
    let m = mark(&t);
    t.answer_choose_moved(P1, &[obj(fm)]);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fervent Mastery"));
    let alt = t.asked()[m..].iter().any(|(p, d)| {
        *p == P1 && matches!(d, Decision::ChooseCastingMethod { .. } | Decision::OptionalCost { .. })
    });
    assert!(!alt, "no alternative cost offered");
    assert!(
        t.asked()[m..]
            .iter()
            .all(|(p, d)| !(*p == P0 && matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("iscard")))),
        "P0 wasn't made to choose discards"
    );
}

#[test]
fn dubious_challenge_choices() {
    cr!("608.2c", "406.3", "701.24a", "115.1");
    ruling!(
        "Dubious Challenge",
        "In a multiplayer game, you must choose the target opponent before you look at your top ten cards and decide what to exile."
    );
    ruling!(
        "Dubious Challenge",
        "The cards that you look at but don't exile never leave your library. They're shuffled along with the rest of your library."
    );
    ruling!(
        "Dubious Challenge",
        "The creature cards you exile are face up. Your opponent can see them while deciding whether to take one and which to take."
    );
    ruling!(
        "Dubious Challenge",
        "You may choose to exile only one creature card. If you do, the target opponent chooses who gets the card."
    );
    supported("Dubious Challenge");
    // Three players: P0 targets P2 as it's cast; two creatures exiled; P2 takes one.
    let mut t = TestGame::new(3);
    let a = t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Hill Giant");
    for _ in 0..8 {
        t.library_top(P0, "Forest");
    }
    let lib = t.library_size(P0);
    let spell = cast_at(&mut t, P0, "Dubious Challenge", &[pl(P2)]);
    assert_eq!(stack_targets(&t, spell), vec![pl(P2)]);
    t.answer_choose_moved(P0, &[obj(a), obj(b)]);
    // When P2 chooses, the exiled cards are face up.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.exile.iter().all(|e| !g.obj(*e).face_down) && g.exile.len() == 2,
    );
    t.answer_choose_moved(P2, &[obj(b)]);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![true]);
    assert_eq!(t.library_size(P0), lib - 2, "the rest never left the library");
    assert!(t.g.turn_events[from..].iter().any(|e| matches!(e,
        mtg_engine::events::Event::Shuffled { player } if *player == P0)));
    let giant = t.named_on_battlefield("Hill Giant");
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!((giant.len(), bears.len()), (1, 1));
    assert_eq!(ctl(&t, giant[0]), P2);
    assert_eq!(ctl(&t, bears[0]), P0);
    // Only one exiled: P1 chooses whether to take it.
    for take in [true, false] {
        let mut t = TestGame::new(2);
        let a = t.library_top(P0, "Grizzly Bears");
        for _ in 0..9 {
            t.library_top(P0, "Forest");
        }
        cast_at(&mut t, P0, "Dubious Challenge", &[pl(P1)]);
        t.answer_choose_moved(P0, &[obj(a)]);
        if take {
            t.answer_choose_moved(P1, &[obj(a)]);
        } else {
            t.answer_choose(P1, &[]);
        }
        let m = mark(&t);
        t.resolve_all();
        assert!(choosers(&t, m).contains(&P1), "P1 chooses");
        let bears = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(bears.len(), 1);
        assert_eq!(ctl(&t, bears[0]), if take { P1 } else { P0 });
    }
}

#[test]
fn dubious_challenge_may_exile_nothing() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Dubious Challenge",
        "You may choose to exile no creature cards, even if there are one or more among the top ten cards of your library."
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    cast_at(&mut t, P0, "Dubious Challenge", &[pl(P1)]);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(t.g.exile.is_empty());
    assert!(!named_here(&t, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn fervent_mastery_opponent_may_discard_nothing() {
    cr!("608.2d", "118.9", "701.9a");
    ruling!(
        "Fervent Mastery",
        "If the {2}{R}{R} cost was paid, the opponent chooses how many and which cards to discard as Fervent Mastery resolves. They may choose to discard no cards."
    );
    supported("Fervent Mastery");
    for chosen in [0usize, 2] {
        let mut t = TestGame::new(2);
        let hand: Vec<_> = (0..3).map(|_| t.hand(P1, "Island")).collect();
        lots(&mut t, P0);
        let fm = t.hand(P0, "Fervent Mastery");
        let method = legal_cast_methods(&mut t, P0, fm)
            .into_iter()
            .find(|m| *m != CastMethod::Normal)
            .expect("the alternative cost");
        t.cast(P0, fm).method(method).go();
        t.answer_choose(
            P1,
            &hand[..chosen].iter().map(|c| obj(*c)).collect::<Vec<_>>(),
        );
        let m = mark(&t);
        t.resolve_all();
        // P1 chooses, on resolution, any number of the cards in their hand (none too).
        let first = t.asked()[m..].iter().find_map(|(p, d)| match d {
            Decision::ChooseEntities {
                candidates, min, ..
            } if *p == P1 => Some((candidates.len(), *min)),
            _ => None,
        });
        assert_eq!(first, Some((3, 0)));
        assert_eq!(t.graveyard_size(P1), chosen);
        assert_eq!(t.hand_size(P1), 3, "discarded {chosen}, drew {chosen}");
    }
}

#[test]
fn see_the_unwritten_puts_creatures_without_casting() {
    cr!("601.2b", "702.37c", "702.33d");
    ruling!(
        "See the Unwritten",
        "Because you’re not casting the creature cards, you can’t pay any additional or alternative costs. You can’t put the cards onto the battlefield face down."
    );
    supported("See the Unwritten");
    let mut t = TestGame::new(2);
    let kavu = t.library_top(P0, "Kavu Titan");
    for _ in 0..7 {
        t.library_top(P0, "Forest");
    }
    let m = mark(&t);
    t.answer_choose_moved(P0, &[obj(kavu)]);
    cast_at(&mut t, P0, "See the Unwritten", &[]);
    t.resolve_all();
    let k = t.named_on_battlefield("Kavu Titan");
    assert_eq!(k.len(), 1);
    assert_eq!(t.counters(k[0], "+1/+1"), 0, "not kicked");
    assert!(!t.obj_now(k[0]).has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    assert!(t.asked()[m..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::OptionalCost { .. })));
    // A morph creature enters face up.
    let mut t = TestGame::new(2);
    let ainok = t.library_top(P0, "Ainok Tracker");
    for _ in 0..7 {
        t.library_top(P0, "Forest");
    }
    t.answer_choose_moved(P0, &[obj(ainok)]);
    cast_at(&mut t, P0, "See the Unwritten", &[]);
    t.resolve_all();
    let a = t.named_on_battlefield("Ainok Tracker");
    assert_eq!(a.len(), 1);
    assert!(!t.obj_now(a[0]).face_down);
}

#[test]
fn court_of_bounty_land_isnt_your_land_play() {
    cr!("305.2a", "305.4");
    ruling!(
        "Court of Bounty",
        "Court of Bounty's ability isn't the same as playing a land. You may still play a land for the turn if you put a land onto the battlefield with it."
    );
    supported("Court of Bounty");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Bounty");
    // P1 is the monarch: P0 may put a land only.
    t.g.monarch = Some(P1);
    t.set_step(P1, Step::End);
    let f = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    t.answer_choose(P0, &[obj(f)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(f));
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, f2).expect("still a land to play");
    assert!(t.on_battlefield(f2));
}

#[test]
fn court_of_ambition_opponents_choose_in_turn_order_then_discard_together() {
    cr!("101.4", "101.4a", "608.2e", "701.9a");
    ruling!(
        "Court of Ambition",
        "As Court of Ambition's second ability resolves, first the next opponent in turn order chooses whether they are discarding or not. If they are, they choose a card without revealing it (or two cards without revealing them, if you're the monarch), then each other opponent in turn order does the same. Then each chosen card is discarded simultaneously, and finally each player who couldn't or didn't discard loses the appropriate amount of life."
    );
    supported("Court of Ambition");
    // P1's Court, at P1's upkeep (P0 is the monarch): the opponents in turn order are P2,
    // then P0. P2 discards, P0 doesn't.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Court of Ambition");
    t.g.monarch = Some(P0);
    t.set_step(P0, Step::End);
    let c2 = t.hand(P2, "Island");
    t.hand(P0, "Island");
    t.answer_yes(P2, true);
    t.answer_choose(P2, &[obj(c2)]);
    t.answer_yes(P0, false);
    // When P0 decides, P2's chosen card hasn't been discarded yet.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.player(P2).hand.len(),
    );
    let m = mark(&t);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let mut order: Vec<PlayerId> = vec![];
    for (p, d) in &t.asked()[m..] {
        if *p != P1
            && matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. })
            && order.last() != Some(p)
        {
            order.push(*p);
        }
    }
    assert_eq!(order, vec![P2, P0]);
    assert_eq!(*seen.lock().unwrap(), vec![1], "nothing discarded before P0 decided");
    assert!(t.in_graveyard(P2, "Island"));
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(t.life(P2), 20);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn regnas_sanction_foes_choose_in_turn_order_then_tap_together() {
    cr!("101.4", "101.4b", "608.2e");
    ruling!(
        "Regna's Sanction",
        "When Regna’s Sanction resolves and your foes perform their actions, first the next foe in turn order (or, if it’s a foe’s turn, that foe) chooses an untapped creature they control, then each other foe in turn order does the same, then all other creatures those players control are tapped at the same time. Foes will know choices made by earlier players when making their choices."
    );
    supported("Regna's Sanction");
    // P1 casts it in P1's turn, choosing foe for everyone (P1 controls no creature): the
    // foes choose in turn order from P1, P2 and then P0.
    let mut t = TestGame::new(3);
    let a1 = t.battlefield(P2, "Grizzly Bears");
    let a2 = t.battlefield(P2, "Hill Giant");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    for _ in 0..3 {
        t.answer(P1, DecisionKind::Option, Answer::Index(1));
    }
    t.answer_choose(P2, &[obj(a2)]);
    t.answer_choose(P0, &[obj(b1)]);
    // When P0 chooses, P2's choice is known and nothing has been tapped yet.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            let tapped = g.battlefield.iter().filter(|o| g.obj(**o).tapped).count();
            let known = g.known_apnap_choices(P0);
            (tapped, known.iter().filter(|(p, c)| *p == P2 && c.is_some()).count())
        },
    );
    let m = mark(&t);
    cast_at(&mut t, P1, "Regna's Sanction", &[]);
    t.resolve_all();
    let order: Vec<PlayerId> = choosers(&t, m)
        .into_iter()
        .filter(|p| *p != P1)
        .collect();
    assert_eq!(order, vec![P2, P0]);
    let seen = seen.lock().unwrap();
    assert_eq!(seen.first().map(|s| s.0), Some(0), "tapped together, after the choices");
    assert!(seen.first().is_some_and(|s| s.1 > 0), "P2's choice is known");
    assert!(tapped(&t, a1) && !tapped(&t, a2));
    assert!(!tapped(&t, b1) && tapped(&t, b2));
}

#[test]
fn mythos_of_snapdax_choices_in_turn_order_then_sacrifice_together() {
    cr!("101.4", "608.2e", "701.21a");
    ruling!(
        "Mythos of Snapdax",
        "As Mythos of Snapdax resolves, first you choose which permanents you control won’t be sacrificed, then each other player in turn order does the same, knowing the choices made before them. Then each nonland permanent not chosen is sacrificed at the same time."
    );
    supported("Mythos of Snapdax");
    // P1 casts it in P1's turn: P1 chooses, then P2, then P0.
    let mut t = TestGame::new(3);
    let p1 = [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Hill Giant"),
    ];
    let p2 = [
        t.battlefield(P2, "Grizzly Bears"),
        t.battlefield(P2, "Hill Giant"),
    ];
    let mine = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Hill Giant"),
    ];
    t.battlefield(P0, "Blood Artist");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_choose(P1, &[obj(p1[1])]);
    t.answer_choose(P2, &[obj(p2[0])]);
    t.answer_choose(P0, &[obj(mine[0])]);
    // When P0 chooses, nothing has been sacrificed yet.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    let m = mark(&t);
    cast_at(&mut t, P1, "Mythos of Snapdax", &[]);
    t.resolve();
    assert_eq!(choosers(&t, m), vec![P1, P2, P0]);
    assert_eq!(*seen.lock().unwrap(), vec![7], "nothing sacrificed before P0 chose");
    assert!(!t.on_battlefield(p1[0]) && t.on_battlefield(p1[1]));
    assert!(t.on_battlefield(p2[0]) && !t.on_battlefield(p2[1]));
    assert!(t.on_battlefield(mine[0]) && !t.on_battlefield(mine[1]));
    // All at once: Blood Artist (sacrificed too) sees every other creature die.
    assert_eq!(triggers_on_stack(&t, "dies"), 4);
}

#[test]
fn clackbridge_troll_other_sacrifices_dont_count() {
    cr!("603.2", "608.2c");
    ruling!(
        "Clackbridge Troll",
        "If a player sacrifices a creature for any other reason (perhaps because it's a Food and the player was hungry), Clackbridge Troll's ability won't tap it or cause you to gain life or draw a card."
    );
    supported("Clackbridge Troll");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[pl(P1)]);
    let troll = t.enter(P0, "Clackbridge Troll");
    t.resolve_all();
    let goats = with_subtype(&t, P1, "Goat");
    assert_eq!(goats.len(), 3);
    // P1 sacrifices a Goat to its Witch's Oven.
    let oven = t.battlefield(P1, "Witch's Oven");
    t.answer_choose(P1, &[obj(goats[0])]);
    act(&mut t, P1, oven, 0, &[]);
    assert!(!t.on_battlefield(goats[0]));
    assert!(!tapped(&t, troll));
    assert_eq!(t.life(P0), 20);
    // At the beginning of combat, a sacrifice to the Troll's ability does.
    let hand = t.hand_size(P0);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(goats[1])]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(goats[1]));
    assert!(tapped(&t, troll));
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn sibilant_spirit_defending_player_may_decline() {
    cr!("603.5", "508.1");
    ruling!(
        "Sibilant Spirit",
        "Your opponent does not have to draw a card if they don’t want to."
    );
    supported("Sibilant Spirit");
    for draw in [false, true] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Sibilant Spirit");
        let hand = t.hand_size(P1);
        t.answer_yes(P1, draw);
        t.set_step(P0, Step::BeginningOfCombat);
        crate::r_s01_common::attack_with(&mut t, &[(s, pl(P1))]);
        t.resolve_all();
        assert_eq!(t.hand_size(P1), hand + draw as usize);
    }
}

#[test]
fn hunted_dragon_knights_are_white_first_strike_knights() {
    cr!("111.4", "111.3");
    ruling!(
        "Hunted Dragon",
        "The tokens are the color white and have name and creature type Knight. They are *not* functionally identical to the card White Knight."
    );
    supported("Hunted Dragon");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Hunted Dragon");
    t.resolve_all();
    let k = tokens(&t, P1);
    assert_eq!(k.len(), 3);
    for id in k {
        let o = t.obj_now(id);
        assert_eq!(o.chars.name, "Knight Token");
        assert!(o.chars.has_subtype("Knight"));
        assert_eq!(
            o.chars.colors,
            mtg_engine::types::ColorSet::single(mtg_engine::types::Color::White)
        );
        assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
        // White Knight has protection from black: the tokens don't.
        assert!(!o.has_keyword(mtg_engine::keywords::KeywordKind::Protection));
        assert_eq!(t.pt(id), (2, 2));
    }
}

#[test]
fn benthic_explorers_counts_replacement_effects_on_the_lands_mana() {
    cr!("106.7", "106.12b");
    ruling!(
        "Benthic Explorers",
        "When determining what types of mana an opponent's land could produce, take into account any applicable replacement effects that would apply to that land's mana abilities (such as Contamination's effect, for example). If there is more than one, consider them in any possible order."
    );
    supported("Contamination");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Benthic Explorers");
    let forest = t.battlefield(P1, "Forest");
    tap(&mut t, forest);
    t.battlefield(P1, "Contamination");
    t.answer_choose(P0, &[obj(forest)]);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let uid = t.obj_now(ex).chars.abilities.iter().find(|a| a.is_mana_ability()).unwrap().uid;
    t.g.activate_ability(P0, ex, uid).expect("activated");
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.total(), 1);
    assert_eq!(
        pool.count(mtg_engine::mana::ManaType::B),
        1,
        "Contamination: only {{B}}"
    );
}
