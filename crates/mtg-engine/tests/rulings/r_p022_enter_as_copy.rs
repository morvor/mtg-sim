//! Rulings batch P022 — permanents that enter as copies of creatures: they can copy only
//! what's already on the battlefield (CR 614.12, 707.5), {X} in the copied mana cost is
//! 0 (CR 107.3, 202.3e, 707.2), and exceptions that set power and toughness drop the
//! copied characteristic-defining ability (CR 707.9d).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn clones_cant_copy_what_enters_with_them() {
    cr!("614.12", "707.5");
    ruling!("Sakashima of a Thousand Faces", "If Sakashima somehow enters the battlefield at the same time as another creature, it can't become a copy of that creature. You may choose only a creature that's already on the battlefield.");
    ruling!("Sakashima's Student", "If Sakashima's Student somehow enters at the same time as another creature, Sakashima's Student can't become a copy of that creature. You may choose only a creature that's already on the battlefield.");
    ruling!("Spark Double", "If Spark Double somehow enters the battlefield at the same time as another permanent, it can’t become a copy of that permanent. You may choose only a creature or planeswalker that’s already on the battlefield.");
    ruling!("Stunt Double", "If Stunt Double somehow enters the battlefield at the same time as another creature, it can’t become a copy of that creature. You may choose only a creature that’s already on the battlefield.");
    ruling!("Visage Bandit", "If Visage Bandit somehow enters the battlefield at the same time as another creature, Visage Bandit can’t become a copy of that creature. You may choose only a creature that’s already on the battlefield.");
    ruling!("Vizier of Many Faces", "If Vizier of Many Faces somehow enters the battlefield at the same time as another creature, Vizier of Many Faces can't become a copy of that creature. You may choose only a creature that's already on the battlefield.");
    ruling!("Wall of Stolen Identity", "If Wall of Stolen Identity somehow enters the battlefield at the same time as another creature, it can’t become a copy of that creature. You may choose only a creature that’s already on the battlefield.");
    // Those that copy "a creature you control" copy P0's Hill Giant, entering with P0's
    // Grizzly Bears; the others P1's.
    for (name, owner) in [
        ("Sakashima of a Thousand Faces", P0),
        ("Sakashima's Student", P1),
        ("Spark Double", P0),
        ("Stunt Double", P1),
        ("Visage Bandit", P0),
        ("Vizier of Many Faces", P1),
        ("Wall of Stolen Identity", P1),
    ] {
        cant_copy_what_enters_with_it(name, P0, "Grizzly Bears", owner, "Hill Giant");
    }
}

#[test]
fn clones_copy_x_as_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("Mercurial Pretender", "If the chosen creature has {X} in its mana cost (such as Genesis Hydra), X is 0.");
    ruling!("Progenitor Mimic", "If the chosen creature has {X} in its mana cost (such as Protean Hydra), X is considered to be zero.");
    ruling!("Evil Twin", "If the chosen creature has {X} in its mana cost, X is considered to be 0.");
    ruling!("Mocking Doppelganger", "If the chosen creature has {X} in its mana cost, X is considered to be 0.");
    ruling!("Phantasmal Image", "If the chosen creature has {X} in its mana cost, X is considered to be 0.");
    ruling!("Sakashima of a Thousand Faces", "If the chosen creature has {X} in its mana cost, X is considered to be 0.");
    ruling!("Vizier of Many Faces", "If the chosen creature has {X} in its mana cost, X is considered to be 0.");
    ruling!("Sakashima's Student", "If the chosen creature has {X} in its mana cost, X is zero.");
    ruling!("Copycrook", "If the chosen creature has {X} in its mana cost, X is 0.");
    // (card, whether the Hydra must be an opponent's)
    for (name, opponents) in [
        ("Mercurial Pretender", false),
        ("Progenitor Mimic", false),
        ("Evil Twin", false),
        ("Mocking Doppelganger", true),
        ("Phantasmal Image", false),
        ("Sakashima of a Thousand Faces", false),
        ("Vizier of Many Faces", false),
        ("Sakashima's Student", false),
        ("Copycrook", false),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let hydra = hydra_cast_with_x3(&mut t, P0);
        if opponents {
            give_control(&mut t, hydra, P1);
        }
        let c = enter_copying(&mut t, P0, name, hydra);
        hydra_with_x0(&mut t, c);
        assert_eq!(t.pt(c), (1, 1), "{name}");
    }
}

#[test]
fn protean_raider_copies_x_as_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("Protean Raider", "If the chosen creature has {X} in its mana cost, X is considered to be 0.");
    supported("Protean Raider");
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    // Raid: P0 attacked this turn with the Hydra.
    unsick(&mut t, hydra);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(hydra, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 16);
    let c = enter_copying(&mut t, P0, "Protean Raider", hydra);
    hydra_with_x0(&mut t, c);
}

#[test]
fn quicksilver_gargantuan_copies_x_as_0_and_stays_7_7() {
    cr!("107.3", "202.3e", "707.9b");
    ruling!("Quicksilver Gargantuan", "If the chosen creature has {X} in its mana cost (such as Protean Hydra), X is considered to be zero.");
    supported("Quicksilver Gargantuan");
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let q = enter_copying(&mut t, P0, "Quicksilver Gargantuan", hydra);
    hydra_with_x0(&mut t, q);
    assert_eq!(t.pt(q), (7, 7));
}

#[test]
fn quicksilver_gargantuan_doesnt_copy_a_characteristic_defining_ability() {
    cr!("707.9d", "604.3");
    ruling!("Quicksilver Gargantuan", "If the chosen creature has a characteristic-defining ability that sets its power and/or toughness, that ability will not be copied. For example, if Quicksilver Gargantuan copies a Nightmare, it will not gain the ability that sets its power and toughness to the number of Swamps you control. As such, it will remain a 7/7.");
    supported("Quicksilver Gargantuan");
    supported("Nightmare");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let nightmare = t.battlefield(P0, "Nightmare");
    assert_eq!(t.pt(nightmare), (3, 3));
    let q = enter_copying(&mut t, P0, "Quicksilver Gargantuan", nightmare);
    assert_eq!(name_of(&t, q), "Nightmare");
    assert_eq!(t.pt(q), (7, 7));
    assert_eq!(abilities_with(&t, q, "Swamps"), 0);
    // More Swamps don't change it.
    t.lands(P0, "Swamp", 2);
    assert_eq!(t.pt(nightmare), (5, 5));
    assert_eq!(t.pt(q), (7, 7));
}

#[test]
fn gigantoplasms_x_ability_doesnt_affect_x_in_its_mana_cost() {
    cr!("107.3", "202.3e", "707.9a");
    ruling!("Gigantoplasm", "If the chosen creature has {X} in its mana cost, X is 0. The ability that Gigantoplasm gives itself doesn't affect an {X} in its mana cost.");
    supported("Gigantoplasm");
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let g = enter_copying(&mut t, P0, "Gigantoplasm", hydra);
    hydra_with_x0(&mut t, g);
    // "{X}: This creature has base power and toughness X/X" with X = 5.
    t.lands(P0, "Wastes", 5);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let i = activated_index(&t, g, "base power and toughness");
    t.activate(P0, g, i, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(t.pt(g), (5, 5));
    assert_eq!(mv_now(&mut t, g), 2);
}

#[test]
fn activated_sleeper_copies_x_as_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("Activated Sleeper", "If the chosen creature card has {X} in its mana cost, X is considered to be zero.");
    supported("Activated Sleeper");
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    kill(&mut t, hydra);
    let card = t.g.current(hydra);
    let s = enter_copying(&mut t, P0, "Activated Sleeper", card);
    hydra_with_x0(&mut t, s);
    assert!(has_subtype(&t, s, "Phyrexian"));
}
