//! CR 601.3e, last sentence: whether a player may begin to cast a card that isn't on the
//! stack is judged with the characteristics the spell would have on the stack, "including
//! effects that would apply to it there" — not only the printed characteristics of the half
//! or face it would be cast as.

use mtg_engine::decision::Action;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` is offered casting `card` with `method` now.
fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, method: m } if *c == card && *m == method))
}

#[test]
fn a_split_half_made_white_by_an_effect_gets_a_white_spell_reduction() {
    cr!("601.3e", "709.3a", "601.2f");
    // Celestial Dawn: "... spells you control and nonland cards you own that aren't on the
    // battlefield [are white]. You may spend white mana as though it were mana of any
    // color." Pearl Medallion: "White spells you cast cost {1} less to cast." Ice ({1}{U})
    // is a white spell costing {U}: one Plains pays for it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    t.battlefield(P0, "Pearl Medallion");
    t.lands(P0, "Plains", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Fire // Ice");
    assert!(offered(&mut t, P0, card, CastMethod::Half(1)));
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.resolve_all();
    assert!(t.g.obj(t.g.current(bears)).tapped);
    // Without Celestial Dawn, Ice is blue: no reduction and no Island.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pearl Medallion");
    t.lands(P0, "Plains", 1);
    t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Fire // Ice");
    assert!(!offered(&mut t, P0, card, CastMethod::Half(1)));
}

#[test]
fn a_room_door_made_an_artifact_by_an_effect_gets_an_artifact_spell_reduction() {
    cr!("601.3e", "709.3a", "601.2f");
    ruling!(
        "Encroaching Mycosynth",
        "Permanent spells you control become artifact spells"
    );
    // Encroaching Mycosynth: "... The same is true for permanent spells you control and
    // nonland permanent cards you own that aren't on the battlefield [they're artifacts]."
    // Etherium Sculptor: "Artifact spells you cast cost {1} less to cast." Locker Room
    // ({4}{U}) is an artifact spell costing {3}{U}: four Islands pay for it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Encroaching Mycosynth");
    t.battlefield(P0, "Etherium Sculptor");
    t.lands(P0, "Island", 4);
    let card = t.hand(P0, "Bottomless Pool // Locker Room");
    assert!(offered(&mut t, P0, card, CastMethod::Half(1)));
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    assert_eq!(t.g.obj(spell).chars.name, "Locker Room");
    // Without Encroaching Mycosynth it costs {4}{U}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Etherium Sculptor");
    t.lands(P0, "Island", 4);
    let card = t.hand(P0, "Bottomless Pool // Locker Room");
    assert!(!offered(&mut t, P0, card, CastMethod::Half(1)));
}
