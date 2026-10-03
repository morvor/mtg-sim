//! Rulings batch P107 — planeswalker abilities with "up to" targets, choices made as the
//! ability resolves, permanents put onto the battlefield at the same time, and casting
//! cards exiled by an ability (CR 115.1, 601.2c, 603.6a, 606, 601.3).

use crate::r_p107_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn kiora_plus_one_needs_no_targets() {
    cr!("115.1", "606.4");
    ruling!(
        "Kiora, Master of the Depths",
        "You can activate Kiora’s first ability with no targets just to put a loyalty counter on her."
    );
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, Master of the Depths");
    let loyalty = t.counters(kiora, counters::LOYALTY);
    act(&mut t, P0, kiora, "Untap up to one", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(kiora, counters::LOYALTY), loyalty + 1);
}

#[test]
fn kiora_minus_two_may_take_nothing_one_or_both() {
    cr!("608.2c");
    ruling!(
        "Kiora, Master of the Depths",
        "With Kiora’s second ability, you could put no cards, a creature card, a land card, or a creature card and a land card into your hand."
    );
    for pick in [&[][..], &[0][..], &[1][..], &[0, 1][..]] {
        let mut t = TestGame::new(2);
        let kiora = t.battlefield(P0, "Kiora, Master of the Depths");
        let lib = stack_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Forest", "Hill Giant", "Island"],
        );
        let chosen: Vec<ObjectId> = pick.iter().map(|i| lib[*i]).collect();
        t.answer_choose(P0, &chosen.iter().map(|c| obj(*c)).collect::<Vec<_>>());
        act(&mut t, P0, kiora, "Reveal the top four", &[]).unwrap();
        t.resolve_all();
        let mut hand = hand_names(&t, P0);
        hand.sort();
        let mut want: Vec<String> = chosen
            .iter()
            .map(|c| t.obj(*c).chars.name.to_string())
            .collect();
        want.sort();
        assert_eq!(hand, want, "{pick:?}");
        assert_eq!(t.graveyard_size(P0), 4 - want.len());
    }
    // Not two creature cards.
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, Master of the Depths");
    let lib = stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Forest", "Hill Giant", "Island"],
    );
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[2])]);
    act(&mut t, P0, kiora, "Reveal the top four", &[]).unwrap();
    t.resolve_all();
    let creatures_in_hand = t
        .g
        .player(P0)
        .hand
        .iter()
        .filter(|c| t.obj(**c).is(CardType::Creature))
        .count();
    assert!(creatures_in_hand <= 1);
}

#[test]
fn nissa_vital_force_can_target_an_untapped_land() {
    cr!("115.1");
    ruling!(
        "Nissa, Vital Force",
        "You can activate Nissa's first ability targeting a land that's already untapped."
    );
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Vital Force");
    let forest = t.battlefield(P0, "Forest");
    act(&mut t, P0, nissa, "Untap target land", &[obj(forest)]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, forest));
    assert_eq!(t.pt(forest), (5, 5));
    assert!(!t.obj_now(forest).tapped);
}

#[test]
fn nissa_genesis_mage_plus_two_with_fewer_targets() {
    cr!("115.1", "601.2c");
    ruling!(
        "Nissa, Genesis Mage",
        "You can activate Nissa’s first ability with fewer than four targets."
    );
    // One creature and two lands; then (another game) no targets at all.
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Genesis Mage");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let l1 = t.battlefield(P0, "Forest");
    let l2 = t.battlefield(P0, "Island");
    for id in [bear, l1, l2] {
        t.g.tap(id);
    }
    t.answer_targets(P0, &[obj(bear)]);
    t.answer_targets(P0, &[obj(l1), obj(l2)]);
    activate_containing(&mut t, P0, nissa, "Untap up to two").unwrap();
    t.resolve_all();
    for id in [bear, l1, l2] {
        assert!(!t.obj_now(id).tapped);
    }
    assert_eq!(t.counters(nissa, counters::LOYALTY), 7);
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Genesis Mage");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    activate_containing(&mut t, P0, nissa, "Untap up to two").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(nissa, counters::LOYALTY), 7);
}

#[test]
fn nissa_genesis_mage_ultimate_puts_everything_onto_the_battlefield_at_once() {
    cr!("603.6a", "603.2");
    ruling!(
        "Nissa, Genesis Mage",
        "While resolving Nissa’s last ability, all creatures and lands put onto the battlefield this way enter at the same time."
    );
    // Two Soul Wardens ("Whenever another creature enters, you gain 1 life") see each
    // other enter.
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Genesis Mage");
    put_counters(&mut t, nissa, counters::LOYALTY, 5);
    let lib = stack_library(&mut t, P0, &["Soul Warden", "Soul Warden", "Forest"]);
    t.answer_choose(P0, &lib.iter().map(|i| obj(*i)).collect::<Vec<_>>());
    activate_containing(&mut t, P0, nissa, "top ten").unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 2);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn chandra_dressed_to_kill_needs_the_mana_and_the_timing() {
    cr!("601.2f", "601.3", "307.1");
    ruling!(
        "Chandra, Dressed to Kill",
        "You must pay all costs for spells cast via Chandra’s last two abilities. For the middle ability, you must also follow all timing restrictions."
    );
    // The middle ability exiles Lava Spike (a red sorcery).
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Dressed to Kill");
    t.library_top(P0, "Lava Spike");
    act(&mut t, P0, chandra, "Exile the top card", &[]).unwrap();
    t.resolve_all();
    let spike = t.g.find_in_zone(Zone::Exile, "Lava Spike")[0];
    // No mana: it can't be cast.
    assert!(!can_cast(&mut t, P0, spike, CastMethod::Normal));
    // With mana, but during combat (not a sorcery's timing): it can't.
    mana(&mut t, P0, ManaType::R, 1);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, spike, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, spike, CastMethod::Normal));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, spike).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn grave_birthing_the_opponent_chooses_and_an_empty_graveyard_still_gives_scion_and_card() {
    cr!("608.2d");
    ruling!(
        "Grave Birthing",
        "The target opponent chooses which card to exile as Grave Birthing resolves."
    );
    ruling!(
        "Grave Birthing",
        "You can cast Grave Birthing targeting an opponent with an empty graveyard."
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(b)]);
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let from = t.asked().len();
    cast_from_hand(&mut t, P0, "Grave Birthing", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.zone(a), Zone::Graveyard(P1));
    assert!(t.in_exile("Hill Giant"));
    // An empty graveyard: still a Scion and a card.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let hand = t.hand_size(P0);
    let spell = cast_from_hand(&mut t, P0, "Grave Birthing", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(with_subtype(&t, P0, "Scion").len(), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}
