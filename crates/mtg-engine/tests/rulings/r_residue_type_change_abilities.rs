//! Item `rulings-residue-type-change` — interaction rulings about when abilities make
//! their choices and check their conditions (on resolution, CR 608.2), target legality
//! on resolution (CR 608.2b), attack requirements (CR 508.1d), and counters that grant
//! abilities and types (CR 122.1b, 613.1d).

use crate::r_residue_type_change_common::*;
use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn whispering_specter_choice_and_poison_count_on_resolution() {
    cr!("608.2c", "608.2h", "702.90b");
    ruling!(
        "Whispering Specter",
        "You choose whether or not to sacrifice Whispering Specter, and the number of poison counters the player has is counted, when the triggered ability resolves."
    );
    supported("Whispering Specter");
    let mut t = TestGame::new(2);
    let spec = t.battlefield(P0, "Whispering Specter");
    for _ in 0..6 {
        t.hand(P1, "Island");
    }
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(spec, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.g.player(P1).counter("poison"), 1);
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    // Two more poison counters while the trigger waits; the choice isn't made yet.
    let from = t.asked().len();
    crate::r_p035_common::put(&mut t, Entity::Player(P1), "poison", 2);
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(spec)), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P1), 3);
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn body_snatcher_discard_chosen_on_resolution() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Body Snatcher",
        "You choose whether to discard a card or not as the enters-the-battlefield ability resolves."
    );
    supported("Body Snatcher");
    let mut t = TestGame::new(2);
    // P0 has no creature card as the ability triggers ...
    let s = t.enter(P0, "Body Snatcher");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // ... but gets one before it resolves, and discards it then.
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(s));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn phyrexian_dreadnought_trigger_chosen_on_resolution() {
    cr!("603.2", "608.2c", "701.21a");
    ruling!(
        "Phyrexian Dreadnought",
        "Reverted to its original wording, this now has an \"enters\" triggered ability. During resolution of the triggered ability, you choose one option or the other."
    );
    ruling!(
        "Phyrexian Dreadnought",
        "If you are unable to sacrifice 12 power worth of creatures, then you cannot choose that option; in that case, you must sacrifice Phyrexian Dreadnought."
    );
    supported("Phyrexian Dreadnought");
    // Enough power as it triggers, but not as it resolves: the Dreadnought is sacrificed.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giants: Vec<_> = (0..4).map(|_| t.battlefield(P0, "Hill Giant")).collect();
    let d = t.enter(P0, "Phyrexian Dreadnought");
    t.settle();
    // A triggered ability: the Dreadnought is on the battlefield with it on the stack.
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(d));
    assert_eq!(pt_now(&mut t, d), (12, 12));
    cast_resolve_top(&mut t, giants[0]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phyrexian Dreadnought"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    // With enough power as it resolves, P0 may sacrifice the creatures instead.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giants: Vec<_> = (0..4).map(|_| t.battlefield(P0, "Hill Giant")).collect();
    let d = t.enter(P0, "Phyrexian Dreadnought");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &giants.iter().map(|g| obj(*g)).collect::<Vec<_>>());
    t.resolve_all();
    assert!(t.on_battlefield(d));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

/// P1 casts Lightning Bolt at the creature and only it resolves.
fn cast_resolve_top(t: &mut TestGame, target: ObjectId) {
    cast_new(t, P1, "Lightning Bolt", &[obj(target)]);
    t.resolve();
}

#[test]
fn aysen_bureaucrats_checks_power_on_activation_and_resolution() {
    cr!("115.1", "608.2b", "602.2b");
    ruling!(
        "Aysen Bureaucrats",
        "This checks the power of the creature on activation and on resolution."
    );
    supported("Aysen Bureaucrats");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.battlefield(P0, "Aysen Bureaucrats");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // On activation: the Hill Giant (power 3) isn't a legal target.
    let from = t.asked().len();
    t.activate(P0, b, 0, &[obj(bears)]).unwrap();
    let cands = asked_since(&t, from)
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(cands.contains(&obj(bears)) && !cands.contains(&obj(giant)));
    // On resolution: the Bears got +3/+3 in response, so the ability does nothing.
    giant_growth(&mut t, bears);
    t.resolve_all();
    assert!(!tapped(&t, bears));
}

#[test]
fn viashino_bey_is_a_constraint_on_declaring_attackers() {
    cr!("508.1d", "508.1c");
    interp!("requirements-of-creatures-unable-to-attack");
    ruling!(
        "Viashino Bey",
        "The ability is not a triggered ability. It is a constraint on declaring an attack."
    );
    supported("Viashino Bey");
    let mut t = TestGame::new(2);
    let bey = t.battlefield(P0, "Viashino Bey");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // A summoning-sick creature and a tapped one can't attack: Viashino Bey can still
    // attack without them (regression: their requirements counted as obeyed only when
    // Bey didn't attack, so Bey couldn't attack at all).
    t.battlefield_sick(P0, "Hill Giant");
    let tapped_elves = t.battlefield(P0, "Llanowar Elves");
    t.g.objects[tapped_elves.0 as usize].tapped = true;
    let p1 = Entity::Player(P1);
    let opts = attack_options(&t.g);
    assert!(!attack_declaration_legal(&t.g, &opts, &[(bey, p1)]));
    assert!(attack_declaration_legal(&t.g, &opts, &[(bears, p1)]));
    assert!(attack_declaration_legal(
        &t.g,
        &opts,
        &[(bey, p1), (bears, p1)]
    ));
    attack_with(&mut t, &[(bey, p1), (bears, p1)]);
    // No triggered ability: nothing goes on the stack.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn phantasmal_form_with_no_targets_or_only_illegal_targets() {
    cr!("608.2b", "601.2c", "115.1");
    ruling!(
        "Phantasmal Form",
        "You may cast Phantasmal Form without choosing any target creatures. You'll just draw a card. However, if you choose any targets and all of those targets become illegal before the spell tries to resolve, the spell won't resolve and you won't draw a card."
    );
    supported("Phantasmal Form");
    // No targets: P0 just draws.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Phantasmal Form");
    let form = t.hand(P0, "Phantasmal Form");
    let n = t.hand_size(P0);
    t.cast(P0, form).targets(&[]).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), n);
    assert_eq!(pt_now(&mut t, bears), (2, 2));
    // Its only target became illegal: no card drawn.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Phantasmal Form");
    let form = t.hand(P0, "Phantasmal Form");
    let n = t.hand_size(P0);
    t.cast(P0, form).targets(&[obj(bears)]).go();
    cast_resolve_top(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), n - 1);
    assert!(t.in_graveyard(P0, "Phantasmal Form"));
}

#[test]
fn minas_morgul_shadow_counter_grants_shadow_and_wraith() {
    cr!("122.1b", "613.1d", "613.1f", "702.28b");
    ruling!(
        "Minas Morgul, Dark Fortress",
        "A permanent with a shadow counter on it has shadow."
    );
    supported("Minas Morgul, Dark Fortress");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mm = t.battlefield(P0, "Minas Morgul, Dark Fortress");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, mm, 1, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "shadow"), 1);
    assert!(has_kw(&mut t, bears, KeywordKind::Shadow));
    assert!(subtype(&mut t, bears, "Wraith") && subtype(&mut t, bears, "Bear"));
    // Without the counter, neither.
    t.g.remove_counters(Entity::Object(bears), "shadow", 1);
    t.settle();
    assert!(!has_kw(&mut t, bears, KeywordKind::Shadow));
    assert!(!subtype(&mut t, bears, "Wraith"));
}

#[test]
fn aurification_gold_counter_makes_a_wall_with_defender() {
    cr!("205.1b", "613.1d", "613.1f", "702.3b");
    ruling!(
        "Aurification",
        "Aurification gives the creature type “Wall” to creatures with gold counters on them in addition to granting them defender."
    );
    supported("Aurification");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aurification");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(bears, "gold"), 1);
    assert_eq!(creature_types(&mut t, bears), vec!["Bear", "Wall"]);
    assert!(has_kw(&mut t, bears, KeywordKind::Defender));
}
