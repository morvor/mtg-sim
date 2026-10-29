//! cards5-referents-statics, family `spell-targets`: a spell already targeted by an
//! ability of another creature with the same name (CR 115.9b).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn goblin_artisans_cant_target_a_spell_another_artisans_targets() {
    cr!("115.9b", "113.7a");
    let def = card("Goblin Artisans");
    assert!(def.unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Artisans");
    let b = t.battlefield(P0, "Goblin Artisans");
    for o in [a, b] {
        t.g.objects[o.0 as usize].summoning_sick = false;
    }
    t.lands(P0, "Plains", 2);
    let orni = t.hand(P0, "Ornithopter");
    let orni = t.cast(P0, orni).go();
    // The first Artisans targets the first Ornithopter spell.
    assert!(t.activate(P0, a, 0, &[Entity::Object(orni)]).is_ok());
    let on_stack = t.stack_len();
    // The second can't target it any more (and there's no other artifact spell).
    let r = t.activate(P0, b, 0, &[Entity::Object(orni)]);
    let targeted_orni = r.is_ok() && t.stack_len() > on_stack && {
        let top = *t.g.stack.last().expect("on the stack");
        t.g.obj(top).stack.as_ref().is_some_and(|s| {
            s.chosen
                .first()
                .is_some_and(|c| c.targets.iter().flatten().any(|e| *e == Entity::Object(orni)))
        })
    };
    assert!(!targeted_orni);
    let mut t2 = TestGame::new(2);
    let a2 = t2.battlefield(P0, "Goblin Artisans");
    t2.g.objects[a2.0 as usize].summoning_sick = false;
    let o2 = t2.hand(P0, "Ornithopter");
    let o2 = t2.cast(P0, o2).go();
    assert!(t2.activate(P0, a2, 0, &[Entity::Object(o2)]).is_ok());
}
