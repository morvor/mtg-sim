//! Rulings batch P004 — the return of a banished card (CR 610.3): immediately after the
//! banisher leaves, with nothing in between (not a triggered ability), under its owner's
//! control (610.3c), even when the banisher's owner leaves the game (800.4a); and
//! assorted interactions of specific banishers.

use crate::r_p004_common::*;
use crate::r_p079_common::concede;
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn the_card_returns_immediately_after_the_banisher_leaves() {
    cr!("610.3", "610.3c", "704.3");
    ruling!(
        "Chained to the Rocks",
        "The exiled card returns to the battlefield immediately after Chained to the Rocks leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Fairgrounds Warden",
        "The exiled card returns to the battlefield immediately after Fairgrounds Warden leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Ixalan's Binding",
        "The exiled card returns to the battlefield immediately after Ixalan's Binding leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Silkwrap",
        "The exiled card returns to the battlefield immediately after Silkwrap leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Thopter Arrest",
        "The exiled card returns to the battlefield immediately after Thopter Arrest leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Hixus, Prison Warden",
        "The exiled cards return to the battlefield immediately after Hixus leaves the battlefield. Nothing happens between the two events, including state-based actions."
    );
    ruling!(
        "Borrowed Time",
        "Borrowed Time's ability is a single ability that creates two one-shot effects: one that exiles the permanent when the ability resolves, and another that returns the exiled card to the battlefield immediately after Borrowed Time leaves the battlefield."
    );
    ruling!(
        "Chained to the Rocks",
        "Chained to the Rocks's ability causes a zone change with a duration, a style of ability introduced in Magic 2014 that's somewhat reminiscent of older cards like Oblivion Ring. However, unlike Oblivion Ring, cards like Chained to the Rocks have a single ability that creates two one-shot effects: one that exiles the creature when the ability resolves, and another that returns the exiled card to the battlefield immediately after Chained to the Rocks leaves the battlefield."
    );
    ruling!(
        "Colossal Whale",
        "Colossal Whale's ability causes a zone change with a duration, a new style of ability that's somewhat reminiscent of older cards like Oblivion Ring. However, unlike Oblivion Ring, cards like Colossal Whale have a single ability that creates two one-shot effects: one that exiles the creature when the ability resolves, and another that returns the exiled card to the battlefield immediately after Colossal Whale leaves the battlefield."
    );
    ruling!(
        "Hixus, Prison Warden",
        "Hixus's ability causes a zone change with a duration, a style of ability that's somewhat reminiscent of older cards like Oblivion Ring. However, unlike Oblivion Ring, cards like Hixus have a single ability that creates two one-shot effects: one that exiles the creature when the ability resolves, and another that returns the exiled card to the battlefield immediately after Hixus leaves the battlefield."
    );
    ruling!(
        "Silkwrap",
        "Silkwrap's ability causes a zone change with a duration, a style of ability that's somewhat reminiscent of older cards like Oblivion Ring. However, unlike Oblivion Ring, cards like Silkwrap have a single ability that creates two one-shot effects: one that exiles the creature when the ability resolves, and another that returns the exiled card to the battlefield immediately after Silkwrap leaves the battlefield."
    );
    ruling!(
        "In the Trenches",
        "The activated ability is a single ability that creates two one-shot effects: one that exiles the permanent when the ability resolves, and another that returns the exiled card to the battlefield immediately after In the Trenches leaves the battlefield."
    );
    for name in [
        "Chained to the Rocks",
        "Fairgrounds Warden",
        "Ixalan's Binding",
        "Silkwrap",
        "Thopter Arrest",
        "Hixus, Prison Warden",
        "Borrowed Time",
        "Colossal Whale",
        "In the Trenches",
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let b = banish_trigger(&mut t, name, bears);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        let stack = t.stack_len();
        // Right after the banisher is destroyed (its events processed), before
        // state-based actions are checked, the card is back under its owner's control;
        // nothing was put on the stack.
        destroy_and_flush(&mut t, b);
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1, "{name}: not back immediately");
        assert_eq!(t.obj_now(back[0]).controller, P1, "{name}");
        assert_eq!(t.stack_len(), stack, "{name}: the return used the stack");
    }
    // The Aura case: Chained to the Rocks leaves because its Mountain left; the card
    // returns once state-based actions put it into the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    banish_trigger(&mut t, "Chained to the Rocks", bears);
    t.resolve_all();
    let mountain = t.named_on_battlefield("Mountain")[0];
    destroy_and_flush(&mut t, mountain);
    assert!(t.in_exile("Grizzly Bears"));
    t.settle();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn the_returning_creature_and_colossal_whale_are_never_on_the_battlefield_together() {
    cr!("610.3", "707.5", "706.1");
    ruling!(
        "Colossal Whale",
        "The exiled card returns to the battlefield immediately after Colossal Whale leaves the battlefield. Nothing happens between the two events, including state-based actions. The two creatures aren't on the battlefield at the same time. For example, if the returning creature is a Clone, it can't enter the battlefield as a copy of Colossal Whale."
    );
    // P1's Clone (a copy of Grizzly Bears) is exiled by Colossal Whale; when the Whale
    // leaves, the Clone returns but can't copy it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(bears)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    let whale = banish_trigger(&mut t, "Colossal Whale", clone);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "the Clone wasn't exiled");
    let from = t.asked().len();
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(whale)]);
    destroy_and_flush(&mut t, whale);
    t.settle();
    assert!(t.named_on_battlefield("Colossal Whale").is_empty(), "copied the Whale");
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty(), "no copy choice");
    assert!(offered.iter().all(|c| !c.contains(&obj(whale))));
}

#[test]
fn the_card_returns_when_the_banishers_owner_leaves_the_game() {
    cr!("800.4a", "610.3", "610.3c");
    ruling!(
        "Borrowed Time",
        "In a multiplayer game, if Borrowed Time's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Chained to the Rocks",
        "In a multiplayer game, if Chained to the Rocks's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Colossal Whale",
        "In a multiplayer game, if Colossal Whale's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Fairgrounds Warden",
        "In a multiplayer game, if Fairgrounds Warden's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Faith Unbroken",
        "In a multiplayer game, if Faith Unbroken's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Hixus, Prison Warden",
        "In a multiplayer game, if Hixus's owner leaves the game, the exiled cards will return to the battlefield. Because the one-shot effect that returns the cards isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Ixalan's Binding",
        "In a multiplayer game, if Ixalan's Binding's owner leaves the game and another player owns the exiled card, the exiled card will return to the battlefield under its owner's control. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Silkwrap",
        "In a multiplayer game, if Silkwrap's owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    ruling!(
        "Thopter Arrest",
        "In a multiplayer game, if Thopter Arrest’s owner leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn’t an ability that goes on the stack, it won’t cease to exist along with the leaving player’s spells and abilities on the stack."
    );
    ruling!(
        "In the Trenches",
        "In a multiplayer game, if the owner of In the Trenches leaves the game, the exiled card will return to the battlefield. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    for name in [
        "Borrowed Time",
        "Chained to the Rocks",
        "Colossal Whale",
        "Fairgrounds Warden",
        "Faith Unbroken",
        "Hixus, Prison Warden",
        "Ixalan's Binding",
        "Silkwrap",
        "Thopter Arrest",
        "In the Trenches",
    ] {
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        banish_trigger(&mut t, name, bears);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        concede(&mut t, P0);
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1, "{name}: the card didn't return");
        assert_eq!(t.obj_now(back[0]).controller, P1, "{name}");
    }
}

#[test]
fn colossal_whale_exiles_before_blockers_are_declared() {
    cr!("508.1m", "508.2", "509.1");
    ruling!(
        "Colossal Whale",
        "Colossal Whale's triggered ability triggers and resolves during the declare attackers step, before blocking creatures are declared."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    banish_trigger(&mut t, "Colossal Whale", bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.in_exile("Grizzly Bears"), "exiled in the declare attackers step");
}

#[test]
fn in_the_trenches_can_be_activated_once_per_object() {
    cr!("602.5b", "400.7", "608.2b");
    ruling!(
        "In the Trenches",
        "The activated ability can only be activated once per game. If the ability is countered or otherwise fails to resolve after being activated, it can't be activated again."
    );
    ruling!(
        "In the Trenches",
        "If In the Trenches leaves the battlefield and later returns to the battlefield, it is a new object on the battlefield with no memory of the last time it was there. Its ability may be activated, even if it was activated a previous time it was on the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let trenches = banish_trigger(&mut t, "In the Trenches", bears);
    // The ability fails to resolve (its target is gone).
    destroy_now(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    assert!(
        t.activate(P0, trenches, 0, &[obj(giant)]).is_err(),
        "activated a second time"
    );
    // A new In the Trenches object (it left and came back) can activate it again.
    let in_hand =
        t.g.move_object(trenches, Zone::Hand(P0), MoveCause::Effect, None).expect("moved");
    let again =
        t.g.move_object(in_hand, Zone::Battlefield, MoveCause::Effect, None).expect("moved");
    t.g.flush_events();
    t.settle();
    mana(&mut t, P0);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, again, 0, &[obj(giant)]).expect("the new object can activate it");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn lagrella_returns_the_cards_even_if_it_lost_its_abilities() {
    cr!("610.3", "603.7", "613.1f");
    ruling!(
        "Lagrella, the Magpie",
        "The last part of Lagrella's ability is a delayed triggered ability that is set up as its enters-the-battlefield ability resolves. It will trigger when Lagrella leaves the battlefield even if it's somehow lost its abilities or become a copy of something else."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant), obj(bears)]);
    t.answer_targets(P0, &[obj(giant), obj(bears)]);
    let lag = t.enter(P0, "Lagrella, the Magpie");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
    // Lagrella loses all abilities.
    cast_now(&mut t, P1, "Turn to Frog", &[obj(lag)]);
    t.resolve_all();
    assert_eq!(t.pt(lag), (1, 1));
    destroy_and_flush(&mut t, lag);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!((g.len(), b.len()), (1, 1));
    assert_eq!(t.obj_now(b[0]).controller, P1);
    // The delayed trigger: the card that entered under P0's control gets the counters.
    assert_eq!(t.counters(g[0], counters::PLUS1), 2);
    assert_eq!(t.counters(b[0], counters::PLUS1), 0);
}

#[test]
fn deputy_of_detention_exiles_same_named_permanents_without_targeting_them() {
    cr!("115.1", "702.11b");
    ruling!(
        "Deputy of Detention",
        "This ability has only one target. The other permanents with that name aren't targeted. For example, a permanent with hexproof will be exiled if it has the same name as the target nonland permanent."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let boots = t.battlefield(P1, "Swiftfoot Boots");
    assert!(t.g.attach(boots, obj(other)));
    t.g.recompute();
    banish_trigger(&mut t, "Deputy of Detention", bears);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "the hexproof one stayed");
    // Only that player's permanents: another opponent's Grizzly Bears stays.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p2s = t.battlefield(P2, "Grizzly Bears");
    banish_trigger(&mut t, "Deputy of Detention", bears);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears"), vec![p2s]);
}

/// P1 controls two face-down 2/2 creatures (Hill Giant and Grizzly Bears turned face
/// down). Returns them.
fn two_face_down(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, a));
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, b));
    t.g.recompute();
    (t.g.current(a), t.g.current(b))
}

#[test]
fn a_face_down_creature_shares_a_name_with_nothing() {
    cr!("708.2a", "201.2c");
    ruling!(
        "Deputy of Detention",
        "A face-down creature has no name, so it can't share a name with anything. This includes other creatures with no name."
    );
    ruling!(
        "Echoing Courage",
        "A face-down creature has no name, so it can't share a name with anything. This includes other creatures with no name."
    );
    ruling!(
        "Legion's End",
        "A face-down creature has no name, so it can't share a name with anything. This includes other creatures with no name."
    );
    // Deputy of Detention exiles only the target.
    let mut t = TestGame::new(2);
    let (a, b) = two_face_down(&mut t);
    banish_trigger(&mut t, "Deputy of Detention", a);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b), "the other face-down creature was exiled");
    // Echoing Courage pumps only the target.
    let mut t = TestGame::new(2);
    let (a, b) = two_face_down(&mut t);
    cast_now(&mut t, P0, "Echoing Courage", &[obj(a)]);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (2, 2));
    // Legion's End exiles only the target.
    let mut t = TestGame::new(2);
    let (a, b) = two_face_down(&mut t);
    cast_now(&mut t, P0, "Legion's End", &[obj(a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b), "the other face-down creature was exiled");
}

#[test]
fn ixalans_binding_names_the_card_as_it_exists_in_exile() {
    cr!("201.2", "101.2", "305.1", "111.7");
    ruling!(
        "Ixalan's Binding",
        "If the exiled card has a different name in exile than it did on the battlefield, Ixalan's Binding stops players from casting spells with the name of that card as it exists in exile."
    );
    ruling!(
        "Ixalan's Binding",
        "If the exiled card is a land card that's become a nonland permanent, Ixalan's Binding won't stop players from playing lands with that name."
    );
    ruling!(
        "Ixalan's Binding",
        "If there is no exiled card (perhaps because the exiled permanent was a token or was a commander that was moved to the command zone in a Commander game), Ixalan's Binding won't stop players from casting spells."
    );
    // A Clone copying Grizzly Bears is exiled: in exile it's Clone.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(bears)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    banish_trigger(&mut t, "Ixalan's Binding", clone);
    t.resolve_all();
    assert!(t.in_exile("Clone"));
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1);
    let c2 = t.hand(P1, "Clone");
    assert!(t.cast(P1, c2).try_go().is_err(), "cast a Clone");
    let gb = t.hand(P1, "Grizzly Bears");
    assert!(t.cast(P1, gb).try_go().is_ok(), "couldn't cast Grizzly Bears");
    // A face-down Forest (a nonland permanent) is exiled: Forests can still be played.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, forest));
    t.g.recompute();
    let forest = t.g.current(forest);
    banish_trigger(&mut t, "Ixalan's Binding", forest);
    t.resolve_all();
    assert!(t.in_exile("Forest"));
    t.set_step(P1, Step::PrecombatMain);
    let f2 = t.hand(P1, "Forest");
    t.play_land(P1, f2).expect("couldn't play a Forest");
    // A token named Grizzly Bears was exiled: there's no exiled card, so nothing is
    // named and Grizzly Bears can still be cast.
    let mut t = TestGame::new(2);
    let spec = mtg_engine::ability::TokenSpec {
        name: "Grizzly Bears".into(),
        colors: types::ColorSet::single(types::Color::Green),
        supertypes: vec![],
        card_types: vec![types::CardType::Creature],
        subtypes: vec!["Bear".into()],
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    };
    let tok = crate::r_p023_common::token_of(&mut t, P1, spec);
    assert_eq!(t.obj_now(tok).chars.name, "Grizzly Bears");
    banish_trigger(&mut t, "Ixalan's Binding", tok);
    t.resolve_all();
    assert!(!t.on_battlefield(tok), "the token wasn't exiled");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1);
    let gb = t.hand(P1, "Grizzly Bears");
    assert!(t.cast(P1, gb).try_go().is_ok(), "the token's name was named");
}

#[test]
fn helvault_returns_only_the_cards_exiled_with_it() {
    cr!("607.2a", "607.1");
    ruling!(
        "Helvault",
        "Helvault’s first and third abilities are linked. Similarly, Helvault’s second and third abilities are linked. Only cards exiled as a result of Helvault’s first or second abilities will be returned to the battlefield by its third ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let h = t.battlefield(P0, "Helvault");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.exile(P1, "Grizzly Bears");
    mana(&mut t, P0);
    t.activate(P0, h, 0, &[obj(mine)]).expect("first ability");
    t.resolve_all();
    t.g.untap(h);
    mana(&mut t, P0);
    t.activate(P0, h, 1, &[obj(theirs)]).expect("second ability");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    destroy_now(&mut t, h);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1, "only the Bears exiled with Helvault returns");
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P1);
    assert!(t.in_exile("Grizzly Bears"), "the unrelated card returned");
}
