//! Rulings batch P167 — Equipment synergies: granted equip and crew abilities (Astor),
//! costs locked in as a spell is cast (CR 601.2f–h), triggers on casting and attacking
//! (CR 603.2), attaching effects and what an Equipment can legally be attached to
//! (CR 301.5c, 701.3b), and counts taken as an ability resolves (CR 608.2h).

use crate::r_p194_common::mana;
use crate::r_p213_common::{equip, equip_nth};
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::*;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{attach_new, attached_to, give_control};
use crate::r_s08_common::legal_cast_methods;
use crate::r_s09_common::attacks_now;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn novice_knight_stays_in_combat_after_it_stops_being_equipped() {
    cr!("506.4", "702.3b");
    ruling!(
        "Novice Knight",
        "Once Novice Knight has attacked, it will remain in combat even if it’s no longer enchanted or equipped."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Novice Knight");
    let sword = attach_new(&mut t, P0, "Bonesplitter", knight);
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    assert_eq!(attacks_now(&t), vec![(knight, Entity::Player(P1))]);
    destroy(&mut t, sword);
    assert_eq!(attacks_now(&t), vec![(knight, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn astor_granted_equip_and_crew_resolve_without_astor() {
    cr!("602.2", "608.2", "702.6a", "702.122a");
    ruling!(
        "Astor, Bearer of Blades",
        "Once either the equip {1} or crew 1 ability is activated, causing Astor, Bearer of Blades to leave the battlefield won't stop the ability from resolving."
    );
    // Equip {1} (Vulshok Morningstar's own equip costs {2}).
    let mut t = TestGame::new(2);
    let astor = t.battlefield(P0, "Astor, Bearer of Blades");
    let star = t.battlefield(P0, "Vulshok Morningstar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 1);
    equip_nth(&mut t, P0, star, 1, bears).expect("equip {1}");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, astor);
    t.resolve_all();
    assert_eq!(attached_to(&t, star), Some(obj(bears)));
    // Crew 1 (Cultivator's Caravan has crew 3), crewed by a 1-power creature.
    let mut t = TestGame::new(2);
    let astor = t.battlefield(P0, "Astor, Bearer of Blades");
    let caravan = t.battlefield(P0, "Cultivator's Caravan");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[obj(elf)]);
    activate_named(&mut t, P0, caravan, "Crew", 1).expect("crew 1");
    t.clear_answers();
    assert!(t.obj_now(elf).tapped);
    destroy(&mut t, astor);
    t.resolve_all();
    assert!(is_creature(&t, caravan));
}

#[test]
fn astor_leaves_the_printed_equip_and_crew_abilities() {
    cr!("113.2c", "702.6a", "702.122a");
    ruling!(
        "Astor, Bearer of Blades",
        "You may still activate any other equip or crew abilities that permanent has if you wish."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Astor, Bearer of Blades");
    let star = t.battlefield(P0, "Vulshok Morningstar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 2);
    equip_nth(&mut t, P0, star, 0, bears).expect("the printed equip {2}");
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(attached_to(&t, star), Some(obj(bears)));
    // The printed crew 3, with a 3-power creature.
    let caravan = t.battlefield(P0, "Cultivator's Caravan");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    activate_named(&mut t, P0, caravan, "Crew", 0).expect("crew 3");
    t.clear_answers();
    t.resolve_all();
    assert!(is_creature(&t, caravan));
}

#[test]
fn sokka_counts_allies_as_the_equipment_spell_is_cast() {
    cr!("601.2f", "601.2h", "117.1");
    ruling!(
        "Sokka, Swordmaster",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't remove Sokka to change by how much Equipment spell costs are reduced."
    );
    ruling!(
        "Sokka, Swordmaster",
        "Sokka will count himself as an Ally for his second ability."
    );
    // Sokka alone: Vulshok Morningstar ({2}) costs {1}; with another Ally, {0}.
    for (allies, cost) in [(0, 1), (1, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sokka, Swordmaster");
        for _ in 0..allies {
            t.battlefield(P0, "Weapons Trainer");
        }
        let star = t.hand(P0, "Vulshok Morningstar");
        mana(&mut t, P0, ManaType::C, cost);
        let from = t.asked().len();
        t.cast(P0, star).go();
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{allies}");
        assert!(
            t.asked()[from..].iter().all(|(p, _)| *p == P0),
            "an opponent was asked something while it was cast"
        );
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Vulshok Morningstar").len(), 1);
    }
    // Not for less than that.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sokka, Swordmaster");
    let star = t.hand(P0, "Vulshok Morningstar");
    assert!(t.cast(P0, star).try_go().is_err());
}

#[test]
fn danitha_losing_control_after_casting_doesnt_affect_the_spell() {
    cr!("601.2", "608.3");
    ruling!(
        "Danitha, New Benalia's Light",
        "Once you begin to cast the spell, losing control of Danitha won't affect the spell."
    );
    let mut t = TestGame::new(2);
    let danitha = t.battlefield(P0, "Danitha, New Benalia's Light");
    let sword = t.graveyard(P0, "Bonesplitter");
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!legal_cast_methods(&mut t, P0, sword).is_empty());
    t.cast(P0, sword).go();
    give_control(&mut t, danitha, P1);
    assert_eq!(t.obj_now(danitha).controller, P1);
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter");
    assert_eq!(s.len(), 1);
    assert_eq!(t.g.obj(s[0]).controller, P0);
}

#[test]
fn vulshok_battlemaster_takes_every_equipment() {
    cr!("701.3a", "701.3c", "301.5d");
    ruling!(
        "Vulshok Battlemaster",
        "Other players' Equipment is moved onto the Battlemaster as well as your own. This doesn't change who controls the Equipment or who can activate its equip ability to move it onto another creature."
    );
    ruling!(
        "Vulshok Battlemaster",
        "The \"enters\" effect moves all Equipment onto the Battlemaster, regardless of whether that Equipment was attached to other creatures."
    );
    let mut t = TestGame::new(2);
    let mine_on = t.battlefield(P0, "Grizzly Bears");
    let theirs_on = t.battlefield(P1, "Hill Giant");
    let mine = attach_new(&mut t, P0, "Bonesplitter", mine_on);
    let theirs = attach_new(&mut t, P1, "Leonin Scimitar", theirs_on);
    let loose = t.battlefield(P1, "Vulshok Morningstar");
    let bm = enter(&mut t, P0, "Vulshok Battlemaster");
    t.resolve_all();
    for e in [mine, theirs, loose] {
        assert_eq!(attached_to(&t, e), Some(obj(bm)));
    }
    assert_eq!(t.obj_now(theirs).controller, P1);
    assert_eq!(t.obj_now(loose).controller, P1);
    // 2/2 + 2/0 + 1/1 + 2/2.
    assert_eq!(t.pt(bm), (7, 5));
    // P1, not P0, can equip the Scimitar to their creature again.
    mana(&mut t, P0, ManaType::C, 1);
    assert!(equip(&mut t, P0, theirs, mine_on).is_err());
    mana(&mut t, P1, ManaType::C, 1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    equip(&mut t, P1, theirs, theirs_on).expect("P1 equips its Scimitar");
    t.resolve_all();
    assert_eq!(attached_to(&t, theirs), Some(obj(theirs_on)));
}

#[test]
fn sram_draws_before_the_spell_resolves_even_if_it_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Sram, Senior Edificer",
        "Sram's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sram, Senior Edificer");
    let sword = t.hand(P0, "Bonesplitter");
    mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, sword).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(top_of_stack(&t), spell);
    // Counter the spell under the trigger.
    let hand = t.hand_size(P0);
    move_to(&mut t, spell, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Bonesplitter"));
}

#[test]
fn reyav_needs_the_attacker_equipped_only_as_it_attacks() {
    cr!("603.2", "508.1m");
    ruling!(
        "Reyav, Master Smith",
        "The creature needs to be enchanted or equipped at the moment it's declared as an attacker to cause Reyav's ability to trigger. If the creature is no longer enchanted or equipped (or no longer attacking) as the ability tries to resolve, the ability still resolves."
    );
    for unequip in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Reyav, Master Smith");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let plain = t.battlefield(P0, "Hill Giant");
        let sword = attach_new(&mut t, P0, "Bonesplitter", bears);
        attack_with(
            &mut t,
            &[(bears, Entity::Player(P1)), (plain, Entity::Player(P1))],
        );
        // Only the equipped attacker triggers it.
        assert_eq!(t.stack_len(), 1);
        if unequip {
            destroy(&mut t, sword);
        }
        t.resolve_all();
        assert!(t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
        assert!(!t.obj_now(plain).has_keyword(KeywordKind::DoubleStrike));
        block_and_finish(&mut t, P1, &[]);
        let bears_dmg = if unequip { 4 } else { 8 };
        assert_eq!(t.life(P1), 20 - bears_dmg - 3, "{unequip}");
    }
}

#[test]
fn armory_automaton_triggers_on_entering_and_on_attacking() {
    cr!("603.2", "603.6a", "508.1m");
    ruling!(
        "Armory Automaton",
        "The triggered ability triggers both when Armory Automaton enters the battlefield and whenever it attacks. You don't have to choose only one."
    );
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Bonesplitter");
    let scimitar = t.battlefield(P0, "Leonin Scimitar");
    t.answer_targets(P0, &[obj(sword)]);
    let aa = enter(&mut t, P0, "Armory Automaton");
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(aa)));
    t.g.objects[aa.0 as usize].summoning_sick = false;
    t.answer_targets(P0, &[obj(scimitar)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(aa, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(attached_to(&t, scimitar), Some(obj(aa)));
    assert_eq!(attached_to(&t, sword), Some(obj(aa)));
}

#[test]
fn brass_squire_attaches_at_instant_speed() {
    cr!("602.5d", "701.3a");
    ruling!(
        "Brass Squire",
        "Unlike the equip ability, Brass Squire's ability can be activated whenever you could cast an instant."
    );
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Brass Squire");
    let sword = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // On the opponent's turn, with a spell on the stack.
    t.set_step(P1, mtg_engine::turn::Step::Upkeep);
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.activate(P0, squire, 0, &[obj(sword), obj(bears)])
        .expect("instant speed");
    // The equip ability can't be.
    mana(&mut t, P0, ManaType::C, 1);
    assert!(equip(&mut t, P0, sword, bears).is_err());
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(bears)));
}

// ---------------------------------------------------------------------------------------
// Nahiri, Forged in Fury
// ---------------------------------------------------------------------------------------

/// Nahiri and an equipped Grizzly Bears that attacks with `top` on top of P0's library:
/// the game stops in the declare attackers step with the card exiled. Returns the card.
fn nahiri_exiles(t: &mut TestGame, top: &str) -> (ObjectId, ObjectId) {
    let nahiri = t.battlefield(P0, "Nahiri, Forged in Fury");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(t, P0, "Bonesplitter", bears);
    let card = t.library_top(P0, top);
    attack_with(t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let card = t.g.current(card);
    assert_eq!(t.zone(card), Zone::Exile);
    (nahiri, card)
}

#[test]
fn nahiri_x_is_zero_unless_you_pay_the_mana_cost() {
    cr!("107.3b", "118.9", "601.2b");
    ruling!(
        "Nahiri, Forged in Fury",
        "One reason you might want to pay its mana cost is if there's an {X} in its mana cost. If you cast such a spell without paying its mana cost, you must choose 0 as the value of X. (If you pay its mana cost, you may choose the value of X as normal.)"
    );
    for free in [true, false] {
        let mut t = TestGame::new(2);
        let (_, sigil) = nahiri_exiles(&mut t, "Sigil of Distinction");
        t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
        let sigil = t.g.current(sigil);
        if free {
            t.cast(P0, sigil)
                .method(mtg_engine::object::CastMethod::Free)
                .x(3)
                .go();
        } else {
            mana(&mut t, P0, ManaType::C, 3);
            t.cast(P0, sigil).x(3).go();
        }
        t.resolve_all();
        let s = t.named_on_battlefield("Sigil of Distinction")[0];
        assert_eq!(t.counters(s, "charge"), if free { 0 } else { 3 }, "{free}");
    }
}

#[test]
fn nahiri_cards_are_played_with_the_normal_timing_and_costs() {
    cr!("601.3", "305.2", "307.1", "118.9");
    ruling!(
        "Nahiri, Forged in Fury",
        "Playing the exiled card follows the normal rules for playing that card. Unless you're casting an Equipment spell this way, you must pay the spell's costs, and you must follow all applicable timing rules. For example, if the card is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
    );
    ruling!(
        "Nahiri, Forged in Fury",
        "Unless an effect allows you to play additional lands that turn, you can play a land card exiled with Nahiri only if you haven't played a land yet that turn."
    );
    use mtg_engine::object::CastMethod;
    use mtg_engine::turn::Step;
    // A creature card: not in combat, not for free, for its mana cost in the main phase.
    let mut t = TestGame::new(2);
    let (_, ogre) = nahiri_exiles(&mut t, "Gray Ogre");
    mana(&mut t, P0, ManaType::C, 2);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(legal_cast_methods(&mut t, P0, ogre).is_empty(), "in combat");
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::C, 2);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!legal_cast_methods(&mut t, P0, ogre).contains(&CastMethod::Free));
    t.cast(P0, ogre).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1);
    // An Equipment card: free, but as a sorcery.
    let mut t = TestGame::new(2);
    let (_, sword) = nahiri_exiles(&mut t, "Vulshok Morningstar");
    assert!(legal_cast_methods(&mut t, P0, sword).is_empty(), "in combat");
    t.advance_to(P0, Step::PostcombatMain);
    t.cast(P0, sword).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Vulshok Morningstar").len(), 1);
    // A land card: only with a land play left.
    for played in [false, true] {
        let mut t = TestGame::new(2);
        if played {
            let f = t.hand(P0, "Forest");
            t.play_land(P0, f).unwrap();
        }
        let (_, land) = nahiri_exiles(&mut t, "Mountain");
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.play_land(P0, land).is_ok(), !played, "{played}");
    }
}

#[test]
fn nahiri_card_stays_playable_after_nahiri_and_the_creature_leave() {
    cr!("611.2c", "400.7");
    ruling!(
        "Nahiri, Forged in Fury",
        "You may play the card exiled with Nahiri's ability even if Nahiri or the equipped creature leaves the battlefield or you lose control of them."
    );
    for lose_control in [false, true] {
        let mut t = TestGame::new(2);
        let (nahiri, ogre) = nahiri_exiles(&mut t, "Gray Ogre");
        let bears = t.named_on_battlefield("Grizzly Bears")[0];
        if lose_control {
            give_control(&mut t, nahiri, P1);
            give_control(&mut t, bears, P1);
        } else {
            destroy(&mut t, nahiri);
            destroy(&mut t, bears);
        }
        t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
        mana(&mut t, P0, ManaType::C, 2);
        mana(&mut t, P0, ManaType::R, 1);
        t.cast(P0, ogre).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1);
    }
}

// ---------------------------------------------------------------------------------------
// Equip cost reductions: Bureau Headmaster, Cloud, Fighter Class
// ---------------------------------------------------------------------------------------

/// Whether, after `setup` (which returns the source and the index among its abilities
/// with text `text`, and the target), `p` can activate that ability with exactly `n`
/// generic mana, spending all of it.
fn activates_for(
    setup: &dyn Fn(&mut TestGame) -> (ObjectId, &'static str, usize, ObjectId),
    n: u32,
) -> bool {
    let mut t = TestGame::new(2);
    let (source, text, nth, target) = setup(&mut t);
    mana(&mut t, P0, ManaType::C, n);
    t.answer_targets(P0, &[obj(target)]);
    let ok = activate_named(&mut t, P0, source, text, nth).is_ok();
    ok && t.g.player(P0).mana_pool.total() == 0
}

/// The generic cost of the activation: the least amount it can be activated for.
fn activation_cost(setup: &dyn Fn(&mut TestGame) -> (ObjectId, &'static str, usize, ObjectId)) -> u32 {
    (0..10)
        .find(|n| activates_for(setup, *n))
        .expect("never activatable")
}

#[test]
fn bureau_headmaster_and_cloud_reduce_equip_variants_but_not_reconfigure() {
    cr!("601.2f", "602.2b", "702.6a", "702.6c", "702.151b");
    ruling!(
        "Bureau Headmaster",
        "Some Equipment creature cards in other sets have reconfigure, a different ability that attaches them to a creature. Reconfigure is not an equip ability, and reconfigure costs are not reduced by Bureau Headmaster’s last ability. Equip variants or equip abilities with restrictions, like the “equip legendary creature” ability of Brotherhood Regalia, are still equip abilities, and their costs will be reduced by Bureau Headmaster’s last ability."
    );
    ruling!(
        "Cloud, Planet's Champion",
        "Some Equipment creature cards in other sets have reconfigure, a different ability that attaches them to a creature. Reconfigure is not an equip ability, and reconfigure costs are not reduced by Cloud's last ability. Equip variants or equip abilities with restrictions, like the \"equip legendary creature\" ability of Wrecking Ball Arm, are still equip abilities, and their costs will be reduced by Cloud's last ability."
    );
    // (reducer, equipment, ability text, nth, cost)
    let cases: [(&str, &str, &'static str, usize, u32); 6] = [
        ("Bureau Headmaster", "Lion Sash", "Reconfigure", 0, 2),
        ("Bureau Headmaster", "Brotherhood Regalia", "Equip", 0, 0),
        ("Bureau Headmaster", "Wrecking Ball Arm", "Equip", 0, 2),
        ("Cloud, Planet's Champion", "Lion Sash", "Reconfigure", 0, 2),
        ("Cloud, Planet's Champion", "Wrecking Ball Arm", "Equip", 0, 1),
        ("Cloud, Planet's Champion", "Wrecking Ball Arm", "Equip", 1, 5),
    ];
    for (reducer, eq, text, nth, cost) in cases {
        let setup = move |t: &mut TestGame| {
            let r = t.battlefield(P0, reducer);
            // A legendary creature to equip: Cloud itself, or Sokka.
            let target = if reducer.starts_with("Cloud") {
                r
            } else {
                t.battlefield(P0, "Sokka, Swordmaster")
            };
            let e = t.battlefield(P0, eq);
            (e, text, nth, target)
        };
        assert_eq!(activation_cost(&setup), cost, "{reducer}: {eq} {text} #{nth}");
    }
}

#[test]
fn fighter_class_reduces_generic_equip_costs_per_class_down_to_zero() {
    cr!("601.2f", "602.2b", "702.6a", "716.2a");
    ruling!(
        "Fighter Class",
        "The discount to equip abilities applies only to generic mana requirements. If you control multiple Fighter Classes that are level 2 or higher, equip abilities cost {2} less for each one. This can't make an ability cost less than {0}."
    );
    // (classes at level 2, equipment, generic cost)
    for (classes, eq, cost) in [
        (1, "Loxodon Warhammer", 1),
        (2, "Loxodon Warhammer", 0),
        (1, "Bonesplitter", 0),
        (0, "Loxodon Warhammer", 3),
    ] {
        let setup = move |t: &mut TestGame| {
            for _ in 0..classes {
                let c = t.battlefield(P0, "Fighter Class");
                mtg_engine::classes::set_level(&mut t.g, c, 2);
            }
            let bears = t.battlefield(P0, "Grizzly Bears");
            let e = t.battlefield(P0, eq);
            (e, "Equip", 0, bears)
        };
        assert_eq!(activation_cost(&setup), cost, "{classes} x {eq}");
    }
    // A level 1 class doesn't reduce; colored equip costs aren't reduced.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Fighter Class");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let e = t.battlefield(P0, "Loxodon Warhammer");
        (e, "Equip", 0, bears)
    };
    assert_eq!(activation_cost(&setup), 3);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Fighter Class");
    mtg_engine::classes::set_level(&mut t.g, c, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Greatsword of Tyr");
    assert!(equip(&mut t, P0, sword, bears).is_err(), "{{W}} isn't reduced");
    mana(&mut t, P0, ManaType::W, 1);
    equip(&mut t, P0, sword, bears).expect("for {W}");
}
