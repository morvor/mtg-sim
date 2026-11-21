//! Rulings batch P086 — face-down and exiled cards: Keeper of the Lens, Lens of Clarity,
//! Lumbering Laundry, Nosy Goblin, Time Reaper, Memory Theft, Thraben Exorcism.

use crate::r_p086_common::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::facedown::can_look_at;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts Hystrodon face down (morph, for {3}); it's left on the stack.
fn morph_spell(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, "Hystrodon");
    let id = t
        .cast(p, card)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    assert!(t.g.obj(id).face_down);
    id
}

/// `p` controls a face-down Hystrodon (cast face down; it resolved).
fn morph(t: &mut TestGame, p: PlayerId) -> ObjectId {
    if t.g.turn.active != p {
        t.set_step(p, Step::PrecombatMain);
    }
    morph_spell(t, p);
    t.resolve_all();
    let id =
        t.g.battlefield
            .iter()
            .copied()
            .find(|o| t.g.obj(*o).face_down && t.g.obj(*o).controller == p)
            .expect("a face-down creature");
    id
}

#[test]
fn look_at_face_down_creatures_any_time_not_spells_not_stopping_their_controller() {
    cr!("708.5");
    ruling!(
        "Keeper of the Lens",
        "Keeper of the Lens allows you to look at face-down creatures you don’t control whenever you want to, even if you don’t have priority. This action doesn’t use the stack."
    );
    ruling!(
        "Keeper of the Lens",
        "Keeper of the Lens doesn’t let you look at other players’ face-down spells while they’re on the stack."
    );
    ruling!(
        "Keeper of the Lens",
        "Keeper of the Lens doesn’t stop your opponents from looking at face-down creatures they control."
    );
    ruling!(
        "Lens of Clarity",
        "Lens of Clarity doesn’t let you look at face-down spells you don’t control on the stack."
    );
    ruling!(
        "Lens of Clarity",
        "Lens of Clarity doesn’t stop your opponents from looking at face-down creatures they control."
    );
    ruling!(
        "Lens of Clarity",
        "Lens of Clarity lets you look at the top card of your library and at face-down creatures you don’t control whenever you want, even if you don’t have priority. This action doesn’t use the stack. Knowing what those cards are becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    for name in ["Keeper of the Lens", "Lens of Clarity"] {
        supported(name);
        let mut t = TestGame::new(2);
        let fd = morph(&mut t, P1);
        assert!(!can_look_at(&t.g, P0, fd), "{name}");
        t.battlefield(P0, name);
        // P1 holds priority with a spell on the stack: P0 can still look. (Looking is what
        // the player may see, not an action: nothing goes on the stack.)
        let spell = morph_spell(&mut t, P1);
        t.g.turn.priority = Some(P1);
        assert!(can_look_at(&t.g, P0, fd), "{name}");
        // Not the face-down spell.
        assert!(!can_look_at(&t.g, P0, spell), "{name}");
        // Its controller still can.
        assert!(can_look_at(&t.g, P1, fd), "{name}");
        assert!(can_look_at(&t.g, P1, spell), "{name}");
        let top = t.g.library_top(P0).expect("library");
        assert_eq!(
            can_look_at(&t.g, P0, top),
            name == "Lens of Clarity",
            "{name}"
        );
    }
}

#[test]
fn lumbering_laundry_until_end_of_turn_not_spells() {
    cr!("708.5", "611.2a", "514.2");
    ruling!(
        "Lumbering Laundry",
        "Lumbering Laundry's first ability doesn't let you look at face-down spells you don't control on the stack."
    );
    ruling!(
        "Lumbering Laundry",
        "Once it has resolved, Lumbering Laundry's first ability lets you look at face-down creatures whenever you want until end of turn, even if you don't have priority. This action doesn't use the stack. Knowing what those cards are becomes part of the information you have access to, just like you can look at the cards in your hand. Once the turn ends, however, you can no longer look at those creatures."
    );
    supported("Lumbering Laundry");
    let mut t = TestGame::new(2);
    let fd = morph(&mut t, P1);
    let ll = t.battlefield(P0, "Lumbering Laundry");
    t.lands(P0, "Wastes", 2);
    activate(&mut t, P0, ll, "look at", &[]);
    assert!(!can_look_at(&t.g, P0, fd));
    t.resolve_all();
    let spell = morph_spell(&mut t, P1);
    t.g.turn.priority = Some(P1);
    assert!(can_look_at(&t.g, P0, fd));
    assert!(!can_look_at(&t.g, P0, spell));
    t.resolve_all();
    next_main(&mut t, P0);
    assert!(!can_look_at(&t.g, P0, fd));
}

#[test]
fn nosy_goblins_target_turned_face_up_is_no_longer_legal() {
    cr!("608.2b", "708.8");
    ruling!(
        "Nosy Goblin",
        "If the target is turned face up before the ability resolves, the creature is no longer a legal target and is not destroyed."
    );
    supported("Nosy Goblin");
    let mut t = TestGame::new(2);
    let fd = morph(&mut t, P1);
    t.set_step(P0, Step::PrecombatMain);
    let goblin = t.battlefield(P0, "Nosy Goblin");
    activate(&mut t, P0, goblin, "Destroy", &[obj(fd)]);
    // P1 turns it face up in response (morph {1}{G}{G}).
    t.lands(P1, "Forest", 3);
    t.g.turn.priority = Some(P1);
    t.g.take_action(P1, Action::Special(SpecialAction::TurnFaceUp { obj: fd }));
    assert!(!t.obj_now(fd).face_down);
    t.resolve_all();
    assert!(t.on_battlefield(fd));
}

#[test]
fn time_reaper_targets_face_up_exiled_cards_only() {
    cr!("406.3", "115.1");
    ruling!(
        "Time Reaper",
        "Most cards in exile are face up and can be targeted by Time Reaper's triggered ability. A card is face down in the exile zone only if the effect or action that put it there specifically says it puts it there face down (like the foretell ability)."
    );
    supported("Time Reaper");
    let mut t = TestGame::new(2);
    let reaper = t.battlefield(P0, "Time Reaper");
    let up = t.exile(P1, "Grizzly Bears");
    let down = t.exile(P1, "Raging Goblin");
    t.g.objects[down.0 as usize].face_down = true;
    let from = t.asked().len();
    attack_with(&mut t, &[(reaper, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(up)));
    assert!(!offered[0].contains(&obj(down)));
    t.resolve_all();
    assert_eq!(t.zone(up), Zone::Library(P1));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn memory_theft_any_face_up_exiled_adventurer_card() {
    cr!("715.1", "406.3");
    ruling!(
        "Memory Theft",
        "Memory Theft allows you to put any face-up exiled card with an Adventure that player owns into their graveyard, not just one that was cast as an Adventure and exiled."
    );
    supported("Memory Theft");
    let mut t = TestGame::new(2);
    // Exiled some other way, not cast as an Adventure.
    let giant = t.exile(P1, "Bonecrusher Giant // Stomp");
    let bears = t.hand(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(giant)]);
    cast_new(&mut t, P0, "Memory Theft", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Bonecrusher Giant"), "{}", t.dump_log());
}

#[test]
fn thraben_exorcism_disturb_means_the_keyword_in_the_text_box() {
    cr!("702.146a", "115.1");
    ruling!(
        "Thraben Exorcism",
        "A creature with disturb is one that has the disturb keyword ability in its text box. Notably, permanents that were cast using the disturb ability rarely end up having disturb on the battlefield. Barring some very weird circumstances with copy effects, if a card's front face has disturb but its back face is on the battlefield, that permanent doesn't have disturb."
    );
    supported("Thraben Exorcism");
    let mut t = TestGame::new(2);
    // Beloved Beggar (a Human Peasant with disturb) can be targeted; Grizzly Bears can't.
    let beggar = t.battlefield(P1, "Beloved Beggar // Generous Soul");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let offered = spell_targets(&mut t, P0, "Thraben Exorcism");
    assert!(offered.contains(&obj(beggar)));
    assert!(!offered.contains(&obj(bears)));
    // Cast with disturb, its back face (Generous Soul, a Spirit) doesn't have disturb.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Beloved Beggar // Generous Soul");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 6);
    t.cast(P1, card)
        .method(CastMethod::Keyword(KeywordKind::Disturb))
        .go();
    t.resolve_all();
    let soul = t.g.current(card);
    assert!(t.on_battlefield(soul));
    assert!(!crate::r_s06_common::has_kw(&t, soul, KeywordKind::Disturb));
}
