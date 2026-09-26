//! CR 508.1e, 508.4–508.8: bands, creatures put onto the battlefield attacking or stated to
//! be attacking, the defending player, "attacking"/"attacked" a player, reselecting what a
//! creature attacks, and skipping steps when nothing attacks.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::combat::{
    make_attacking, player_has_attacked, player_is_attacking, reselect_attack_target,
};
use mtg_engine::eval::Ctx;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const P4: PlayerId = PlayerId(4);

fn band_of(t: &TestGame, a: ObjectId) -> Option<u32> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attacker(a))
        .and_then(|x| x.band)
}

#[test]
fn attacking_player_announces_bands() {
    cr!("508.1e");
    let mut t = TestGame::new(2);
    let h1 = t.battlefield(P0, "Benalish Hero");
    let h2 = t.battlefield(P0, "Benalish Hero");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Hill Giant");
    let p1 = Entity::Player(P1);
    declare(&mut t, &[(h1, p1), (h2, p1), (b1, p1), (b2, p1)]);
    // Band: both Heroes and one creature without banding.
    t.answer_choose(P0, &[Entity::Object(h2), Entity::Object(b1)]);
    go_to(&mut t, Step::DeclareAttackers);
    let band = band_of(&t, h1);
    assert!(band.is_some());
    assert_eq!(band_of(&t, h2), band);
    assert_eq!(band_of(&t, b1), band);
    assert_eq!(band_of(&t, b2), None);

    // A band can't contain two creatures without banding.
    let mut t = TestGame::new(2);
    let h1 = t.battlefield(P0, "Benalish Hero");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Hill Giant");
    declare(&mut t, &[(h1, p1), (b1, p1), (b2, p1)]);
    t.answer_choose(P0, &[Entity::Object(b1), Entity::Object(b2)]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(band_of(&t, h1), None);
    assert_eq!(band_of(&t, b1), None);
}

#[test]
fn controller_chooses_what_a_creature_entering_attacking_attacks() {
    cr!("508.4");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    // "Create a 1/1 Soldier token that's tapped and attacking" from a non-attacking source:
    // its controller chooses Jace.
    t.answer_choose(P0, &[Entity::Object(jace)]);
    let horn = bf(
        &mut t,
        P0,
        custom_with("Rally Horn", "Artifact", None, vec![]),
    );
    let mut ctx = Ctx::new(Some(horn), P0);
    t.g.exec(
        &Effect::CreateToken {
            spec: TokenSpec {
                name: "Soldier".into(),
                colors: ColorSet::NONE,
                supertypes: vec![],
                card_types: vec![CardType::Creature],
                subtypes: vec!["Soldier".into()],
                power: Some(1),
                toughness: Some(1),
                abilities: vec![],
                scryfall_name: None,
            },
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: true,
            attacking: true,
        },
        &mut ctx,
    );
    t.g.flush_events();
    let token = ctx.var_objects(vars::CREATED)[0];
    assert_eq!(attack_target(&t, token), Some(Entity::Object(jace)));
    // It's attacking, but it never "attacked".
    let f = Filter::AttackedThisTurn;
    assert!(!t.g.matches(token, &f, &Ctx::new(None, P0)));
    assert!(t.g.matches(bears, &f, &Ctx::new(None, P0)));
    // It remains attacking until the combat phase ends.
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.g.is_attacking(token));
    go_to(&mut t, Step::PostcombatMain);
    assert!(!t.g.is_attacking(token));
}

#[test]
fn effect_stating_a_creature_is_attacking() {
    cr!("508.4", "508.4b");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let borrowed = t.battlefield(P0, "Hill Giant");
    t.g.tap(borrowed);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    // "...tap it, and it's attacking that player."
    assert!(make_attacking(&mut t.g, borrowed, Entity::Player(P2)));
    assert_eq!(attack_target(&t, borrowed), Some(Entity::Player(P2)));
    // If the specified player has left the game, it doesn't become attacking.
    let other = t.battlefield(P0, "Craw Wurm");
    t.g.player_loses(P2);
    assert!(!make_attacking(&mut t.g, other, Entity::Player(P2)));
    assert!(!t.g.is_attacking(other));
    // Nor if the specified planeswalker isn't on the battlefield anymore.
    let jace = t.battlefield(P1, "Jace Beleren");
    t.g.destroy(jace, None);
    assert!(!make_attacking(&mut t.g, other, Entity::Object(jace)));
    assert!(!t.g.is_attacking(other));
}

#[test]
fn creatures_entering_attacking_ignore_attack_restrictions_and_requirements() {
    cr!("508.4c");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    let lands = t.lands(P0, "Plains", 2);
    to_combat(&mut t, P0);
    // Wall of Stone has defender; it enters summoning sick; Propaganda would demand {2}.
    let wall = enter_with(
        &mut t,
        P0,
        (*card("Wall of Stone")).clone(),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.obj_now(wall).summoning_sick);
    assert!(t.g.is_attacking(wall));
    assert!(
        lands.iter().all(|l| !t.obj_now(*l).tapped),
        "no attack cost paid"
    );
    // Same for a creature stated to be attacking.
    let pacified = t.battlefield(P0, "Grizzly Bears");
    apply(
        &mut t,
        P1,
        Effect::AddRestriction {
            restriction: Restriction::CantAttack(Filter::In(Box::new(Sel::Target(0)))),
            duration: Duration::EndOfTurn,
        },
        &[pacified],
    );
    assert!(!t.g.can_attack(pacified));
    assert!(make_attacking(&mut t.g, pacified, Entity::Player(P1)));
}

#[test]
fn defending_player_is_the_player_the_creature_attacks() {
    cr!("508.5");
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P2, "Jace Beleren");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    set_protector(&mut t, battle, P1);
    let c = t.battlefield(P0, "Craw Wurm");
    declare(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Object(jace)),
            (c, Entity::Object(battle)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    let dp = |t: &TestGame, id| t.g.defending_player_for(&Ctx::new(Some(id), P0));
    assert_eq!(dp(&t, a), Some(P1));
    assert_eq!(dp(&t, b), Some(P2), "the planeswalker's controller");
    assert_eq!(dp(&t, c), Some(P1), "the battle's protector");
    // After it's removed from combat, it's still the player it was attacking.
    mtg_engine::combat::remove_from_combat(&mut t.g, b);
    assert!(!t.g.is_attacking(b));
    assert_eq!(dp(&t, b), Some(P2));
}

#[test]
fn defending_player_is_determined_for_each_attacking_creature() {
    cr!("508.5a");
    // "Whenever this creature attacks, defending player loses 1 life."
    let raider = || {
        custom_card(
            "Needling Raider",
            "Creature — Human",
            Some((1, 1)),
            "Whenever this creature attacks, defending player loses 1 life.",
        )
    };
    let mut t = TestGame::new(3);
    let a = bf(&mut t, P0, raider());
    let b = bf(&mut t, P0, raider());
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    // Each trigger refers to one specific defending player — not all of them.
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P2), 19);
    let mut t = TestGame::new(3);
    let a = bf(&mut t, P0, raider());
    t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 19);
}

#[test]
fn attacking_and_attacked_a_player() {
    cr!("508.6");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(player_is_attacking(&t.g, P0, P1));
    assert!(player_has_attacked(&t.g, P0, P1));
    // Put onto the battlefield attacking P2: P0 is attacking P2 but hasn't attacked P2.
    enter_with(
        &mut t,
        P0,
        vanilla("Late", 1, 1),
        Some(Entity::Player(P2)),
        None,
    );
    assert!(player_is_attacking(&t.g, P0, P2));
    assert!(!player_has_attacked(&t.g, P0, P2));
    // After combat, P0 isn't attacking anyone but has still attacked P1 this turn.
    go_to(&mut t, Step::PostcombatMain);
    assert!(!player_is_attacking(&t.g, P0, P1));
    assert!(player_has_attacked(&t.g, P0, P1));
}

#[test]
fn reselecting_what_a_creature_attacks() {
    cr!("508.7", "508.7a", "508.7b");
    let mut t = TestGame::new(3);
    let a = bf(
        &mut t,
        P0,
        custom_card(
            "Charging Herald",
            "Creature — Human",
            Some((2, 2)),
            "Whenever this creature attacks, you gain 1 life.",
        ),
    );
    // P2 has Propaganda: attacking P2 would normally cost {2}.
    t.battlefield(P2, "Propaganda");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert!(reselect_attack_target(&mut t.g, a, Entity::Player(P2)));
    t.g.flush_events();
    t.resolve_all();
    // Not removed from combat, not attacking a second time.
    assert!(t.g.is_attacking(a));
    assert_eq!(t.life(P0), 21);
    assert_eq!(attack_target(&t, a), Some(Entity::Player(P2)));
    let info = t.g.combat.as_ref().unwrap().attacker(a).unwrap().clone();
    assert_eq!(info.original_target, Some(Entity::Player(P1)));
    assert!(player_has_attacked(&t.g, P0, P1));
    assert!(!player_has_attacked(&t.g, P0, P2));
    assert_eq!(t.g.defending_player_for(&Ctx::new(Some(a), P0)), Some(P2));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn reselected_target_must_be_an_opponent_or_their_permanent() {
    cr!("508.7c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let my_jace = t.battlefield(P0, "Jace Beleren");
    let their_jace = t.battlefield(P1, "Jace Beleren");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!reselect_attack_target(&mut t.g, a, Entity::Player(P0)));
    assert!(!reselect_attack_target(
        &mut t.g,
        a,
        Entity::Object(my_jace)
    ));
    assert!(!reselect_attack_target(
        &mut t.g,
        a,
        Entity::Object(their_bears)
    ));
    assert_eq!(attack_target(&t, a), Some(Entity::Player(P1)));
    assert!(reselect_attack_target(
        &mut t.g,
        a,
        Entity::Object(their_jace)
    ));
    assert_eq!(attack_target(&t, a), Some(Entity::Object(their_jace)));
}

#[test]
fn reselection_limited_to_the_chosen_defending_player_without_attack_multiple_players() {
    cr!("508.7d");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: false,
            ..Default::default()
        },
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    let jace1 = t.battlefield(P1, "Jace Beleren");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!reselect_attack_target(&mut t.g, a, Entity::Player(P2)));
    assert!(reselect_attack_target(&mut t.g, a, Entity::Object(jace1)));
}

#[test]
fn reselection_limited_by_range_of_influence() {
    cr!("508.7e");
    let mut t = TestGame::with_config(
        5,
        GameConfig {
            range_of_influence: Some(1),
            ..Default::default()
        },
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    // A Siege controlled by P4 (in range) but protected by P3 (out of range), and one
    // controlled by P2 (out of range) but protected by P1 (in range).
    let far = t.battlefield(P4, "Invasion of Azgol");
    set_protector(&mut t, far, P3);
    let near = t.battlefield(P2, "Invasion of Azgol");
    set_protector(&mut t, near, P1);
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(
        t.g.combat.as_ref().unwrap().defending_players,
        vec![P1, P4],
        "only opponents within range are attacked"
    );
    assert!(!reselect_attack_target(&mut t.g, a, Entity::Player(P2)));
    assert!(!reselect_attack_target(&mut t.g, a, Entity::Object(far)));
    assert!(reselect_attack_target(&mut t.g, a, Entity::Object(near)));
    assert!(reselect_attack_target(&mut t.g, a, Entity::Player(P4)));
}

#[test]
fn creature_put_onto_battlefield_attacking_keeps_the_blockers_and_damage_steps() {
    cr!("508.8");
    let mut t = TestGame::new(2);
    // No attackers are declared...
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
    // ...but a creature is put onto the battlefield attacking during the step.
    enter_with(
        &mut t,
        P0,
        vanilla("Ambusher", 3, 3),
        Some(Entity::Player(P1)),
        None,
    );
    go_to(&mut t, Step::PostcombatMain);
    assert!(steps_this_turn(&t).contains(&Step::DeclareBlockers));
    assert!(steps_this_turn(&t).contains(&Step::CombatDamage));
    assert_eq!(t.life(P1), 17);

    // With no attackers at all, both steps are skipped.
    let mut t = TestGame::new(2);
    go_to(&mut t, Step::PostcombatMain);
    assert!(!steps_this_turn(&t).contains(&Step::DeclareBlockers));
    assert!(!steps_this_turn(&t).contains(&Step::CombatDamage));
}

#[test]
fn skipping_applies_only_to_that_combat_phase() {
    cr!("508.8");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_extra_combat(true);
    // First combat: no attack. Second combat: the bears attack.
    declare(&mut t, &[]);
    go_to(&mut t, Step::EndOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    // The additional main phase, then the second combat.
    go_to(&mut t, Step::PostcombatMain);
    go_to(&mut t, Step::EndOfCombat);
    let log = steps_this_turn(&t);
    assert_eq!(
        log.iter().filter(|s| **s == Step::DeclareBlockers).count(),
        1
    );
    assert_eq!(t.life(P1), 18);
}
