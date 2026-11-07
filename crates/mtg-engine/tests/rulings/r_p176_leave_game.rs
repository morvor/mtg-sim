//! Rulings batch P176 — a player who leaves a multiplayer game (CR 800.4a) takes the
//! cards they own with them; permanents and spells they control from other players'
//! cards are exiled, and effects giving them control end: Whispersteel Dagger, Athreos,
//! Zara, Ashiok, Dredge the Mire, Tomb of the Dusk Rose, Geth and Macabre Mockery.

use crate::r_p176_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s22_common::attack_p1_unblocked;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A three-player free-for-all in P0's precombat main phase.
fn game() -> TestGame {
    let mut t = ffa(3);
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// The one permanent named `name` P0 controls.
fn p0s(t: &mut TestGame, name: &str) -> ObjectId {
    t.g.recompute();
    let v: Vec<ObjectId> = t
        .named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj_now(*id).controller == P0)
        .collect();
    assert_eq!(v.len(), 1, "P0 controls one {name}");
    v[0]
}

/// P0 leaves the game: the permanent (P1's card) is exiled rather than returned.
fn p0_leaves_and_it_is_exiled(t: &mut TestGame, id: ObjectId) {
    concede(t, P0);
    assert_eq!(t.zone(id), Zone::Exile, "exiled");
}

#[test]
fn whispersteel_dagger_spells_and_permanents_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Whispersteel Dagger",
        "If you leave the game, any spells and/or permanents you control from Whispersteel Dagger’s ability are exiled."
    );
    supported("Whispersteel Dagger");
    for on_stack in [false, true] {
        let mut t = game();
        let bears = t.battlefield(P0, "Grizzly Bears");
        let dagger = t.battlefield(P0, "Whispersteel Dagger");
        t.g.attach(dagger, obj(bears));
        let giant = t.graveyard(P1, "Hill Giant");
        attack_p1_unblocked(&mut t, bears);
        t.advance_to(P0, Step::PostcombatMain);
        t.lands(P0, "Swamp", 4);
        let spell = t.cast(P0, giant).go();
        if on_stack {
            concede(&mut t, P0);
            assert_eq!(t.zone(spell), Zone::Exile, "the spell is exiled");
        } else {
            t.resolve_all();
            let g = p0s(&mut t, "Hill Giant");
            p0_leaves_and_it_is_exiled(&mut t, g);
        }
    }
}

#[test]
fn athreos_creatures_you_control_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Athreos, Shroud-Veiled",
        "If you leave the game, any creatures you control from Athreos's ability are exiled."
    );
    supported("Athreos, Shroud-Veiled");
    let mut t = game();
    t.battlefield(P0, "Athreos, Shroud-Veiled");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(obj(bears), "coin", 1, None);
    destroy(&mut t, bears);
    t.resolve_all();
    let b = p0s(&mut t, "Grizzly Bears");
    p0_leaves_and_it_is_exiled(&mut t, b);
}

/// P0's Zara attacks P1 and puts P1's Hill Giant from P1's hand onto the battlefield
/// attacking.
fn zara() -> (TestGame, ObjectId) {
    supported("Zara, Renegade Recruiter");
    let mut t = game();
    let z = t.battlefield(P0, "Zara, Renegade Recruiter");
    let giant = t.hand(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    attack_with(&mut t, &[(z, Entity::Player(P1))]);
    t.resolve_all();
    let g = p0s(&mut t, "Hill Giant");
    (t, g)
}

#[test]
fn zara_creature_leaves_with_either_player() {
    cr!("800.4a");
    ruling!(
        "Zara, Renegade Recruiter",
        "If you lose the game before the creature put onto the battlefield under your control returns to its owner's hand, that creature is exiled. If that creature's owner loses the game while you control the creature, the creature leaves the game with them."
    );
    let (mut t, g) = zara();
    p0_leaves_and_it_is_exiled(&mut t, g);
    let (mut t, g) = zara();
    concede(&mut t, P1);
    assert!(!t.on_battlefield(g), "left with its owner");
    assert!(!t.in_exile("Hill Giant"));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn ashiok_reanimated_creatures_are_exiled_but_stolen_ones_return() {
    cr!("800.4a");
    ruling!(
        "Ashiok, Sculptor of Fears",
        "If you leave the game, any permanents you control from Ashiok’s second ability are exiled, and the control effect of Ashiok’s last ability ends."
    );
    supported("Ashiok, Sculptor of Fears");
    let mut t = game();
    let ashiok = t.battlefield(P0, "Ashiok, Sculptor of Fears");
    set_loyalty(&mut t, ashiok, 20);
    // −5: P1's Hill Giant from P1's graveyard.
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, ashiok, "graveyard onto the battlefield").unwrap();
    t.resolve_all();
    let g = p0s(&mut t, "Hill Giant");
    // −11 (next turn's activation, allowed by resetting the per-turn count): P2's Bears.
    let bears = t.battlefield(P2, "Grizzly Bears");
    let a = t.g.current(ashiok);
    t.g.objects[a.0 as usize].activations_this_turn.clear();
    t.answer_targets(P0, &[Entity::Player(P2)]);
    activate_containing(&mut t, P0, ashiok, "Gain control").unwrap();
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    concede(&mut t, P0);
    assert_eq!(t.zone(g), Zone::Exile, "reanimated: exiled");
    assert!(t.on_battlefield(bears), "stolen: control effect ends");
    assert_eq!(controller(&mut t, bears), P2);
}

#[test]
fn dredge_the_mire_creatures_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Dredge the Mire",
        "If you leave the game, any permanents you control from Dredge the Mire are exiled."
    );
    let mut t = game();
    let giant = t.graveyard(P1, "Hill Giant");
    let bears = t.graveyard(P2, "Grizzly Bears");
    t.answer_choose(P1, &[obj(giant)]);
    t.answer_choose(P2, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Dredge the Mire", &[]);
    let g = p0s(&mut t, "Hill Giant");
    let b = p0s(&mut t, "Grizzly Bears");
    concede(&mut t, P0);
    assert_eq!(t.zone(g), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
}

#[test]
fn tomb_of_the_dusk_rose_creatures_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Profane Procession // Tomb of the Dusk Rose",
        "If you leave the game, any permanents you control from Tomb of the Dusk Rose's ability are exiled."
    );
    supported("Profane Procession // Tomb of the Dusk Rose");
    let mut t = game();
    let pp = t.battlefield(P0, "Profane Procession // Tomb of the Dusk Rose");
    for name in ["Grizzly Bears", "Hill Giant", "Craw Wurm"] {
        let c = t.battlefield(P1, name);
        add_mana(&mut t, P0, ManaType::W, 1);
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::C, 3);
        t.answer_targets(P0, &[obj(c)]);
        activate_containing(&mut t, P0, pp, "Exile target creature").expect("activate");
        t.resolve_all();
    }
    let tomb = t.g.current(pp);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    crate::r_s22_common::choose_named_when_offered(&mut t, P0, "Hill Giant");
    activate_containing(&mut t, P0, tomb, "onto the battlefield").expect("activate");
    t.resolve_all();
    let g = p0s(&mut t, "Hill Giant");
    p0_leaves_and_it_is_exiled(&mut t, g);
}

#[test]
fn geth_creatures_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Geth, Lord of the Vault",
        "If you leave the game, any spells and/or permanents you control from Geth's last ability are exiled."
    );
    supported("Geth, Lord of the Vault");
    let mut t = game();
    let geth = t.battlefield(P0, "Geth, Lord of the Vault");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, geth, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    let b = p0s(&mut t, "Grizzly Bears");
    p0_leaves_and_it_is_exiled(&mut t, b);
}

#[test]
fn macabre_mockery_creature_is_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Macabre Mockery",
        "If you leave the game, the creature you control from Macabre Mockery’s effect is exiled."
    );
    let mut t = game();
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Macabre Mockery", &[obj(bears)]);
    let b = p0s(&mut t, "Grizzly Bears");
    p0_leaves_and_it_is_exiled(&mut t, b);
}
