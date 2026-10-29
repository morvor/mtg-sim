//! Rulings batch P188 — cards in graveyards and libraries: "[cards] in graveyards [and
//! libraries] can't enter the battlefield" checks the card as it exists in that zone
//! (Kunoros, Hound of Athreos; Grafdigger's Cage; Weathered Runestone; Soulless Jailer),
//! their "can't cast spells from [zones]" halves, and playing lands from a graveyard with
//! Ramunap Excavator (CR 305.1, 305.2).

use crate::r_s01_common::supported;
use crate::r_s28_common::cast_card;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` casts `name` targeting `target` (lands for it are added) and it resolves.
fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    t.answer_targets(p, &[o(target)]);
    cast_card(t, p, name);
    t.resolve_all();
}

/// Whether `p` may cast Think Twice from their graveyard with flashback now.
fn can_flashback_think_twice(t: &mut TestGame, p: PlayerId) -> bool {
    let tt = t.graveyard(p, "Think Twice");
    t.lands(p, "Island", 3);
    t.cast(p, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .try_go()
        .is_ok()
}

fn can_play_land(t: &mut TestGame, p: PlayerId, land: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::PlayLand { card: land })
}

// ---------------------------------------------------------------------------------------
// Kunoros, Hound of Athreos
// ---------------------------------------------------------------------------------------

#[test]
fn kunoros_checks_the_card_as_it_exists_in_the_graveyard() {
    cr!("614.17", "700.5", "707.2");
    ruling!(
        "Kunoros, Hound of Athreos",
        "Look at the card as it exists in your graveyard to determine whether it can enter the battlefield. For example, Sculpting Steel (a noncreature card in the graveyard) can be put onto the battlefield as a copy of a creature, but Phyrexian Metamorph (a creature card in the graveyard) can’t be put onto the battlefield, even if it would copy a noncreature artifact. A Theros Beyond Death God creature card can’t be put onto the battlefield regardless of your devotion to its colors."
    );
    supported("Kunoros, Hound of Athreos");
    supported("Argivian Restoration");
    supported("Zombify");
    for kunoros in [false, true] {
        // Sculpting Steel enters as a copy of Ornithopter (an artifact creature).
        let mut t = TestGame::new(2);
        if kunoros {
            t.battlefield(P1, "Kunoros, Hound of Athreos");
        }
        let thopter = t.battlefield(P1, "Ornithopter");
        let steel = t.graveyard(P0, "Sculpting Steel");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(thopter)]);
        cast_at(&mut t, P0, "Argivian Restoration", steel);
        let s = t.g.current(steel);
        assert!(t.on_battlefield(s), "kunoros: {kunoros}");
        assert!(t.obj(s).is(CardType::Creature));
        assert_eq!(t.obj(s).chars.name, "Ornithopter");

        // Phyrexian Metamorph would copy Sol Ring (a noncreature artifact).
        let mut t = TestGame::new(2);
        if kunoros {
            t.battlefield(P1, "Kunoros, Hound of Athreos");
        }
        let ring = t.battlefield(P1, "Sol Ring");
        let meta = t.graveyard(P0, "Phyrexian Metamorph");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(ring)]);
        cast_at(&mut t, P0, "Argivian Restoration", meta);
        assert_eq!(t.on_battlefield(meta), !kunoros);
        if kunoros {
            assert!(t.in_graveyard(P0, "Phyrexian Metamorph"));
        }

        // Heliod, Sun-Crowned: a God creature card in the graveyard.
        let mut t = TestGame::new(2);
        if kunoros {
            t.battlefield(P1, "Kunoros, Hound of Athreos");
        }
        let heliod = t.graveyard(P0, "Heliod, Sun-Crowned");
        cast_at(&mut t, P0, "Zombify", heliod);
        assert_eq!(t.on_battlefield(heliod), !kunoros);
        if !kunoros {
            // With no devotion it isn't a creature on the battlefield.
            assert!(!t.obj_now(heliod).is(CardType::Creature));
        }
    }
}

#[test]
fn kunoros_lets_a_card_exiled_from_a_graveyard_be_cast_from_exile() {
    cr!("601.2", "601.3");
    ruling!(
        "Kunoros, Hound of Athreos",
        "If an effect exiles a card from a graveyard and allows a player to cast it, that player may do so. The spell is cast from exile, not a graveyard."
    );
    supported("Practiced Scrollsmith");
    supported("Think Twice");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Kunoros, Hound of Athreos");
    // Casting from a graveyard is forbidden.
    assert!(!can_flashback_think_twice(&mut t, P0));
    // Practiced Scrollsmith: "When this creature enters, exile target noncreature, nonland
    // card from your graveyard. Until the end of your next turn, you may cast that card."
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[o(bolt)]);
    t.enter(P0, "Practiced Scrollsmith");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let exiled = t.g.current(bolt);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, exiled).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn kunoros_and_grafdigger_cage_let_lands_be_played_from_graveyards() {
    cr!("305.1", "305.2");
    ruling!(
        "Kunoros, Hound of Athreos",
        "Players can still play lands from graveyards if an effect allows them to do so."
    );
    ruling!(
        "Grafdigger's Cage",
        "Players can still play lands from graveyards or libraries if an effect allows them to do so."
    );
    ruling!(
        "Weathered Runestone",
        "Players can still play lands from graveyards if an effect allows them to."
    );
    supported("Ramunap Excavator");
    supported("Future Sight");
    for hoser in [
        "Kunoros, Hound of Athreos",
        "Grafdigger's Cage",
        "Weathered Runestone",
    ] {
        supported(hoser);
        let mut t = TestGame::new(2);
        t.battlefield(P1, hoser);
        t.battlefield(P0, "Ramunap Excavator");
        let forest = t.graveyard(P0, "Forest");
        assert!(can_play_land(&mut t, P0, forest), "{hoser}");
        t.play_land(P0, forest).expect("play from graveyard");
        assert!(t.on_battlefield(forest), "{hoser}");
    }
    // And from the top of the library with Future Sight.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grafdigger's Cage");
    t.battlefield(P0, "Future Sight");
    let island = t.library_top(P0, "Island");
    t.play_land(P0, island).expect("play from library");
    assert!(t.on_battlefield(island));
}

// ---------------------------------------------------------------------------------------
// Grafdigger's Cage, Weathered Runestone, Soulless Jailer
// ---------------------------------------------------------------------------------------

#[test]
fn grafdiggers_cage_stops_creature_cards_from_graveyards_and_libraries() {
    cr!("614.17", "601.3");
    ruling!(
        "Grafdigger's Cage",
        "Look at the card as it exists in your graveyard to determine whether it can enter the battlefield. For example, Sculpting Steel can be put onto the battlefield as a copy of a creature, but Phyrexian Metamorph can't be put onto the battlefield, even if it would copy a noncreature artifact."
    );
    supported("Grafdigger's Cage");
    supported("Natural Order");
    // "Creature cards in graveyards and libraries can't enter the battlefield. Players
    // can't cast spells from graveyards or libraries."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grafdigger's Cage");
    let thopter = t.battlefield(P1, "Ornithopter");
    let steel = t.graveyard(P0, "Sculpting Steel");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(thopter)]);
    cast_at(&mut t, P0, "Argivian Restoration", steel);
    assert!(t.on_battlefield(steel));
    let ring = t.battlefield(P1, "Sol Ring");
    let meta = t.graveyard(P0, "Phyrexian Metamorph");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(ring)]);
    cast_at(&mut t, P0, "Argivian Restoration", meta);
    assert!(t.in_graveyard(P0, "Phyrexian Metamorph"));
    // A creature card in a library: Natural Order finds it, but it can't enter.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.library_top(P0, "Craw Wurm");
    t.answer_choose(P0, &[o(elves)]);
    t.answer_choose(P0, &[o(giant)]);
    cast_card(&mut t, P0, "Natural Order");
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    assert!(!t.on_battlefield(elves));
    // Spells can't be cast from a graveyard or a library.
    assert!(!can_flashback_think_twice(&mut t, P0));
    t.battlefield(P0, "Future Sight");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    assert!(t.cast(P0, bolt).target(Entity::Player(P1)).try_go().is_err());
}

#[test]
fn weathered_runestone_stops_nonland_permanent_cards_and_casting_from_those_zones() {
    cr!("614.17", "601.3");
    ruling!(
        "Weathered Runestone",
        "Look at the card as it exists in your graveyard or library to determine whether it can enter the battlefield."
    );
    supported("Weathered Runestone");
    // "Nonland permanent cards in graveyards and libraries can't enter the battlefield.
    // Players can't cast spells from graveyards or libraries."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Weathered Runestone");
    let ring = t.graveyard(P0, "Sol Ring");
    cast_at(&mut t, P0, "Argivian Restoration", ring);
    assert!(t.in_graveyard(P0, "Sol Ring"));
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.library_top(P0, "Craw Wurm");
    let wurm = t.g.library_top(P0).unwrap();
    t.answer_choose(P0, &[o(elves)]);
    t.answer_choose(P0, &[o(wurm)]);
    cast_card(&mut t, P0, "Natural Order");
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    assert!(!can_flashback_think_twice(&mut t, P0));
    // Without it, Argivian Restoration returns the Sol Ring.
    let mut t = TestGame::new(2);
    let ring = t.graveyard(P0, "Sol Ring");
    cast_at(&mut t, P0, "Argivian Restoration", ring);
    assert!(t.on_battlefield(ring));
    // Sculpting Steel (a nonland card in the graveyard) can't enter even as a copy of an
    // artifact land, which it would be on the battlefield; without the Runestone it can.
    for runestone in [true, false] {
        let mut t = TestGame::new(2);
        if runestone {
            t.battlefield(P1, "Weathered Runestone");
        }
        let seat = t.battlefield(P1, "Seat of the Synod");
        let steel = t.graveyard(P0, "Sculpting Steel");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(seat)]);
        cast_at(&mut t, P0, "Argivian Restoration", steel);
        let s = t.g.current(steel);
        assert_eq!(t.on_battlefield(s), !runestone);
        if !runestone {
            assert!(t.obj(s).is(CardType::Land));
        }
    }
}

#[test]
fn soulless_jailer_stops_permanent_cards_in_graveyards_and_noncreature_spells() {
    cr!("614.17", "601.2");
    ruling!(
        "Soulless Jailer",
        "Putting a permanent card onto the battlefield from a graveyard is an impossible action while Soulless Jailer is on the battlefield."
    );
    ruling!(
        "Soulless Jailer",
        "Players may still cast creature spells from graveyards and from exile if an effect allows them to do so."
    );
    supported("Soulless Jailer");
    supported("Gravecrawler");
    supported("Bonecrusher Giant // Stomp");
    // "Permanent cards in graveyards can't enter the battlefield. Players can't cast
    // noncreature spells from graveyards or exile."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soulless Jailer");
    let bears = t.graveyard(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Zombify", bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let ring = t.graveyard(P0, "Sol Ring");
    cast_at(&mut t, P0, "Argivian Restoration", ring);
    assert!(t.in_graveyard(P0, "Sol Ring"));
    // A noncreature spell can't be cast from a graveyard...
    assert!(!can_flashback_think_twice(&mut t, P0));
    // ...but a creature spell can: Gravecrawler ("You may cast this card from your
    // graveyard as long as you control a Zombie").
    t.battlefield(P0, "Walking Corpse");
    let crawler = t.graveyard(P0, "Gravecrawler");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, crawler).go();
    t.resolve_all();
    assert!(t.on_battlefield(crawler));
    // And from exile: Bonecrusher Giant after its Adventure.
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    let exiled = t.g.current(giant);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.lands(P0, "Mountain", 3);
    t.cast(P0, exiled).go();
    t.resolve_all();
    assert!(t.on_battlefield(exiled));
}

// ---------------------------------------------------------------------------------------
// Ramunap Excavator
// ---------------------------------------------------------------------------------------

#[test]
fn ramunap_excavator_doesnt_allow_activating_abilities_of_lands_in_graveyards() {
    cr!("305.1", "702.29a");
    ruling!(
        "Ramunap Excavator",
        "Ramunap Excavator doesn't allow you to activate activated abilities (such as cycling) of land cards in your graveyard."
    );
    supported("Ramunap Excavator");
    supported("Desert of the Fervent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ramunap Excavator");
    t.lands(P0, "Mountain", 3);
    let desert = t.graveyard(P0, "Desert of the Fervent");
    t.g.turn.priority = Some(P0);
    let actions = t.g.legal_actions(P0);
    assert!(actions.contains(&Action::PlayLand { card: desert }));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == desert)));
}

#[test]
fn ramunap_excavator_doesnt_change_when_lands_can_be_played() {
    cr!("305.1", "305.2");
    ruling!(
        "Ramunap Excavator",
        "Ramunap Excavator doesn't change the times when you can play those land cards. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ramunap Excavator");
    let forest = t.graveyard(P0, "Forest");
    // Not while the stack isn't empty.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, forest));
    // Not in combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    // Not during the opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(!t
        .g
        .legal_actions(P0)
        .contains(&Action::PlayLand { card: forest }));
    // Only one land per turn: after a land from hand, not this one.
    t.set_step(P0, Step::PrecombatMain);
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).expect("land from hand");
    assert!(!can_play_land(&mut t, P0, forest));
}

#[test]
fn grafdiggers_cage_makes_manifesting_from_a_library_impossible() {
    cr!("701.40a", "701.40f", "708.3");
    ruling!(
        "Grafdigger's Cage",
        "Manifesting a card from a graveyard or library is an impossible action while Grafdigger's Cage is on the battlefield."
    );
    supported("Grafdigger's Cage");
    supported("Soul Summons");
    // The card is turned face down before it moves: a 2/2 face-down creature card in the
    // library, even if it's a noncreature card face up. Kunoros (graveyards only) doesn't
    // stop it.
    for (blocker, stopped) in [
        (None, false),
        (Some("Grafdigger's Cage"), true),
        (Some("Kunoros, Hound of Athreos"), false),
    ] {
        for top in ["Lightning Bolt", "Hill Giant"] {
            let mut t = TestGame::new(2);
            if let Some(b) = blocker {
                t.battlefield(P1, b);
            }
            let card = t.library_top(P0, top);
            t.lands(P0, "Plains", 2);
            let spell = t.hand(P0, "Soul Summons");
            t.cast(P0, spell).go();
            t.resolve_all();
            let c = t.g.current(card);
            let msg = format!("{blocker:?} {top}");
            if stopped {
                // CR 701.40f: it isn't manifested; it stays in the library, face up.
                assert_eq!(t.zone(c), Zone::Library(P0), "{msg}");
                assert!(!t.obj(c).face_down, "{msg}");
                assert_eq!(t.obj(c).chars.name, top, "{msg}");
            } else {
                assert!(t.on_battlefield(c), "{msg}");
                assert!(t.obj(c).face_down, "{msg}");
            }
        }
    }
}
