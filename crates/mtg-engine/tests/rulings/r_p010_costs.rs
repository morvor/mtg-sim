//! Rulings batch P010 — burn with additional and alternative costs: sacrifices paid while
//! casting or activating (CR 601.2h, 602.2b), trap conditions (alternative costs,
//! CR 118.9), cost reductions, and mana value with an additional cost (CR 202.3).

use crate::r_p009_common::*;
use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The alternative-cost method of the card `id` (the ability whose text has `needle`).
fn alt_method(t: &TestGame, id: ObjectId, needle: &str) -> CastMethod {
    let uid = t
        .obj(id)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such ability");
    CastMethod::Alternative(uid)
}

/// Casts `name` from `p`'s hand for its trap cost ({R}, paid with one Mountain).
fn cast_trap(t: &mut TestGame, p: PlayerId, name: &str, target: Entity) -> bool {
    t.lands(p, "Mountain", 1);
    let card = t.hand(p, name);
    let m = alt_method(t, card, "rather than pay");
    let r = t.cast(p, card).method(m).target(target).try_go();
    t.clear_answers();
    r.is_ok()
}

#[test]
fn inferno_trap_counts_damage_from_two_different_creatures() {
    cr!("118.9", "120.1");
    ruling!(
        "Inferno Trap",
        "Inferno Trap's alternative cost condition checks for any damage, not just combat damage."
    );
    ruling!(
        "Inferno Trap",
        "The alternative cost condition doesn't check who controlled the creatures that dealt damage to you. Damage dealt to you by your own creatures counts."
    );
    ruling!(
        "Inferno Trap",
        "The alternative cost condition is satisfied only if two different creatures dealt damage to you, not if one creature dealt damage to you twice. The damage may have been dealt to you simultaneously (as with combat damage), or at separate times."
    );
    supported("Inferno Trap");
    // Noncombat damage from P1's creature and from P0's own creature, at separate times.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let theirs = t.battlefield(P1, "Prodigal Sorcerer");
    let mine = t.battlefield(P0, "Prodigal Sorcerer");
    t.g.deal_damage(theirs, pl(P0), 1, false);
    t.settle();
    assert!(!cast_trap(&mut t, P0, "Inferno Trap", obj(wurm)));
    t.g.deal_damage(mine, pl(P0), 1, false);
    t.settle();
    assert!(cast_trap(&mut t, P0, "Inferno Trap", obj(wurm)));
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 0, "the Wurm died");
    assert!(!t.on_battlefield(wurm));
    // One creature twice isn't enough.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let theirs = t.battlefield(P1, "Prodigal Sorcerer");
    t.g.deal_damage(theirs, pl(P0), 1, false);
    t.g.deal_damage(theirs, pl(P0), 1, false);
    t.settle();
    assert!(!cast_trap(&mut t, P0, "Inferno Trap", obj(wurm)));
    // Combat damage from two attackers at once is.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(a, pl(P0)), (b, pl(P0))], &[]);
    assert_eq!(t.life(P0), 16);
    assert!(cast_trap(&mut t, P0, "Inferno Trap", obj(wurm)));
}

#[test]
fn runeflare_trap_may_target_any_player() {
    cr!("118.9");
    ruling!(
        "Runeflare Trap",
        "The player you target with Runeflare Trap doesn't have to be the opponent that drew three or more cards this turn."
    );
    supported("Runeflare Trap");
    let mut t = TestGame::new(3);
    t.g.draw_cards(P1, 3);
    for _ in 0..4 {
        t.hand(P2, "Grizzly Bears");
    }
    assert!(cast_trap(&mut t, P0, "Runeflare Trap", pl(P2)));
    t.resolve_all();
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sacrifices_are_paid_while_casting() {
    cr!("601.2h", "601.2i");
    ruling!(
        "Goblin Grenade",
        "Players can only respond once Goblin Grenade has been cast and all its costs have been paid. No one can try and destroy the Goblin to prevent you from casting Goblin Grenade."
    );
    ruling!(
        "Shrapnel Blast",
        "Players can only respond once this spell has been cast and all its costs have been paid. No one can try to destroy the artifact you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Kazuul's Fury // Kazuul's Cliffs",
        "Players can respond only once this spell has been cast and all its costs have been paid. No one can try to interfere with the creature you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Artillerize",
        "The artifact or creature is sacrificed as costs are paid. Players can’t respond to the paying of costs by trying to destroy that artifact or creature."
    );
    // (card, permanent to sacrifice, damage dealt)
    let cases = [
        ("Goblin Grenade", "Goblin Piker", 5),
        ("Shrapnel Blast", "Mind Stone", 5),
        ("Kazuul's Fury // Kazuul's Cliffs", "Hill Giant", 3),
        ("Artillerize", "Ornithopter", 5),
    ];
    for (card, fodder, amount) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, fodder);
        let asked = t.asked().len();
        let spell = cast_new(&mut t, P0, card, &[pl(P1)]);
        // Already sacrificed while the spell was cast; P1 wasn't given priority meanwhile.
        assert!(!t.on_battlefield(f), "{card}");
        assert!(t.in_graveyard(P0, fodder), "{card}");
        assert!(t.stack.contains(&spell), "{card}");
        assert!(!crate::r_s01_common::asked_since(&t, asked)
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. })));
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - amount, "{card}");
    }
}

#[test]
fn goblin_grenade_sacrifices_exactly_one_goblin_of_any_type() {
    cr!("601.2b", "205.3d");
    ruling!(
        "Goblin Grenade",
        "The Goblin you sacrifice to cast Goblin Grenade doesn't have to be a creature. For example, you could sacrifice Boggart Shenanigans (a kindred enchantment with the subtype Goblin)."
    );
    ruling!(
        "Goblin Grenade",
        "You can't sacrifice more than one Goblin to get a greater effect."
    );
    supported("Boggart Shenanigans");
    let mut t = TestGame::new(2);
    let bs = t.battlefield(P0, "Boggart Shenanigans");
    cast_new(&mut t, P0, "Goblin Grenade", &[pl(P1)]);
    assert!(!t.on_battlefield(bs));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Two Goblins: one is sacrificed, and it's still 5 damage.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Piker");
    let b = t.battlefield(P0, "Goblin Piker");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    cast_new(&mut t, P0, "Goblin Grenade", &[pl(P1)]);
    assert_eq!(
        [a, b].iter().filter(|g| t.on_battlefield(**g)).count(),
        1
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn siege_gang_commander_can_sacrifice_itself() {
    cr!("602.2b", "117.1");
    ruling!(
        "Siege-Gang Commander",
        "You can sacrifice any Goblin you control to activate Siege-Gang Commander's activated ability, including Siege-Gang Commander itself."
    );
    supported("Siege-Gang Commander");
    let mut t = TestGame::new(2);
    let sgc = t.battlefield(P0, "Siege-Gang Commander");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, sgc, 0, &[pl(P1)]).unwrap();
    assert!(!t.on_battlefield(sgc));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn final_flare_sacrificing_an_enchantment_creature_is_just_one_sacrifice() {
    cr!("601.2b");
    ruling!(
        "Final Flare",
        "Final Flare has no extra effect if you sacrifice an enchantment creature."
    );
    supported("Final Flare");
    let mut t = TestGame::new(2);
    let courser = t.battlefield(P0, "Nessian Courser");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Final Flare", &[obj(wurm)]);
    assert!(!t.on_battlefield(courser));
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 5);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn wizards_lightning_cost_is_locked_in_as_it_is_cast() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Wizard's Lightning",
        "Once you announce that you’re casting Wizard’s Lightning, no player may take other actions until the spell’s been paid for."
    );
    supported("Wizard's Lightning");
    let mut t = TestGame::new(2);
    let wizard = t.battlefield(P0, "Prodigal Sorcerer");
    t.lands(P0, "Mountain", 1);
    let wl = t.hand(P0, "Wizard's Lightning");
    let asked = t.asked().len();
    t.cast(P0, wl).target(pl(P1)).go();
    assert!(!crate::r_s01_common::asked_since(&t, asked)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. })));
    // Removing the Wizard now changes nothing.
    kill(&mut t, wizard);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn lightning_axe_mana_value_is_one() {
    cr!("202.3", "601.2f");
    ruling!(
        "Lightning Axe",
        "The mana value of Lightning Axe is 1, no matter which additional cost you paid."
    );
    supported("Lightning Axe");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 6);
    let axe = t.hand(P0, "Lightning Axe");
    // No card to discard: it's the {5}.
    let spell = t.cast(P0, axe).target(obj(wurm)).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 6);
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 5);
}

#[test]
fn barrel_down_sokenzan_returns_mountains_on_resolution() {
    cr!("608.2c", "702.69a");
    ruling!(
        "Barrel Down Sokenzan",
        "The lands are returned on resolution; this is not a cost of casting the spell."
    );
    supported("Barrel Down Sokenzan");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let extra = t.lands(P0, "Mountain", 2);
    cast_new(&mut t, P0, "Barrel Down Sokenzan", &[obj(wurm)]);
    // Nothing returned yet: all three Mountains are still there.
    let mountains = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Mountain"))
        .count();
    assert_eq!(mountains, 3);
    t.answer_choose(P0, &[obj(extra[0]), obj(extra[1])]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(dmg(&t, wurm), 4);
}

#[test]
fn night_is_the_games_day_night_designation() {
    cr!("726.2", "601.2f");
    ruling!(
        "Moonrager's Slash",
        "The phrase \"if it's night\" refers to the day or night designation the game can have."
    );
    ruling!(
        "Olivia's Midnight Ambush",
        "The phrase \"if it's night\" refers to the day or night designation the game can have."
    );
    supported("Moonrager's Slash");
    supported("Olivia's Midnight Ambush");
    // (designation, Moonrager's Slash castable with one Mountain, Ambush's -N/-N)
    for (day, cheap) in [(None, false), (Some(true), false), (Some(false), true)] {
        let mut t = TestGame::new(2);
        t.g.day = day;
        t.lands(P0, "Mountain", 1);
        let slash = t.hand(P0, "Moonrager's Slash");
        let ok = t.cast(P0, slash).target(pl(P1)).try_go().is_ok();
        t.clear_answers();
        assert_eq!(ok, cheap, "{day:?}");

        let mut t = TestGame::new(2);
        t.g.day = day;
        let maw = t.battlefield(P1, "Colossal Dreadmaw");
        cast_new(&mut t, P0, "Olivia's Midnight Ambush", &[obj(maw)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(maw), !cheap, "{day:?}");
        if !cheap {
            assert_eq!(t.pt(maw), (4, 4));
        }
    }
}
