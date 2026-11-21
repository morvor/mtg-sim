//! Rulings batch P094 — spells that cost their caster life: an illegal target means none
//! of the effects happen (no life lost), Orzhov Charm's modes, Vendetta and umbra armor,
//! Forge Devil, Agonizing Remorse, Malakir Rebirth, Death Wish and Toxic Deluge.

use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Puts the card into P0's hand with lands for its mana cost and casts it with `targets`
/// (one per slot) and `modes`.
fn cast_with(t: &mut TestGame, name: &str, modes: &[usize], targets: &[Entity]) -> ObjectId {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    let mut b = t.cast(P0, c);
    if !modes.is_empty() {
        b = b.modes(modes);
    }
    b.targets(targets).go()
}

#[test]
fn illegal_target_none_of_the_effects_happen_no_life_lost() {
    cr!("608.2b");
    ruling!(
        "Malakir Rebirth // Malakir Mire",
        "If the target creature is an illegal target by the time Malakir Rebirth tries to resolve, the spell doesn't resolve. You don't lose 2 life."
    );
    ruling!(
        "Withering Torment",
        "If the target creature or enchantment is an illegal target as Withering Torment tries to resolve, it won't resolve and none of its effects will happen. You won't lose life."
    );
    ruling!(
        "Anguished Unmaking",
        "If the target nonland permanent is an illegal target as Anguished Unmaking tries to resolve, it won't resolve and none of its effects will happen. You won't lose 3 life."
    );
    ruling!(
        "Vendetta",
        "If the targeted creature is an illegal target by the time Vendetta resolves, the spell doesn't resolve. You won't lose life."
    );
    ruling!(
        "Orzhov Charm",
        "For each of the modes, if the target creature or creature card is an illegal target when Orzhov Charm tries to resolve, it won't resolve and none of its effects will happen."
    );
    // (card, modes, P0's own target?, life lost if it resolves)
    let cases: [(&str, &[usize], bool, i32); 7] = [
        ("Malakir Rebirth // Malakir Mire", &[], false, 2),
        ("Withering Torment", &[], false, 2),
        ("Anguished Unmaking", &[], false, 3),
        ("Vendetta", &[], false, 2),
        ("Orzhov Charm", &[0], true, 0),
        ("Orzhov Charm", &[1], false, 2),
        ("Orzhov Charm", &[2], true, 0),
    ];
    for (name, modes, own, loss) in cases {
        supported(name);
        for illegal in [false, true] {
            let mut t = TestGame::new(2);
            let target = if modes == [2] {
                t.graveyard(P0, "Memnite")
            } else if own {
                t.battlefield(P0, "Grizzly Bears")
            } else {
                t.battlefield(P1, "Grizzly Bears")
            };
            let spell = cast_with(&mut t, name, modes, &[obj(target)]);
            if illegal {
                let to = if modes == [2] {
                    Zone::Exile
                } else {
                    Zone::Graveyard(t.obj_now(target).owner)
                };
                move_to(&mut t, target, to);
            }
            let moved_before = t.g.current(target);
            t.resolve_all();
            let label = format!("{name} {modes:?} illegal: {illegal}");
            assert_eq!(t.life(P0), if illegal { 20 } else { 20 - loss }, "{label}");
            if illegal {
                // The target (now a new object) wasn't touched either.
                assert_eq!(t.g.current(target), moved_before, "{label}");
                assert_eq!(t.zone(spell), Zone::Graveyard(P0), "{label}");
            } else if !name.starts_with("Malakir") {
                assert_ne!(t.g.current(target), target, "{label}");
            }
        }
    }
}

#[test]
fn dark_favor_with_an_illegal_target_doesnt_enter_or_trigger() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Dark Favor",
        "If the creature targeted by Dark Favor is an illegal target when Dark Favor tries to resolve, Dark Favor won't resolve. It won't enter and its \"enters\" ability won't trigger."
    );
    supported("Dark Favor");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let favor = cast_with(&mut t, "Dark Favor", &[], &[obj(bears)]);
        if illegal {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        if illegal {
            assert!(t.in_graveyard(P0, "Dark Favor"));
            assert_eq!(t.life(P0), 20);
        } else {
            assert!(t.on_battlefield(favor));
            assert_eq!(t.life(P0), 19);
            assert_eq!(t.pt(bears), (5, 3));
        }
    }
}

#[test]
fn spicy_oatmeal_pizza_illegal_target_you_arent_dealt_damage() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Spicy Oatmeal Pizza",
        "If the target of Spicy Oatmeal Pizza's first ability is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't be dealt damage."
    );
    supported("Spicy Oatmeal Pizza");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(giant)]);
        t.enter(P0, "Spicy Oatmeal Pizza");
        t.settle();
        assert_eq!(triggers_on_stack(&t, "4 damage"), 1);
        if illegal {
            destroy(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if illegal { 20 } else { 17 });
        assert!(t.in_graveyard(P1, "Hill Giant"));
    }
}

#[test]
fn orzhov_charm_first_mode_returns_your_auras_others_fall_off() {
    cr!("704.5m", "115.1", "608.2c");
    ruling!(
        "Orzhov Charm",
        "If you choose the first mode, any Aura controlled by another player attached to the creature will be put into its owner’s graveyard as a state-based action after the creature leaves the battlefield."
    );
    ruling!(
        "Orzhov Charm",
        "Orzhov Charm’s first mode doesn’t target any of the Auras attached to the target creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    attach_new(&mut t, P1, "Pacifism", bears);
    let from = t.asked().len();
    let spell = cast_with(&mut t, "Orzhov Charm", &[0], &[obj(bears)]);
    let targets: Vec<Entity> = t.obj_now(spell).stack.as_ref().unwrap().chosen[0]
        .targets
        .concat();
    assert_eq!(targets, vec![obj(bears)]);
    // Only one target was asked for, among creatures P0 controls.
    let asks = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(asks, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Holy Strength"));
    assert!(t.in_graveyard(P1, "Pacifism"));
}

#[test]
fn orzhov_charm_second_mode_loses_life_by_last_known_toughness() {
    cr!("608.2h", "608.2c", "702.12b");
    ruling!(
        "Orzhov Charm",
        "If you choose the second mode, and Orzhov Charm resolves but the creature isn’t destroyed (perhaps because it has indestructible or it regenerates), you’ll still lose life equal to the creature’s toughness."
    );
    ruling!(
        "Orzhov Charm",
        "If you choose the second mode, you’ll lose life equal to the creature’s toughness when it was last on the battlefield."
    );
    // Indestructible Darksteel Gargoyle (3/3) survives; P0 still loses 3.
    let mut t = TestGame::new(2);
    let garg = t.battlefield(P1, "Darksteel Gargoyle");
    cast_with(&mut t, "Orzhov Charm", &[1], &[obj(garg)]);
    t.resolve_all();
    assert!(t.on_battlefield(garg));
    assert_eq!(t.life(P0), 17);
    // Grizzly Bears with Holy Strength (+1/+2) is a 3/4 as it last existed: lose 4.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Holy Strength", bears);
    cast_with(&mut t, "Orzhov Charm", &[1], &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn vendetta_umbra_armor_saves_it_you_lose_its_current_toughness() {
    cr!("702.89a", "608.2h", "608.2c");
    ruling!(
        "Vendetta",
        "If the targeted creature isn't destroyed due to umbra armor's effect, Vendetta continues to resolve. You lose life equal to the creature's current toughness (after the Aura with umbra armor has been destroyed)."
    );
    supported("Hyena Umbra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Hyena Umbra", bears);
    assert_eq!(t.pt(bears), (3, 3));
    cast_with(&mut t, "Vendetta", &[], &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Hyena Umbra"));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn forge_devil_alone_must_target_itself() {
    cr!("603.3d", "115.1");
    ruling!(
        "Forge Devil",
        "Forge Devil’s ability is mandatory. If it’s the only creature on the battlefield, you’ll have to choose it as the target and have it deal 1 damage to itself and 1 damage to you."
    );
    supported("Forge Devil");
    let mut t = TestGame::new(2);
    t.enter(P0, "Forge Devil");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "1 damage"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forge Devil"));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn agonizing_remorse_rulings() {
    cr!("608.2c", "608.2d", "701.20a");
    ruling!(
        "Agonizing Remorse",
        "If there are no nonland cards in their hand and no cards in their graveyard, then you just lose 1 life."
    );
    ruling!(
        "Agonizing Remorse",
        "If you choose a card from their graveyard, it may be a land card."
    );
    ruling!(
        "Agonizing Remorse",
        "No player may take actions between the time you choose a card and the time it's exiled."
    );
    supported("Agonizing Remorse");
    // Only lands in hand, empty graveyard: just lose 1 life.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    cast_with(&mut t, "Agonizing Remorse", &[], &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Forest"));
    assert_eq!(t.life(P0), 19);
    // A land from the graveyard, exiled with no priority in between.
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let mountain = t.graveyard(P1, "Mountain");
    t.answer_choose(P0, &[obj(mountain)]);
    cast_with(&mut t, "Agonizing Remorse", &[], &[Entity::Player(P1)]);
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    assert!(!asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.in_exile("Mountain"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn malakir_rebirth_works_only_once() {
    cr!("400.7", "611.2c");
    ruling!(
        "Malakir Rebirth // Malakir Mire",
        "Malakir Rebirth's effect works only once. If the targeted creature dies and is then returned to the battlefield, it's considered to be a new creature. If that new creature dies, it won't come back."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_with(
        &mut t,
        "Malakir Rebirth // Malakir Mire",
        &[],
        &[obj(bears)],
    );
    t.resolve_all();
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert!(t.obj_now(back).tapped);
    destroy(&mut t, back);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn death_wish_finding_nothing_still_loses_half_your_life() {
    cr!("107.1a", "400.11a");
    ruling!(
        "Death Wish",
        "If you fail to find something, you still lose half your life, rounded up."
    );
    supported("Death Wish");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 15;
    cast_with(&mut t, "Death Wish", &[], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 7);
    assert!(t.in_exile("Death Wish"));
}

#[test]
fn toxic_deluge_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Toxic Deluge",
        "All creatures on the battlefield when Toxic Deluge resolves are affected. Ones that enter the battlefield or become creatures later in the turn are not."
    );
    supported("Toxic Deluge");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let deluge = t.hand(P0, "Toxic Deluge");
    t.cast(P0, deluge).x(2).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(giant), (1, 1));
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn toxic_deluge_cast_free_still_chooses_x_and_pays_life() {
    cr!("107.3", "118.9", "601.2b");
    ruling!(
        "Toxic Deluge",
        "If you cast Toxic Deluge without paying its mana cost, you'll still choose a value for X and pay X life. This is because it doesn't have {X} in its mana cost."
    );
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.battlefield(P1, "Hill Giant");
    let deluge = t.hand(P0, "Toxic Deluge");
    t.cast(P0, deluge).method(CastMethod::Free).x(3).go();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}
