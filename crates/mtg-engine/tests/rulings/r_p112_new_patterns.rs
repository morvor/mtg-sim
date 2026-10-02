//! Rulings batch P112 — cards newly compiled by this batch's patterns: "becomes a [type],
//! gets +N/+N, and gains [keywords]" (Dragonsoul Knight, Paragon of the Amesha), "~ and
//! each other creature with the same name as it get" (Cylian Sunsinger), "where X is the
//! number of colors among those creatures" (General Tazri), an intervening "if it has
//! oil counters on it" about the entering creature (Ichorplate Golem) and "counter that
//! spell unless its controller discards a card" (Reality Smasher).

use crate::r_s01_common::*;
use crate::r_s06_common::has_kw;
use crate::r_s13_common::add;
use crate::r_s18_common::rainbow_lands;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn subtypes(t: &TestGame, id: ObjectId) -> Vec<String> {
    let mut v: Vec<String> = t
        .obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn dragonsoul_knight_becomes_only_a_dragon_and_can_activate_again() {
    cr!("205.1a", "613.1d", "613.4c");
    ruling!(
        "Dragonsoul Knight",
        "When Dragonsoul Knight’s activated ability resolves, Dragonsoul Knight will gain flying and trample in addition to its other abilities. However, becoming a Dragon will make it lose all other creature types. It will no longer be a Human or a Knight."
    );
    ruling!(
        "Dragonsoul Knight",
        "You can activate Dragonsoul Knight’s ability more than once during a turn. The second time it resolves, it will probably have no visible effect other than giving Dragonsoul Knight another +5/+3."
    );
    supported("Dragonsoul Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Dragonsoul Knight");
    rainbow_lands(&mut t, P0, 2);
    t.activate(P0, knight, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(subtypes(&t, knight), vec!["Dragon".to_string()]);
    assert_eq!(t.pt(knight), (7, 5));
    for kw in [KeywordKind::FirstStrike, KeywordKind::Flying, KeywordKind::Trample] {
        assert!(has_kw(&t, knight, kw), "{kw:?}");
    }
    t.activate(P0, knight, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(subtypes(&t, knight), vec!["Dragon".to_string()]);
    assert_eq!(t.pt(knight), (12, 8));
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.g.recompute();
    assert_eq!(subtypes(&t, knight), vec!["Human".to_string(), "Knight".to_string()]);
    assert_eq!(t.pt(knight), (2, 2));
    assert!(!has_kw(&t, knight, KeywordKind::Flying));
}

#[test]
fn paragon_of_the_amesha_becomes_an_angel_and_extra_lifelink_is_redundant() {
    cr!("205.1a", "702.15f", "613.4c");
    ruling!(
        "Paragon of the Amesha",
        "When Paragon of the Amesha’s activated ability resolves, Paragon of the Amesha will gain flying and lifelink in addition to its other abilities. However, becoming an Angel will make it lose all other creature types. It will no longer be a Human or a Knight."
    );
    ruling!(
        "Paragon of the Amesha",
        "You can activate Paragon of the Amesha’s ability more than once during a turn. The second time it resolves, it will get another +3/+3 and gain another instance of lifelink. However, multiple instances of lifelink on the same creature are redundant so you won’t gain extra life from the new one."
    );
    supported("Paragon of the Amesha");
    let mut t = TestGame::new(2);
    let paragon = t.battlefield(P0, "Paragon of the Amesha");
    rainbow_lands(&mut t, P0, 2);
    t.activate(P0, paragon, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(subtypes(&t, paragon), vec!["Angel".to_string()]);
    assert_eq!(t.pt(paragon), (5, 5));
    assert!(has_kw(&t, paragon, KeywordKind::Flying));
    assert!(has_kw(&t, paragon, KeywordKind::Lifelink));
    assert!(has_kw(&t, paragon, KeywordKind::FirstStrike));
    t.activate(P0, paragon, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(paragon), (8, 8));
    let lifelinks = t
        .obj_now(paragon)
        .chars
        .abilities
        .iter()
        .filter(|a| {
            matches!(&a.kind, mtg_engine::ability::AbilityKind::Keyword(k)
                if k.kind == KeywordKind::Lifelink)
        })
        .count();
    assert!(lifelinks >= 1);
    t.attack(&[(paragon, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.life(P0), 28);
}

#[test]
fn cylian_sunsinger_pumps_itself_and_creatures_sharing_its_name() {
    cr!("201.2a", "611.2c", "608.2h");
    ruling!(
        "Cylian Sunsinger",
        "When the ability resolves, it gives the +3/+3 bonus to the source of the ability and each other creature that shares a name with that source, even if that source isn't actually named Cylian Sunsinger."
    );
    supported("Cylian Sunsinger");
    supported("Mirrorweave");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cylian Sunsinger");
    let b = t.battlefield(P1, "Cylian Sunsinger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for l in ["Mountain", "Forest", "Plains"] {
        t.lands(P0, l, 1);
    }
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(a), (5, 5));
    assert_eq!(t.pt(b), (5, 5));
    assert_eq!(t.pt(bears), (2, 2));
    // Its name changes in response (Mirrorweave makes it a Grizzly Bears): the bonus goes
    // to it and each other creature named Grizzly Bears.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cylian Sunsinger");
    let other = t.battlefield(P0, "Cylian Sunsinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    for l in ["Mountain", "Forest", "Plains"] {
        t.lands(P0, l, 1);
    }
    t.activate(P0, a, 0, &[]).unwrap();
    cast_new(&mut t, P0, "Mirrorweave", &[bears.into()]);
    t.resolve();
    assert_eq!(t.obj_now(a).chars.name, "Grizzly Bears");
    t.resolve_all();
    for id in [a, other, bears, giant] {
        assert_eq!(t.pt(id), (5, 5), "{:?}", t.obj_now(id).chars.name);
    }
}

/// P0's General Tazri (a white Ally) and `allies`; returns the game and their ids.
fn tazri_with(allies: &[&str]) -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported("General Tazri");
    let mut t = TestGame::new(2);
    let tazri = t.battlefield(P0, "General Tazri");
    let ids = allies.iter().map(|n| t.battlefield(P0, n)).collect();
    rainbow_lands(&mut t, P0, 1);
    (t, tazri, ids)
}

#[test]
fn general_tazri_counts_colors_among_your_allies_as_it_resolves() {
    cr!("105.2c", "608.2h", "611.2c");
    ruling!(
        "General Tazri",
        "The last ability checks only Ally creatures you control as the ability resolves. The value of X will be between 0 and 5. Colorless is not a color."
    );
    // White (Tazri) and colorless (Stonework Puma): X = 1. A blue Ally an opponent
    // controls doesn't count.
    let (mut t, tazri, ids) = tazri_with(&["Stonework Puma"]);
    t.battlefield(P1, "Coralhelm Guide");
    t.activate(P0, tazri, 0, &[]).unwrap();
    // A black Ally enters in response: it's counted (and gets the bonus).
    let late = t.battlefield(P0, "Hagra Sharpshooter");
    t.resolve_all();
    assert_eq!(t.pt(tazri), (5, 6));
    assert_eq!(t.pt(ids[0]), (4, 4));
    assert_eq!(t.pt(late), (4, 4));
}

#[test]
fn general_tazri_bonus_is_locked_in() {
    cr!("608.2h", "611.2c");
    ruling!(
        "General Tazri",
        "Once the last ability resolves, the bonus given doesn’t change, even if the number of colors among your Allies does."
    );
    let (mut t, tazri, ids) = tazri_with(&["Coralhelm Guide"]);
    t.activate(P0, tazri, 0, &[]).unwrap();
    t.resolve_all();
    // White and blue: +2/+2.
    assert_eq!(t.pt(tazri), (5, 6));
    let black = t.battlefield(P0, "Hagra Sharpshooter");
    crate::r_s02_common::destroy(&mut t, ids[0]);
    assert_eq!(t.pt(tazri), (5, 6));
    assert_eq!(t.pt(black), (2, 2));
}

#[test]
fn ichorplate_golem_gives_one_bonus_for_any_number_of_oil_counters() {
    cr!("122.1", "613.4c");
    ruling!(
        "Ichorplate Golem",
        "If a creature has at least one oil counter on it, it gets +1/+1 due to Ichorplate Golem's effect. It does not get any additional bonus for having more oil counters on it."
    );
    supported("Ichorplate Golem");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichorplate Golem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, bears, "oil", 1);
    add(&mut t, giant, "oil", 3);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn ichorplate_golem_trigger_checks_oil_counters_on_entry_and_resolution() {
    cr!("603.4", "614.1c");
    ruling!(
        "Ichorplate Golem",
        "The first ability will check to see if a creature has any oil counters on it at the moment it entered the battlefield. If it doesn't, the ability won't trigger at all."
    );
    supported("Glistener Seer");
    // Entering with oil counters: one more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichorplate Golem");
    let seer = t.enter(P0, "Glistener Seer");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(seer, "oil"), 4);
    // Entering without: no trigger, even if counters are put on it right after.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichorplate Golem");
    let bears = t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    add(&mut t, bears, "oil", 1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(bears, "oil"), 1);
    // The counters are gone when it resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichorplate Golem");
    let seer = t.enter(P0, "Glistener Seer");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let id = t.g.current(seer);
    t.g.objects[id.0 as usize].counters.remove("oil");
    t.resolve_all();
    assert_eq!(t.counters(seer, "oil"), 0);
}

#[test]
fn reality_smasher_triggers_on_any_opponents_spell_including_an_aura() {
    cr!("603.2", "115.1", "303.4a");
    ruling!(
        "Reality Smasher",
        "The last ability will trigger due to any spell controlled by an opponent (including an Aura spell) that targets Reality Smasher."
    );
    supported("Reality Smasher");
    // P1's Pacifism: countered unless P1 discards a card.
    for discard in [false, true] {
        let mut t = TestGame::new(2);
        let smasher = t.battlefield(P0, "Reality Smasher");
        t.hand(P1, "Grizzly Bears");
        t.g.turn.active = P1;
        cast_new(&mut t, P1, "Pacifism", &[smasher.into()]);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        t.answer_yes(P1, discard);
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), discard);
        assert_eq!(t.in_graveyard(P1, "Pacifism"), !discard);
        assert_eq!(t.named_on_battlefield("Pacifism").len(), usize::from(discard));
    }
    // P1's Lightning Bolt: also. P0's own Giant Growth: no trigger.
    let mut t = TestGame::new(2);
    let smasher = t.battlefield(P0, "Reality Smasher");
    t.answer_yes(P1, false);
    cast_new(&mut t, P1, "Lightning Bolt", &[smasher.into()]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    let mut t = TestGame::new(2);
    let smasher = t.battlefield(P0, "Reality Smasher");
    cast_new(&mut t, P0, "Giant Growth", &[smasher.into()]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(smasher), (8, 8));
}
