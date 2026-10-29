//! Curse of Leeches // Leeching Lurker (hand-written, `src/cards/curse_of_leeches.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const CURSE: &str = "Curse of Leeches // Leeching Lurker";

fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// P0 casts Curse of Leeches on `target` by day; then it becomes night.
fn cast_then_night(t: &mut TestGame, target: PlayerId) -> ObjectId {
    t.lands(P0, "Swamp", 3);
    t.set_step(P0, Step::PrecombatMain);
    let card = t.hand(P0, CURSE);
    t.cast(P0, card).target(Entity::Player(target)).go();
    t.resolve_all();
    let id = t.named_on_battlefield("Curse of Leeches")[0];
    assert_eq!(t.obj_now(id).attached_to, Some(Entity::Player(target)));
    t.g.set_day(false);
    t.settle();
    assert_eq!(name(t, id), "Leeching Lurker");
    id
}

#[test]
fn curse_of_leeches_compiles() {
    let def = mtg_engine::card::card(CURSE);
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
}

#[test]
fn as_it_transforms_into_the_curse_its_controller_attaches_it_to_a_player() {
    cr!("712.20", "712.18", "701.3a");
    ruling!(
        "Curse of Leeches // Leeching Lurker",
        "After Curse of Leeches transforms into Leeching Lurker, it will become unattached from the player it's attached to as a state-based action."
    );
    ruling!(
        "Curse of Leeches // Leeching Lurker",
        "Attaching Curse of Leeches as it transforms doesn't target a player, so it can be attached to a player with hexproof this way."
    );
    let mut t = TestGame::new(2);
    // Cast on P0 itself, so attaching to P1 later is the new choice.
    let id = cast_then_night(&mut t, P0);
    assert_eq!(t.obj_now(id).attached_to, None);
    // P1 has hexproof: still a legal choice, since attaching doesn't target.
    t.battlefield(P1, "Leyline of Sanctity");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.g.set_day(true);
    t.settle();
    assert_eq!(name(&t, id), "Curse of Leeches");
    assert!(t.on_battlefield(id));
    assert_eq!(t.obj_now(id).attached_to, Some(Entity::Player(P1)));
    // "At the beginning of enchanted player's upkeep, they lose 1 life and you gain 1
    // life."
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn the_choice_is_among_players_it_can_enchant() {
    cr!("701.3a", "702.16c", "712.20");
    interp!("attach-as-it-transforms-to-a-legal-player");
    let mut t = TestGame::new(2);
    let id = cast_then_night(&mut t, P1);
    // P1 gains protection from everything: Curse of Leeches can't enchant P1.
    t.lands(P1, "Plains", 3);
    let tp = t.hand(P1, "Teferi's Protection");
    t.cast(P1, tp).go();
    t.resolve_all();
    t.clear_answers();
    t.g.set_day(true);
    t.settle();
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt == "Attach it to a player" && p == P0 => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(offered, [Entity::Player(P0)]);
    assert_eq!(t.obj_now(id).attached_to, Some(Entity::Player(P0)));
}
