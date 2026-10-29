//! Rulings batch P225 — transforming permanents and zone changes: a permanent that
//! transforms stays the same object (CR 712.18), so "until this leaves the battlefield"
//! effects keep going (CR 610.3); a permanent that left the battlefield is a new object
//! that abilities can't find (CR 400.7), so they don't exile, return or transform it; and
//! a transformed Aura that isn't attached goes to the graveyard (CR 704.5m) without dying.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::*;
use mtg_engine::events::Event;
use mtg_engine::object::{FaceState, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const CATHAR: &str = "Brutal Cathar // Moonrage Brute";
const MISSIONARIES: &str = "Avacynian Missionaries // Lunarch Inquisitors";
const BOLAS: &str = "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen";
const SEIZER: &str = "Soul Seizer // Ghastly Haunting";
const CECIL: &str = "Cecil, Dark Knight // Cecil, Redeemed Paladin";
const SEPHIROTH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";
const WITCH: &str = "Accursed Witch // Infectious Curse";
const STRANGLER: &str = "Vengeful Strangler // Strangling Grasp";
const RAL: &str = "Ral, Monsoon Mage // Ral, Leyline Prodigy";

/// It becomes night (daybound permanents transform), then state-based actions and
/// triggers are settled.
fn make_night(t: &mut TestGame) {
    t.g.day = Some(false);
    t.g.recompute();
    t.settle();
}

#[test]
fn brutal_cathars_exile_survives_transforming_but_not_leaving_first() {
    cr!("712.18", "610.3c");
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "If it transforms with its triggered ability on the stack, that ability will still exile the target creature."
    );
    ruling!(
        "Brutal Cathar // Moonrage Brute",
        "If Brutal Cathar leaves the battlefield before its enters-the-battlefield/transforms-into ability resolves, the target creature won't be exiled."
    );
    supported(CATHAR);
    // It transforms after exiling: the card stays exiled until the Brute leaves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cathar = t.enter(P0, CATHAR);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    make_night(&mut t);
    assert_eq!(name_of(&t, cathar), "Moonrage Brute");
    assert!(t.in_exile("Grizzly Bears"));
    // It transforms with the ability on the stack: the ability still exiles.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cathar = t.enter(P0, CATHAR);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    make_night(&mut t);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, cathar);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // It leaves with the ability on the stack: nothing is exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cathar = t.enter(P0, CATHAR);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.settle();
    destroy(&mut t, cathar);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn lunarch_inquisitors_exile_lasts_through_transforming_back() {
    cr!("712.18", "610.3c");
    ruling!(
        "Avacynian Missionaries // Lunarch Inquisitors",
        "If Lunarch Inquisitors somehow transforms into Avacynian Missionaries, the creature will remain exiled."
    );
    supported(MISSIONARIES);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, MISSIONARIES);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    transform(&mut t, m);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Lunarch Inquisitors");
    assert!(t.in_exile("Grizzly Bears"));
    transform(&mut t, m);
    assert_eq!(name_of(&t, m), "Avacynian Missionaries");
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, m);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn nicol_bolas_that_left_isnt_exiled_or_returned() {
    cr!("400.7", "608.2h");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "If Nicol Bolas leaves the battlefield after his creature face's activated ability has been activated but before it resolves"
    );
    supported(BOLAS);
    let mut t = TestGame::new(2);
    let bolas = t.battlefield(P0, BOLAS);
    for land in ["Island", "Swamp", "Mountain"] {
        t.lands(P0, land, 1);
    }
    t.lands(P0, "Wastes", 4);
    t.set_step(P0, Step::PrecombatMain);
    activate_containing(&mut t, P0, bolas, "{4}{U}{B}{R}").unwrap();
    destroy(&mut t, bolas);
    t.resolve_all();
    assert_eq!(t.zone(bolas), Zone::Graveyard(P0));
    assert_eq!(face(&t, bolas), FaceState::Front);
    assert!(t.named_on_battlefield("Nicol Bolas, the Arisen").is_empty());
}

#[test]
fn soul_seizer_that_left_doesnt_transform_or_attach() {
    cr!("400.7", "701.27a");
    ruling!(
        "Soul Seizer // Ghastly Haunting",
        "If Soul Seizer leaves the battlefield in response to its triggered ability"
    );
    supported(SEIZER);
    let mut t = TestGame::new(2);
    let seizer = t.battlefield(P0, SEIZER);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(seizer, Entity::Player(P1))], &[]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, seizer);
    t.resolve_all();
    assert_eq!(t.zone(seizer), Zone::Graveyard(P0));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn cecil_dealt_lethal_damage_still_triggers_but_doesnt_untap_or_transform() {
    cr!("603.10a", "510.2", "400.7");
    ruling!(
        "Cecil, Dark Knight // Cecil, Redeemed Paladin",
        "You'll still lose life, but you won't untap or transform Cecil, Dark Knight"
    );
    supported(CECIL);
    let mut t = TestGame::new(2);
    t.g.players[0].life = 12;
    let cecil = t.battlefield(P0, CECIL);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(cecil, Entity::Player(P1))], &[(giant, cecil)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.zone(cecil), Zone::Graveyard(P0));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.zone(cecil), Zone::Graveyard(P0));
    assert_eq!(face(&t, cecil), FaceState::Front);
}

#[test]
fn sephiroth_dying_with_other_creatures_triggers_for_each_but_doesnt_transform() {
    cr!("603.10a", "400.7");
    ruling!(
        "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel",
        "its last ability will trigger for each of those other creatures. (It won't transform, though.)"
    );
    supported(SEPHIROTH);
    let mut t = TestGame::new(2);
    let seph = t.battlefield(P0, SEPHIROTH);
    let mut all = vec![seph];
    for _ in 0..4 {
        all.push(t.battlefield(P1, "Grizzly Bears"));
    }
    t.g.destroy_all(all, None, false);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.zone(seph), Zone::Graveyard(P0));
    assert_eq!(face(&t, seph), FaceState::Front);
    assert!(!t
        .g
        .command
        .iter()
        .any(|id| t.obj(*id).kind == ObjKind::Emblem));
}

#[test]
fn a_creature_transforming_into_an_aura_on_the_battlefield_is_put_into_the_graveyard() {
    cr!("704.5m", "700.4");
    ruling!(
        "Accursed Witch // Infectious Curse",
        "then Infectious Curse doesn't become attached to any player and is put into your graveyard"
    );
    ruling!(
        "Vengeful Strangler // Strangling Grasp",
        "then Strangling Grasp doesn't become attached to any object and is put into your graveyard"
    );
    // (Infectious Curse's cost reduction doesn't compile; it isn't involved.)
    supported(STRANGLER);
    for name in [WITCH, STRANGLER] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.battlefield(P0, "Blood Artist");
        transform(&mut t, c);
        assert_eq!(t.zone(c), Zone::Graveyard(P0), "{name}");
        // Neither its own "when this creature dies" nor Blood Artist triggers.
        assert_eq!(t.stack_len(), 0, "{name}");
    }
}

#[test]
fn rals_other_triggers_do_nothing_after_he_returned_transformed() {
    cr!("400.7", "705.2", "608.2b");
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "if the controller of one of those abilities wins the flip, nothing will happen"
    );
    // (Ral, Leyline Prodigy's extra loyalty doesn't compile; it isn't involved.) The coin
    // flips are random: find a seed where both flips are won.
    let wins = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .chain(t.g.events.iter())
            .filter(|e| matches!(e, Event::CoinFlipped { won: true, .. }))
            .count()
    };
    let mut checked = false;
    for seed in 0..64 {
        let mut t = TestGame::new(2);
        t.g.rng = ChaCha8Rng::seed_from_u64(seed);
        let ral = t.battlefield(P0, RAL);
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 2);
        for _ in 0..2 {
            let bolt = t.hand(P0, "Lightning Bolt");
            t.cast(P0, bolt).target(P1).go();
            t.settle();
        }
        assert_eq!(triggers_on_stack(&t, "flip a coin"), 2);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve();
        if wins(&t) != 1 {
            continue;
        }
        let pw = t.g.current(ral);
        assert_eq!(name_of(&t, pw), "Ral, Leyline Prodigy");
        t.resolve_all();
        if wins(&t) != 2 {
            continue;
        }
        // The second won flip neither exiled the planeswalker nor dealt damage.
        assert_eq!(t.g.current(pw), pw);
        assert!(t.on_battlefield(pw));
        assert_eq!(t.life(P0), 20);
        checked = true;
        break;
    }
    assert!(checked, "no seed won both flips");
}
