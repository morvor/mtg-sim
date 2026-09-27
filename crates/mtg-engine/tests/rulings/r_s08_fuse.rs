//! Rulings batch S08 — fuse (CR 702.102): "You may cast one or both halves of this card
//! from your hand." A split card's characteristics (CR 709.3, 709.4), fused split spells
//! (CR 702.102c, 702.102d), and naming split cards (CR 709.4a).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::*;
use crate::r_s05_common::*;
use crate::r_s08_common::*;
use mtg_engine::ability::{Duration, Filter};
use mtg_engine::casting::PlayGrant;
use mtg_engine::decision::Answer;
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FUSED: CastMethod = CastMethod::Keyword(KeywordKind::Fuse);

#[test]
fn both_halves_of_a_fused_spell_can_target_the_same_creature() {
    cr!("702.102d", "115.3");
    ruling!(
        "Protect // Serve",
        "You can choose the same object as the target of each half of a fused split spell, if appropriate."
    );
    supported("Protect // Serve");
    // Protect: "Target creature gets +2/+4 until end of turn." Serve: "Target creature
    // gets -6/-0 until end of turn."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Protect // Serve");
    t.cast(P0, card).method(FUSED).target(wurm).target(wurm).go();
    t.resolve_all();
    // 6/4 + 2/4 - 6/0.
    assert_eq!(t.pt(wurm), (2, 8));
}

#[test]
fn a_split_card_has_the_chosen_name_if_either_half_has_it() {
    cr!("709.4a", "702.102c");
    ruling!(
        "Wear // Tear",
        "If a player names a card, the player may name either half of a split card, but not both. A split card has the chosen name if one of its two names matches the chosen name."
    );
    supported("Wear // Tear");
    supported("Meddling Mage");
    // Meddling Mage: "As this creature enters, choose a nonland card name. Spells with the
    // chosen name can't be cast."
    let mut t = TestGame::new(2);
    let card = t.hand(P1, "Wear // Tear");
    t.g.recompute();
    let ctx = Ctx::new(None, P0);
    assert!(t.g.matches(card, &Filter::Named("Wear".into()), &ctx));
    assert!(t.g.matches(card, &Filter::Named("Tear".into()), &ctx));
    // Both names at once aren't a legal choice.
    t.answer(P0, DecisionKind::Name, Answer::Text("Wear // Tear".into()));
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some(""));
    t.answer(P0, DecisionKind::Name, Answer::Text("Wear".into()));
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some("Wear"));
    // Wear can't be cast, nor the fused spell (it has both names); Tear can.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Plains", 1);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Glorious Anthem");
    assert!(!can_cast(&mut t, P1, card, CastMethod::Half(0)));
    assert!(!can_cast(&mut t, P1, card, FUSED));
    assert!(can_cast(&mut t, P1, card, CastMethod::Half(1)));
}

#[test]
fn a_split_card_with_two_multicolored_halves_is_always_multicolored() {
    cr!("709.3b", "709.4b", "709.4d", "105.2b");
    ruling!(
        "Ready // Willing",
        "Some split cards with fuse have two halves that are both multicolored. That card is multicolored no matter which half is cast, or if both halves are cast. It's also multicolored while not on the stack."
    );
    ruling!(
        "Alive // Well",
        "Some split cards with fuse have two halves that are both multicolored. That card is multicolored no matter which half is cast, or if both halves are cast. It’s also multicolored while not on the stack."
    );
    supported("Ready // Willing");
    // Ready {1}{G}{W} // Willing {1}{W}{B}.
    let gw = ColorSet::single(Color::Green).union(ColorSet::single(Color::White));
    let wb = ColorSet::single(Color::White).union(ColorSet::single(Color::Black));
    for (method, colors) in [
        (CastMethod::Half(0), gw),
        (CastMethod::Half(1), wb),
        (FUSED, gw.union(wb)),
    ] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Plains", 2);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Wastes", 2);
        let card = t.hand(P0, "Ready // Willing");
        t.g.recompute();
        assert_eq!(colors_of(&t, card), gw.union(wb));
        let spell = t.cast(P0, card).method(method.clone()).go();
        assert_eq!(colors_of(&t, spell), colors, "{method:?}");
        assert!(colors_of(&t, spell).is_multicolored());
        t.resolve_all();
        let gy = t.g.player(P0).graveyard[0];
        assert!(colors_of(&t, gy).is_multicolored());
    }
}

fn colors_of(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj(id).chars.colors
}

#[test]
fn an_unfused_half_on_the_stack_has_only_that_halfs_characteristics() {
    cr!("709.3b", "709.4", "709.4b");
    ruling!(
        "Wear // Tear",
        "On the stack, a split spell that hasn't been fused has only that half's characteristics and mana value. The other half is treated as though it didn't exist."
    );
    ruling!(
        "Armed // Dangerous",
        "On the stack, a split spell that hasn’t been fused has only that half’s characteristics and mana value. The other half is treated as though it didn’t exist."
    );
    supported("Armed // Dangerous");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Armed // Dangerous");
    t.g.recompute();
    // In the hand: both halves, mana value 6.
    assert_eq!(mana_value(&t, card), 6);
    assert!(t.obj(card).chars.colors.contains(Color::Green));
    let spell = t
        .cast(P0, card)
        .method(CastMethod::Half(0))
        .target(bears)
        .go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Armed");
    assert!(!c.has_name("Dangerous"));
    assert_eq!(c.colors, ColorSet::single(Color::Red));
    assert_eq!(mana_value(&t, spell), 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    // Wear, on the stack: a red instant with mana value 2 that isn't named Tear.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Wear // Tear");
    let spell = t
        .cast(P0, card)
        .method(CastMethod::Half(0))
        .target(thopter)
        .go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Wear");
    assert!(!c.has_name("Tear"));
    assert_eq!(c.colors, ColorSet::single(Color::Red));
    assert_eq!(mana_value(&t, spell), 2);
}

#[test]
fn a_fused_spell_with_one_illegal_target_still_resolves_for_the_other() {
    cr!("702.102d", "608.2b");
    ruling!(
        "Armed // Dangerous",
        "When resolving a fused split spell with multiple targets, treat it as you would any spell with multiple targets. If all targets are illegal when the spell tries to resolve, the spell doesn’t resolve and none of its effects happen."
    );
    // Armed ("Target creature gets +1/+1 and gains double strike until end of turn.") on
    // P0's Grizzly Bears and Dangerous ("All creatures able to block target creature this
    // turn do so.") on P0's Hill Giant; the Bears die before it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Armed // Dangerous");
    let spell = t.cast(P0, card).method(FUSED).target(bears).target(giant).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(resolved_spell(&t, spell));
    // Armed did nothing to the Giant; Dangerous applies: the Elves must block it.
    assert_eq!(t.pt(giant), (3, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(elves));
    // Both targets illegal: nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Armed // Dangerous");
    let spell = t.cast(P0, card).method(FUSED).target(bears).target(giant).go();
    destroy(&mut t, bears);
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(!resolved_spell(&t, spell));
    assert!(t.in_graveyard(P0, "Armed // Dangerous"));
}

/// Whether the spell `spell` resolved (it wasn't countered on resolution).
fn resolved_spell(t: &TestGame, spell: ObjectId) -> bool {
    crate::r_s07_common::resolved(t, spell)
}

#[test]
fn a_fuse_card_cast_from_outside_the_hand_is_cast_as_one_half() {
    cr!("702.102a");
    ruling!(
        "Alive // Well",
        "If you’re casting a split card with fuse from any zone other than your hand, you can’t cast both halves. You’ll only be able to cast one half or the other."
    );
    supported("Alive // Well");
    // Alive {3}{G}: "Create a 3/3 green Centaur creature token." Well {W}: "You gain 2
    // life for each creature you control."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.exile(P0, "Alive // Well");
    t.g.play_grants.push(PlayGrant {
        player: P0,
        object: card,
        duration: Duration::EndOfTurn,
        free: false,
        source: None,
        turn: 1,
    });
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(0)));
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(1)));
    assert!(!can_cast(&mut t, P0, card, FUSED));
    assert!(t.cast(P0, card).method(FUSED).try_go().is_err());
    // From the hand, it can be fused.
    let card = t.hand(P0, "Alive // Well");
    assert!(can_cast(&mut t, P0, card, FUSED));
    t.cast(P0, card).method(FUSED).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Centaur").len(), 1);
    assert_eq!(t.life(P0), 22);
}
