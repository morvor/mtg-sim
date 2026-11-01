//! Rulings batch P123 — "Whenever you gain life, put a +1/+1 counter on ~" (Ajani's
//! Pridemate and friends): one trigger per life-gaining event however much life is gained
//! (CR 119.9, 603.2), one event per lifelink source dealing combat damage (CR 510.2,
//! 702.15b), a creature dealt lethal damage at the same time dies before the counter
//! arrives (CR 704.5g, 603.3), only your own life gain in Two-Headed Giant (CR 810.9),
//! and last known information (CR 608.2h).

use crate::r_p123_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A "whenever you gain life" permanent: the counters it gets and how many per trigger.
struct Gainer {
    name: &'static str,
    kind: &'static str,
    per: u32,
}

const fn g(name: &'static str, kind: &'static str, per: u32) -> Gainer {
    Gainer { name, kind, per }
}

const ALL: [Gainer; 10] = [
    g("Ajani's Pridemate", "+1/+1", 1),
    g("Blood Researcher", "+1/+1", 1),
    g("Bloodthirsty Aerialist", "+1/+1", 1),
    g("Celestial Unicorn", "+1/+1", 1),
    g("Excalibur II", "charge", 1),
    g("Gideon's Company", "+1/+1", 2),
    g("MJ, Rising Star", "+1/+1", 1),
    g("Scion of the Swarm", "+1/+1", 1),
    g("Vampire Scrivener", "+1/+1", 1),
    g("Wall of Limbs", "+1/+1", 1),
];

/// The triggers of `gainer` an event produced: its counters before and after, divided by
/// the counters per trigger.
fn triggers(t: &mut TestGame, gainer: &Gainer, id: ObjectId, f: impl FnOnce(&mut TestGame)) -> u32 {
    let before = t.counters(id, gainer.kind);
    f(t);
    t.resolve_all();
    (t.counters(id, gainer.kind) - before) / gainer.per
}

#[test]
fn one_trigger_per_life_gaining_event_however_much() {
    cr!("119.9", "603.2");
    ruling!(
        "Ajani's Pridemate",
        "The ability of Ajani's Pridemate triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    ruling!(
        "Bloodthirsty Aerialist",
        "The ability of Bloodthirsty Aerialist triggers just once for each life-gaining event, whether it's 1 life from Impassioned Orator or 4 life from Dawning Angel."
    );
    ruling!(
        "Celestial Unicorn",
        "The ability of Celestial Unicorn triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    ruling!(
        "Excalibur II",
        "Excalibur II's first ability triggers just once for each life-gaining event, whether it's 1 life from Al Bhed Salvagers or 3 life from Balamb T-Rexaur."
    );
    ruling!(
        "Gideon's Company",
        "The ability of Gideon’s Company triggers just once for each life-gaining event, whether it’s 1 life from Ajani’s Welcome or 2 life from Battlefield Promotion."
    );
    ruling!(
        "MJ, Rising Star",
        "The ability of MJ, Rising Star triggers just once for each life-gaining event, regardless of how much life is gained."
    );
    ruling!(
        "Wall of Limbs",
        "The ability triggers just once for each life-gaining event, whether it’s 1 life from Soulmender or 8 life from Meditation Puzzle."
    );
    ruling!(
        "Vampire Scrivener",
        "The first ability of Vampire Scrivener triggers just once for each life-gaining or life-losing event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    ruling!(
        "Scion of the Swarm",
        "The last ability of Scion of the Swarm triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    for gainer in &ALL {
        supported(gainer.name);
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, gainer.name);
        for n in [1, 4, 8] {
            let k = triggers(&mut t, gainer, id, |t| gain_life(t, P0, n));
            assert_eq!(k, 1, "{} gaining {n}", gainer.name);
        }
    }
    // Vampire Scrivener's other ability: once per life-losing event.
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, "Vampire Scrivener");
    for n in [1, 4] {
        let k = triggers(&mut t, &ALL[8], id, |t| {
            t.g.lose_life(P0, n);
            t.g.flush_events();
            t.settle();
        });
        assert_eq!(k, 1, "losing {n}");
    }
}

#[test]
fn life_for_each_is_one_event() {
    cr!("119.9", "603.2");
    ruling!(
        "Ajani's Pridemate",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Ajani's Pridemate's ability triggers only once."
    );
    ruling!(
        "Celestial Unicorn",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Celestial Unicorn's ability triggers only once."
    );
    ruling!(
        "Excalibur II",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Excalibur II's first ability triggers only once."
    );
    ruling!(
        "MJ, Rising Star",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and MJ, Rising Star's ability triggers only once."
    );
    ruling!(
        "Scion of the Swarm",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Scion of the Swarm's ability triggers only once."
    );
    ruling!(
        "Vampire Scrivener",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Vampire Scrivener's first ability triggers only once."
    );
    ruling!(
        "Gideon's Company",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability of Gideon’s Company triggers only once."
    );
    ruling!(
        "Bloodthirsty Aerialist",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Bloodthirsty Aerialist triggers only once."
    );
    supported("Congregate");
    for gainer in &ALL {
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, gainer.name);
        t.lands(P0, "Grizzly Bears", 3);
        // Congregate: 2 life for each creature on the battlefield.
        let k = triggers(&mut t, gainer, id, |t| {
            cast_resolve_at(t, P0, "Congregate", &[Entity::Player(P0)]);
        });
        assert!(t.life(P0) >= 26, "{}", gainer.name);
        assert_eq!(k, 1, "{}", gainer.name);
    }
}

#[test]
fn each_lifelink_creature_is_a_separate_event() {
    cr!("510.2", "702.15b", "119.9", "603.2");
    ruling!(
        "Blood Researcher",
        "Each creature with lifelink dealing combat damage causes a separate life gain event. For example, if two creatures you control with lifelink deal combat damage at the same time, a “whenever you gain life” ability will trigger twice."
    );
    ruling!(
        "Bloodthirsty Aerialist",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Bloodthirsty Aerialist's ability will trigger twice."
    );
    ruling!(
        "Excalibur II",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Excalibur II's ability will trigger twice."
    );
    ruling!(
        "Gideon's Company",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Gideon’s Company’s ability will trigger twice."
    );
    ruling!(
        "MJ, Rising Star",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, MJ, Rising Star's ability will trigger twice."
    );
    ruling!(
        "Scion of the Swarm",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Scion of hte Swarm's ability will trigger twice."
    );
    ruling!(
        "Vampire Scrivener",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Vampire Scrivener's  firstability will trigger twice."
    );
    let leech = custom_card(
        "Trampling Leech",
        "Creature — Leech",
        "{4}",
        Some((5, 5)),
        "Trample\nLifelink",
    );
    for gainer in ALL.iter().filter(|g| g.name != "Wall of Limbs") {
        // Two lifelink creatures deal combat damage at the same time: two triggers.
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, gainer.name);
        let a = t.battlefield(P0, "Vampire Nighthawk");
        let b = t.battlefield(P0, "Vampire Nighthawk");
        let k = triggers(&mut t, gainer, id, |t| {
            t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
        });
        assert_eq!(t.life(P0), 24);
        assert_eq!(k, 2, "{}: two lifelink creatures", gainer.name);
        // One lifelink creature dealing damage to a blocker and the player: one trigger.
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, gainer.name);
        let l = t.custom(P0, leech.clone(), Zone::Battlefield);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let k = triggers(&mut t, gainer, id, |t| {
            t.attack(&[(l, Entity::Player(P1))], &[(bears, l)]);
        });
        assert_eq!(t.life(P0), 25);
        assert!(t.life(P1) < 20, "trample damage to the player");
        assert_eq!(k, 1, "{}: one trampling lifelink creature", gainer.name);
    }
}

#[test]
fn dealt_lethal_damage_as_you_gain_life_dies_first() {
    cr!("704.5g", "603.3", "510.2");
    ruling!(
        "Bloodthirsty Aerialist",
        "If Bloodthirsty Aerialist is dealt lethal damage at the same time that you gain life, it won't receive a counter from its ability in time to save it."
    );
    ruling!(
        "Celestial Unicorn",
        "If Celestial Unicorn is dealt lethal damage at the same time that you gain life, it won't receive a counter from its ability in time to save it."
    );
    ruling!(
        "Gideon's Company",
        "If Gideon’s Company is dealt lethal damage at the same time that you gain life, it won’t receive counters from its ability in time to save it."
    );
    ruling!(
        "MJ, Rising Star",
        "If MJ, Rising Star is dealt lethal damage at the same time that you gain life, she won't receive a counter from her ability in time to save her."
    );
    ruling!(
        "Scion of the Swarm",
        "If Scion of the Swarm is dealt lethal damage at the same time that you gain life, it won't receive a counter from its ability in time to save it."
    );
    ruling!(
        "Vampire Scrivener",
        "If Vampire Scrivener is dealt lethal damage at the same time that you gain or lose life during your turn, it won't receive a counter from its ability in time to save it."
    );
    ruling!(
        "Blood Researcher",
        "If you gain life at the same time Blood Researcher is dealt lethal damage, it dies before its ability can put a +1/+1 counter on it."
    );
    ruling!(
        "Wall of Limbs",
        "If you gain life at the same time Wall of Limbs is dealt lethal damage (probably because you control a blocking creature with lifelink), Wall of Limbs’s triggered ability will trigger, but Wall of Limbs will be destroyed before the ability resolves."
    );
    // On P1's turn the creature blocks P1's Craw Wurm (6/4) while P0's Vampire Nighthawk
    // blocks a Grizzly Bears; Vampire Scrivener ("during your turn") instead attacks on
    // P0's turn and is blocked by Giant Spider (reach) while a Nighthawk deals damage to P1.
    for gainer in ALL.iter().filter(|g| g.name != "Excalibur II" && g.name != "Ajani's Pridemate") {
        let mut t = TestGame::new(2);
        let (id, ap) = if gainer.name != "Vampire Scrivener" {
            let wall = t.battlefield(P0, gainer.name);
            let hawk = t.battlefield(P0, "Vampire Nighthawk");
            let wurm = t.battlefield(P1, "Craw Wurm");
            let bears = t.battlefield(P1, "Grizzly Bears");
            to_combat(&mut t, P1);
            to_blockers(
                &mut t,
                &[(wurm, Entity::Player(P0)), (bears, Entity::Player(P0))],
                &[(wall, wurm), (hawk, bears)],
            );
            (wall, P1)
        } else {
            let id = t.battlefield(P0, gainer.name);
            let hawk = t.battlefield(P0, "Vampire Nighthawk");
            let wurm = t.battlefield(P1, "Giant Spider");
            to_combat(&mut t, P0);
            to_blockers(
                &mut t,
                &[(id, Entity::Player(P1)), (hawk, Entity::Player(P1))],
                &[(wurm, id)],
            );
            (id, P0)
        };
        t.advance_to(ap, Step::CombatDamage);
        t.settle();
        assert_eq!(t.life(P0), 22, "{}", gainer.name);
        assert_eq!(t.zone(id), Zone::Graveyard(P0), "{}: died", gainer.name);
        assert_eq!(
            triggers_on_stack(&t, "gain life"),
            1,
            "{}: it triggered",
            gainer.name
        );
        t.resolve_all();
        assert_eq!(t.zone(id), Zone::Graveyard(P0));
    }
}

#[test]
fn two_headed_giant_teammate_life_doesnt_trigger() {
    cr!("810.9", "603.2");
    ruling!(
        "Ajani's Pridemate",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Ajani's Pridemate's ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "Celestial Unicorn",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Celestial Unicorn's ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "Excalibur II",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Excalibur II's first ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "MJ, Rising Star",
        "In a Two-Headed Giant game, life gained by your teammate won't cause MJ, Rising Star's ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "Scion of the Swarm",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Scion of the Swarm's ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "Vampire Scrivener",
        "In a Two-Headed Giant game, life gained or lost by your teammate during your turn won't cause either of Vampire Scrivener's abilities to trigger, even though it caused your team's life total to increase or decrease."
    );
    for gainer in &ALL {
        let mut t = two_headed_giant();
        let id = t.battlefield(P0, gainer.name);
        let life = t.life(P0);
        let k = triggers(&mut t, gainer, id, |t| gain_life(t, P1, 3));
        assert_eq!(t.life(P0), life + 3, "the team's life total");
        assert_eq!(k, 0, "{}: the teammate gained life", gainer.name);
        let k = triggers(&mut t, gainer, id, |t| gain_life(t, P0, 3));
        assert_eq!(k, 1, "{}: you gained life", gainer.name);
    }
    // Vampire Scrivener: the teammate losing life during your turn.
    let mut t = two_headed_giant();
    let id = t.battlefield(P0, "Vampire Scrivener");
    let k = triggers(&mut t, &ALL[8], id, |t| {
        t.g.lose_life(P1, 2);
        t.g.flush_events();
        t.settle();
    });
    assert_eq!(k, 0);
}

#[test]
fn gideons_company_loyalty_counter_doesnt_activate_anything() {
    cr!("606.3", "122.1");
    ruling!(
        "Gideon's Company",
        "Putting a loyalty counter on a Gideon planeswalker doesn’t cause any of its activated abilities to become activated."
    );
    let mut t = TestGame::new(2);
    let company = t.battlefield(P0, "Gideon's Company");
    let gideon = t.battlefield(P0, "Gideon, Ally of Zendikar");
    let loyalty = t.counters(gideon, "loyalty");
    let tokens = crate::r_s01_common::tokens(&t, P0).len();
    t.lands(P0, "Plains", 4);
    t.answer_targets(P0, &[obj(gideon)]);
    activate_containing(&mut t, P0, company, "loyalty counter").expect("activated");
    t.resolve_all();
    assert_eq!(t.counters(gideon, "loyalty"), loyalty + 1);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), tokens);
    assert_eq!(t.stack_len(), 0);
    // A loyalty ability can still be activated this turn.
    assert!(can_activate(&mut t, P0, gideon));
}

#[test]
fn wall_of_limbs_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Wall of Limbs",
        "Use Wall of Limbs’s power as it last existed on the battlefield, including any +1/+1 counters it had, to determine the value of X."
    );
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Limbs");
    give_plus1(&mut t, wall, 3);
    t.lands(P0, "Swamp", 7);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, wall, "Sacrifice").expect("activated");
    assert!(t.in_graveyard(P0, "Wall of Limbs"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
