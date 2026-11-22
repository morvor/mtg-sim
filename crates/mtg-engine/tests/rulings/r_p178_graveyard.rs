//! Rulings batch P178 — River Kelpie (permanents entering from a graveyard vs. spells
//! cast from one), Tergrid (sacrifices as a cost, Auras put onto the battlefield),
//! Witchstalker (a cast copy), Treasure Nabber (improvise isn't tapping for mana).

use crate::r_p178_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts cards from `p`'s graveyard onto the battlefield at the same time.
fn return_together(t: &mut TestGame, p: PlayerId, cards: &[ObjectId]) -> Vec<ObjectId> {
    let moves = cards
        .iter()
        .map(|c| mtg_engine::replacement::MoveEv {
            obj: *c,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(p),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids: Vec<ObjectId> = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    t.settle();
    ids
}

#[test]
fn river_kelpie_sees_another_permanent_returning_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "River Kelpie",
        "If River Kelpie and another permanent are each put onto the battlefield from a graveyard at the same time, River Kelpie's first ability will trigger twice."
    );
    supported("River Kelpie");
    let mut t = TestGame::new(2);
    let kelpie = t.graveyard(P0, "River Kelpie");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let ids = return_together(&mut t, P0, &[kelpie, bears]);
    assert_eq!(triggers_from(&t, ids[0]), 2);
    let h = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 2);
}

#[test]
fn river_kelpie_spells_cast_from_a_graveyard() {
    cr!("601.2a", "603.2", "603.6a");
    ruling!(
        "River Kelpie",
        "If you cast River Kelpie itself from your graveyard (by using Yawgmoth's Will, for example), neither ability triggers."
    );
    ruling!(
        "River Kelpie",
        "If you cast another artifact, creature, enchantment, or planeswalker spell from a graveyard, only the second ability triggers."
    );
    ruling!(
        "River Kelpie",
        "If you play a land card from a graveyard (by using Crucible of Worlds, for example), only the first ability triggers."
    );
    // River Kelpie itself, cast from the graveyard.
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Yawgmoth's Will", &[]);
    let kelpie = t.graveyard(P0, "River Kelpie");
    give_mana_for(&mut t, P0, "River Kelpie");
    t.cast(P0, kelpie).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger for the cast");
    t.resolve();
    assert_eq!(t.stack_len(), 0, "no trigger for it entering");
    assert!(t.on_battlefield(kelpie));

    // Another creature spell cast from the graveyard: only the cast trigger.
    let mut t = TestGame::new(2);
    let kelpie = t.battlefield(P0, "River Kelpie");
    cast_resolve(&mut t, P0, "Yawgmoth's Will", &[]);
    let bears = t.graveyard(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(triggers_from(&t, kelpie), 1);
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(triggers_from(&t, kelpie), 0, "entering from the stack");

    // A land played from the graveyard: only the enters trigger.
    let mut t = TestGame::new(2);
    let kelpie = t.battlefield(P0, "River Kelpie");
    t.battlefield(P0, "Crucible of Worlds");
    let forest = t.graveyard(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, kelpie), 1);
}

#[test]
fn river_kelpie_gives_no_permission_to_cast_from_graveyards() {
    cr!("601.2a");
    ruling!(
        "River Kelpie",
        "River Kelpie doesn't give you the ability to cast spells from graveyards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "River Kelpie");
    let bears = t.graveyard(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Grizzly Bears");
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        bears,
        mtg_engine::object::CastMethod::Normal
    ));
}

#[test]
fn tergrid_trigger_from_a_sacrifice_cost_resolves_first() {
    cr!("603.3", "602.2b");
    ruling!(
        "Tergrid, God of Fright // Tergrid's Lantern",
        "If an opponent sacrifices a nontoken permanent or discards a permanent card as part of paying the cost of a spell or ability, Tergrid's ability will trigger and goes on the stack on top of that spell or ability. Tergrid's ability will resolve before that spell or ability."
    );
    supported("Tergrid, God of Fright // Tergrid's Lantern");
    let mut t = TestGame::new(2);
    let tergrid = t.battlefield(P0, "Tergrid, God of Fright // Tergrid's Lantern");
    let elder = t.battlefield(P1, "Sakura-Tribe Elder");
    t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    activate_containing(&mut t, P1, elder, "Sacrifice").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_from(&t, tergrid), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(elder), Zone::Battlefield);
    assert_eq!(controller(&mut t, elder), P0);
    assert_eq!(t.stack_len(), 1, "the Elder's ability hasn't resolved");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn tergrid_aura_put_onto_the_battlefield_isnt_targeted() {
    cr!("303.4f", "303.4a");
    ruling!(
        "Tergrid, God of Fright // Tergrid's Lantern",
        "If you put an Aura onto the battlefield using Tergrid's ability, you choose what the Aura will enchant just before it enters the battlefield."
    );
    // Betrayal: "Enchant creature an opponent controls". P1's hexproof creature may be
    // chosen; one with protection from enchantments may not.
    for (creature, attaches) in [("Gladecover Scout", true), ("Azorius First-Wing", false)] {
        supported(creature);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Tergrid, God of Fright // Tergrid's Lantern");
        let c = t.battlefield(P1, creature);
        let betrayal = t.hand(P1, "Betrayal");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(c)]);
        t.g.discard(P1, betrayal, None);
        t.g.flush_events();
        t.resolve_all();
        if attaches {
            assert_eq!(t.zone(betrayal), Zone::Battlefield, "{creature}");
            assert_eq!(t.obj_now(betrayal).attached_to, Some(obj(c)));
            assert_eq!(controller(&mut t, betrayal), P0);
        } else {
            assert_eq!(t.zone(betrayal), Zone::Graveyard(P1), "{creature}: stays");
        }
    }
}

#[test]
fn witchstalker_triggers_on_a_cast_copy() {
    cr!("707.12", "603.2");
    ruling!(
        "Witchstalker",
        "If an opponent casts a copy of a card that’s blue or black during your turn (as opposed to putting a copy of such a spell directly on the stack), the ability will also trigger."
    );
    supported("Witchstalker");
    supported("Isochron Scepter");
    let mut t = TestGame::new(2);
    let ws = t.battlefield(P0, "Witchstalker");
    let opt = t.hand(P1, "Opt");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(opt)]);
    let scepter = t.enter(P1, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(opt), Zone::Exile);
    t.lands(P1, "Wastes", 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    activate_containing(&mut t, P1, scepter, "{2}").unwrap();
    t.resolve();
    // The copy was cast during P0's turn.
    assert_eq!(triggers_from(&t, ws), 1);
    t.resolve_all();
    assert_eq!(t.counters(ws, "+1/+1"), 1);
}

#[test]
fn treasure_nabber_improvise_isnt_tapping_for_mana() {
    cr!("702.126a", "106.12");
    ruling!(
        "Treasure Nabber",
        "Notably, tapping an artifact to pay for a spell with the improvise keyword is not tapping that artifact for mana."
    );
    supported("Reverse Engineer");
    let mut t = TestGame::new(2);
    let nabber = t.battlefield(P0, "Treasure Nabber");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 2);
    let ring = t.battlefield(P1, "Sol Ring");
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Ornithopter");
    t.answer_choose(P1, &[obj(ring), obj(a), obj(b)]);
    let re = t.hand(P1, "Reverse Engineer");
    t.cast(P1, re).go();
    t.settle();
    assert!(tapped(&t, ring), "tapped for improvise");
    assert_eq!(triggers_from(&t, nabber), 0);
    t.resolve_all();
    assert_eq!(controller(&mut t, ring), P1);
}
