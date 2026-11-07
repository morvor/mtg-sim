//! Rulings batch P220 — overload (CR 702.96): March of Progress, Corporeal Projection,
//! Chemister's Trick, Dragonshift, Teleportal, Mind Rake.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::untapped_lands;
use crate::r_s06_common::give_control;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

fn lands(t: &mut TestGame, p: PlayerId, spec: &[(&str, usize)]) {
    for (name, n) in spec {
        t.lands(p, name, *n);
    }
}

/// Whether target choices were asked since decision `from`.
fn targets_asked(t: &TestGame, from: usize) -> bool {
    asked_since(t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
}

#[test]
fn overloaded_spells_keep_their_mana_cost() {
    cr!("702.96a", "118.9", "202.3");
    ruling!(
        "Corporeal Projection",
        "Casting a spell with overload doesn't change that spell's mana cost. You just pay the overload cost instead."
    );
    ruling!(
        "March of Progress",
        "Casting a spell with overload doesn’t change that spell’s mana cost. You just pay the overload cost instead."
    );
    supported("March of Progress");
    for (name, mv, pay) in [
        (
            "Corporeal Projection",
            2,
            vec![("Island", 2), ("Mountain", 2), ("Wastes", 3)],
        ),
        ("March of Progress", 3, vec![("Island", 1), ("Wastes", 6)]),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ornithopter");
        lands(&mut t, P0, &pay);
        let c = t.hand(P0, name);
        let cost = format!("{}", t.obj(c).chars.mana_cost.clone().unwrap());
        let spell = t.cast(P0, c).method(OVERLOAD).go();
        assert_eq!(untapped_lands(&t, P0), 0, "{name}");
        assert_eq!(t.g.mana_value_of(spell), mv, "{name}");
        assert_eq!(
            format!("{}", t.obj(spell).chars.mana_cost.clone().unwrap()),
            cost,
            "{name}"
        );
    }
}

#[test]
fn march_of_progress_has_one_target_or_none_when_overloaded() {
    cr!("702.96a", "702.96b", "707.2");
    ruling!(
        "March of Progress",
        "If you don’t pay the overload cost of a spell, that spell will have a single target. If you pay the overload cost, the spell won’t have any targets."
    );
    // Normally: one target artifact creature you control; one token copy of it.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Steel Wall");
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "March of Progress");
    t.cast(P0, m).target(thopter).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
    assert_eq!(t.named_on_battlefield("Steel Wall").len(), 1);
    // Overloaded: no targets; a copy of each artifact creature you control.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Steel Wall");
    t.battlefield(P1, "Ornithopter");
    lands(&mut t, P0, &[("Island", 1), ("Wastes", 6)]);
    let m = t.hand(P0, "March of Progress");
    let from = t.asked().len();
    t.cast(P0, m).method(OVERLOAD).go();
    assert!(!targets_asked(&t, from));
    t.resolve_all();
    let mine = |name: &str| {
        t.named_on_battlefield(name)
            .into_iter()
            .filter(|x| t.obj(*x).controller == P0)
            .count()
    };
    assert_eq!(mine("Ornithopter"), 2);
    assert_eq!(mine("Steel Wall"), 2);
    assert_eq!(tokens(&t, P1).len(), 0);
}

#[test]
fn overloaded_march_of_progress_ignores_hexproof() {
    cr!("702.96a", "702.16b", "115.1");
    ruling!(
        "March of Progress",
        "Because a spell with overload doesn’t target when its overload cost is paid, it may affect permanents with hexproof or with protection from the appropriate color."
    );
    // Sword of Body and Mind: "Equipped creature gets +2/+2 and has protection from
    // green and from blue." The equipped Ornithopter can't be the target of the blue
    // March of Progress, but the overloaded one copies it.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let sword = t.battlefield(P0, "Sword of Body and Mind");
    t.g.attach(sword, Entity::Object(thopter));
    t.g.recompute();
    lands(&mut t, P0, &[("Island", 1), ("Wastes", 6)]);
    let m = t.hand(P0, "March of Progress");
    // It can't be targeted: the targeted mode isn't castable.
    assert!(!can_cast(&mut t, P0, m, CastMethod::Normal));
    t.cast(P0, m).method(OVERLOAD).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
}

#[test]
fn march_of_progress_cast_without_paying_its_mana_cost_cant_be_overloaded() {
    cr!("702.96a", "118.9a");
    ruling!(
        "March of Progress",
        "If you are instructed to cast a spell with overload “without paying its mana cost,” you can’t choose to pay its overload cost instead."
    );
    // Plotted (cast "without paying its mana cost"), the only way to cast it is the
    // plotted one: not overloaded.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    lands(&mut t, P0, &[("Island", 1), ("Wastes", 6)]);
    let m = t.exile(P0, "March of Progress");
    mtg_engine::kw::plot::make_plotted(&mut t.g, m);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    lands(&mut t, P0, &[("Island", 1), ("Wastes", 6)]);
    t.g.turn.priority = Some(P0);
    let methods: Vec<CastMethod> = t
        .g
        .legal_actions(P0)
        .into_iter()
        .filter_map(|a| match a {
            mtg_engine::decision::Action::Cast { card, method } if card == m => Some(method),
            _ => None,
        })
        .collect();
    assert!(methods.contains(&CastMethod::Keyword(KeywordKind::Plot)));
    assert!(!methods.contains(&OVERLOAD));
    t.cast(P0, m)
        .method(CastMethod::Keyword(KeywordKind::Plot))
        .target(thopter)
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
}

#[test]
fn overload_doesnt_change_when_march_of_progress_can_be_cast() {
    cr!("702.96a", "307.1");
    ruling!(
        "March of Progress",
        "Overload doesn’t change when you can cast the spell."
    );
    // A sorcery: not in combat, overloaded or not.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    lands(&mut t, P0, &[("Island", 1), ("Wastes", 6)]);
    let m = t.hand(P0, "March of Progress");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, m, OVERLOAD));
    assert!(!can_cast(&mut t, P0, m, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, m, OVERLOAD));
}

#[test]
fn overloaded_effects_lock_in_the_creatures_as_they_resolve() {
    cr!("702.96a", "611.2c");
    ruling!(
        "Chemister's Trick",
        "If you cast Chemister's Trick with overload, only creatures you don't control that are on the battlefield when Chemister's Trick resolves are affected. Creatures that come under another player's control later in the turn are not."
    );
    ruling!(
        "Dragonshift",
        "If you cast Dragonshift using overload, only creatures you control when it resolves will be affected. Creatures that come under your control later in the turn will not."
    );
    supported("Chemister's Trick");
    supported("Dragonshift");
    // Chemister's Trick overloaded: P1's Hill Giant gets -2/-0; P0's Bears, given to P1
    // afterwards, don't.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands(&mut t, P0, &[("Island", 1), ("Mountain", 1), ("Wastes", 3)]);
    let c = t.hand(P0, "Chemister's Trick");
    t.cast(P0, c).method(OVERLOAD).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 3));
    give_control(&mut t, bears, P1);
    assert_eq!(t.obj(bears).controller, P1);
    assert_eq!(t.pt(bears), (2, 2));
    // Dragonshift overloaded: P0's Bears become 4/4 Dragons with flying; P1's Hill
    // Giant, gained afterwards, doesn't.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands(&mut t, P0, &[("Island", 2), ("Mountain", 2), ("Wastes", 3)]);
    let c = t.hand(P0, "Dragonshift");
    t.cast(P0, c).method(OVERLOAD).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj(bears).chars.has_subtype("Dragon"));
    assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
    give_control(&mut t, giant, P0);
    assert_eq!(t.obj(giant).controller, P0);
    assert_eq!(t.pt(giant), (3, 3));
    assert!(!t.obj(giant).chars.has_subtype("Dragon"));
    assert!(!t.obj(giant).has_keyword(KeywordKind::Flying));
}

#[test]
fn overloaded_teleportal_locks_the_bonus_but_not_the_evasion() {
    cr!("702.96a", "611.2c", "509.1b");
    ruling!(
        "Teleportal",
        "The set of creatures that gets +1/+0 from an overloaded Teleportal is determined as the spell resolves. Creatures you begin to control later in the turn won't get +1/+0. However, because the second part of Teleportal's effect doesn't change the characteristics of any permanents, the set of creatures that can't be blocked is constantly updated."
    );
    supported("Teleportal");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    lands(&mut t, P0, &[("Island", 1), ("Mountain", 1), ("Wastes", 3)]);
    let c = t.hand(P0, "Teleportal");
    t.cast(P0, c).method(OVERLOAD).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    // P0 gains control of the Hill Giant afterwards: no +1/+0, but it can't be blocked.
    give_control(&mut t, giant, P0);
    t.g.objects[giant.0 as usize].summoning_sick = false;
    assert_eq!(t.pt(giant), (3, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.g.recompute();
    assert!(!t.g.can_block(wall, bears));
    assert!(!t.g.can_block(wall, giant));
}

#[test]
fn overloaded_mind_rake_makes_each_player_discard_including_you() {
    cr!("702.96a", "701.9a");
    ruling!(
        "Mind Rake",
        "While most spells with overload affect only permanents you control or only permanents your opponents control, Mind Rake affects all players, including you, if it's overloaded."
    );
    supported("Mind Rake");
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        for name in ["Island", "Grizzly Bears", "Shock"] {
            t.hand(p, name);
        }
    }
    lands(&mut t, P0, &[("Swamp", 1), ("Wastes", 1)]);
    let mr = t.hand(P0, "Mind Rake");
    t.cast(P0, mr).method(OVERLOAD).go();
    t.resolve_all();
    for p in [P0, P1, P2] {
        assert_eq!(t.hand_size(p), 1, "{p:?}");
    }
}
