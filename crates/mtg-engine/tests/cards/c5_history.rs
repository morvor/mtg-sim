//! cards5-referents-statics, family `history`: what happened earlier in the game, read by
//! an ability (CR 603.10a, 700.4).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn krovikan_vampire_returns_the_creatures_it_damaged_that_died() {
    cr!("603.4", "603.10a", "700.4");
    ruling!(
        "Krovikan Vampire",
        "it doesn’t matter whether the damage was what caused the creatures to die"
    );
    supported("Krovikan Vampire");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // The Giant survives 1 damage from the Vampire, then is destroyed; the Elves die
    // without being dealt damage by it.
    t.g.deal_damage(kv, Entity::Object(giant), 1, false);
    t.g.deal_damage(kv, Entity::Object(bears), 2, false);
    t.settle();
    t.g.destroy(giant, None);
    t.g.destroy(elves, None);
    t.settle();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for (name, back) in [("Hill Giant", true), ("Grizzly Bears", true), ("Llanowar Elves", false)] {
        let on: Vec<ObjectId> = t.named_on_battlefield(name);
        assert_eq!(on.len(), usize::from(back), "{name}");
        if back {
            assert_eq!(t.obj_now(on[0]).controller, P0, "{name}");
        }
    }
    // Losing control of the Vampire (it leaves the battlefield): they're sacrificed.
    t.g.destroy(kv, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn krovikan_vampire_does_not_trigger_without_such_a_death() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(kv, Entity::Object(giant), 1, false);
    t.settle();
    let stack_before = t.stack_len();
    t.advance_to(P0, Step::End);
    assert_eq!(t.stack_len(), stack_before);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P1);
}

/// Runs P0's next combat with Territorial Hellkite, declaring no attackers (its
/// requirement makes it attack anyway): whom it attacked, and whether it was tapped by its
/// ability.
fn hellkite_combat(t: &mut TestGame, hk: ObjectId) -> (Option<Entity>, bool) {
    use mtg_engine::decision::Answer;
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tapped = t.obj_now(hk).tapped;
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareAttackers);
    let at = t
        .g
        .combat
        .as_ref()
        .and_then(|c| c.attack_target(t.g.current(hk)));
    t.advance_to(P0, Step::EndOfCombat);
    (at, tapped)
}

#[test]
fn territorial_hellkite_attacks_an_opponent_it_didnt_attack_last_combat() {
    cr!("508.1b", "508.1d");
    ruling!(
        "Territorial Hellkite",
        "then any opponent can be chosen at random"
    );
    supported("Territorial Hellkite");
    let mut t = TestGame::new(3);
    let hk = t.battlefield(P0, "Territorial Hellkite");
    // First combat: no last combat, any opponent; it attacks that player.
    let (first, tapped) = hellkite_combat(&mut t, hk);
    assert!(!tapped);
    let first = first.expect("it attacks the chosen opponent");
    assert!(matches!(first, Entity::Player(p) if p != P0));
    // Next turn: the other opponent.
    let (second, tapped) = hellkite_combat(&mut t, hk);
    assert!(!tapped);
    let second = second.expect("it attacks the chosen opponent");
    assert!(matches!(second, Entity::Player(p) if p != P0));
    assert_ne!(first, second);
}

#[test]
fn territorial_hellkite_taps_if_no_opponent_can_be_chosen() {
    cr!("508.1b", "508.1d");
    supported("Territorial Hellkite");
    let mut t = TestGame::new(2);
    let hk = t.battlefield(P0, "Territorial Hellkite");
    let (at, tapped) = hellkite_combat(&mut t, hk);
    assert_eq!((at, tapped), (Some(Entity::Player(P1)), false));
    // It attacked the only opponent last combat: it's tapped and doesn't attack.
    let (at, tapped) = hellkite_combat(&mut t, hk);
    assert_eq!((at, tapped), (None, true));
    // It didn't attack during the last combat: P1 can be chosen again.
    let (at, tapped) = hellkite_combat(&mut t, hk);
    assert_eq!((at, tapped), (Some(Entity::Player(P1)), false));
}

#[test]
fn territorial_hellkite_last_combat_is_an_earlier_combat_of_this_turn() {
    cr!("500.8", "508.1d");
    supported("Territorial Hellkite");
    let mut t = TestGame::new(2);
    let hk = t.battlefield(P0, "Territorial Hellkite");
    let (at, tapped) = hellkite_combat(&mut t, hk);
    assert_eq!((at, tapped), (Some(Entity::Player(P1)), false));
    let turn = t.g.turn.number;
    // An additional combat phase this turn: its last combat is the one just fought, in
    // which it attacked the only opponent.
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 4);
    let ra = t.hand(P0, "Relentless Assault");
    t.cast(P0, ra).go();
    t.resolve_all();
    assert!(!t.obj_now(hk).tapped);
    let (at, tapped) = hellkite_combat(&mut t, hk);
    assert_eq!(t.g.turn.number, turn);
    assert_eq!((at, tapped), (None, true));
}
