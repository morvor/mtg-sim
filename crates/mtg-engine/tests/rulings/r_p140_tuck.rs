//! Rulings batch P140 — putting permanents and spells into their owners' libraries:
//! Vedalken Dismisser, Void Stalker, Aether Gust, Rebuking Ceremony, Hunting Drake,
//! Isolation at Orthanc, Lost to Legend, Oust, Quarry Colossus, Dramatic Accusation,
//! Primal Command, Trial of a Time Lord, Shattered Ego, Desynchronize, Unlucky Drop and
//! the charms.

use crate::r_p140_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

#[test]
fn a_lone_dismisser_or_colossus_must_target_itself() {
    cr!("603.3d", "601.2c");
    ruling!(
        "Vedalken Dismisser",
        "If Vedalken Dismisser is the only creature on the battlefield when its ability triggers, you’ll have to choose it as the target."
    );
    ruling!(
        "Quarry Colossus",
        "Quarry Colossus's ability is mandatory. If you control the only creatures when that ability triggers, you must choose a creature you control (which could be Quarry Colossus itself) as the target."
    );
    for name in ["Vedalken Dismisser", "Quarry Colossus"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Island");
        let from = t.asked().len();
        let c = t.enter(P0, name);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}: the trigger isn't removed");
        let offered = target_candidates(&t, P0, from);
        assert_eq!(offered, vec![vec![obj(c)]], "{name}: only itself to target");
        t.resolve_all();
        assert_eq!(zone_of(&t, c), Zone::Library(P0), "{name}");
    }
}

#[test]
fn void_stalker_gone_still_shuffles_the_target_away_and_stays_where_it_is() {
    cr!("608.2b", "701.24c");
    ruling!(
        "Void Stalker",
        "If Void Stalker leaves the battlefield before its ability resolves, the target creature will be shuffled into its owner’s library but Void Stalker will remain wherever it is."
    );
    supported("Void Stalker");
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Void Stalker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate_lots(&mut t, P0, vs, 0, &[obj(bears)]);
    move_to(&mut t, vs, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(zone_of(&t, bears), Zone::Library(P1));
    assert_eq!(zone_of(&t, vs), Zone::Hand(P0), "the Stalker stays in hand");
    assert!(shuffles(&t).contains(&P1));
}

#[test]
fn void_stalker_with_an_illegal_target_does_nothing_at_all() {
    cr!("608.2b");
    ruling!(
        "Void Stalker",
        "If the target creature is an illegal target when the ability tries to resolve, it won’t resolve and none of its effects will happen. No creature will leave the battlefield and no library will be shuffled."
    );
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Void Stalker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate_lots(&mut t, P0, vs, 0, &[obj(bears)]);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(vs)), "the Stalker stays");
    assert_eq!(zone_of(&t, bears), Zone::Hand(P1));
    assert!(shuffles(&t).is_empty(), "no library is shuffled");
}

#[test]
fn void_stalker_targeting_itself_only_you_shuffle() {
    cr!("701.24a");
    ruling!(
        "Void Stalker",
        "You can activate Void Stalker’s ability targeting a creature you control, including Void Stalker itself. In that case, only you will shuffle your library."
    );
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Void Stalker");
    t.battlefield(P1, "Grizzly Bears");
    activate_lots(&mut t, P0, vs, 0, &[obj(vs)]);
    t.resolve_all();
    assert_eq!(zone_of(&t, vs), Zone::Library(P0));
    let s = shuffles(&t);
    assert!(s.contains(&P0) && !s.contains(&P1), "{s:?}");
    // Another creature of yours: both go into your library, and only you shuffle.
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Void Stalker");
    let mine = t.battlefield(P0, "Grizzly Bears");
    activate_lots(&mut t, P0, vs, 0, &[obj(mine)]);
    t.resolve_all();
    assert_eq!(zone_of(&t, vs), Zone::Library(P0));
    assert_eq!(zone_of(&t, mine), Zone::Library(P0));
    assert!(!shuffles(&t).contains(&P1));
}

#[test]
fn spells_put_into_or_returned_from_the_stack_are_not_countered() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Aether Gust",
        "If a spell is put into its owner’s library, it’s removed from the stack and thus will not resolve. The spell isn’t countered; it no longer exists. This works against a spell that can’t be countered."
    );
    ruling!(
        "Reprieve",
        "If a spell is returned to its owner's hand, it's removed from the stack and thus will not resolve. This works against a spell that can't be countered."
    );
    // Aether Gust: Carnage Tyrant (green, can't be countered) goes into its owner's library.
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let tyrant = cast_lots(&mut t, P1, "Carnage Tyrant", &[]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    cast_lots(&mut t, P0, "Aether Gust", &[obj(tyrant)]);
    t.resolve_all();
    assert_eq!(zone_of(&t, tyrant), Zone::Library(P1));
    assert!(t.named_on_battlefield("Carnage Tyrant").is_empty());
    assert!(!anything_countered(&t));
    // Reprieve: Abrupt Decay (can't be countered) returns to its owner's hand; you draw.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let decay = cast_lots(&mut t, P1, "Abrupt Decay", &[obj(bears)]);
    let hand = t.hand_size(P0);
    cast_lots(&mut t, P0, "Reprieve", &[obj(decay)]);
    t.resolve_all();
    assert_eq!(zone_of(&t, decay), Zone::Hand(P1));
    assert!(t.on_battlefield(bears), "Abrupt Decay never resolved");
    assert_eq!(t.hand_size(P0), hand + 1, "Reprieve drew a card");
    assert!(!anything_countered(&t));
}

#[test]
fn the_owner_chooses_top_or_bottom_for_aether_gust_desynchronize_and_unlucky_drop() {
    cr!("608.2d", "400.3");
    ruling!(
        "Aether Gust",
        "The owner of the target spell or permanent chooses to put it either on the top or bottom of their library."
    );
    ruling!(
        "Desynchronize",
        "The permanent's owner chooses whether to put it on the top or bottom of their library."
    );
    ruling!(
        "Unlucky Drop",
        "The permanent's owner chooses whether to put it on the top or bottom of their library."
    );
    // P0 controls a creature P1 owns: P1 (the owner), not P0, makes the choice.
    for name in ["Aether Gust", "Desynchronize", "Unlucky Drop"] {
        for bottom in [false, true] {
            let mut t = TestGame::new(2);
            let ape = t.battlefield(P1, "Raging Goblin");
            give_control(&mut t, ape, P0);
            assert_eq!(t.obj(ape).controller, P0);
            t.answer(P1, DecisionKind::Option, Answer::Index(bottom as usize));
            // P0's own answer would be the opposite, were P0 asked.
            t.answer(P0, DecisionKind::Option, Answer::Index(!bottom as usize));
            let from = t.asked().len();
            cast_lots(&mut t, P0, name, &[obj(ape)]);
            t.resolve_all();
            let pos = from_top(&t, P1, ape).expect("in its owner's library");
            let len = t.library_size(P1);
            assert_eq!(pos, if bottom { len - 1 } else { 0 }, "{name} bottom={bottom}");
            assert!(
                !asked_since(&t, from)
                    .iter()
                    .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { .. })),
                "{name}: the controller isn't asked"
            );
        }
    }
}

#[test]
fn a_melded_permanent_goes_all_on_top_or_all_on_the_bottom() {
    cr!("712.21a", "401.4");
    ruling!(
        "Desynchronize",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that permanent's owner puts all the cards on top or all the cards on the bottom."
    );
    ruling!(
        "Unlucky Drop",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that permanent's owner puts all the cards on top or all the cards on the bottom."
    );
    for name in ["Desynchronize", "Unlucky Drop"] {
        for bottom in [false, true] {
            let mut t = TestGame::new(2);
            let rats = t.exile(P1, "Graf Rats");
            let scav = t.exile(P1, "Midnight Scavengers");
            let host =
                mtg_engine::merge::meld(&mut t.g, rats, scav, card("Chittering Host"), P1)
                    .expect("meld");
            t.g.recompute();
            t.answer(P1, DecisionKind::Option, Answer::Index(bottom as usize));
            cast_lots(&mut t, P0, name, &[obj(host)]);
            t.resolve_all();
            let len = t.library_size(P1);
            let mut pos: Vec<usize> = t
                .g
                .player(P1)
                .library
                .iter()
                .rev()
                .enumerate()
                .filter(|(_, c)| {
                    let n = t.obj(**c).chars.name.as_str();
                    n == "Graf Rats" || n == "Midnight Scavengers"
                })
                .map(|(i, _)| i)
                .collect();
            pos.sort();
            let want = if bottom { vec![len - 2, len - 1] } else { vec![0, 1] };
            assert_eq!(pos, want, "{name} bottom={bottom}");
        }
    }
}

#[test]
fn rebuking_ceremony_the_owner_orders_two_cards_of_theirs() {
    cr!("401.4");
    ruling!(
        "Rebuking Ceremony",
        "If both cards are owned by the same player, that player decides their order on top of their library."
    );
    supported("Rebuking Ceremony");
    // The two answers put the cards in opposite orders.
    let mut tops = vec![];
    for flip in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Ornithopter");
        let b = t.battlefield(P1, "Millstone");
        t.answer(
            P1,
            DecisionKind::Order,
            Answer::Indices(if flip { vec![1, 0] } else { vec![0, 1] }),
        );
        let from = t.asked().len();
        cast_lots(&mut t, P0, "Rebuking Ceremony", &[obj(a), obj(b)]);
        t.resolve_all();
        let (pa, pb) = (from_top(&t, P1, a).unwrap(), from_top(&t, P1, b).unwrap());
        assert!(pa <= 1 && pb <= 1, "both on top");
        let orders: Vec<PlayerId> = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Order { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(orders, vec![P1], "the owner orders them");
        let top = *t.g.player(P1).library.last().unwrap();
        tops.push(t.obj(top).chars.name.to_string());
    }
    assert_ne!(tops[0], tops[1], "{tops:?}");
}

#[test]
fn hunting_drake_works_if_the_target_turns_from_red_to_green() {
    cr!("608.2b", "105.3");
    ruling!(
        "Hunting Drake",
        "If the target changes color from red to green or green to red between triggering and resolution, the ability still works."
    );
    supported("Hunting Drake");
    for (to, works) in [(Color::Green, true), (Color::Blue, false)] {
        let mut t = TestGame::new(2);
        let goblin = t.battlefield(P1, "Raging Goblin");
        t.answer_targets(P0, &[obj(goblin)]);
        t.enter(P0, "Hunting Drake");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        modify(
            &mut t,
            goblin,
            vec![mtg_engine::ability::Modification::SetColors(ColorSet::single(to))],
        );
        t.resolve_all();
        let z = zone_of(&t, goblin);
        if works {
            assert_eq!(z, Zone::Library(P1), "now green: still a legal target");
        } else {
            assert_eq!(z, Zone::Battlefield, "now blue: an illegal target");
        }
    }
}

#[test]
fn nth_from_the_top_of_a_short_library_is_the_bottom() {
    cr!("401.7");
    ruling!(
        "Isolation at Orthanc",
        "If the target creature's controller has zero cards left in their library when Isolation at Orthanc resolves, that creature becomes the top card of their library."
    );
    ruling!(
        "Lost to Legend",
        "If the target permanent's controller has two or fewer cards left in their library when Lost to Legend resolves, that permanent becomes the bottom card of their library."
    );
    ruling!(
        "Quarry Colossus",
        "If there are fewer than X cards in that player's library, put the creature on the bottom of that library."
    );
    // Isolation at Orthanc into an empty library: the only (top) card.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    empty_library(&mut t, P1);
    cast_lots(&mut t, P0, "Isolation at Orthanc", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.g.player(P1).library, vec![t.g.current(bears)]);
    // Lost to Legend: two cards left (bottom) and five (fourth from the top).
    for (n, want) in [(2usize, 2usize), (5, 3)] {
        let mut t = TestGame::new(2);
        let legend = t.battlefield(P1, "Isamaru, Hound of Konda");
        library_of(&mut t, P1, n);
        cast_lots(&mut t, P0, "Lost to Legend", &[obj(legend)]);
        t.resolve_all();
        assert_eq!(from_top(&t, P1, legend), Some(want), "{n} cards");
    }
    // Quarry Colossus with three Plains: one card left (bottom), five (fourth).
    for (n, want) in [(1usize, 1usize), (5, 3)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        library_of(&mut t, P1, n);
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Quarry Colossus");
        t.resolve_all();
        assert_eq!(from_top(&t, P1, bears), Some(want), "{n} cards");
    }
}

#[test]
fn oust_a_token_ceases_to_exist_its_controller_gains_the_life() {
    cr!("111.7", "704.5d");
    ruling!(
        "Oust",
        "If the targeted creature is a token, it will cease to exist after it’s put into its owner’s library."
    );
    ruling!(
        "Oust",
        "Note that the creature’s controller, not its owner, is the one who gains life."
    );
    supported("Oust");
    // A token.
    let mut t = TestGame::new(2);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let lib = t.library_size(P1);
    cast_lots(&mut t, P0, "Oust", &[obj(token)]);
    t.resolve_all();
    assert!(tokens_of(&t, P1).is_empty());
    assert_eq!(t.library_size(P1), lib, "the token isn't in the library");
    assert_eq!(t.life(P1), 23);
    // A creature P0 owns that P1 controls: into P0's library; P1 gains 3 life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    cast_lots(&mut t, P0, "Oust", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(from_top(&t, P0, bears), Some(1), "second from the top of P0's");
    assert_eq!((t.life(P0), t.life(P1)), (20, 23));
}

#[test]
fn oust_with_an_illegal_target_gains_no_one_life() {
    cr!("608.2b");
    ruling!(
        "Oust",
        "If the targeted creature is an illegal target by the time Oust resolves, the spell doesn’t resolve. No one gains life."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_lots(&mut t, P0, "Oust", &[obj(bears)]);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(zone_of(&t, bears), Zone::Hand(P1));
}

#[test]
fn only_dramatic_accusations_controller_may_activate_it() {
    cr!("602.2");
    ruling!(
        "Dramatic Accusation",
        "Only the controller of Dramatic Accusation may activate its ability."
    );
    supported("Dramatic Accusation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = cast_lots(&mut t, P0, "Dramatic Accusation", &[obj(bears)]);
    t.resolve_all();
    let aura = t.g.current(aura);
    assert!(t.on_battlefield(aura));
    lots(&mut t, P1);
    assert!(!can_activate(&mut t, P1, aura), "the enchanted creature's controller can't");
    lots(&mut t, P0);
    assert!(can_activate(&mut t, P0, aura), "the Aura's controller can");
    activate_lots(&mut t, P0, aura, 0, &[]);
    t.resolve_all();
    assert_eq!(zone_of(&t, bears), Zone::Library(P1));
}

#[test]
fn shattered_ego_tucks_the_creature_even_if_it_is_no_longer_attached() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Shattered Ego",
        "The creature will be put into its owner's library even if Shattered Ego is no longer attached to it as the ability resolves."
    );
    supported("Shattered Ego");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ego = cast_lots(&mut t, P0, "Shattered Ego", &[obj(bears)]);
    t.resolve_all();
    let ego = t.g.current(ego);
    activate_lots(&mut t, P0, ego, 0, &[]);
    destroy(&mut t, ego);
    assert!(!t.on_battlefield(t.g.current(ego)));
    t.resolve_all();
    assert_eq!(from_top(&t, P1, bears), Some(2), "third from the top");
}

#[test]
fn primal_command_is_not_shuffled_in_by_its_own_third_mode() {
    cr!("608.2n", "608.2c");
    ruling!(
        "Primal Command",
        "Primal Command won't be put into your graveyard until after it's finished resolving, which means it won't be shuffled into your library as part of its own effect if you target yourself with its third mode."
    );
    supported("Primal Command");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    lots(&mut t, P0);
    let pc = t.hand(P0, "Primal Command");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.cast(P0, pc).modes(&[0, 2]).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
    assert_eq!(t.library_size(P0), lib + 1, "only the Bears went in");
    assert!(t.in_graveyard(P0, "Primal Command"));
}

#[test]
fn primal_commands_modes_happen_in_order_so_the_tucked_card_is_shuffled_away() {
    cr!("700.2", "608.2c");
    ruling!(
        "Primal Command",
        "Primal Command's modes are performed in the order listed. If you put a noncreature permanent on top of its owner's library and have that player shuffle their graveyard into their library, that card is shuffled away."
    );
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    t.graveyard(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let pc = t.hand(P0, "Primal Command");
    t.answer_targets(P0, &[obj(stone)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, pc).modes(&[1, 2]).go();
    let mark = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(zone_of(&t, stone), Zone::Library(P1));
    // The Millstone went on top first; then the library was shuffled.
    let ev = &t.g.turn_events[mark..];
    let moved = ev
        .iter()
        .position(|e| matches!(e, Event::ZoneChange { old, to: Zone::Library(_), .. } if *old == stone))
        .expect("tucked");
    let shuffled = ev
        .iter()
        .position(|e| matches!(e, Event::Shuffled { player } if *player == P1))
        .expect("shuffled");
    assert!(moved < shuffled, "the second mode comes before the third");
}

/// The modes P0 may choose as it begins casting `card`.
fn offered_modes(t: &mut TestGame, card: ObjectId) -> Vec<usize> {
    let from = t.asked().len();
    let _ = t.cast(P0, card).try_go();
    asked_since(t, from)
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .expect("asked for modes")
}

#[test]
fn modal_charms_have_only_the_chosen_modes_targets() {
    cr!("700.2a", "601.2b", "601.2c");
    ruling!(
        "Bant Charm",
        "While the spell is on the stack, treat it as though its only text is the chosen mode. The other two modes are treated as though they don't exist. You don't choose targets for those modes."
    );
    ruling!(
        "Naya Charm",
        "While the spell is on the stack, treat it as though its only text is the chosen mode. The other two modes are treated as though they don't exist. You don't choose targets for those modes."
    );
    ruling!(
        "Bant Charm",
        "You can choose a mode only if you can choose legal targets for that mode. If you can't choose legal targets for any of the modes, you can't cast the spell."
    );
    ruling!(
        "Naya Charm",
        "You can choose a mode only if you can choose legal targets for that mode. If you can't choose legal targets for any of the modes, you can't cast the spell."
    );
    for (name, mode, target) in [("Bant Charm", 1usize, "Grizzly Bears"), ("Naya Charm", 0, "Grizzly Bears")] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, target);
        t.battlefield(P1, "Ornithopter");
        t.graveyard(P1, "Lightning Bolt");
        lots(&mut t, P0);
        let card_ = t.hand(P0, name);
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(bears)]);
        let spell = t.cast(P0, card_).modes(&[mode]).go();
        let n_target_choices = target_candidates(&t, P0, from).len();
        assert_eq!(n_target_choices, 1, "{name}: one target, for the chosen mode");
        let chosen = &t.obj(spell).stack.as_ref().unwrap().chosen;
        assert_eq!(chosen.len(), 1, "{name}: one mode");
        assert_eq!(chosen[0].mode, Some(mode));
        let n: usize = chosen.iter().flat_map(|c| c.targets.iter()).map(|v| v.len()).sum();
        assert_eq!(n, 1, "{name}: {chosen:?}");
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{name}");
    }
    // Nothing to target for any mode: neither charm can be cast. (Naya Charm's third mode
    // targets a player, so it always has a legal target.)
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let bant = t.hand(P0, "Bant Charm");
    assert!(!can_cast(&mut t, P0, bant, mtg_engine::object::CastMethod::Normal));
    // A mode without legal targets can't be chosen: only an artifact, no creature, no
    // card in a graveyard.
    t.battlefield(P1, "Millstone");
    assert!(can_cast(&mut t, P0, bant, mtg_engine::object::CastMethod::Normal));
    assert_eq!(offered_modes(&mut t, bant), vec![0], "Bant Charm: only the artifact mode");
    let naya = t.hand(P0, "Naya Charm");
    assert_eq!(offered_modes(&mut t, naya), vec![2], "Naya Charm: only the player mode");
}

#[test]
fn trial_of_a_time_lord_returns_the_cards_not_put_on_the_bottom() {
    cr!("610.3", "714.4", "701.38a");
    ruling!(
        "Trial of a Time Lord",
        "The cards that didn't get put on the bottom of their owners' libraries will return to the battlefield as Trial of a Time Lord leaves the battlefield."
    );
    supported("Trial of a Time Lord");
    for guilty in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(bears)]);
        let trial = t.enter(P0, "Trial of a Time Lord");
        t.resolve_all();
        assert_eq!(zone_of(&t, bears), Zone::Exile);
        t.answer_targets(P0, &[obj(giant)]);
        crate::r_s19_common::add_lore(&mut t, trial, 1);
        t.resolve_all();
        assert_eq!(zone_of(&t, giant), Zone::Exile);
        crate::r_s19_common::add_lore(&mut t, trial, 1);
        t.resolve_all();
        for p in [P0, P1] {
            t.answer(p, DecisionKind::Option, Answer::Index(guilty as usize));
        }
        crate::r_s19_common::add_lore(&mut t, trial, 1);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Trial of a Time Lord"), "sacrificed after IV");
        for c in [bears, giant] {
            let z = zone_of(&t, c);
            if guilty {
                assert_eq!(z, Zone::Library(P1), "guilty: on the bottom");
                assert_eq!(from_top(&t, P1, c).map(|i| i + 2 >= t.library_size(P1)), Some(true));
            } else {
                assert_eq!(z, Zone::Battlefield, "innocent: back as the Saga leaves");
            }
        }
    }
}
