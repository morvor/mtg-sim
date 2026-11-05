//! CR 601.3e (709.3a, 712.11c, 715.3a, 718.3b, 702.37c): whether a card can be cast (the
//! legal actions a player is offered) is judged with the characteristics it would have as
//! the spell cast that way, including the cost increases and reductions that would then
//! apply to it — the same ones that apply once it is being cast (CR 601.2f).

use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
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
fn an_adventure_gets_the_cost_changes_of_an_instant() {
    cr!("601.3e", "715.3a", "601.2f");
    const ADVENTURE: CastMethod = CastMethod::Half(1);
    // Petty Theft ({1}{U} instant — Adventure) of Brazen Borrower (a creature card).
    // Goblin Electromancer: "Instant and sorcery spells you cast cost {1} less to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Brazen Borrower // Petty Theft");
    assert!(offered(&mut t, P0, card, ADVENTURE));
    t.cast(P0, card).target(bears).method(ADVENTURE).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Thalia, Guardian of Thraben: "Noncreature spells cost {1} more to cast." The
    // Adventure costs {2}{U}: two Islands aren't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Brazen Borrower // Petty Theft");
    assert!(!offered(&mut t, P0, card, ADVENTURE));
    assert!(t
        .cast(P0, card)
        .target(bears)
        .method(ADVENTURE)
        .try_go()
        .is_err());
    t.lands(P0, "Island", 1);
    assert!(offered(&mut t, P0, card, ADVENTURE));
}

#[test]
fn a_face_down_spell_gets_only_the_cost_changes_of_a_colorless_creature() {
    cr!("601.3e", "702.37c", "601.2f");
    const MORPH: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);
    // Battering Craghorn is a red card with morph; Ruby Medallion: "Red spells you cast
    // cost {1} less to cast." Cast face down it's a colorless spell costing {3}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Battering Craghorn");
    assert!(!offered(&mut t, P0, card, MORPH));
    assert!(t.cast(P0, card).method(MORPH).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    assert!(offered(&mut t, P0, card, MORPH));
    let spell = t.cast(P0, card).method(MORPH).go();
    assert!(t.g.obj(spell).face_down);
}

#[test]
fn a_split_half_or_back_face_gets_the_cost_changes_of_that_half_or_face() {
    cr!("601.3e", "709.3a", "712.11c", "601.2f");
    // Fire // Ice is a red and blue card; Ice ({1}{U}) is a blue spell, which Ruby
    // Medallion ("Red spells you cast cost {1} less to cast.") doesn't reduce.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Island", 1);
    t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Fire // Ice");
    assert!(!offered(&mut t, P0, card, CastMethod::Half(1)));
    // Sword of the Realms ({1}{W} legendary artifact, the back face of Halvar, God of
    // Battle) is an artifact spell: Etherium Sculptor ("Artifact spells you cast cost {1}
    // less to cast.") reduces it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherium Sculptor");
    t.lands(P0, "Plains", 1);
    let card = t.hand(P0, "Halvar, God of Battle // Sword of the Realms");
    assert!(offered(&mut t, P0, card, CastMethod::Half(1)));
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(spell).chars.name, "Sword of the Realms");
}

#[test]
fn a_prototyped_spell_gets_the_cost_changes_of_its_color() {
    cr!("601.3e", "718.3b", "601.2f");
    const PROTOTYPE: CastMethod = CastMethod::Keyword(KeywordKind::Prototype);
    // Phyrexian Fleshgorger is a colorless card; prototyped ({1}{B}{B}) it's a black
    // spell, which Jet Medallion ("Black spells you cast cost {1} less to cast.")
    // reduces to {B}{B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jet Medallion");
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Phyrexian Fleshgorger");
    assert!(offered(&mut t, P0, card, PROTOTYPE));
    let spell = t.cast(P0, card).method(PROTOTYPE).go();
    t.resolve_all();
    assert_eq!(t.pt(spell), (3, 3));
}
