//! Abilities that name the object granting them (`src/granted_by.rs`): "Equipped creature
//! has \"{T}, Sacrifice Blazing Torch: Blazing Torch deals 2 damage to any target.\""
//! The ability is the creature's (CR 113.7, 602.2): its controller activates it and it
//! targets; the name still means the Equipment (CR 201.5), which must be sacrificed, by
//! a player who can only sacrifice permanents they control (CR 701.21a), and which is the
//! source of the damage (CR 608.2h).

use mtg_engine::ability::AbilityKind;
use mtg_engine::eval::Ctx;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn granted_index(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
        - 1
}

#[test]
fn blazing_torch_sacrifices_itself_and_deals_the_damage() {
    cr!("113.7", "701.21a", "608.2h");
    ruling!("Blazing Torch", "The source of the damage is Blazing Torch");
    assert_supported(&["Blazing Torch", "Ninja's Kunai", "Deconstruction Hammer", "Citizen's Crowbar"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let torch = t.battlefield(P0, "Blazing Torch");
    t.g.attach(torch, Entity::Object(bears));
    let i = granted_index(&mut t, bears);
    t.activate(P0, bears, i, &[Entity::Player(P1)]).unwrap();
    assert!(t.in_graveyard(P0, "Blazing Torch"));
    assert!(t.obj_now(bears).tapped);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // The targeting is the creature's ability: a red creature's can't target a creature
    // with protection from red.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let torch = t.battlefield(P0, "Blazing Torch");
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.attach(torch, Entity::Object(goblin));
    t.g.recompute();
    let i = granted_index(&mut t, goblin);
    let spec = t
        .obj_now(goblin)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(x) => Some(x.body.targets[0].clone()),
            _ => None,
        })
        .nth(i)
        .unwrap();
    let ctx = Ctx::new(Some(goblin), P0);
    let cands = t.g.legal_target_candidates(&spec, &ctx, goblin);
    assert!(!cands.contains(&Entity::Object(paladin)));
    assert!(cands.contains(&Entity::Object(bears)));
}

#[test]
fn ninjas_kunai_controlled_by_another_player_cant_be_sacrificed() {
    cr!("701.21a");
    ruling!(
        "Ninja's Kunai",
        "you cannot sacrifice permanents you don't control"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let kunai = t.battlefield(P1, "Ninja's Kunai");
    t.g.attach(kunai, Entity::Object(bears));
    t.lands(P0, "Mountain", 1);
    let i = granted_index(&mut t, bears);
    assert!(t.activate(P0, bears, i, &[Entity::Player(P1)]).is_err());
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(kunai));
}

#[test]
fn deconstruction_hammer_destroys_an_artifact() {
    cr!("113.7", "701.21a");
    ruling!(
        "Deconstruction Hammer",
        "as they can't pay the cost of sacrificing Deconstruction Hammer"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hammer = t.battlefield(P0, "Deconstruction Hammer");
    let other = t.battlefield(P0, "Deconstruction Hammer");
    t.g.attach(hammer, Entity::Object(bears));
    let target = t.battlefield(P1, "Bonesplitter");
    t.lands(P0, "Plains", 3);
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
    let i = granted_index(&mut t, bears);
    t.activate(P0, bears, i, &[Entity::Object(target)]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(target));
    // The attached Hammer was sacrificed, not the other one.
    assert!(!t.on_battlefield(hammer));
    assert!(t.on_battlefield(other));
}
