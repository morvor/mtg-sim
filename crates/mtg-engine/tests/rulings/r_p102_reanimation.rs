//! Rulings batch P102 — mass reanimation: Auras returned without being cast (CR 303.4f,
//! 303.4i), Open the Vaults / Triumphant Reckoning / Redress Fate / Archon of Falling
//! Stars, Finale of Eternity's targets and X, and Funeral Room leaving the battlefield
//! together with dying creatures (CR 603.10a).

use crate::r_p102_common::*;
use crate::r_p137_common::choosers_since;
use crate::r_s02_common::destroy;
use crate::r_s06_common::attached_to;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts the real card `name` from hand with plenty of mana and resolves everything.
fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str) {
    supported(name);
    lots(t, p);
    lots(t, p);
    let c = t.hand(p, name);
    t.cast(p, c).go();
    t.resolve_all();
}

#[test]
fn returned_auras_cant_enchant_what_enters_with_them() {
    cr!("303.4f", "303.4i");
    ruling!(
        "Open the Vaults",
        "An Aura put onto the battlefield this way can't enchant an artifact or enchantment that's also being put onto the battlefield with Open the Vaults."
    );
    ruling!(
        "Triumphant Reckoning",
        "An Aura returning to the battlefield this way can't be attached to anything else entering the battlefield at the same time."
    );
    ruling!(
        "Redress Fate",
        "If an Aura is being put onto the battlefield at the same time as a creature, that Aura can't enchant that creature."
    );
    // Holy Strength (enchant creature) and Ornithopter (an artifact creature) return
    // together: with Grizzly Bears on the battlefield it enchants the Bears; without one,
    // it stays in the graveyard.
    for spell in ["Open the Vaults", "Triumphant Reckoning", "Redress Fate"] {
        for with_bears in [true, false] {
            let mut t = TestGame::new(2);
            let bears = with_bears.then(|| t.battlefield(P0, "Grizzly Bears"));
            let aura = t.graveyard(P0, "Holy Strength");
            let thopter = t.graveyard(P0, "Ornithopter");
            if let Some(b) = bears {
                t.answer_choose_moved(P0, &[o(b)]);
            }
            cast_resolve(&mut t, P0, spell);
            assert!(t.on_battlefield(thopter), "{spell}");
            match bears {
                Some(b) => {
                    assert_eq!(attached_to(&t, aura), Some(o(b)), "{spell}");
                    assert_eq!(t.pt(thopter), (0, 2), "{spell}");
                }
                None => assert_eq!(t.zone(aura), Zone::Graveyard(P0), "{spell}"),
            }
        }
    }
}

#[test]
fn returned_auras_dont_target_but_obey_enchant() {
    cr!("303.4f", "702.11b", "115.1");
    ruling!(
        "Triumphant Reckoning",
        "If you return an Aura card to the battlefield this way, the Aura isn't being cast, so you choose what it will enchant as is enters the battlefield."
    );
    ruling!(
        "Redress Fate",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it comes back onto the battlefield."
    );
    ruling!(
        "Archon of Falling Stars",
        "If an Aura is put onto the battlefield without being cast, the Aura's controller-to-be chooses what it will enchant as it comes back onto the battlefield."
    );
    // Pacifism onto P1's hexproof Slippery Bogle.
    for spell in ["Triumphant Reckoning", "Redress Fate"] {
        let mut t = TestGame::new(2);
        let bogle = t.battlefield(P1, "Slippery Bogle");
        let aura = t.graveyard(P0, "Pacifism");
        t.answer_choose_moved(P0, &[o(bogle)]);
        cast_resolve(&mut t, P0, spell);
        assert_eq!(attached_to(&t, aura), Some(o(bogle)), "{spell}");
        // Nothing it can enchant: it stays in the graveyard.
        let mut t = TestGame::new(2);
        let aura = t.graveyard(P0, "Pacifism");
        cast_resolve(&mut t, P0, spell);
        assert_eq!(t.zone(aura), Zone::Graveyard(P0), "{spell}");
    }
    // Archon of Falling Stars: "When this creature dies, you may return target
    // enchantment card from your graveyard to the battlefield."
    supported("Archon of Falling Stars");
    let mut t = TestGame::new(2);
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let archon = t.battlefield(P0, "Archon of Falling Stars");
    let aura = t.graveyard(P0, "Pacifism");
    t.answer_targets(P0, &[o(aura)]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(bogle)]);
    destroy(&mut t, archon);
    t.resolve_all();
    assert_eq!(attached_to(&t, aura), Some(o(bogle)));
    // Nothing to enchant: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Archon of Falling Stars");
    let aura = t.graveyard(P0, "Pacifism");
    t.answer_targets(P0, &[o(aura)]);
    t.answer_yes(P0, true);
    destroy(&mut t, archon);
    t.resolve_all();
    assert_eq!(t.zone(aura), Zone::Graveyard(P0));
}

#[test]
fn open_the_vaults_auras_must_return_if_they_can() {
    cr!("303.4f", "608.2c");
    ruling!(
        "Open the Vaults",
        "If an Aura can be returned to the battlefield with Open the Vaults, it must be, even if its owner doesn't like what it would enchant."
    );
    // P1's Pacifism returns, and the only creature is P1's own Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.graveyard(P1, "Pacifism");
    cast_resolve(&mut t, P0, "Open the Vaults");
    assert_eq!(attached_to(&t, aura), Some(o(bears)));
    assert_eq!(t.obj_now(aura).controller, P1);
}

#[test]
fn open_the_vaults_auras_are_announced_in_apnap_order() {
    cr!("101.4", "303.4f", "608.2c");
    ruling!(
        "Open the Vaults",
        "If multiple players have Auras in their graveyards, the player whose turn it is announces what their Auras will enchant, then each other player in turn order does the same"
    );
    // It's P1's turn (P1 casts it); P0 and P1 each have an Aura and a creature to choose.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let b0 = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let a0 = t.graveyard(P0, "Holy Strength");
    let a1 = t.graveyard(P1, "Holy Strength");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.answer_choose_moved(P0, &[o(b1)]);
    t.answer_choose_moved(P1, &[o(b0)]);
    let from = t.asked().len();
    cast_resolve(&mut t, P1, "Open the Vaults");
    assert_eq!(choosers_since(&t, from), vec![P1, P0]);
    assert_eq!(attached_to(&t, a0), Some(o(b1)));
    assert_eq!(attached_to(&t, a1), Some(o(b0)));
    // All entered at the same time: neither Aura could go on the Ornithopter.
    assert!(t.on_battlefield(thopter));
}

/// P0 casts Finale of Eternity with X = `x` and the given targets (none for an empty
/// list).
fn finale(t: &mut TestGame, x: i64, targets: &[ObjectId]) -> ObjectId {
    supported("Finale of Eternity");
    lots(t, P0);
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, 8);
    let f = t.hand(P0, "Finale of Eternity");
    let es: Vec<Entity> = targets.iter().map(|id| o(*id)).collect();
    t.cast(P0, f).x(x).targets(&es).go()
}

#[test]
fn finale_of_eternity_checks_each_targets_toughness() {
    cr!("601.2c", "115.1");
    ruling!(
        "Finale of Eternity",
        "Finale of Eternity can target up to three creatures that each have toughness X or less."
    );
    // X = 3: three 3-toughness creatures (total 9) can be targeted; a 4-toughness one
    // can't.
    let mut t = TestGame::new(2);
    let g: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Hill Giant")).collect();
    let wall = t.battlefield(P1, "Steel Wall");
    let from = t.asked().len();
    finale(&mut t, 3, &g);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered[0].contains(&o(wall)));
    t.resolve_all();
    assert!(g.iter().all(|id| in_gy(&t, *id)));
    assert!(t.on_battlefield(wall));
}

#[test]
fn finale_of_eternity_returns_with_some_or_no_targets_destroyed() {
    cr!("608.2b", "107.3a");
    ruling!(
        "Finale of Eternity",
        "If some but not all of the target creatures you choose are illegal targets as Finale of Eternity resolves, the legal targets are still destroyed and, if X is 10 or more, you still return cards to the battlefield."
    );
    ruling!(
        "Finale of Eternity",
        "If some or all of the targets are legal targets but aren’t destroyed, most likely because they have indestructible, you’ll still return cards to the battlefield if X is 10 or more."
    );
    ruling!(
        "Finale of Eternity",
        "If you target a creature you own and X is 10 or more, that card will be among those returned to the battlefield."
    );
    // Some targets illegal: P1's Bears leaves; P1's Giant is destroyed; P0's own Hill
    // Giant (targeted) dies and comes back with the Bears in P0's graveyard.
    let mut t = TestGame::new(2);
    let gone = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    let old = t.graveyard(P0, "Grizzly Bears");
    finale(&mut t, 10, &[gone, theirs, mine]);
    destroy(&mut t, gone);
    t.resolve_all();
    assert!(in_gy(&t, theirs));
    assert!(t.on_battlefield(mine), "returned");
    assert!(t.on_battlefield(old));
    // Legal but indestructible (Darksteel Myr): still returns.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let old = t.graveyard(P0, "Grizzly Bears");
    finale(&mut t, 10, &[myr]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert!(t.on_battlefield(old));
}

#[test]
fn finale_of_eternity_without_targets_or_with_all_targets_illegal() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Finale of Eternity",
        "You may cast Finale of Eternity without choosing any target creatures. If X is 10 or more, you’ll just return all creature cards from your graveyard to the battlefield."
    );
    let mut t = TestGame::new(2);
    let old = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    finale(&mut t, 10, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(old));
    // All chosen targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let old = t.graveyard(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    finale(&mut t, 10, &[bears]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(in_gy(&t, old));
    assert!(t.in_graveyard(P0, "Finale of Eternity"));
}

#[test]
fn finale_of_eternity_triggers_wait_and_go_on_in_apnap_order() {
    cr!("603.3b", "101.4", "608.2");
    ruling!(
        "Finale of Eternity",
        "Any abilities that trigger on the target creatures dying or on the creatures returning to the battlefield will all be put onto the stack after Finale of Eternity has finished resolving."
    );
    // P1's Doomed Traveler dies (P1's trigger); P0's Elvish Visionary returns (P0's
    // trigger). On P0's turn, P0's goes on the stack first, so P1's resolves first.
    let mut t = TestGame::new(2);
    let dt = t.battlefield(P1, "Doomed Traveler");
    let ev = t.graveyard(P0, "Elvish Visionary");
    finale(&mut t, 10, &[dt]);
    t.resolve();
    assert!(t.on_battlefield(ev));
    assert_eq!(t.stack_len(), 2);
    let top = t.g.stack[t.g.stack.len() - 1];
    assert_eq!(t.obj(top).controller, P1);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P1, "Spirit").len(), 1);
    assert_eq!(t.hand_size(P0), hand);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn funeral_room_leaving_with_dying_creatures_still_triggers() {
    cr!("603.10a", "709.5");
    ruling!(
        "Funeral Room // Awakening Hall",
        "If Funeral Room is unlocked and this Room leaves the battlefield at the same time one or more creatures you control die, its ability will trigger once for each of those creatures."
    );
    supported("Funeral Room // Awakening Hall");
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let room = t.hand(P0, "Funeral Room // Awakening Hall");
    t.cast(P0, room)
        .method(mtg_engine::object::CastMethod::Half(0))
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(room));
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    // A board wipe that also destroys enchantments (Nevinyrral's Disk).
    let disk = t.battlefield(P0, "Nevinyrral's Disk");
    t.activate(P0, disk, 0, &[]).unwrap();
    t.resolve_all();
    assert!(in_gy(&t, room));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}
