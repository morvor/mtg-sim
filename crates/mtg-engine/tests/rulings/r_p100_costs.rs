//! Rulings batch P100 — costs that depend on mana values: no player acts while a spell is
//! cast or an ability activated (CR 601.2, 602.2, 117.3c), and a card cast from the
//! graveyard leaves it first, before its cost is determined (601.2a, 601.2f).

use crate::r_p100_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 holds a Shock with {R} to cast it: a chance to act, if any were given.
fn p1_can_shock(t: &mut TestGame) {
    t.hand(P1, "Shock");
    mana(t, P1, "R");
}

#[test]
fn no_player_may_act_while_a_mana_value_matters_spell_is_cast_or_ability_activated() {
    cr!("601.2", "601.2h", "602.2", "117.3c");
    ruling!(
        "Pyre of Heroes",
        "Once you announce that you're activating the activated ability, no player may take actions until the ability has been paid for. Notably, opponents can't try to remove one of your creatures to stop you from sacrificing it."
    );
    ruling!(
        "The Capitoline Triad",
        "Once you announce you’re casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can’t try to remove historic cards from your graveyard."
    );
    ruling!(
        "Anchor to Reality",
        "Players can only respond once this spell has been cast and all its costs have been paid. No one can try to interfere with the permanent you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Neoform",
        "Players can respond to Neoform only after it's been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to stop you from casting this spell."
    );
    for c in ["Pyre of Heroes", "The Capitoline Triad", "Anchor to Reality", "Neoform"] {
        supported(c);
    }
    // Pyre of Heroes sacrificing Grizzly Bears: P1 gets no decision until the ability is
    // on the stack, and the Bears is already gone.
    let mut t = TestGame::new(2);
    let pyre = t.battlefield(P0, "Pyre of Heroes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    p1_can_shock(&mut t);
    mana(&mut t, P0, "2");
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    act(&mut t, P0, pyre, "Sacrifice a creature", &[]).expect("Pyre");
    assert!(others_asked_since(&t, from, P0).is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
    // The Capitoline Triad with five Sol Rings in the graveyard, cast for {5}.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.graveyard(P0, "Sol Ring");
    }
    p1_can_shock(&mut t);
    let triad = t.hand(P0, "The Capitoline Triad");
    mana(&mut t, P0, "5");
    let from = t.asked().len();
    t.cast(P0, triad).go();
    assert!(others_asked_since(&t, from, P0).is_empty());
    assert_eq!(pool(&t, P0), 0);
    // Anchor to Reality sacrificing a Mind Stone, Neoform sacrificing Grizzly Bears.
    for (name, cost, sacrificed) in [("Anchor to Reality", "2UU", "Mind Stone"), ("Neoform", "GU", "Grizzly Bears")] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, sacrificed);
        p1_can_shock(&mut t);
        let card = t.hand(P0, name);
        mana(&mut t, P0, cost);
        t.answer_choose(P0, &[obj(s)]);
        let from = t.asked().len();
        t.cast(P0, card).go();
        assert!(others_asked_since(&t, from, P0).is_empty(), "{name}");
        assert!(t.in_graveyard(P0, sacrificed), "{name}");
        assert_eq!(t.stack_len(), 1, "{name}");
    }
}

#[test]
fn the_capitoline_triad_cast_from_the_graveyard_doesnt_count_itself() {
    cr!("601.2a", "601.2f");
    ruling!(
        "The Capitoline Triad",
        "The first step of casting a spell is to move it to the stack. If this causes the number of historic cards in your graveyard to change (probably because you are casting The Capitoline Triad from your graveyard), that new number will be used to determine the cost reduction."
    );
    supported("The Capitoline Triad");
    supported("Yawgmoth's Will");
    // Yawgmoth's Will: cast from the graveyard with five other historic cards there: the
    // Triad itself has left the graveyard as it's cast, so {5}, not {4}.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.graveyard(P0, "Sol Ring");
    }
    let triad = t.graveyard(P0, "The Capitoline Triad");
    cast_resolve(&mut t, P0, "Yawgmoth's Will", "2B", &[]);
    mana(&mut t, P0, "4");
    assert!(t.cast(P0, triad).try_go().is_err(), "{{4}} isn't enough: {}", t.dump_log());
    assert!(!can_cast(&mut t, P0, triad, mtg_engine::object::CastMethod::Normal), "{{4}} isn't enough");
    mana(&mut t, P0, "1");
    assert!(can_cast(&mut t, P0, triad, mtg_engine::object::CastMethod::Normal));
    t.cast(P0, triad).go();
    assert_eq!(pool(&t, P0), 0, "{{5}} paid");
    assert_eq!(t.zone(triad), Zone::Stack);
}

#[test]
fn the_capitoline_triads_cost_is_locked_in_before_mana_abilities_are_activated() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "The Capitoline Triad",
        "Once you determine the cost to cast The Capitoline Triad, you may activate mana abilities to pay that cost. If the number of historic cards in your graveyard changes while activating mana abilities, the cost to cast The Capitoline Triad remains what you previously determined."
    );
    supported("The Capitoline Triad");
    supported("Ashnod's Altar");
    // Five Sol Ring cards in the graveyard: the Triad costs {5}. P0 has {3} and pays the
    // rest by sacrificing an Ornithopter to Ashnod's Altar ("Sacrifice a creature: Add
    // {C}{C}.") while paying: a sixth historic card in the graveyard, but the cost stays
    // {5} rather than becoming {4}: all five mana are spent.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.graveyard(P0, "Sol Ring");
    }
    t.battlefield(P0, "Ashnod's Altar");
    let thopter = t.battlefield(P0, "Ornithopter");
    let triad = t.hand(P0, "The Capitoline Triad");
    mana(&mut t, P0, "3");
    t.cast(P0, triad).go();
    assert_eq!(t.zone(triad), Zone::Stack);
    assert_eq!(t.zone(thopter), Zone::Graveyard(P0), "sacrificed for {{C}}{{C}}: {}", t.dump_log());
    assert_eq!(pool(&t, P0), 0, "{{5}} paid, not {{4}}: {}", t.dump_log());
}
