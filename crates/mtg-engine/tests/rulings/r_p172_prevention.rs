//! Rulings batch P172 — prevention effects of white cards: Acolyte's Reward's prevention
//! shield that deals the prevented damage itself (CR 615.5, 615.7, 608.2b), Iroas's
//! prevention for attacking creatures, and Chant of Vitu-Ghazi's shield that gains life
//! (CR 615.5).

use crate::r_p076_common::mana;
use crate::r_p117_common::modify;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::{respond, to_blockers};
use crate::r_s05_common::move_to;
use crate::r_s06_common::damage;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0 controls permanents for devotion to white `n` (1 to 4): Serra Angels ({W}{W}) and
/// Savannah Lions ({W}).
fn devotion(t: &mut TestGame, n: u32) {
    for _ in 0..n / 2 {
        t.battlefield(P0, "Serra Angel");
    }
    if n % 2 == 1 {
        t.battlefield(P0, "Savannah Lions");
    }
}

/// P0 casts Acolyte's Reward with the two targets; it's left on the stack.
fn cast_reward(t: &mut TestGame, first: ObjectId, second: Entity) -> ObjectId {
    supported("Acolyte's Reward");
    let ar = t.hand(P0, "Acolyte's Reward");
    mana(t, P0, ManaType::W, 2);
    t.cast(P0, ar).target(first).target(second).go()
}

fn damage_of(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// Damage events dealt by an object named `name`: (target, amount, combat).
fn damage_by(t: &TestGame, name: &str) -> Vec<(Entity, u32, bool)> {
    t.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                combat,
            } if t.g.obj(*source).chars.name == name => Some((*target, *amount, *combat)),
            _ => None,
        })
        .collect()
}

#[test]
fn acolytes_reward_prevents_the_next_damage_from_any_source_and_deals_it() {
    cr!("615.7", "615.5", "115.1a");
    ruling!(
        "Acolyte's Reward",
        "Acolyte's Reward has two targets: the creature that would be dealt damage and the permanent or player that Acolyte's Reward will deal damage to. These targets are chosen as you cast Acolyte's Reward."
    );
    ruling!(
        "Acolyte's Reward",
        "You don't choose a source of damage. The prevention shield will apply to the next X damage that would be dealt to the first target, no matter where that damage comes from."
    );
    ruling!(
        "Acolyte's Reward",
        "If the amount of damage that would be dealt to the first target is in excess of the amount of damage that Acolyte's Reward would prevent, the source deals its excess damage to the first target at the same time that the rest of it is prevented. Then Acolyte's Reward deals its damage."
    );
    let mut t = TestGame::new(2);
    devotion(&mut t, 3);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let spell = cast_reward(&mut t, giant, Entity::Player(P1));
    // Both targets were chosen as it was cast.
    let si = t.obj(spell).stack.as_ref().unwrap();
    assert_eq!(
        si.chosen[0].targets,
        vec![vec![obj(giant)], vec![Entity::Player(P1)]]
    );
    t.resolve_all();
    // 2 from one source: all prevented and dealt to P1.
    damage(&mut t, bears, 2, giant);
    assert_eq!(damage_of(&t, giant), 0);
    assert_eq!(t.life(P1), 18);
    // 2 from another source: 1 left on the shield; the excess is dealt.
    damage(&mut t, goblin, 2, giant);
    assert_eq!(damage_of(&t, giant), 1);
    assert_eq!(t.life(P1), 17);
    // Used up.
    damage(&mut t, goblin, 1, giant);
    assert_eq!(damage_of(&t, giant), 2);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn acolytes_reward_after_it_resolves_target_legality_doesnt_matter() {
    cr!("608.2b", "615.5");
    ruling!(
        "Acolyte's Reward",
        "After Acolyte's Reward resolves, it no longer matters whether either target is still legal."
    );
    // The second target gains hexproof: it's still dealt damage.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let src = t.battlefield(P1, "Grizzly Bears");
    cast_reward(&mut t, giant, obj(wurm));
    t.resolve_all();
    modify(
        &mut t,
        wurm,
        vec![Modification::AddKeyword(KeywordKind::Hexproof.into())],
    );
    damage(&mut t, src, 2, giant);
    assert_eq!(damage_of(&t, giant), 0);
    assert_eq!(damage_of(&t, wurm), 2);
    // The second target left the battlefield: the damage is still prevented.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let src = t.battlefield(P1, "Grizzly Bears");
    cast_reward(&mut t, giant, obj(wurm));
    t.resolve_all();
    move_to(&mut t, wurm, Zone::Hand(P1));
    damage(&mut t, src, 2, giant);
    assert_eq!(damage_of(&t, giant), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn acolytes_reward_with_illegal_targets() {
    cr!("608.2b", "115.1a");
    ruling!(
        "Acolyte's Reward",
        "As Acolyte's Reward tries to resolve, if only the first target is illegal, Acolyte's Reward won't prevent any damage that would be dealt to that creature and, because of this, Acolyte's Reward won't deal damage to the second target. If only the second target is illegal, damage that would be dealt to the first target will be prevented, but Acolyte's Reward won't deal damage. If both targets are illegal, Acolyte's Reward won't resolve."
    );
    let shroud = || vec![Modification::AddKeyword(KeywordKind::Shroud.into())];
    // Only the first target is illegal.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let src = t.battlefield(P1, "Grizzly Bears");
    cast_reward(&mut t, giant, Entity::Player(P1));
    modify(&mut t, giant, shroud());
    t.resolve_all();
    damage(&mut t, src, 2, giant);
    assert_eq!(damage_of(&t, giant), 2);
    assert_eq!(t.life(P1), 20);
    // Only the second target is illegal.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let src = t.battlefield(P1, "Grizzly Bears");
    cast_reward(&mut t, giant, obj(wurm));
    modify(&mut t, wurm, shroud());
    t.resolve_all();
    damage(&mut t, src, 2, giant);
    assert_eq!(damage_of(&t, giant), 0);
    assert_eq!(damage_of(&t, wurm), 0);
    // Both: it doesn't resolve.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let src = t.battlefield(P1, "Grizzly Bears");
    cast_reward(&mut t, giant, obj(wurm));
    modify(&mut t, wurm, shroud());
    modify(&mut t, giant, shroud());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Acolyte's Reward"));
    damage(&mut t, src, 2, giant);
    assert_eq!(damage_of(&t, giant), 2);
}

#[test]
fn acolytes_reward_deals_its_damage_before_the_spell_resumes() {
    cr!("615.5", "608.2c");
    ruling!(
        "Acolyte's Reward",
        "If Acolyte's Reward prevents damage, it deals its damage immediately afterward as part of that same prevention effect."
    );
    supported("Lightning Helix");
    let mut t = TestGame::new(2);
    devotion(&mut t, 3);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_reward(&mut t, giant, Entity::Player(P1));
    t.resolve_all();
    // P1's Lightning Helix: its damage is prevented and Acolyte's Reward deals 3 to P1
    // before P1 gains 3 life.
    let helix = t.hand(P1, "Lightning Helix");
    mana(&mut t, P1, ManaType::R, 1);
    mana(&mut t, P1, ManaType::W, 1);
    t.cast(P1, helix).target(giant).go();
    t.resolve_all();
    assert_eq!(damage_of(&t, giant), 0);
    assert_eq!(t.life(P1), 20);
    let events: Vec<&Event> = t.turn_events.iter().chain(t.g.events.iter()).collect();
    let dealt = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Damage {
                    target: Entity::Player(P1),
                    amount: 3,
                    ..
                }
            )
        })
        .expect("Acolyte's Reward dealt damage");
    let gained = events
        .iter()
        .position(|e| matches!(e, Event::LifeGained { player: P1, .. }))
        .expect("P1 gained life");
    assert!(dealt < gained);
}

/// Orders the damage a prevention shield applies to: Typhoid Rats's damage first.
fn rats_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::Order { prompt, items } if prompt.contains("prevention shield") => {
            let rats = items.iter().position(|i| i.contains("Typhoid Rats"))?;
            let mut order = vec![rats];
            order.extend((0..items.len()).filter(|i| *i != rats));
            Some(Answer::Indices(order))
        }
        _ => None,
    }
}

#[test]
fn acolytes_reward_chooses_which_combat_damage_to_prevent() {
    cr!("615.7", "510.2");
    ruling!(
        "Acolyte's Reward",
        "If the first target would be dealt combat damage by multiple creatures, you choose which of that damage to prevent."
    );
    supported("Typhoid Rats");
    for prevent_rats in [true, false] {
        let mut t = TestGame::new(2);
        devotion(&mut t, 1);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let rats = t.battlefield(P1, "Typhoid Rats");
        cast_reward(&mut t, giant, Entity::Player(P1));
        t.resolve_all();
        if prevent_rats {
            respond(&mut t, P0, rats_first);
        }
        to_blockers(
            &mut t,
            &[(giant, Entity::Player(P1))],
            &[(bears, giant), (rats, giant)],
        );
        t.advance_to(P0, Step::EndOfCombat);
        // The deathtouch damage prevented: the giant survives.
        assert_eq!(t.on_battlefield(giant), prevent_rats);
        assert_eq!(t.life(P1), 19);
    }
}

#[test]
fn acolytes_reward_locks_in_devotion_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Acolyte's Reward",
        "The amount of damage the prevention shield will prevent is based on your devotion to white as Acolyte's Reward resolves. That amount won't change later in the turn, even if your devotion to white does."
    );
    let mut t = TestGame::new(2);
    devotion(&mut t, 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let src = t.battlefield(P1, "Grizzly Bears");
    cast_reward(&mut t, giant, Entity::Player(P1));
    t.resolve_all();
    devotion(&mut t, 2);
    damage(&mut t, src, 2, giant);
    assert_eq!(damage_of(&t, giant), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn acolytes_reward_is_the_source_of_its_own_damage() {
    cr!("615.5", "120.3");
    ruling!(
        "Acolyte's Reward",
        "The damage will be dealt by Acolyte's Reward as it existed on the stack, not as it exists when the damage is dealt."
    );
    ruling!(
        "Acolyte's Reward",
        "The effect of Acolyte's Reward isn't a redirection effect. If it prevents damage, Acolyte's Reward (not the source of that damage) deals damage to the second target as part of that prevention effect."
    );
    supported("Satyr Firedancer");
    supported("Vampire Nighthawk");
    // An instant spell you control dealt damage to an opponent: Satyr Firedancer triggers.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    t.battlefield(P0, "Satyr Firedancer");
    let giant = t.battlefield(P0, "Hill Giant");
    let src = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_reward(&mut t, giant, Entity::Player(P1));
    t.resolve_all();
    t.answer_targets(P0, &[obj(wurm)]);
    damage(&mut t, src, 2, giant);
    assert_eq!(t.life(P1), 18);
    t.resolve_all();
    assert_eq!(damage_of(&t, wurm), 2);
    // Prevented combat damage from a deathtouch, lifelink creature: Acolyte's Reward's
    // damage is noncombat, without deathtouch or lifelink.
    let mut t = TestGame::new(2);
    devotion(&mut t, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_reward(&mut t, giant, obj(wurm));
    t.resolve_all();
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(hawk, giant)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(giant));
    assert_eq!(damage_of(&t, giant), 0);
    assert!(t.on_battlefield(wurm));
    assert_eq!(damage_of(&t, wurm), 2);
    assert_eq!(t.life(P1), 20);
    assert_eq!(
        damage_by(&t, "Acolyte's Reward"),
        vec![(obj(wurm), 2, false)]
    );
}

#[test]
fn iroas_prevents_all_damage_to_attacking_creatures() {
    cr!("615.1a", "506.4");
    ruling!(
        "Iroas, God of Victory",
        "Iroas's last ability prevents all damage dealt to attacking creatures you control, not just combat damage."
    );
    supported("Iroas, God of Victory");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Iroas, God of Victory");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let src = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    damage(&mut t, src, 5, bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_of(&t, bears), 0);
    // Not attacking: not prevented.
    damage(&mut t, src, 2, other);
    assert_eq!(damage_of(&t, other), 2);
}

#[test]
fn chant_of_vitu_ghazi_gains_life_each_time_it_prevents_damage() {
    cr!("615.5", "615.1a");
    ruling!(
        "Chant of Vitu-Ghazi",
        "The damage prevention shield lasts for the rest of the turn, and you gain life each time that shield prevents 1 or more damage."
    );
    supported("Chant of Vitu-Ghazi");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let chant = t.hand(P0, "Chant of Vitu-Ghazi");
    mana(&mut t, P0, ManaType::W, 8);
    t.cast(P0, chant).go();
    t.resolve_all();
    damage(&mut t, giant, 3, bears);
    assert_eq!(t.life(P0), 23);
    damage(&mut t, giant, 2, bears);
    assert_eq!(t.life(P0), 25);
    assert_eq!(damage_of(&t, bears), 0);
}
