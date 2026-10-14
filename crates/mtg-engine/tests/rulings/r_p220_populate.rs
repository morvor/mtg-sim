//! Rulings batch P220 — populate (CR 701.36): Ghired, Conclave Exile, Arboreal Alliance,
//! Full Flowering, Selesnya Eulogist, Trostani's Judgment, Sundering Growth.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, create_token};
use crate::r_s03_common::choice_candidates;
use crate::r_s12_common::attack_target;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The tokens `p` controls that aren't in `before`.
fn new_tokens(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect()
}

#[test]
fn ghired_populated_token_enters_attacking_even_if_it_cant_attack() {
    cr!("701.36a", "508.4", "506.3");
    ruling!(
        "Ghired, Conclave Exile",
        "Any effects that say that the token can't attack (such as that of Doomed Artisan, or if you copy a token with defender) affect only the declaration of attackers. They won't stop the token you populate from entering the battlefield attacking."
    );
    supported("Ghired, Conclave Exile");
    let mut t = TestGame::new(2);
    // Ghired enters: a 4/4 green Rhino token with trample.
    let ghired = t.enter(P0, "Ghired, Conclave Exile");
    t.resolve_all();
    let rhino = with_subtype(&t, P0, "Rhino");
    assert_eq!(rhino.len(), 1);
    t.g.objects[ghired.0 as usize].summoning_sick = false;
    t.g.objects[rhino[0].0 as usize].summoning_sick = false;
    // "Creature tokens can't attack.": the Rhino can't be declared as an attacker.
    t.custom(
        P1,
        custom_card(
            "Token Ban",
            "Enchantment",
            "{1}",
            None,
            "Creature tokens can't attack.",
        ),
        Zone::Battlefield,
    );
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, rhino[0]));
    let before = tokens(&t, P0);
    attack_with(&mut t, &[(ghired, Entity::Player(P1))]);
    t.resolve_all();
    // Populate: the copy of the Rhino enters tapped and attacking.
    let new = new_tokens(&t, P0, &before);
    assert_eq!(new.len(), 1);
    let copy = new[0];
    assert!(t.obj(copy).chars.has_subtype("Rhino"));
    assert!(t.obj(copy).tapped);
    assert!(t.g.is_attacking(copy));
    assert_eq!(attack_target(&t, copy), Some(Entity::Player(P1)));
    assert!(!t.g.is_attacking(rhino[0]));
}

#[test]
fn arboreal_alliance_populates_once_however_many_elves_attack() {
    cr!("701.36a", "603.2c", "107.3m");
    ruling!(
        "Arboreal Alliance",
        "Arboreal Alliance's last ability triggers whenever you attack with at least one Elf. You populate exactly once as it resolves, no matter how many Elves you attacked with."
    );
    supported("Arboreal Alliance");
    let mut t = TestGame::new(2);
    // Cast for X = 3: a 3/3 green Treefolk token (the spell's X, CR 107.3m).
    t.lands(P0, "Forest", 5);
    let aa = t.hand(P0, "Arboreal Alliance");
    t.cast(P0, aa).x(3).go();
    t.resolve_all();
    let tree = with_subtype(&t, P0, "Treefolk");
    assert_eq!(tree.len(), 1);
    assert_eq!(t.pt(tree[0]), (3, 3));
    let e1 = t.battlefield(P0, "Llanowar Elves");
    let e2 = t.battlefield(P0, "Llanowar Elves");
    let before = tokens(&t, P0);
    attack_with(&mut t, &[(e1, Entity::Player(P1)), (e2, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let new = new_tokens(&t, P0, &before);
    assert_eq!(new.len(), 1);
    assert_eq!(t.pt(new[0]), (3, 3));
    assert!(t.obj(new[0]).chars.has_subtype("Treefolk"));
}

#[test]
fn full_flowering_can_copy_a_different_token_each_time_and_x_0_does_nothing() {
    cr!("701.36a", "107.3a");
    ruling!(
        "Full Flowering",
        "Each time you populate, you can pick any creature token you control to copy. You don't have to create X tokens that are all copies of the same creature token."
    );
    ruling!("Full Flowering", "If X is 0, you don't populate at all.");
    supported("Full Flowering");
    let mut t = TestGame::new(2);
    let goblin = create_token(&mut t, P0, "Goblin");
    let elf = create_token(&mut t, P0, "Elf");
    let before = tokens(&t, P0);
    t.lands(P0, "Forest", 5);
    let ff = t.hand(P0, "Full Flowering");
    t.answer_choose(P0, &[Entity::Object(goblin)]);
    t.answer_choose(P0, &[Entity::Object(elf)]);
    t.cast(P0, ff).x(2).go();
    t.resolve_all();
    let new = new_tokens(&t, P0, &before);
    assert_eq!(new.len(), 2);
    let mut names: Vec<String> = new
        .iter()
        .map(|x| t.obj(*x).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Elf", "Goblin"]);
    // X = 0: no populate (no choice is asked, no token is created).
    let mut t = TestGame::new(2);
    create_token(&mut t, P0, "Goblin");
    let before = tokens(&t, P0);
    t.lands(P0, "Forest", 1);
    let ff = t.hand(P0, "Full Flowering");
    let from = t.asked().len();
    t.cast(P0, ff).x(0).go();
    t.resolve_all();
    assert!(new_tokens(&t, P0, &before).is_empty());
    assert!(choice_candidates(&t, from, "Populate").is_empty());
}

#[test]
fn selesnya_eulogist_with_an_illegal_target_doesnt_populate() {
    cr!("608.2b", "701.36a");
    ruling!(
        "Selesnya Eulogist",
        "If the target card is an illegal target by the time Selesnya Eulogist's ability tries to resolve, the ability doesn't resolve. You won't populate."
    );
    supported("Selesnya Eulogist");
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let eu = t.battlefield(P0, "Selesnya Eulogist");
        create_token(&mut t, P0, "Goblin");
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.lands(P0, "Forest", 3);
        let before = tokens(&t, P0);
        t.activate(P0, eu, 0, &[Entity::Object(bears)])
            .expect("activate");
        if remove {
            t.g.exile_object(bears, None);
        }
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        let n = new_tokens(&t, P0, &before).len();
        assert_eq!(n, if remove { 0 } else { 1 }, "removed: {remove}");
    }
}

#[test]
fn trostanis_judgment_exiles_the_target_token_before_populating() {
    cr!("608.2c", "701.36a", "701.36b");
    ruling!(
        "Trostani's Judgment",
        "If you choose a creature token you control as the target, it will be exiled before you populate and you won't be able to choose that creature to copy."
    );
    supported("Trostani's Judgment");
    // The only creature token is the target: no token to copy afterwards.
    let mut t = TestGame::new(2);
    let goblin = create_token(&mut t, P0, "Goblin");
    t.lands(P0, "Plains", 6);
    let tj = t.hand(P0, "Trostani's Judgment");
    t.cast(P0, tj).target(goblin).go();
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // With another token, only that one can be chosen.
    let mut t = TestGame::new(2);
    let goblin = create_token(&mut t, P0, "Goblin");
    let elf = create_token(&mut t, P0, "Elf");
    t.lands(P0, "Plains", 6);
    let tj = t.hand(P0, "Trostani's Judgment");
    let from = t.asked().len();
    t.cast(P0, tj).target(goblin).go();
    t.resolve_all();
    for c in choice_candidates(&t, from, "Populate") {
        assert!(!c.contains(&Entity::Object(goblin)));
    }
    let left: Vec<String> = tokens(&t, P0)
        .iter()
        .map(|x| t.obj(*x).chars.name.to_string())
        .collect();
    assert_eq!(left, vec!["Elf", "Elf"]);
    assert!(t.on_battlefield(elf));
}

#[test]
fn a_spell_that_populates_with_an_illegal_target_doesnt_populate() {
    cr!("608.2b", "701.36a");
    ruling!(
        "Trostani's Judgment",
        "If that creature is an illegal target when Trostani's Judgment tries to resolve, it won't resolve and none of its effects will happen. You won't populate."
    );
    ruling!(
        "Sundering Growth",
        "If that artifact or enchantment is an illegal target when Sundering Growth tries to resolve, it won't resolve and none of its effects will happen. You won't populate."
    );
    supported("Sundering Growth");
    for (spell, target, land) in [
        ("Trostani's Judgment", "Grizzly Bears", "Plains"),
        ("Sundering Growth", "Sol Ring", "Forest"),
    ] {
        for remove in [false, true] {
            let mut t = TestGame::new(2);
            create_token(&mut t, P0, "Goblin");
            let tgt = t.battlefield(P1, target);
            t.lands(P0, land, 6);
            let s = t.hand(P0, spell);
            let before = tokens(&t, P0);
            t.cast(P0, s).target(tgt).go();
            if remove {
                t.g.exile_object(tgt, None);
                t.settle();
            }
            t.resolve_all();
            let n = new_tokens(&t, P0, &before).len();
            assert_eq!(n, if remove { 0 } else { 1 }, "{spell}, removed: {remove}");
            assert!(!t.on_battlefield(tgt));
        }
    }
}
