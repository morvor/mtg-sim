//! CR 614.1c / 122.6 audit: the counters a permanent enters with. Every "enters with"
//! effect that applies (its own, a copied one, a copy effect's exception, another
//! permanent's) adds to the counters it's given as it enters; that is one event of
//! putting counters on it, so an effect modifying "one or more +1/+1 counters would be
//! put" applies to it once (614.5), not once for each effect that contributed.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Spark Double, copying the Spike Feeder P0 controls.
fn spark_double_copying_feeder(t: &mut TestGame) -> ObjectId {
    supported("Spark Double");
    supported("Spike Feeder");
    let feeder = t.named_on_battlefield("Spike Feeder")[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(feeder)]);
    let sd = cast_resolve(t, P0, "Spark Double", &[]);
    t.g.current(sd)
}

#[test]
fn spark_double_copying_spike_feeder_enters_with_its_counters_and_one_more() {
    cr!("614.1c", "614.12", "707.9b");
    ruling!(
        "Spark Double",
        "“[this permanent] enters the battlefield with” abilities of the chosen permanent will also work."
    );
    ruling!(
        "Spark Double",
        "Spark Double enters with a +1/+1 counter on it, plus any counters that will be put on it from abilities it copied"
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Spike Feeder");
    let sd = spark_double_copying_feeder(&mut t);
    assert_eq!(t.obj_now(sd).chars.name.as_str(), "Spike Feeder");
    assert_eq!(plus_counters(&t, sd), 3);
}

#[test]
fn hardened_scales_adds_one_counter_to_spark_doubles_whole_entry() {
    cr!("614.1c", "614.5", "122.6", "614.16");
    ruling!(
        "Hardened Scales",
        "If a creature you control would enter the battlefield with a number of +1/+1 counters on it, it enters with that many plus one instead."
    );
    interp!("entering-counters-one-event");
    let mut t = TestGame::new(2);
    t.enter(P0, "Spike Feeder");
    t.battlefield(P0, "Hardened Scales");
    let sd = spark_double_copying_feeder(&mut t);
    // Two from the copied ability, one from the copy's exception: three counters put on
    // it as it enters, plus one.
    assert_eq!(plus_counters(&t, sd), 4);
}

#[test]
fn hardened_scales_adds_one_counter_to_a_feeder_entering_under_master_biomancer() {
    cr!("614.1c", "614.5", "122.6");
    interp!("entering-counters-one-event");
    supported("Master Biomancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Biomancer");
    t.battlefield(P0, "Hardened Scales");
    // Spike Feeder's two, Master Biomancer's two (its power), plus one.
    let feeder = cast_resolve(&mut t, P0, "Spike Feeder", &[]);
    assert_eq!(plus_counters(&t, feeder), 5);
}

#[test]
fn corpsejack_doubles_all_the_counters_a_feeder_enters_with() {
    cr!("614.1c", "614.5", "122.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Biomancer");
    t.battlefield(P0, "Corpsejack Menace");
    let feeder = cast_resolve(&mut t, P0, "Spike Feeder", &[]);
    assert_eq!(plus_counters(&t, feeder), 8);
}

#[test]
fn a_clone_copying_nyxathid_chooses_an_opponent_of_its_own_controller() {
    cr!("616.1c", "616.1f", "614.12a", "707.2");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let nyx = cast_resolve(&mut t, P0, "Nyxathid", &[]);
    let nyx = t.g.current(nyx);
    assert_eq!(t.obj_now(nyx).choices.player, Some(P1));
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    // P1's Clone copies it; as it enters, P1 chooses one of P1's opponents (P2).
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(nyx)]);
    t.answer_choose(P1, &[Entity::Player(P2)]);
    let clone = cast_resolve(&mut t, P1, "Clone", &[]);
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Nyxathid");
    assert_eq!(t.obj_now(clone).choices.player, Some(P2));
    let asked_opponents: Vec<Vec<Entity>> = t
        .asked()
        .iter()
        .filter(|(p, _)| *p == P1)
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
                if candidates.iter().all(|e| e.player().is_some()) =>
            {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(asked_opponents.len(), 1);
    let mut cands = asked_opponents[0].clone();
    cands.sort_by_key(|e| e.player().map(|p| p.0));
    assert_eq!(cands, vec![Entity::Player(P0), Entity::Player(P2)]);
}
