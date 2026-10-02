//! Exiling cards face down (CR 406.3): "[instruction] face down" and "look at the top N
//! cards of [a library], exile one of them face down, then put the rest ..." (patterns in
//! `src/oracle/patterns/face_status_grammar.rs`).

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::zones;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Puts the named cards on top of `p`'s library, the last one on top; returns them.
fn stack(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.library_top(p, n)).collect()
}

/// The face-down cards in exile.
fn exiled_face_down(t: &TestGame) -> Vec<ObjectId> {
    t.g.exile
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).face_down)
        .collect()
}

#[test]
fn face_down_exile_cards_compile() {
    assert_compiles(&[
        "Bottled Cloister",
        "Decadent Dragon // Expensive Taste",
        "Vivien, Champion of the Wilds",
        "Clone Shell",
        "Extract Power",
        "Discover the Impossible",
        "Siphon Insight",
        "Outrageous Robbery",
        "Inverter of Truth",
        "Lobelia, Defender of Bag End",
        "Gonti, Lord of Luxury",
        "Induced Amnesia",
        "Thief of Sanity",
        "Petty Larceny",
        "Gandalf, Goblins' Bane // Flameshape",
    ]);
}

#[test]
fn bottled_cloister_exiles_the_hand_face_down_and_returns_it() {
    cr!("406.3", "607.2a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bottled Cloister");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.hand(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    let exiled = exiled_face_down(&t);
    assert_eq!(exiled.len(), 2);
    // No player may look at them (CR 406.3).
    for o in &exiled {
        assert!(!zones::may_look(&t.g, P0, *o));
        assert!(!zones::may_look(&t.g, P1, *o));
    }
    // At the beginning of P0's upkeep they come back, then P0 draws.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn expensive_taste_exiles_the_top_two_cards_of_target_opponents_library_face_down() {
    cr!("406.3");
    let mut t = TestGame::new(2);
    let top = stack(&mut t, P1, &["Grizzly Bears", "Shock"]);
    let card = t.hand(P0, "Decadent Dragon // Expensive Taste");
    t.lands(P0, "Swamp", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    for c in top {
        let now = t.g.current(c);
        assert_eq!(t.zone(now), Zone::Exile);
        assert!(t.g.obj(now).face_down);
        // "You may look at and play those cards for as long as they remain exiled."
        assert!(zones::may_look(&t.g, P0, now));
        assert!(!zones::may_look(&t.g, P1, now));
    }
}

#[test]
fn induced_amnesia_target_player_exiles_their_hand_face_down_and_draws_that_many() {
    cr!("406.3", "121.1");
    ruling!("Induced Amnesia", "No player may look at the exiled cards.");
    let mut t = TestGame::new(2);
    for n in ["Lightning Bolt", "Shock", "Grizzly Bears"] {
        t.hand(P1, n);
    }
    let library = t.library_size(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Induced Amnesia");
    t.resolve_all();
    let exiled = exiled_face_down(&t);
    assert_eq!(exiled.len(), 3);
    assert!(exiled.iter().all(|o| t.g.obj(*o).owner == P1));
    assert!(exiled
        .iter()
        .all(|o| !zones::may_look(&t.g, P0, *o) && !zones::may_look(&t.g, P1, *o)));
    assert_eq!(t.hand_size(P1), 3);
    assert_eq!(t.library_size(P1), library - 3);
}

#[test]
fn hoarding_broodlord_searches_for_a_card_and_exiles_it_face_down() {
    cr!("406.3", "701.23a");
    ruling!(
        "Hoarding Broodlord",
        "As long as that card remains exiled, you may look at it."
    );
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.enter(P0, "Hoarding Broodlord");
    t.resolve_all();
    let now = t.g.current(bolt);
    assert_eq!(t.zone(now), Zone::Exile);
    assert!(t.g.obj(now).face_down);
    // Having found it, the searcher may go on looking at it (CR 406.3).
    assert!(zones::may_look(&t.g, P0, now));
    assert!(!zones::may_look(&t.g, P1, now));
}

#[test]
fn praetors_grasp_only_its_caster_may_look_at_the_card() {
    cr!("406.3", "701.23a");
    ruling!(
        "Praetor's Grasp",
        "Other players, including the card’s owner, can’t look at the card while it remains exiled."
    );
    let mut t = TestGame::new(2);
    let card = t.library_top(P1, "Serra Angel");
    let spell = t.hand(P0, "Praetor's Grasp");
    t.lands(P0, "Swamp", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    let now = t.g.current(card);
    assert_eq!(t.zone(now), Zone::Exile);
    assert!(t.g.obj(now).face_down);
    assert!(zones::may_look(&t.g, P0, now));
    assert!(!zones::may_look(&t.g, P1, now));
}

#[test]
fn vivien_exiles_one_of_the_top_three_face_down_and_the_rest_go_to_the_bottom() {
    cr!("406.3", "701.20b");
    let mut t = TestGame::new(2);
    let cards = stack(&mut t, P0, &["Grizzly Bears", "Shock", "Llanowar Elves"]);
    let vivien = t.battlefield(P0, "Vivien, Champion of the Wilds");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.activate(P0, vivien, 1, &[]).unwrap();
    t.resolve_all();
    let exiled = t.g.current(cards[0]);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.g.obj(exiled).face_down);
    // The other two are on the bottom of P0's library.
    let lib = &t.g.player(P0).library;
    let mut bottom = lib[..2].to_vec();
    bottom.sort();
    let mut expected = vec![cards[1], cards[2]];
    expected.sort();
    assert_eq!(bottom, expected);
}

#[test]
fn clone_shell_must_exile_one_of_the_cards_even_if_none_is_a_creature() {
    cr!("406.3", "607.2a");
    ruling!(
        "Clone Shell",
        "As Clone Shell's first ability resolves, you must exile one of the cards you look at, even if none of them is a creature card."
    );
    let mut t = TestGame::new(2);
    stack(&mut t, P0, &["Shock", "Lightning Bolt", "Counterspell", "Island"]);
    let size = t.library_size(P0);
    t.enter(P0, "Clone Shell");
    t.resolve_all();
    assert_eq!(exiled_face_down(&t).len(), 1);
    assert_eq!(t.library_size(P0), size - 1);
}

#[test]
fn gonti_its_controller_chooses_the_card_from_the_opponents_library() {
    cr!("406.3", "608.2c");
    ruling!(
        "Gonti, Lord of Luxury",
        "Gonti doesn't change when you can cast the exiled card."
    );
    let mut t = TestGame::new(2);
    let cards = stack(
        &mut t,
        P1,
        &["Grizzly Bears", "Shock", "Llanowar Elves", "Giant Growth"],
    );
    let size = t.library_size(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(cards[2])]);
    t.enter(P0, "Gonti, Lord of Luxury");
    t.resolve_all();
    let elves = t.g.current(cards[2]);
    assert_eq!(t.zone(elves), Zone::Exile);
    assert!(t.g.obj(elves).face_down);
    assert_eq!(t.library_size(P1), size - 1);
    // The other three are on the bottom of P1's library.
    let lib = t.g.player(P1).library.clone();
    let mut bottom = lib[..3].to_vec();
    bottom.sort();
    let mut expected = vec![cards[0], cards[1], cards[3]];
    expected.sort();
    assert_eq!(bottom, expected);
    // P0 (not the library's owner) chose the card.
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
            if candidates.len() == 4)));
    // P0 may cast it.
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, elves).go();
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(elves)).controller, P0);
    assert!(t.on_battlefield(t.g.current(elves)));
}

#[test]
fn thief_of_sanity_exiles_one_and_puts_the_rest_into_their_graveyard() {
    cr!("406.3", "510.2");
    let mut t = TestGame::new(2);
    let cards = stack(&mut t, P1, &["Grizzly Bears", "Shock", "Llanowar Elves"]);
    let thief = t.battlefield(P0, "Thief of Sanity");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(cards[1])]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    t.resolve_all();
    let shock = t.g.current(cards[1]);
    assert_eq!(t.zone(shock), Zone::Exile);
    assert!(t.g.obj(shock).face_down);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn siphon_insight_puts_the_other_card_on_the_bottom_of_that_library() {
    cr!("406.3");
    let mut t = TestGame::new(2);
    let cards = stack(&mut t, P1, &["Grizzly Bears", "Shock"]);
    let spell = t.hand(P0, "Siphon Insight");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    let bears = t.g.current(cards[0]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.g.obj(bears).face_down);
    assert_eq!(t.g.player(P1).library[0], cards[1]);
}

#[test]
fn lobelia_exiles_the_top_card_of_each_opponents_library_face_down() {
    cr!("406.3");
    let mut t = TestGame::new(3);
    let a = t.library_top(P1, "Shock");
    let b = t.library_top(P2, "Grizzly Bears");
    let mine = t.library_top(P0, "Lightning Bolt");
    t.enter(P0, "Lobelia, Defender of Bag End");
    t.resolve_all();
    for c in [a, b] {
        let now = t.g.current(c);
        assert_eq!(t.zone(now), Zone::Exile);
        assert!(t.g.obj(now).face_down);
        // Having looked at them, P0 may look at them in exile.
        assert!(zones::may_look(&t.g, P0, now));
    }
    assert_eq!(t.zone(mine), Zone::Library(P0));
}

// ---------------------------------------------------------------------------
// Turning face up
// ---------------------------------------------------------------------------

fn destroy(t: &mut TestGame, id: ObjectId) {
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

#[test]
fn face_up_cards_compile() {
    assert_compiles(&[
        "Clone Shell",
        "The Creation of Avacyn",
        "Hustle // Bustle",
        "Showstopping Surprise",
    ]);
}

#[test]
fn clone_shell_turns_the_exiled_card_face_up_and_puts_a_creature_onto_the_battlefield() {
    cr!("406.3", "607.2a", "603.10a");
    let mut t = TestGame::new(2);
    let cards = stack(&mut t, P0, &["Grizzly Bears", "Shock", "Island", "Swamp"]);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    let shell = t.enter(P0, "Clone Shell");
    t.resolve_all();
    let exiled = t.g.current(cards[0]);
    assert!(t.g.obj(exiled).face_down);
    destroy(&mut t, shell);
    t.resolve_all();
    let bears = t.g.current(cards[0]);
    assert!(t.on_battlefield(bears));
    assert!(!t.g.obj(bears).face_down);
    assert_eq!(t.g.obj(bears).controller, P0);
}

#[test]
fn clone_shell_leaves_a_noncreature_card_face_up_in_exile() {
    cr!("406.3");
    ruling!(
        "Clone Shell",
        "As Clone Shell's second ability resolves, if the exiled card is not a creature card, it simply remains in exile face up."
    );
    let mut t = TestGame::new(2);
    let cards = stack(&mut t, P0, &["Shock", "Island", "Swamp", "Plains"]);
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    let shell = t.enter(P0, "Clone Shell");
    t.resolve_all();
    destroy(&mut t, shell);
    t.resolve_all();
    let shock = t.g.current(cards[0]);
    assert_eq!(t.zone(shock), Zone::Exile);
    assert!(!t.g.obj(shock).face_down);
    // Face up in exile: any player may examine it (CR 406.3).
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, shock));
}

#[test]
fn creation_of_avacyn_turns_the_exiled_card_face_up_and_you_lose_life() {
    cr!("406.3", "714.2b");
    let mut t = TestGame::new(2);
    let angel = t.library_top(P0, "Serra Angel");
    t.answer_choose(P0, &[Entity::Object(angel)]);
    let saga = t.enter(P0, "The Creation of Avacyn");
    t.resolve_all();
    let exiled = t.g.current(angel);
    assert!(t.g.obj(exiled).face_down);
    // Chapter II.
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    let exiled = t.g.current(angel);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(!t.g.obj(exiled).face_down);
    assert_eq!(t.life(P0), 20 - 5);
}

#[test]
fn showstopping_surprise_turns_a_face_down_creature_face_up_before_it_deals_damage() {
    cr!("708.8", "120.3");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Hill Giant");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, ogre));
    t.g.recompute();
    assert_eq!(t.pt(ogre), (2, 2));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    let spell = t.hand(P0, "Showstopping Surprise");
    t.lands(P0, "Mountain", 5);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(ogre)).go();
    t.resolve_all();
    assert!(!t.g.obj(ogre).face_down);
    // A 3/3 deals 3 damage.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.g.obj(wall).damage, 3);
}

#[test]
fn bustle_may_turn_a_creature_you_control_face_up() {
    cr!("708.8");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, giant));
    t.g.recompute();
    let spell = t.hand(P0, "Hustle // Bustle");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Mountain", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert!(!t.g.obj(giant).face_down);
    assert_eq!(t.pt(giant), (5, 5));
}

// ---------------------------------------------------------------------------
// Entering the battlefield face down
// ---------------------------------------------------------------------------

#[test]
fn face_down_entry_cards_compile() {
    assert_compiles(&[
        "Ashcloud Phoenix",
        "Yedora, Grave Gardener",
        "Death in Heaven",
        "The Cyber-Controller",
        "Tezzeret, Cruel Machinist",
        "Deathmist Raptor",
        "Yarus, Roar of the Old Gods",
    ]);
}

#[test]
fn ashcloud_phoenix_returns_face_down_and_can_be_turned_up_for_its_morph_cost() {
    cr!("708.2a", "708.3", "702.37e");
    ruling!(
        "Ashcloud Phoenix",
        "If Ashcloud Phoenix is face down, you can turn it face up for its morph cost, even if you didn't cast Ashcloud Phoenix face down using its morph ability."
    );
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Ashcloud Phoenix");
    destroy(&mut t, phoenix);
    t.resolve_all();
    let now = t.g.current(phoenix);
    assert!(t.on_battlefield(now));
    assert!(t.g.obj(now).face_down);
    assert_eq!(t.g.obj(now).controller, P0);
    assert_eq!(t.pt(now), (2, 2));
    assert!(t.g.obj(now).chars.name.is_empty());
    // Turned face up for {4}{R}{R}: "it deals 2 damage to each player".
    t.lands(P0, "Mountain", 6);
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    t.g
        .perform_action(
            P0,
            mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
                obj: now,
            }),
        )
        .expect("turned face up");
    t.g.flush_events();
    t.resolve_all();
    assert!(!t.g.obj(now).face_down);
    assert_eq!(t.pt(now), (4, 1));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn yedora_returns_the_creature_face_down_as_a_forest_land() {
    cr!("708.2a", "305.6");
    ruling!(
        "Yedora, Grave Gardener",
        "The face-down card has no name or colors. Its only type is land, its only subtype is Forest"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yedora, Grave Gardener");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    destroy(&mut t, bears);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now));
    let o = t.g.obj(now);
    assert!(o.face_down);
    assert!(o.is(CardType::Land));
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Forest"));
    assert!(o.chars.name.is_empty());
}

#[test]
fn cyber_controller_puts_milled_creature_cards_onto_the_battlefield_as_face_down_cybermen() {
    cr!("708.2a", "708.8");
    ruling!(
        "The Cyber-Controller",
        "Each creature turned face down this way or put onto the battlefield this way is a 2/2 Cyberman artifact creature with no name and no color."
    );
    ruling!(
        "The Cyber-Controller",
        "If, for any reason, the face-down creature is turned face up, the effect making it a Cyberman ends."
    );
    let mut t = TestGame::new(2);
    let angel = t.library_top(P1, "Serra Angel");
    let bolt = t.library_top(P1, "Lightning Bolt");
    let controller = t.hand(P0, "The Cyber-Controller");
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, controller).x(2).go();
    t.resolve_all();
    let angel = t.g.current(angel);
    assert!(t.on_battlefield(angel));
    assert_eq!(t.g.obj(angel).controller, P0);
    assert!(t.g.obj(angel).face_down);
    assert!(t.g.obj(angel).is(CardType::Artifact));
    assert!(t.g.obj(angel).chars.has_subtype("Cyberman"));
    // 2/2, +1/+1 from The Cyber-Controller.
    assert_eq!(t.pt(angel), (3, 3));
    // The noncreature card stays in the graveyard.
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P1));
    // Turned face up, it's the Serra Angel printed on the card.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::TurnFaceUp {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::FaceDown),
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(!t.g.obj(angel).face_down);
    assert!(!t.g.obj(angel).is(CardType::Artifact));
    assert_eq!(t.pt(angel), (4, 4));
}

#[test]
fn deathmist_raptor_may_return_face_down() {
    cr!("708.3");
    ruling!(
        "Deathmist Raptor",
        "You choose whether Deathmist Raptor will enter the battlefield face up or face down as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let raptor = t.graveyard(P0, "Deathmist Raptor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, bears));
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::TurnFaceUp {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::FaceDown),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
    let now = t.g.current(raptor);
    assert!(t.on_battlefield(now));
    assert!(t.g.obj(now).face_down);
}

// ---------------------------------------------------------------------------
// Manifesting and cloaking particular cards; looking at face-down permanents
// ---------------------------------------------------------------------------

#[test]
fn manifest_and_look_cards_compile() {
    assert_compiles(&[
        "Thieving Amalgam",
        "Orochi Soul-Reaver",
        "Scroll of Fate",
        "Omarthis, Ghostfire Initiate",
        "Vannifar, Evolved Enigma",
        "Smoke Teller",
        "Aven Soulgazer",
        "Revealing Wind",
    ]);
}

#[test]
fn thieving_amalgam_manifests_the_top_card_of_the_opponents_library() {
    cr!("701.40a", "708.5");
    ruling!(
        "Thieving Amalgam",
        "Your opponents can't look at the card they own that you manifested."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thieving Amalgam");
    let top = t.library_top(P1, "Serra Angel");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let now = t.g.current(top);
    assert!(t.on_battlefield(now));
    let o = t.g.obj(now);
    assert!(o.face_down);
    assert_eq!(o.controller, P0);
    assert_eq!(o.owner, P1);
    assert_eq!(t.pt(now), (2, 2));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, now));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, now));
}

#[test]
fn scroll_of_fate_manifests_a_card_from_your_hand() {
    cr!("701.40a");
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "Scroll of Fate");
    let angel = t.hand(P0, "Serra Angel");
    t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(angel)]);
    t.activate(P0, scroll, 0, &[]).unwrap();
    t.resolve_all();
    let now = t.g.current(angel);
    assert!(t.on_battlefield(now));
    assert!(t.g.obj(now).face_down);
    assert_eq!(t.pt(now), (2, 2));
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn omarthis_manifests_as_many_cards_as_it_had_counters() {
    cr!("701.40e", "603.10a");
    let mut t = TestGame::new(2);
    let omarthis = t.battlefield(P0, "Omarthis, Ghostfire Initiate");
    t.g.add_counters(Entity::Object(omarthis), "+1/+1", 3, None);
    t.g.flush_events();
    let before = t.g.permanents().filter(|o| o.face_down).count();
    destroy(&mut t, omarthis);
    t.resolve_all();
    let face_down = t.g.permanents().filter(|o| o.face_down).count();
    assert_eq!(face_down - before, 3);
}

#[test]
fn smoke_teller_looks_at_target_face_down_creature() {
    cr!("708.5");
    let mut t = TestGame::new(2);
    let teller = t.battlefield(P0, "Smoke Teller");
    let theirs = t.battlefield(P1, "Serra Angel");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, theirs));
    t.g.recompute();
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, theirs));
    t.lands(P0, "Island", 2);
    t.activate(P0, teller, 0, &[Entity::Object(theirs)]).unwrap();
    t.resolve_all();
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, theirs));
    // Nothing else changes: it's still a face-down 2/2.
    assert!(t.g.obj(theirs).face_down);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn wall_of_mourning_exiles_a_card_face_down_for_each_opponent() {
    cr!("406.3");
    let mut t = TestGame::new(3);
    let size = t.library_size(P0);
    t.enter(P0, "Wall of Mourning");
    t.resolve_all();
    assert_eq!(t.library_size(P0), size - 2);
    assert_eq!(exiled_face_down(&t).len(), 2);
}

// ---------------------------------------------------------------------------
// More face-down and face-up wordings
// ---------------------------------------------------------------------------

#[test]
fn more_face_cards_compile() {
    assert_compiles(&[
        "Duplicity",
        "Dermoplasm",
        "Cybership",
        "Experimental Lab // Staff Room",
    ]);
}

#[test]
fn duplicity_swaps_your_hand_for_the_other_exiled_cards() {
    cr!("406.3", "607.2a");
    let mut t = TestGame::new(2);
    let top: Vec<ObjectId> = (0..5)
        .map(|_| t.library_top(P0, "Grizzly Bears"))
        .collect();
    t.enter(P0, "Duplicity");
    t.resolve_all();
    assert_eq!(exiled_face_down(&t).len(), 5);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // The five cards are in hand; the Bolt is exiled face down in their place.
    for c in &top {
        assert_eq!(t.zone(t.g.current(*c)), Zone::Hand(P0));
    }
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.g.obj(bolt).face_down);
}

#[test]
fn dermoplasm_puts_a_morph_creature_from_hand_onto_the_battlefield_face_up() {
    cr!("708.8", "702.37e");
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Dermoplasm");
    let phoenix = t.hand(P0, "Ashcloud Phoenix");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, plasm));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(phoenix)]);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::TurnFaceUp {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::FaceDown),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
    let phoenix = t.g.current(phoenix);
    assert!(t.on_battlefield(phoenix));
    assert!(!t.g.obj(phoenix).face_down);
    assert!(t.in_hand(P0, "Dermoplasm"));
}

#[test]
fn staff_room_turns_the_creature_face_up_or_puts_a_counter_on_it() {
    cr!("708.8", "709.5");
    let mut t = TestGame::new(2);
    let room = t.hand(P0, "Experimental Lab // Staff Room");
    t.lands(P0, "Forest", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, room).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, giant));
    t.g.recompute();
    // Choose the first option: turn it face up.
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.g.obj(giant).face_down);
    assert_eq!(t.counters(giant, "+1/+1"), 0);
}
