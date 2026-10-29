//! Illusionary Mask (hand-written, `src/cards/illusionary_mask.rs`): cast a creature card
//! face down for the mana spent on {X}; the creature is turned face up instead as it would
//! deal or be dealt damage or become tapped.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the Mask with X paid by `lands`, choosing `card` and casting it face down;
/// returns the face-down permanent.
fn mask(t: &mut TestGame, lands: &str, n: usize, card: ObjectId) -> Option<ObjectId> {
    let m = t.battlefield(P0, "Illusionary Mask");
    t.lands(P0, lands, n);
    t.answer(P0, DecisionKind::X, Answer::Number(n as i64));
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.answer_yes(P0, true);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    t.g.battlefield.iter().copied().find(|o| t.obj(*o).face_down)
}

#[test]
fn a_creature_cast_face_down_is_a_2_2_and_turns_up_when_it_becomes_tapped() {
    cr!("708.2a", "614.1a");
    ruling!("Illusionary Mask", "it's a 2/2 creature with no name, mana cost, color");
    ruling!("Illusionary Mask", "The effect that turns it face-up is a replacement effect.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.hand(P0, "Hill Giant");
    let fd = mask(&mut t, "Mountain", 4, giant).expect("a face-down creature");
    assert_eq!(t.pt(fd), (2, 2));
    assert!(t.obj_now(fd).chars.name.is_empty());
    // It becomes tapped (attacking): it's turned face up first.
    t.g.tap(fd);
    t.g.recompute();
    assert!(!t.obj_now(fd).face_down);
    assert!(t.obj_now(fd).tapped);
    assert_eq!(t.obj_now(fd).chars.name, "Hill Giant");
    assert_eq!(t.pt(fd), (3, 3));
}

#[test]
fn it_turns_face_up_before_it_is_dealt_damage() {
    cr!("614.1a", "708.8");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.hand(P0, "Hill Giant");
    let fd = mask(&mut t, "Mountain", 4, giant).expect("a face-down creature");
    // 2 damage would kill a 2/2; the Hill Giant survives it.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(fd).go();
    t.resolve_all();
    assert!(t.on_battlefield(fd));
    assert!(!t.obj_now(fd).face_down);
    assert_eq!(t.obj_now(fd).damage, 2);
}

#[test]
fn the_card_must_be_payable_by_the_amount_and_types_spent() {
    cr!("708.4");
    ruling!("Illusionary Mask", "Both the amount and types of mana you spend on {X} are taken into account");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // {R}{R} can't pay for {1}{G} (Grizzly Bears): nothing to choose.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(mask(&mut t, "Mountain", 2, bears).is_none());
    assert!(t.g.player(P0).hand.contains(&bears));
    // {G}{G} can: it's cast face down and becomes a face-down 2/2.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    let fd = mask(&mut t, "Forest", 2, bears).expect("a face-down creature");
    assert_eq!(t.pt(fd), (2, 2));
    assert!(t.obj_now(fd).face_down);
}

#[test]
fn it_turns_face_up_before_assigning_combat_damage_even_after_the_mask_is_gone() {
    cr!("510.1", "614.1a");
    ruling!("Illusionary Mask", "Illusionary Mask's ability will continue to apply to creatures cast face down with it, even if Illusionary Mask has left the battlefield.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.hand(P0, "Craw Wurm");
    let fd = mask(&mut t, "Forest", 6, wurm).expect("a face-down creature");
    let m = t.named_on_battlefield("Illusionary Mask")[0];
    t.g.destroy(m, None);
    t.settle();
    assert!(t.named_on_battlefield("Illusionary Mask").is_empty());
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::BeginningOfCombat);
    // Blocking doesn't tap it: it's still face down when combat damage is assigned.
    t.attack(&[(giant, Entity::Player(P0))], &[(fd, giant)]);
    // Turned face up first, it assigns the Craw Wurm's 6 damage (not a 2/2's 2) and
    // survives the Giant's 3.
    assert!(!t.obj_now(fd).face_down);
    assert_eq!(t.obj_now(fd).chars.name, "Craw Wurm");
    assert!(t.on_battlefield(fd));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn the_face_down_spell_can_be_countered() {
    cr!("708.4", "701.6a");
    ruling!("Illusionary Mask", "You actually cast the card face-down, much as when playing a spell with Morph. It can be responded to and countered.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.hand(P0, "Hill Giant");
    let m = t.battlefield(P0, "Illusionary Mask");
    t.lands(P0, "Mountain", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve();
    // The ability has resolved; the face-down spell is on the stack.
    assert_eq!(t.stack_len(), 1);
    let spell = t.g.current(giant);
    assert!(t.obj_now(spell).face_down);
    assert_eq!(t.pt(spell), (2, 2));
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.g.battlefield.iter().all(|o| !t.obj(*o).face_down));
}

#[test]
fn an_ordinary_face_down_creature_isnt_turned_face_up() {
    cr!("708.2a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Illusionary Mask");
    t.lands(P0, "Island", 3);
    let egotist = t.hand(P0, "Scornful Egotist");
    t.cast(P0, egotist)
        .method(object::CastMethod::FaceDown(keywords::KeywordKind::Morph))
        .go();
    t.resolve();
    let fd = t.g.current(egotist);
    assert!(t.obj_now(fd).face_down);
    t.g.tap(fd);
    t.g.recompute();
    assert!(t.obj_now(fd).face_down && t.obj_now(fd).tapped);
}
