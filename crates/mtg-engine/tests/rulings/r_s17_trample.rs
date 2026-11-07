//! Rulings batch S17 — trample (the shared rulings of cards with trample): untapping
//! during your upkeep (Black Carriage), casting from a graveyard (The Indomitable), and
//! counting other Aurochs (Bull Aurochs).

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s04_common::next_upkeep;
use crate::r_s08_common::is_tapped;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Taps the permanent as an effect would.
fn tap(t: &mut TestGame, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::Tap {
            what: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

#[test]
fn the_upkeep_untap_ability_can_be_activated_any_number_of_times() {
    cr!("602.5", "502.3");
    ruling!(
        "Black Carriage",
        "There is no restriction on how many times it can be untapped during your upkeep with this ability."
    );
    supported("Black Carriage");
    // Black Carriage: "This creature doesn't untap during your untap step. Sacrifice a
    // creature: Untap this creature. Activate only during your upkeep."
    let mut t = TestGame::new(2);
    let carriage = t.battlefield(P0, "Black Carriage");
    let bears: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    tap(&mut t, carriage);
    // Not during the main phase.
    t.answer_choose(P0, &[Entity::Object(bears[0])]);
    assert!(t.activate(P0, carriage, 0, &[]).is_err());
    t.clear_answers();
    next_upkeep(&mut t, P0);
    // It didn't untap during the untap step.
    assert!(is_tapped(&t, carriage));
    // Untapped, tapped again, and untapped again, in the same upkeep.
    for b in &bears {
        t.answer_choose(P0, &[Entity::Object(*b)]);
        t.activate(P0, carriage, 0, &[]).unwrap();
        t.resolve();
        assert!(!is_tapped(&t, carriage));
        assert!(!t.on_battlefield(*b));
        tap(&mut t, carriage);
    }
    assert_eq!(t.g.turn.step, Step::Upkeep);
}

#[test]
fn a_spell_cast_from_a_graveyard_moves_to_the_stack_as_casting_begins() {
    cr!("601.2", "601.2a", "601.2i", "603.3");
    ruling!(
        "The Indomitable",
        "Once you begin casting a spell from your graveyard, it immediately moves to the stack. Players can't take any actions until you've finished casting the spell."
    );
    supported("The Indomitable");
    supported("Fang, Fearless l'Cie");
    // The Indomitable: "You may cast this card from your graveyard as long as you control
    // three or more tapped Pirates and/or Vehicles." Fang: "Whenever one or more cards
    // leave your graveyard, you draw a card and you lose 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fang, Fearless l'Cie");
    for _ in 0..3 {
        let pirate = create_token(&mut t, P0, "Pirate");
        tap(&mut t, pirate);
    }
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    let card = t.graveyard(P0, "The Indomitable");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    let spell = t.cast(P0, card).go();
    // No player got priority while it was being cast.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    // It left the graveyard as casting began: Fang's ability triggered, and it's put on
    // the stack only once casting is done — above the spell.
    assert_eq!(t.zone(spell), Zone::Stack);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], spell);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

#[test]
fn aurochs_abilities_count_every_other_creature_with_the_creature_type_aurochs() {
    cr!("205.3m", "702.73a", "508.1m");
    ruling!(
        "Bull Aurochs",
        "Abilities of Aurochs care about other creatures that have creature type Aurochs"
    );
    supported("Bull Aurochs");
    supported("Changeling Outcast");
    // Bull Aurochs: "Whenever this creature attacks, it gets +1/+0 until end of turn for
    // each other attacking Aurochs." Another Bull Aurochs and a changeling count; a Bear
    // doesn't.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Bull Aurochs");
    let b = t.battlefield(P0, "Bull Aurochs");
    let changeling = t.battlefield(P0, "Changeling Outcast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let (p, _) = t.pt(a);
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (changeling, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.pt(a).0, p + 2);
    assert_eq!(t.pt(b).0, p + 2);
    // Alone, it doesn't get a bonus.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Bull Aurochs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(a).0, p);
}
