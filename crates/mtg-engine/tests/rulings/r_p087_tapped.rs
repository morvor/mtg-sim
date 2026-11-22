//! Rulings batch P087 — hate for tapped permanents ("hate-tapped"): Tamiyo, the Moon Sage,
//! "target tapped creature" spells whose target becomes illegal, until-leaves exile whose
//! source leaves first, and "whenever ... becomes tapped" triggers.

use crate::r_p087_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Tamiyo, the Moon Sage ----------------------------------------------------------------

fn tamiyo(t: &mut TestGame, loyalty: u32) -> ObjectId {
    supported("Tamiyo, the Moon Sage");
    let tam = t.battlefield(P0, "Tamiyo, the Moon Sage");
    set_loyalty(t, tam, loyalty);
    tam
}

fn emblems(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::Emblem && o.zone == Zone::Command && o.controller == p)
        .map(|o| o.id)
        .collect()
}

#[test]
fn tamiyo_plus_one_can_target_a_tapped_permanent_which_then_skips_its_next_untap() {
    cr!("701.26a", "502.3", "115.1");
    ruling!(
        "Tamiyo, the Moon Sage",
        "The ability can target a tapped permanent. If the targeted permanent is already tapped when it resolves, that permanent just remains tapped and doesn’t untap during its controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let tam = tamiyo(&mut t, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    activate(&mut t, P0, tam, "Tap target permanent", &[obj(bears)]);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    next_upkeep(&mut t, P1);
    assert!(is_tapped(&t, bears));
    next_upkeep(&mut t, P1);
    assert!(!is_tapped(&t, bears));
}

#[test]
fn tamiyo_plus_one_tracks_the_permanent_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Tamiyo, the Moon Sage",
        "Tamiyo’s first ability tracks the permanent, but not its controller. If the permanent changes controllers before its first controller’s next untap step has come around, then it won’t untap during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let tam = tamiyo(&mut t, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate(&mut t, P0, tam, "Tap target permanent", &[obj(bears)]);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    // P0 gains control of it before P1's next untap step.
    give_control(&mut t, bears, P0);
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1's untap step: not P1's permanent any more.
    next_upkeep(&mut t, P1);
    assert!(is_tapped(&t, bears));
    // P0's next untap step is its new controller's next one: it stays tapped.
    next_upkeep(&mut t, P0);
    assert!(is_tapped(&t, bears));
    next_upkeep(&mut t, P0);
    assert!(!is_tapped(&t, bears));
}

#[test]
fn tamiyo_minus_two_counts_tapped_creatures_on_resolution() {
    cr!("608.2h", "606.4");
    ruling!(
        "Tamiyo, the Moon Sage",
        "The number of tapped creatures the player controls is determined when Tamiyo’s second ability resolves."
    );
    let mut t = TestGame::new(2);
    let tam = tamiyo(&mut t, 4);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, a);
    activate(
        &mut t,
        P0,
        tam,
        "Draw a card for each",
        &[Entity::Player(P1)],
    );
    // In response, a second creature becomes tapped.
    tap(&mut t, b);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0) - hand, 2);
}

#[test]
fn tamiyo_emblem_is_one_emblem_and_returns_cards_never_in_hand() {
    cr!("114.1", "114.2", "603.6");
    ruling!(
        "Tamiyo, the Moon Sage",
        "Tamiyo’s third ability creates one emblem with two abilities."
    );
    ruling!(
        "Tamiyo, the Moon Sage",
        "Cards that are put into your graveyard can be returned to your hand by the emblem’s ability even if they were never in your hand."
    );
    let mut t = TestGame::new(2);
    let tam = tamiyo(&mut t, 9);
    activate(&mut t, P0, tam, "emblem", &[]);
    t.resolve_all();
    let e = emblems(&t, P0);
    assert_eq!(e.len(), 1);
    assert_eq!(t.g.obj(e[0]).chars.abilities.len(), 2);
    // A card milled from the library (never in hand) can be returned.
    let top = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.mill(P0, 1);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(top)), Zone::Hand(P0));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn tamiyo_using_minus_eight_from_eight_loyalty_dies_before_the_emblem_exists() {
    cr!("704.5i", "606.4", "114.2");
    ruling!(
        "Tamiyo, the Moon Sage",
        "If you activate Tamiyo’s third ability when she has eight loyalty counters on her, she’ll be put into your graveyard before the emblem is created. She won’t return to your hand."
    );
    let mut t = TestGame::new(2);
    let tam = tamiyo(&mut t, 8);
    t.answer_yes(P0, true);
    activate(&mut t, P0, tam, "emblem", &[]);
    // State-based actions put her into the graveyard while the ability is on the stack.
    assert!(t.in_graveyard(P0, "Tamiyo, the Moon Sage"));
    assert!(emblems(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(emblems(&t, P0).len(), 1);
    assert!(t.in_graveyard(P0, "Tamiyo, the Moon Sage"));
    assert!(!t.in_hand(P0, "Tamiyo, the Moon Sage"));
}

// --- "Target tapped creature" ---------------------------------------------------------------

#[test]
fn glimmerdust_nap_targets_only_a_tapped_creature_and_fizzles_if_it_untaps() {
    cr!("303.4a", "608.3b");
    ruling!(
        "Glimmerdust Nap",
        "Glimmerdust Nap can target only a tapped creature."
    );
    ruling!(
        "Glimmerdust Nap",
        "If the targeted creature has become untapped by the time Glimmerdust Nap would resolve, Glimmerdust Nap doesn’t resolve for having an illegal target."
    );
    supported("Glimmerdust Nap");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    let untapped = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, tapped);
    let from = t.asked().len();
    let nap = cast_new(&mut t, P0, "Glimmerdust Nap", &[obj(tapped)]);
    // Only the tapped creature was offered.
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![obj(tapped)]]);
    assert!(!offered[0].contains(&obj(untapped)));
    untap(&mut t, tapped);
    t.resolve_all();
    assert!(!t.on_battlefield(t.g.current(nap)));
    assert!(t.in_graveyard(P0, "Glimmerdust Nap"));
    assert_eq!(attached_to(&t, tapped), None);
}

#[test]
fn tapped_creature_spells_with_an_illegal_target_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Deadly Riposte",
        "If the target creature is an illegal target as Deadly Riposte tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    ruling!(
        "Eriette's Lullaby",
        "If the target creature is an illegal target as Eriette's Lullaby tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    ruling!(
        "Radiant Strike",
        "If the target permanent is an illegal target as Radiant Strike tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain life."
    );
    for (name, gain) in [
        ("Deadly Riposte", 2),
        ("Eriette's Lullaby", 2),
        ("Radiant Strike", 3),
    ] {
        supported(name);
        for illegal in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            tap(&mut t, bears);
            cast_new(&mut t, P0, name, &[obj(bears)]);
            if illegal {
                // It untaps in response: no longer a tapped creature.
                untap(&mut t, bears);
            }
            t.resolve_all();
            assert_eq!(t.on_battlefield(bears), illegal, "{name}");
            assert_eq!(t.life(P0), if illegal { 20 } else { 20 + gain }, "{name}");
            assert!(t.in_graveyard(P0, name), "{name}");
        }
    }
}

#[test]
fn dovins_dismissal_with_an_illegal_target_doesnt_search() {
    cr!("608.2b");
    ruling!(
        "Dovin's Dismissal",
        "If you choose a target tapped creature and it becomes an illegal target for Dovin’s Dismissal, the spell doesn’t resolve. You don’t search for Dovin."
    );
    supported("Dovin's Dismissal");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let dovin = t.library_top(P0, "Dovin, Architect of Law");
        let bears = t.battlefield(P1, "Grizzly Bears");
        tap(&mut t, bears);
        cast_new(&mut t, P0, "Dovin's Dismissal", &[obj(bears)]);
        if illegal {
            untap(&mut t, bears);
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(dovin)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), illegal);
        assert_eq!(t.in_hand(P0, "Dovin, Architect of Law"), !illegal);
    }
}

#[test]
fn until_leaves_exile_of_a_tapped_creature_does_nothing_if_the_source_left() {
    cr!("610.3b", "603.2");
    ruling!(
        "Henchbots",
        "If Henchbots leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Seal Away",
        "If Seal Away leaves the battlefield before its triggered ability resolves, the target creature won’t be exiled."
    );
    for name in ["Henchbots", "Seal Away"] {
        for leaves in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            tap(&mut t, bears);
            t.answer_targets(P0, &[obj(bears)]);
            let src = t.enter(P0, name);
            t.g.flush_events();
            t.settle();
            assert_eq!(t.stack_len(), 1, "{name}");
            if leaves {
                destroy(&mut t, src);
            }
            t.resolve_all();
            assert_eq!(t.on_battlefield(bears), leaves, "{name}");
        }
    }
}

#[test]
fn banishing_slash_targets_and_its_token() {
    cr!("608.2b", "115.6", "205.2b");
    ruling!(
        "Banishing Slash",
        "If you control an artifact that is also an enchantment as Banishing Slash resolves, you will create a Samurai token."
    );
    ruling!(
        "Banishing Slash",
        "You may cast Banishing Slash without choosing a target. If you do, and you control both an artifact and an enchantment as it resolves, you'll just create a Samurai token. However, if you do choose a target and that target is an illegal target as Banishing Slash tries to resolve, Banishing Slash won't resolve and none of its effects will happen. The illegal target won't be destroyed and you won't create a Samurai token."
    );
    supported("Banishing Slash");
    let samurai = |t: &TestGame| crate::r_s05_common::tokens_with_subtype(t, P0, "Samurai").len();
    // An artifact that is also an enchantment (Hammer of Purphoros), no target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hammer of Purphoros");
    cast_new(&mut t, P0, "Banishing Slash", &[]);
    t.resolve_all();
    assert_eq!(samurai(&t), 1);
    // A target that became illegal: nothing happens, even with the artifact and enchantment.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hammer of Purphoros");
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    cast_new(&mut t, P0, "Banishing Slash", &[obj(bears)]);
    untap(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(samurai(&t), 0);
    // A legal target: destroyed and a token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hammer of Purphoros");
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    cast_new(&mut t, P0, "Banishing Slash", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(samurai(&t), 1);
}

// --- "Whenever ... becomes tapped" -------------------------------------------------------------

#[test]
fn gideons_avenger_doesnt_see_a_creature_entering_tapped() {
    cr!("603.2", "603.2e", "110.5b");
    ruling!(
        "Gideon's Avenger",
        "If an opponent's creature enters tapped, Gideon's Avenger's ability doesn't trigger."
    );
    supported("Gideon's Avenger");
    let mut t = TestGame::new(2);
    let av = t.battlefield(P0, "Gideon's Avenger");
    let ghoul = t.enter(P1, "Diregraf Ghoul");
    t.g.flush_events();
    t.settle();
    assert!(is_tapped(&t, ghoul));
    assert_eq!(t.stack_len(), 0);
    // Becoming tapped later does trigger.
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(av, counters::PLUS1), 1);
}

#[test]
fn insolence_triggers_on_becoming_tapped_however_but_not_on_entering_attached_to_a_tapped_one() {
    cr!("603.2", "603.2e");
    ruling!(
        "Insolence",
        "It does not trigger immediately when cast targeting an already tapped creature."
    );
    ruling!(
        "Insolence",
        "The damaging ability triggers no matter how the creature becomes tapped."
    );
    supported("Insolence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    cast_new(&mut t, P0, "Insolence", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(t.named_on_battlefield("Insolence")[0]));
    assert_eq!(t.life(P1), 20);
    // Tapped by an effect.
    untap(&mut t, bears);
    tap(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Tapped by Sigardian Priest's ability ("{1}, {T}: Tap target non-Human creature.").
    untap(&mut t, bears);
    let priest = t.battlefield(P0, "Sigardian Priest");
    t.lands(P0, "Plains", 1);
    activate(&mut t, P0, priest, "Tap target", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Tapped to attack.
    untap(&mut t, bears);
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn psychic_venom_triggers_whenever_the_land_is_tapped_for_any_reason() {
    cr!("603.2", "106.12");
    ruling!(
        "Psychic Venom",
        "Whenever the land is tapped for any reason, the ability triggers."
    );
    supported("Psychic Venom");
    let mut t = TestGame::new(2);
    let island = t.battlefield(P1, "Island");
    attach_new(&mut t, P0, "Psychic Venom", island);
    // Tapped for mana.
    assert!(tap_for_mana(&mut t, P1, island, "Add"));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Tapped by an effect.
    untap(&mut t, island);
    tap(&mut t, island);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn roots_of_life_counts_lands_of_every_opponent() {
    cr!("603.2", "806.1");
    ruling!(
        "Roots of Life",
        "It affects all opponents in a multiplayer game."
    );
    supported("Roots of Life");
    let mut t = TestGame::new(3);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Roots of Life");
    // The first option is Island.
    let island_first = t.asked()[from..].iter().any(|(p, d)| {
        *p == P0
            && matches!(d, Decision::ChooseOption { options, .. }
                if options.first().is_some_and(|o| o.contains("Island")))
    });
    assert!(island_first, "{:?}", &t.asked()[from..]);
    let i1 = t.battlefield(P1, "Island");
    let i2 = t.battlefield(P2, "Island");
    let mine = t.battlefield(P0, "Island");
    let swamp = t.battlefield(P1, "Swamp");
    for l in [i1, i2, mine, swamp] {
        tap(&mut t, l);
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 22);
}
