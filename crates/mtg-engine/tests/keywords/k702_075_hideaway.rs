//! CR 702.75 Hideaway.

use crate::common_k702_011_017::{assert_supported, give_mana_for};
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::{destroy, on_top, stack_triggers};
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::zones;
use mtg_engine::*;

/// Puts four named cards on top of P0's library (the last one on top) and returns them.
fn stack_library(t: &mut TestGame) -> Vec<ObjectId> {
    [
        "Grizzly Bears",
        "Counterspell",
        "Llanowar Elves",
        "Lightning Bolt",
    ]
    .iter()
    .map(|n| on_top(t, P0, n))
    .collect()
}

/// The ids of the bottom `n` cards of `p`'s library.
fn bottom(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p).library[..n].to_vec()
}

#[test]
fn hideaway_exiles_one_of_the_top_n_cards_face_down() {
    cr!("702.75", "702.75a");
    ruling!(
        "Mosswort Bridge",
        "Exile one of them face down and put the rest on the bottom of your library in a random order. The exiled card gains 'The player who controls the permanent that exiled this card may look at this card in the exile zone.'"
    );
    ruling!(
        "Evercoat Ursine",
        "Exile one of them face down and put the rest on the bottom of your library in a random order."
    );
    ruling!(
        "Watcher for Tomorrow",
        "Hideaway now causes you to put the rest of the cards on the bottom of your library in a random order instead of any order."
    );
    assert_supported("Watcher for Tomorrow");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let size = t.library_size(P0);
    give_mana_for(&mut t, P0, "Watcher for Tomorrow");
    let watcher = t.hand(P0, "Watcher for Tomorrow");
    t.cast(P0, watcher).go();
    t.resolve();
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 4").len(), 1);
    // Exile the Counterspell (third from the top).
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    t.resolve();
    let exiled = t.g.current(cards[1]);
    assert_eq!(t.g.obj(exiled).zone, Zone::Exile);
    assert!(t.g.obj(exiled).face_down);
    // The other three are on the bottom.
    let mut rest = bottom(&t, P0, 3);
    rest.sort();
    let mut expected = vec![cards[0], cards[2], cards[3]];
    expected.sort();
    assert_eq!(rest, expected);
    assert_eq!(t.library_size(P0), size - 1);
    // Its controller may look at it; the opponent may not.
    assert!(zones::may_look(&t.g, P0, exiled));
    assert!(!zones::may_look(&t.g, P1, exiled));
}

#[test]
fn the_linked_ability_returns_the_exiled_card() {
    cr!("702.75a", "607.2a");
    ruling!(
        "Watcher for Tomorrow",
        "You don't reveal the exiled card when you put it into its owner's hand."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    // "When this creature leaves the battlefield, put the exiled card into its owner's
    // hand."
    destroy(&mut t, watcher);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn a_card_that_left_exile_is_no_longer_the_exiled_card() {
    cr!("702.75a", "400.7");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    let exiled = t.g.current(cards[3]);
    assert_eq!(t.g.obj(exiled).zone, Zone::Exile);
    // Another effect puts the card into its owner's graveyard.
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P1,
        mtg_engine::ability::Effect::Move {
            what: mtg_engine::ability::Sel::Target(0),
            to: mtg_engine::ability::Destination::zone(mtg_engine::ability::ZoneKind::Graveyard),
        },
        &[Entity::Object(exiled)],
    );
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    destroy(&mut t, watcher);
    t.resolve_all();
    // It stays in the graveyard.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(!t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn leaving_before_hideaway_resolves_strands_the_card() {
    cr!("702.75a");
    ruling!(
        "Watcher for Tomorrow",
        "If Watcher for Tomorrow leaves the battlefield before its triggered ability from hideaway resolves, its leaves-the-battlefield ability resolves and does nothing."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.settle();
    destroy(&mut t, watcher);
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    assert!(!t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn hideaway_n_looks_at_n_cards() {
    cr!("702.75a");
    assert_supported("Fight Rigging");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let fifth = on_top(&mut t, P0, "Island");
    let before = t.library_size(P0);
    t.enter(P0, "Fight Rigging");
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 5").len(), 1);
    // The Grizzly Bears was fifth from the top: it could be chosen.
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    let mut rest = bottom(&t, P0, 4);
    rest.sort();
    let mut expected = vec![cards[1], cards[2], cards[3], fifth];
    expected.sort();
    assert_eq!(rest, expected);
    assert_eq!(t.library_size(P0), before - 1);
}

#[test]
fn a_new_controller_of_the_hideaway_permanent_may_look_at_the_card() {
    cr!("702.75a");
    ruling!(
        "Mosswort Bridge",
        "Any player who has controlled a permanent with a hideaway ability since a card was exiled with it may look at that card."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let watcher = t.enter(P0, "Watcher for Tomorrow");
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    let exiled = t.g.current(cards[3]);
    assert!(!zones::may_look(&t.g, P1, exiled));
    // P1 gains control of it.
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(watcher).go();
    t.resolve_all();
    assert_eq!(t.obj_now(watcher).controller, P1);
    assert!(zones::may_look(&t.g, P1, exiled));
    // The previous controller still may too.
    assert!(zones::may_look(&t.g, P0, exiled));
}

#[test]
fn older_hideaway_cards_have_hideaway_four_and_enter_tapped() {
    cr!("702.75b");
    ruling!(
        "Mosswort Bridge",
        "Older cards have received errata to have an additional paragraph"
    );
    assert_supported("Mosswort Bridge");
    let def = mtg_engine::card::card("Mosswort Bridge");
    let chars = &def.front().chars;
    let kw: Vec<_> = chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Hideaway)
        .collect();
    assert_eq!(kw.len(), 1);
    assert_eq!(kw[0].n, Some(4));
    let mut t = TestGame::new(2);
    stack_library(&mut t);
    let bridge = t.enter(P0, "Mosswort Bridge");
    assert!(t.obj_now(bridge).tapped);
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 4").len(), 1);
}

#[test]
fn the_exiled_card_can_be_played_by_the_hideaway_permanents_linked_ability() {
    cr!("702.75a", "607.2a");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let bridge = t.enter(P0, "Mosswort Bridge");
    // Exile the Grizzly Bears (fourth from the top).
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    let bridge_now = t.g.current(bridge);
    t.g.objects[bridge_now.0 as usize].tapped = false;
    t.lands(P0, "Forest", 1);
    let uid = t
        .obj_now(bridge)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(&a.kind, AbilityKind::Activated(x) if !x.is_mana_ability))
        .map(|a| a.text.clone())
        .expect("play ability");
    // "... if creatures you control have total power 10 or greater": not yet.
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, bridge, &uid, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    // With 12 power, it's played for free.
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Craw Wurm");
    let now = t.g.current(bridge);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, bridge, &uid, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

/// The text of the first nonmana activated ability of `id`.
fn play_ability(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(&a.kind, AbilityKind::Activated(x) if !x.is_mana_ability))
        .map(|a| a.text.clone())
        .expect("play ability")
}

#[test]
fn an_exiled_land_card_is_played_only_with_a_land_play_left() {
    cr!("702.75a");
    ruling!(
        "Evercoat Ursine",
        "If one of the exiled cards is a land card, you may play it only if you have an available land play remaining this turn."
    );
    assert_supported("Shelldock Isle");
    let mut t = TestGame::new(2);
    let forest = on_top(&mut t, P0, "Forest");
    let isle = t.enter(P0, "Shelldock Isle");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    // "... if a library has twenty or fewer cards in it": any library.
    ruling!(
        "Shelldock Isle",
        "It doesn't matter which library has twenty or fewer cards in it"
    );
    while t.library_size(P1) > 20 {
        let top = *t.g.player(P1).library.last().unwrap();
        t.g.players[1].library.pop();
        let _ = top;
    }
    let text = play_ability(&t, isle);
    // A land was already played this turn: the Forest stays exiled.
    t.g.players[0].lands_played_this_turn = 1;
    let now = t.g.current(isle);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, isle, &text, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    // With a land play left, it's played.
    t.g.players[0].lands_played_this_turn = 0;
    let now = t.g.current(isle);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, isle, &text, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert!(!t.obj_now(forest).face_down);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
}

#[test]
fn cards_exiled_by_several_hideaway_abilities_are_all_exiled_with_it() {
    cr!("702.75a", "607.2a");
    ruling!(
        "Evercoat Ursine",
        "You choose and play the card while Evercoat Ursine's last ability is resolving and still on the stack."
    );
    assert_supported("Evercoat Ursine");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    // Hideaway 3, hideaway 3: two triggers, each exiling one card.
    let bear = t.enter(P0, "Evercoat Ursine");
    t.settle();
    assert_eq!(stack_triggers(&t, "Hideaway 3").len(), 2);
    // The first looks at the Lightning Bolt, Llanowar Elves, and Counterspell; the second
    // at the Grizzly Bears and the two cards under it.
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve();
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    // "Whenever this creature deals combat damage to a player, if there are cards exiled
    // with it, you may play one of them without paying its mana cost."
    t.g.objects[bear.0 as usize].summoning_sick = false;
    crate::common_k702_011_017::attack_with(&mut t, &[(bear, Entity::Player(P1))]);
    crate::common_k702_018_026::declare_blocks(&mut t, P1, &[]);
    // Play the Grizzly Bears, exiled by the second ability (a creature card: cast without
    // paying its mana cost).
    let exiled_bears = t.g.current(cards[0]);
    t.answer_choose(P0, &[Entity::Object(exiled_bears)]);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    // The other exiled card (an instant) can't be played later in the turn.
    t.lands(P0, "Mountain", 1);
    assert!(!crate::common_k702_027_037::can_cast(
        &mut t,
        P0,
        cards[3],
        mtg_engine::object::CastMethod::Normal
    ));
}

#[test]
fn windbrisk_heights_counts_the_different_creatures_declared_as_attackers() {
    cr!("702.75a");
    ruling!(
        "Windbrisk Heights",
        "A creature declared as an attacker in two different attack phases counts only once. A creature that entered attacking (such as a token created by Militia's Pride) doesn't count because you never attacked with it."
    );
    assert_supported("Windbrisk Heights");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let heights = t.enter(P0, "Windbrisk Heights");
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    let text = play_ability(&t, heights);
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.hand(P0, "Llanowar Elves");
    crate::common_k702_011_017::attack_with(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    // A third creature put onto the battlefield attacking wasn't declared as an attacker.
    let mut to = mtg_engine::ability::Destination::battlefield();
    to.attacking = true;
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::Move {
            what: mtg_engine::ability::Sel::Target(0),
            to,
        },
        &[Entity::Object(c)],
    );
    crate::common_k702_018_026::declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    // An additional combat: the first creature attacks again.
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::ExtraCombat { after_this: true },
        &[],
    );
    let now = t.g.current(a);
    t.g.objects[now.0 as usize].tapped = false;
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(a, Entity::Player(P1))]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::DeclareAttackers);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    // Two different creatures attacked: not enough.
    let now = t.g.current(heights);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, heights, &text, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    // Three different creatures declared as attackers: the card is played.
    let d = t.battlefield(P0, "Llanowar Elves");
    t.g.objects[d.0 as usize].tapped = false;
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::ExtraCombat { after_this: true },
        &[],
    );
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(d, Entity::Player(P1))]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::DeclareAttackers);
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    let now = t.g.current(heights);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    activate_named(&mut t, P0, heights, &text, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn spinerock_knoll_counts_all_damage_dealt_to_an_opponent_this_turn() {
    cr!("702.75a");
    ruling!(
        "Spinerock Knoll",
        "It doesn't matter how the opponent was dealt damage or by whom, as long as the total damage is 7 or more."
    );
    ruling!(
        "Spinerock Knoll",
        "You'll get to play the card even if Spinerock Knoll wasn't on the battlefield at the time some or all of the 7 damage was dealt."
    );
    assert_supported("Spinerock Knoll");
    let mut t = TestGame::new(2);
    // 4 damage before the Knoll is on the battlefield, 3 more by another source later.
    let giant = t.battlefield(P0, "Hill Giant");
    let deal = |t: &mut TestGame, src: ObjectId, n: i32| {
        crate::common_k702_052_066::run_effect(
            t,
            None,
            P0,
            mtg_engine::ability::Effect::DealDamage {
                source: mtg_engine::ability::Sel::Target(0),
                amount: mtg_engine::ability::Value::c(n),
                to: mtg_engine::ability::Sel::Players(mtg_engine::ability::PlayerRef::Player(P1)),
            },
            &[Entity::Object(src)],
        );
    };
    deal(&mut t, giant, 4);
    let cards = stack_library(&mut t);
    let knoll = t.enter(P0, "Spinerock Knoll");
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    let text = play_ability(&t, knoll);
    let activate = |t: &mut TestGame| {
        let now = t.g.current(knoll);
        t.g.objects[now.0 as usize].tapped = false;
        t.lands(P0, "Mountain", 1);
        t.answer_yes(P0, true);
        activate_named(t, P0, knoll, &text, 0).unwrap();
        t.resolve_all();
    };
    activate(&mut t);
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    let other = t.battlefield(P0, "Grizzly Bears");
    deal(&mut t, other, 3);
    assert_eq!(t.life(P1), 13);
    activate(&mut t);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn howltooth_hollow_needs_every_hand_to_be_empty() {
    cr!("702.75a");
    assert_supported("Howltooth Hollow");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let hollow = t.enter(P0, "Howltooth Hollow");
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.resolve_all();
    let text = play_ability(&t, hollow);
    let card_in_hand = t.hand(P1, "Grizzly Bears");
    let activate = |t: &mut TestGame| {
        let now = t.g.current(hollow);
        t.g.objects[now.0 as usize].tapped = false;
        t.lands(P0, "Swamp", 1);
        t.answer_yes(P0, true);
        activate_named(t, P0, hollow, &text, 0).unwrap();
        t.resolve_all();
    };
    activate(&mut t);
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    // The opponent's last card leaves their hand.
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P1,
        mtg_engine::ability::Effect::Move {
            what: mtg_engine::ability::Sel::Target(0),
            to: mtg_engine::ability::Destination::zone(mtg_engine::ability::ZoneKind::Graveyard),
        },
        &[Entity::Object(card_in_hand)],
    );
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), 0);
    activate(&mut t);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

/// Smuggler's Buggy (a 5/5 Vehicle) with the Lightning Bolt exiled by its hideaway ability,
/// made a creature and attacking P1 unblocked; stops in the postcombat main phase.
fn buggy_hits(cast_it: bool) -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t);
    let buggy = t.enter(P0, "Smuggler's Buggy");
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.resolve_all();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![mtg_engine::ability::Modification::AddTypes(vec![
                mtg_engine::types::CardType::Creature,
            ])],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &[Entity::Object(buggy)],
    );
    t.g.objects[buggy.0 as usize].summoning_sick = false;
    crate::common_k702_011_017::attack_with(&mut t, &[(buggy, Entity::Player(P1))]);
    crate::common_k702_018_026::declare_blocks(&mut t, P1, &[]);
    // "Whenever this Vehicle deals combat damage to a player, you may cast the exiled card
    // without paying its mana cost. If you do, return this Vehicle to its owner's hand."
    t.answer_yes(P0, cast_it);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    t.resolve_all();
    (t, cards)
}

#[test]
fn a_hideaway_permanent_can_cast_the_exiled_card_for_free() {
    cr!("702.75a", "607.2a");
    assert_supported("Smuggler's Buggy");
    let (t, cards) = buggy_hits(true);
    // 5 combat damage, then the Lightning Bolt cast for free.
    assert_eq!(t.life(P1), 12);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.g.permanents().all(|o| !o.tapped || !o.chars.is_land()));
    // "If you do": the Vehicle returns to its owner's hand.
    assert!(t.in_hand(P0, "Smuggler's Buggy"));
    let _ = cards;
    // Not cast: the card stays exiled and the Vehicle stays.
    let (t, cards) = buggy_hits(false);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    assert_eq!(t.named_on_battlefield("Smuggler's Buggy").len(), 1);
}

#[test]
fn fight_rigging_plays_the_exiled_card_once_you_control_a_big_creature() {
    cr!("702.75a", "607.2a");
    assert_supported("Fight Rigging");
    for (creature, played) in [("Hill Giant", false), ("Craw Wurm", true)] {
        let mut t = TestGame::new(2);
        let cards = stack_library(&mut t);
        let c = t.battlefield(P0, creature);
        t.enter(P0, "Fight Rigging");
        // Hideaway 5: exile the Grizzly Bears.
        t.answer_choose(P0, &[Entity::Object(cards[0])]);
        t.resolve_all();
        // "At the beginning of combat on your turn, put a +1/+1 counter on target creature
        // you control. Then if you control a creature with power 7 or greater, you may play
        // the exiled card without paying its mana cost."
        t.answer_targets(P0, &[Entity::Object(c)]);
        t.answer_yes(P0, true);
        t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
        t.resolve_all();
        assert_eq!(t.counters(c, mtg_engine::types::counters::PLUS1), 1);
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            played as usize,
            "{creature}"
        );
        assert_eq!(t.zone(cards[0]) == Zone::Exile, !played);
    }
}

const CLIVE: &str = "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno";
/// Vanilla legendary creatures (no abilities that change costs or land plays).
const LEGENDS: [&str; 3] = [
    "Isamaru, Hound of Konda",
    "Caelorna, Coral Tyrant",
    "Barktooth Warbeard",
];

/// Clive's Hideaway enters and hides `top` (put on top of P0's library); returns the land
/// and the exiled card.
fn clives_hideaway(t: &mut TestGame, top: &str) -> (ObjectId, ObjectId) {
    let card = on_top(t, P0, top);
    let land = t.enter(P0, "Clive's Hideaway");
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.obj_now(card).face_down);
    (land, card)
}

/// Activates Clive's Hideaway's "{2}, {T}: You may play the exiled card ..." (untapping it
/// first and giving P0 the mana), answering yes or no to playing the card.
fn activate_clives_hideaway(t: &mut TestGame, land: ObjectId, play: bool) {
    let now = t.g.current(land);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, play);
    let text = play_ability(t, land);
    activate_named(t, P0, land, &text, 0).unwrap();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn clives_hideaway_checks_its_legendary_creatures_and_plays_the_card_as_the_ability_resolves() {
    cr!("702.75a", "607.2a", "602.1b", "608.2h", "608.2g", "305.2a", "305.2b", "305.3", "406.3a");
    // Scryfall's data (source wotc, 2025-06-06) attaches these three FINAL FANTASY
    // release-note rulings for Clive's Hideaway to Clive, Ifrit's Dominant, which has no
    // exiled card to play (Clive's Hideaway itself has only the Town note). They are tested
    // here on Clive's Hideaway; Clive is one of the legendary creatures it counts.
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "Clive's Hideaway's last ability only cares if you control four or more legendary creatures at the time it resolves. You don't even have to control four or more legendary creatures when you activate it."
    );
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "If the exiled card is a land card, you may play it only if you have an available land play remaining this turn."
    );
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "You play the card while Clive's Hideaway's last ability is resolving and still on the stack. You can't wait to play it later in the turn."
    );
    assert_supported("Clive's Hideaway");
    // Clive goes onto the battlefield without its enters trigger ("you may discard your
    // hand"), which would otherwise go on the stack above the hideaway ability.
    let four = |t: &mut TestGame| -> Vec<ObjectId> {
        let mut v: Vec<ObjectId> = LEGENDS.iter().map(|n| t.battlefield(P0, n)).collect();
        v.push(t.battlefield(P0, CLIVE));
        v
    };

    // Three legendary creatures as it's activated, four (Clive arrives) as it resolves:
    // the exiled card is cast while the ability resolves (it's on the stack once the
    // ability is gone).
    let mut t = TestGame::new(2);
    let (land, bears) = clives_hideaway(&mut t, "Grizzly Bears");
    for n in LEGENDS {
        t.battlefield(P0, n);
    }
    activate_clives_hideaway(&mut t, land, true);
    t.battlefield(P0, CLIVE);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Battlefield);

    // Four as it's activated, three (Clive destroyed) as it resolves: nothing is played.
    let mut t = TestGame::new(2);
    let (land, bears) = clives_hideaway(&mut t, "Grizzly Bears");
    let legends = four(&mut t);
    activate_clives_hideaway(&mut t, land, true);
    destroy(&mut t, legends[3]);
    assert_eq!(t.zone(legends[3]), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.clear_answers();

    // An exiled land: not played with no land play left (CR 305.2b), played with one
    // (face up, counting as the land play, CR 305.2a, 406.3a).
    let mut t = TestGame::new(2);
    let (land, forest) = clives_hideaway(&mut t, "Forest");
    four(&mut t);
    t.g.players[0].lands_played_this_turn = 1;
    activate_clives_hideaway(&mut t, land, true);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
    t.g.players[0].lands_played_this_turn = 0;
    activate_clives_hideaway(&mut t, land, true);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert!(!t.obj_now(forest).face_down);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);

    // Not on another player's turn (CR 305.3), even with no land played.
    let mut t = TestGame::new(2);
    let (land, forest) = clives_hideaway(&mut t, "Forest");
    four(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    activate_clives_hideaway(&mut t, land, true);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 0);

    // On P0's own turn outside a main phase it is played: a land played while an ability
    // resolves ignores the timing of the special action.
    let mut t = TestGame::new(2);
    let (land, forest) = clives_hideaway(&mut t, "Forest");
    four(&mut t);
    t.set_step(P0, Step::DeclareBlockers);
    activate_clives_hideaway(&mut t, land, true);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Battlefield);

    // Declined while the ability resolves: the card stays exiled and can't be cast later
    // (the ability gives no lasting permission), even with mana for it; another activation
    // can still cast it.
    let mut t = TestGame::new(2);
    let (land, bolt) = clives_hideaway(&mut t, "Lightning Bolt");
    four(&mut t);
    activate_clives_hideaway(&mut t, land, false);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    t.g.turn.priority = Some(P0);
    let now = t.g.current(bolt);
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == now)));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_clives_hideaway(&mut t, land, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 17);
}
