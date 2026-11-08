//! Rulings batch P061 — the other "force draw" cards: mana value under cost reductions and
//! X (CR 202.3, 107.3f, 601.2f), new objects (CR 400.7), copies of spells and the tokens
//! they become (CR 707.10, 111.12), cast triggers that resolve first (CR 603.3, 608.2b),
//! values computed on resolution (CR 608.2h), illegal targets (CR 608.2b), tokens leaving
//! the battlefield (CR 111.7, 111.8), searches by name (CR 701.23), "can't be sacrificed"
//! (CR 701.21a), the monarch (CR 724), casting during resolution (CR 608.2g), and
//! reflexive triggers and crimes (CR 603.12, 700.13).

use crate::r_p061_common::*;
use crate::r_s01_common::{tokens, triggers_on_stack};
use crate::r_s02_common::create_token;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::*;

#[test]
fn the_end_always_has_mana_value_four() {
    cr!("202.3", "601.2f", "118.7");
    ruling!(
        "The End",
        "The cost-reduction ability of The End doesn't change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of The End is always 4."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    let bears = t.battlefield(P1, "Grizzly Bears");
    let end = t.hand(P0, "The End");
    assert_eq!(t.g.mana_value_of(end), 4);
    t.lands(P0, "Swamp", 2);
    let spell = t.cast(P0, end).target(bears).go();
    assert_eq!(
        crate::r_s01_common::tapped_lands(&t, P0),
        2,
        "paid {{B}}{{B}}"
    );
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn triad_of_fates_flickered_creature_has_no_fate_counter() {
    cr!("400.7", "122.2");
    ruling!(
        "Triad of Fates",
        "The creature that returns to the battlefield during the resolution of Triad of the Fate's middle ability does so as a new object, with no memory of its previous existence. It won't have a fate counter on it."
    );
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, "Triad of Fates");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let i = ability(&mut t, triad, "{1}");
    t.activate(P0, triad, i, &[o(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "fate"), 1);
    t.g.untap(triad);
    let i = ability(&mut t, triad, "{W}");
    t.activate(P0, triad, i, &[o(bears)]).unwrap();
    t.resolve_all();
    let now = t.g.current(bears);
    assert_ne!(now, bears);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).controller, P1);
    assert_eq!(t.counters(now, "fate"), 0);
}

/// P0 controls Gandalf (and `extra`), each opponent's library has `top` on top; P0 casts
/// Craw Wurm (mana value 6) and everything resolves.
fn gandalf_wurm(players: usize, top: &str, extra: &[&str]) -> TestGame {
    supported("Gandalf, Westward Voyager");
    let mut t = TestGame::new(players);
    t.battlefield(P0, "Gandalf, Westward Voyager");
    for e in extra {
        t.battlefield(P0, e);
    }
    for i in 1..players {
        stack_library(&mut t, PlayerId(i as u8), &[top]);
    }
    cast_resolve(&mut t, P0, "Craw Wurm", &[]);
    t
}

#[test]
fn gandalf_copies_the_spell_once_and_the_token_isnt_created() {
    cr!("707.10", "111.12", "603.2");
    ruling!(
        "Gandalf, Westward Voyager",
        "You copy the spell you cast, not any of the revealed cards."
    );
    ruling!(
        "Gandalf, Westward Voyager",
        "You create only one copy, no matter how many revealed cards share a card type with the spell you cast or how many card types those cards share with the spell you cast."
    );
    // Both opponents reveal Dryad Arbor (a land creature: it shares the creature type).
    let t = gandalf_wurm(3, "Dryad Arbor", &[]);
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 2, "the card and one token");
    assert_eq!(wurms.iter().filter(|w| t.g.obj(**w).is_token()).count(), 1);
    assert!(t.named_on_battlefield("Dryad Arbor").is_empty());
    assert_eq!(
        (t.hand_size(P1), t.hand_size(P2)),
        (1, 1),
        "each opponent draws"
    );

    ruling!(
        "Gandalf, Westward Voyager",
        "The token that a resolving copy of a spell becomes isn't said to have been \"created\" and won't interact with abilities that care about tokens being created, such as that of Rosie Cotton of South Lane."
    );
    // Rosie Cotton: "Whenever you create a token, put a +1/+1 counter on target creature
    // you control other than Rosie Cotton."
    let t = gandalf_wurm(2, "Grizzly Bears", &["Rosie Cotton of South Lane"]);
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 2);
    let gandalf = t.named_on_battlefield("Gandalf, Westward Voyager")[0];
    assert_eq!(t.counters(gandalf, PLUS1), 0);
    assert!(wurms.iter().all(|w| t.counters(*w, PLUS1) == 0));
}

#[test]
fn gandalf_uses_x_for_the_mana_value_on_the_stack() {
    cr!("107.3f", "202.3e", "603.2");
    ruling!(
        "Gandalf, Westward Voyager",
        "While a spell with {X} in its mana cost is on the stack, use the value of X that was chosen when it was cast to determine its mana value."
    );
    // Blaze ({X}{R}): X = 4 is mana value 5 (it triggers); X = 3 is 4 (it doesn't).
    for (x, triggers) in [(4, 1), (3, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Gandalf, Westward Voyager");
        t.lands(P0, "Mountain", 5);
        let blaze = t.hand(P0, "Blaze");
        t.cast(P0, blaze).x(x).target(pl(P1)).go();
        t.settle();
        assert_eq!(triggers_on_stack(&t, "mana value 5"), triggers, "X = {x}");
    }
}

#[test]
fn wedding_ring_counts_any_artifact_named_wedding_ring() {
    cr!("201.2", "603.2");
    ruling!(
        "Wedding Ring",
        "The trigger conditions of Wedding Ring's second and third abilities care about opponents who control any artifact named Wedding Ring, not just the token copy created by its first ability."
    );
    // P1's Wedding Ring is a card, not P0's token copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wedding Ring");
    t.battlefield(P1, "Wedding Ring");
    t.set_step(P1, Step::PrecombatMain);
    let before = libraries(&t);
    cast_resolve(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    // (P1's Ring doesn't trigger for P0's draw: that's not during P0's turn.)
    assert_eq!(drawn_since(&t, &before), vec![2, 2]);
    cast_resolve(&mut t, P1, "Healing Salve", &[pl(P1)]);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "608.2", "702.33d");
    ruling!(
        "Depth Defiler",
        "The triggered ability will resolve before Depth Defiler does. If Depth Defiler is countered or otherwise leaves the stack in response to its triggered ability, the triggered ability will still resolve as normal. If Depth Defiler was kicked, you'll still follow the instructions of both modes."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    t.lands(P0, "Wastes", 1);
    let defiler = t.hand(P0, "Depth Defiler");
    t.answer_targets(P0, &[o(bears)]);
    t.answer_targets(P0, &[pl(P0)]);
    let spell = t.cast(P0, defiler).kicked(true).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast(&mut t, P1, "Cancel", &[o(spell)]);
    let before = libraries(&t);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Depth Defiler"));
    assert!(t.in_hand(P1, "Grizzly Bears"), "first mode");
    assert_eq!(drawn_since(&t, &before)[0], 2, "second mode");

    ruling!(
        "Thief of Existence",
        "Thief of Existence's triggered ability will resolve before Thief of Existence does. If Thief of Existence is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Forest", 1);
    let thief = t.hand(P0, "Thief of Existence");
    t.answer_targets(P0, &[o(anthem)]);
    let spell = t.cast(P0, thief).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast(&mut t, P1, "Cancel", &[o(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Thief of Existence"));
    assert!(t.in_exile("Glorious Anthem"));
}

#[test]
fn communal_brewing_values_on_resolution_and_illegal_targets() {
    cr!("608.2h", "608.2b", "115.10a");
    ruling!(
        "Communal Brewing",
        "The value of X is calculated as Communal Brewing's last ability resolves."
    );
    // One ingredient counter as the creature spell is cast; another added in response.
    let mut t = TestGame::new(2);
    let brewing = t.battlefield(P0, "Communal Brewing");
    put(&mut t, brewing, "ingredient", 1);
    let bears = cast(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    put(&mut t, brewing, "ingredient", 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, PLUS1), 2);

    ruling!(
        "Communal Brewing",
        "You don't have to choose any target opponents for Communal Brewing's first ability. However, if you do, and all of those targets are illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. No players will draw cards, and you won't put any ingredient counters on Communal Brewing."
    );
    // No targets: one ingredient counter; two opponents who each draw: one plus two (the
    // "for each card drawn this way" is only the second instruction's).
    for targets in [vec![], vec![pl(P1), pl(P2)]] {
        let mut t = TestGame::new(3);
        t.answer_targets(P0, &targets);
        let brewing = t.enter(P0, "Communal Brewing");
        let before = libraries(&t);
        t.resolve_all();
        let drawn = drawn_since(&t, &before);
        assert_eq!(drawn.iter().sum::<usize>(), targets.len());
        assert_eq!(t.counters(brewing, "ingredient"), 1 + targets.len() as u32);
    }
    // P1 targeted, then hexproof: nothing.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[pl(P1)]);
    let brewing = t.enter(P0, "Communal Brewing");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_p007_common::player_hexproof(&mut t, P1);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(t.counters(brewing, "ingredient"), 0);
    assert_eq!(drawn_since(&t, &before), vec![0, 0]);
}

#[test]
fn sway_of_the_stars_tokens_leave_and_arent_shuffled() {
    cr!("111.7", "111.8", "704.5d");
    ruling!(
        "Sway of the Stars",
        "Tokens are included in the effect but shouldn’t actually be included in the shuffle. Tokens will cease to exist immediately after the spell resolves."
    );
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P0, "Soldier");
    t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    give_mana_for(&mut t, P0, "Sway of the Stars");
    let sway = t.hand(P0, "Sway of the Stars");
    let owned_permanents =
        t.g.permanents()
            .filter(|o| o.owner == P0 && !o.is_token())
            .count();
    let expected =
        t.library_size(P0) + (t.hand_size(P0) - 1) + t.graveyard_size(P0) + owned_permanents - 7;
    t.cast(P0, sway).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), expected);
    assert!(!t.g.players[0].library.contains(&t.g.current(token)));
    assert!(!t.g.battlefield.contains(&t.g.current(token)));
    assert!(tokens(&t, P0).is_empty());
    assert_eq!((t.hand_size(P0), t.life(P0), t.life(P1)), (7, 7, 7));
}

#[test]
fn deadly_cover_up_can_exile_a_basic_land() {
    cr!("701.23a", "201.2");
    ruling!(
        "Deadly Cover-Up",
        "Unlike many other effects of this kind, Deadly Cover-Up allows its controller to exile a basic land."
    );
    let mut t = TestGame::new(2);
    let dragon = t.graveyard(P0, "Shivan Dragon");
    let gy = t.graveyard(P1, "Forest");
    let hand = t.hand(P1, "Forest");
    let lib = t.library_top(P1, "Forest");
    t.hand(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Deadly Cover-Up");
    let s = t.hand(P0, "Deadly Cover-Up");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[o(dragon)]);
    t.answer_choose(P0, &[o(gy)]);
    t.answer_choose_moved(P0, &[o(hand), o(lib)]);
    t.cast(P0, s).go();
    let before = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.zone(dragon), Zone::Exile);
    for c in [gy, hand, lib] {
        assert_eq!(t.zone(c), Zone::Exile, "{}", t.dump_log());
    }
    assert_eq!(
        t.hand_size(P1),
        before,
        "one Forest exiled from hand, one card drawn"
    );
}

/// P0 activates The Stone Brain targeting P1 (naming `name` as it resolves), choosing
/// `chosen` among the cards found.
fn stone_brain(t: &mut TestGame, name: &str, chosen: &[ObjectId]) -> usize {
    let sb = t.battlefield(P0, "The Stone Brain");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    t.answer_choose(P0, &chosen.iter().map(|c| o(*c)).collect::<Vec<_>>());
    let from = t.asked().len();
    t.activate(P0, sb, 0, &[pl(P1)]).unwrap();
    from
}

#[test]
fn the_stone_brain_names_as_it_resolves() {
    cr!("701.23a", "201.3", "608.2c");
    ruling!(
        "The Stone Brain",
        "You target a player as The Stone Brain's activated ability is put on the stack, but you don't name a card until it resolves. Notably, this means that players don't get to respond to the ability knowing what card you will name."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let from = stone_brain(&mut t, "Grizzly Bears", &[bears]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. })));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. })));
    assert_eq!(t.zone(bears), Zone::Exile);

    ruling!(
        "The Stone Brain",
        "You don't have to choose the name of a card that you've already seen this game; you may guess at a card you think your opponent might have in their deck, if you'd like."
    );
    // Shivan Dragon is hidden in P1's library; nobody has seen it.
    let mut t = TestGame::new(2);
    let dragon = t.library_top(P1, "Shivan Dragon");
    stone_brain(&mut t, "Shivan Dragon", &[dragon]);
    t.resolve_all();
    assert_eq!(t.zone(dragon), Zone::Exile);
    // A name that isn't there at all: nothing is found.
    let mut t = TestGame::new(2);
    stone_brain(&mut t, "Serra Angel", &[]);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

#[test]
fn the_stone_brain_basic_land_names_up_to_four() {
    cr!("701.23a", "701.23d");
    ruling!(
        "The Stone Brain",
        "Unlike most effects of this kind, The Stone Brain's ability allows its controller to choose the name of a basic land. Note that you may only search for up to four copies of that card, even if you find more of them."
    );
    ruling!(
        "The Stone Brain",
        "You don't have to exile each card you find with that name. If you want to leave one in their graveyard, for example, you may."
    );
    // Five Forests: two in P1's graveyard, two in hand, one in library.
    let mut t = TestGame::new(2);
    let g1 = t.graveyard(P1, "Forest");
    let g2 = t.graveyard(P1, "Forest");
    let h1 = t.hand(P1, "Forest");
    let h2 = t.hand(P1, "Forest");
    let l1 = t.library_top(P1, "Forest");
    // Leave one in the graveyard; take the other four.
    let from = stone_brain(&mut t, "Forest", &[g1, h1, h2, l1]);
    t.resolve_all();
    let max = t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities {
                max, candidates, ..
            } if *p == P0 && candidates.contains(&o(g2)) => Some(*max),
            _ => None,
        })
        .expect("search choice");
    assert_eq!(max, 4);
    for c in [g1, h1, h2, l1] {
        assert_eq!(t.zone(c), Zone::Exile);
    }
    assert_eq!(t.zone(g2), Zone::Graveyard(P1));
    // P1 drew two cards for the two from their hand.
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn garland_stolen_creatures_cant_be_sacrificed() {
    cr!("701.21a", "118.3");
    ruling!(
        "Garland, Royal Kidnapper",
        "While Garland is on the battlefield, creatures you control but don't own can't be sacrificed for any reason. If an effect instructs you to sacrifice one of them, you can't and it remains on the battlefield. You also can't sacrifice one of them to pay a cost that requires you to sacrifice a creature."
    );
    // P0 controls Garland and P1's Grizzly Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garland, Royal Kidnapper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, bears, P0);
    assert_eq!(t.pt(bears), (4, 4));
    // A cost: Fallen Angel's "Sacrifice a creature" can't use the Bears.
    let angel = t.battlefield(P0, "Fallen Angel");
    let from = t.asked().len();
    let i = ability(&mut t, angel, "Sacrifice");
    t.answer_choose(P0, &[o(angel)]);
    t.activate(P0, angel, i, &[]).unwrap();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty());
    assert!(
        offered.iter().all(|c| !c.contains(&o(bears))),
        "{offered:?}"
    );
    t.resolve_all();
    // An effect: Innocent Blood ("Each player sacrifices a creature.") — P0 can't
    // sacrifice the Bears, so it sacrifices Garland.
    cast_resolve(&mut t, P0, "Innocent Blood", &[]);
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Garland, Royal Kidnapper"));
}

#[test]
fn garland_gains_nothing_if_the_opponent_is_no_longer_the_monarch() {
    cr!("724.1", "611.2b");
    ruling!(
        "Garland, Royal Kidnapper",
        "While Garland's second ability is on the stack, if that opponent stops being the monarch before it resolves, you won't gain control of the target creature at all."
    );
    for steal_back in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[o(bears)]);
        t.enter(P0, "Garland, Royal Kidnapper");
        t.resolve();
        assert_eq!(t.g.monarch, Some(P1));
        t.settle();
        assert_eq!(t.stack_len(), 1, "the control trigger");
        if steal_back {
            mtg_engine::designations::become_monarch(&mut t.g, P0);
            t.g.flush_events();
        }
        t.resolve_all();
        let controller = t.obj_now(bears).controller;
        assert_eq!(controller, if steal_back { P1 } else { P0 }, "{steal_back}");
    }
}

#[test]
fn chandra_ablazes_ultimate_casts_from_the_graveyard_during_resolution() {
    cr!("608.2g", "601.2", "307.1", "405.5");
    ruling!(
        "Chandra Ablaze",
        "You cast red instant cards and red sorcery cards from your graveyard as part of the resolution of Chandra Ablaze's third ability. You don't choose which ones to cast until you're actually doing so as the ability resolves. You cast only the ones you want to, and you may cast them in any order. Timing restrictions based on the card's type (if it's a sorcery) are ignored. Other restrictions are not (such as \"Cast [this card] only during combat\"). Each card you cast this way is put on the stack, then the ability finishes resolving. Those spells will then resolve as normal, one at a time, in the opposite order that they were put on the stack. They'll go back to the graveyard as they resolve."
    );
    // Lava Spike (a sorcery) is cast while the ability resolves: its timing is ignored.
    let mut t = TestGame::new(2);
    let ch = crate::r_p007_common::pw(&mut t, P0, "Chandra Ablaze", 7);
    let spike = t.graveyard(P0, "Lava Spike");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let hammer = t.graveyard(P0, "Volcanic Hammer");
    let cry = t.graveyard(P0, "Gorilla War Cry");
    // Lava Spike first, then Lightning Bolt (so Bolt resolves first).
    t.answer_choose(P0, &[o(spike), o(bolt)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.asked().len();
    let i = ability(&mut t, ch, "−7");
    t.activate(P0, ch, i, &[]).unwrap();
    t.settle();
    // The choice of cards wasn't made on activation.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    t.g.resolve_top();
    t.g.flush_events();
    // Gorilla War Cry ("Cast this spell only during combat before blockers are
    // declared.") wasn't among the cards offered.
    let offered = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("cards offered");
    assert!(offered.contains(&o(spike)) && offered.contains(&o(bolt)));
    assert!(!offered.contains(&o(cry)), "{offered:?}");
    // Both are on the stack, Bolt on top; Volcanic Hammer wasn't cast.
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Lightning Bolt");
    assert_eq!(t.zone(hammer), Zone::Graveyard(P0));
    t.resolve();
    assert_eq!(t.life(P1), 17, "Bolt first");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P0, "Lava Spike") && t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn unscrupulous_contractor_targets_with_its_reflexive_trigger() {
    cr!("603.12", "700.13");
    ruling!(
        "Unscrupulous Contractor",
        "You don’t choose a target for Unscrupulous Contractor’s ability at the time it triggers. Rather, a second “reflexive” ability triggers if you sacrifice a creature this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal. Notably, the triggered ability isn’t a crime, but the reflexive triggered ability potentially is."
    );
    for target in [P1, P0] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        t.enter(P0, "Unscrupulous Contractor");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
        assert_eq!(t.g.history.crimes.get(&P0).copied().unwrap_or(0), 0);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(bears)]);
        t.answer_targets(P0, &[pl(target)]);
        t.resolve();
        // The reflexive trigger is on the stack, targeting.
        assert_eq!(t.stack_len(), 1);
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        let crimes = t.g.history.crimes.get(&P0).copied().unwrap_or(0);
        assert_eq!(crimes, u32::from(target == P1), "{target}");
        let before = libraries(&t);
        t.resolve_all();
        assert_eq!(drawn_since(&t, &before)[target.idx()], 2);
        assert_eq!(t.life(target), 18);
    }
}
