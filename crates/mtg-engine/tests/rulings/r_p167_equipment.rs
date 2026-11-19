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

/// The permanents attached to `id`.
fn attachments_of(t: &TestGame, id: ObjectId) -> Vec<ObjectId> {
    let id = t.g.current(id);
    t.g.permanents()
        .filter(|o| o.attached_to == Some(Entity::Object(id)))
        .map(|o| o.id)
        .collect()
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
    cr!("602.2", "113.7a", "702.6a", "702.122a");
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
    cr!("701.3a", "301.5d");
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
    // (what happens before the ability resolves, the Bears' combat damage)
    for (case, bears_dmg) in [
        ("nothing", 8),
        ("unequipped", 4),
        ("removed from combat", 0),
    ] {
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
        match case {
            "unequipped" => destroy(&mut t, sword),
            "removed from combat" => mtg_engine::combat::remove_from_combat(&mut t.g, bears),
            _ => {}
        }
        t.resolve_all();
        assert!(
            t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike),
            "{case}"
        );
        assert!(!t.obj_now(plain).has_keyword(KeywordKind::DoubleStrike));
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), 20 - bears_dmg - 3, "{case}");
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
    cr!("601.3", "305.2", "301.1", "302.1", "118.9");
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
    assert!(
        legal_cast_methods(&mut t, P0, sword).is_empty(),
        "in combat"
    );
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
    cr!("611.2a");
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
fn activation_cost(
    setup: &dyn Fn(&mut TestGame) -> (ObjectId, &'static str, usize, ObjectId),
) -> u32 {
    (0..10)
        .find(|n| activates_for(setup, *n))
        .expect("never activatable")
}

#[test]
fn bureau_headmaster_and_cloud_reduce_equip_variants_but_not_reconfigure() {
    cr!("601.2f", "602.2b", "702.6a", "702.6c", "702.151a");
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
        (
            "Cloud, Planet's Champion",
            "Wrecking Ball Arm",
            "Equip",
            0,
            1,
        ),
        (
            "Cloud, Planet's Champion",
            "Wrecking Ball Arm",
            "Equip",
            1,
            5,
        ),
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
        assert_eq!(
            activation_cost(&setup),
            cost,
            "{reducer}: {eq} {text} #{nth}"
        );
    }
}

#[test]
fn fighter_class_reduces_generic_equip_costs_per_class_down_to_zero() {
    cr!("601.2f", "602.2b", "118.7a", "702.6a", "716.2a");
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
    assert!(
        equip(&mut t, P0, sword, bears).is_err(),
        "{{W}} isn't reduced"
    );
    mana(&mut t, P0, ManaType::W, 1);
    equip(&mut t, P0, sword, bears).expect("for {W}");
}

// ---------------------------------------------------------------------------------------
// Counts taken as a spell or ability resolves (CR 608.2h), last known information
// ---------------------------------------------------------------------------------------

#[test]
fn wyleth_counts_attachments_on_resolution_or_as_it_last_existed() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Wyleth, Soul of Steel",
        "The number of Auras and Equipment attached to Wyleth is determined as its triggered ability resolves. If Wyleth is no longer on the battlefield at that time, its last existence on the battlefield is checked to determine the number of Auras and Equipment attached to it."
    );
    // (what happens in response, cards drawn)
    for (response, drawn) in [("attach", 3), ("destroy", 2), ("none", 2)] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Wyleth, Soul of Steel");
        attach_new(&mut t, P0, "Bonesplitter", w);
        attach_new(&mut t, P0, "Holy Strength", w);
        attack_with(&mut t, &[(w, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        match response {
            "attach" => {
                attach_new(&mut t, P0, "Leonin Scimitar", w);
            }
            "destroy" => destroy(&mut t, w),
            _ => {}
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + drawn, "{response}");
    }
}

#[test]
fn kemba_counts_equipment_on_resolution_or_as_it_last_existed() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Kemba, Kha Regent",
        "The number of Equipment attached to Kemba is determined as the ability resolves. If Kemba is no longer on the battlefield at that time, its last existence on the battlefield is checked to determine the number of Equipment attached to it."
    );
    for (response, cats) in [("attach", 2), ("destroy", 1), ("unattach", 0)] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kemba, Kha Regent");
        let sword = attach_new(&mut t, P0, "Bonesplitter", k);
        next_upkeep(&mut t, P0);
        assert_eq!(t.stack_len(), 1);
        match response {
            "attach" => {
                attach_new(&mut t, P0, "Leonin Scimitar", k);
            }
            "destroy" => destroy(&mut t, k),
            _ => destroy(&mut t, sword),
        }
        t.resolve_all();
        assert_eq!(
            crate::r_s05_common::tokens_with_subtype(&t, P0, "Cat").len(),
            cats,
            "{response}"
        );
    }
}

#[test]
fn judgment_bolt_slash_of_light_and_barret_count_as_they_resolve() {
    cr!("608.2h");
    ruling!(
        "Judgment Bolt",
        "The value of X is calculated only once, as Judgment Bolt resolves."
    );
    ruling!(
        "Slash of Light",
        "Use the number of creatures you control and the number of Equipment you control when Slash of Light resolves to determine how much damage is dealt."
    );
    ruling!(
        "Barret Wallace",
        "Use the number of equipped creatures you control at the time Barret Wallace's last ability resolves to determine how much damage to deal."
    );
    // Judgment Bolt: one Equipment as it's cast, two as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bonesplitter");
    let giant = t.battlefield(P1, "Craw Wurm");
    let bolt = t.hand(P0, "Judgment Bolt");
    give_mana_for(&mut t, P0, "Judgment Bolt");
    t.cast(P0, bolt).target(obj(giant)).go();
    t.battlefield(P0, "Leonin Scimitar");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(giant));
    // Slash of Light: 1 creature + 1 Equipment cast, 2 creatures + 2 Equipment resolving.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Bonesplitter");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let slash = t.hand(P0, "Slash of Light");
    give_mana_for(&mut t, P0, "Slash of Light");
    t.cast(P0, slash).target(obj(wurm)).go();
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Leonin Scimitar");
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 4);
    // Barret Wallace: one equipped creature as it attacks, two as the trigger resolves.
    let mut t = TestGame::new(2);
    let barret = t.battlefield(P0, "Barret Wallace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Bonesplitter", bears);
    attack_with(&mut t, &[(barret, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    attach_new(&mut t, P0, "Leonin Scimitar", barret);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Statics that care about Equipment
// ---------------------------------------------------------------------------------------

#[test]
fn equipment_statics_table() {
    cr!("613.4c", "301.5");
    ruling!(
        "Stone Haven Outfitter",
        "The bonuses gained from Stone Haven Outfitter's abilities are in addition to any bonuses given by the Equipment."
    );
    ruling!(
        "Weapons Trainer",
        "Weapons Trainer's ability applies even if the Equipment you control isn't attached to any creature."
    );
    ruling!(
        "Weapons Trainer",
        "Weapons Trainer's ability gives just +1/+0, no matter how many Equipment you control beyond the first."
    );
    // Stone Haven Outfitter + Bonesplitter on Grizzly Bears: 2/2 + 2/0 + 1/1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stone Haven Outfitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (5, 3));
    // Weapons Trainer with (n) unattached Equipment: Grizzly Bears' power.
    for (n, power) in [(0, 2), (1, 3), (2, 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Weapons Trainer");
        let bears = t.battlefield(P0, "Grizzly Bears");
        for _ in 0..n {
            t.battlefield(P0, "Leonin Scimitar");
        }
        assert_eq!(t.pt(bears), (power, 2), "{n}");
    }
}

#[test]
fn strength_of_arms_token_table() {
    cr!("608.2c", "115.1");
    ruling!(
        "Strength of Arms",
        "The Human Soldier token that you'll create can't be the target to get +2/+2."
    );
    ruling!(
        "Strength of Arms",
        "You'll create a Human Soldier token if you control any Equipment, not just one attached to the target creature, or even to any creature at all."
    );
    ruling!(
        "Strength of Arms",
        "You'll create only one token, no matter how many Equipment you control beyond the first."
    );
    // (Equipment: none / attached to another creature / unattached x2, tokens)
    for (setup, tokens) in [("none", 0), ("other", 1), ("loose2", 1)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        match setup {
            "other" => {
                let giant = t.battlefield(P0, "Hill Giant");
                attach_new(&mut t, P0, "Bonesplitter", giant);
            }
            "loose2" => {
                t.battlefield(P0, "Bonesplitter");
                t.battlefield(P0, "Leonin Scimitar");
            }
            _ => {}
        }
        let s = t.hand(P0, "Strength of Arms");
        give_mana_for(&mut t, P0, "Strength of Arms");
        t.cast(P0, s).target(obj(bears)).go();
        t.resolve_all();
        let made = crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier");
        assert_eq!(made.len(), tokens, "{setup}");
        for tok in made {
            assert_eq!(t.pt(tok), (1, 1), "the token didn't get +2/+2");
        }
        assert_eq!(t.pt(bears).1, 4);
    }
}

#[test]
fn warchanter_skald_has_no_way_to_tap_itself() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Warchanter Skald",
        "Warchanter Skald’s triggered ability doesn’t allow you to tap it. You have to find some other way to tap it. You could attack with it, I suppose?"
    );
    let mut t = TestGame::new(2);
    let skald = t.battlefield(P0, "Warchanter Skald");
    attach_new(&mut t, P0, "Short Sword", skald);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, skald));
    attack_with(&mut t, &[(skald, Entity::Player(P1))]);
    assert!(t.obj_now(skald).tapped);
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Dwarf").len(),
        1
    );
}

#[test]
fn stoneforge_mystic_puts_any_equipment_from_hand_unattached() {
    cr!("301.5b");
    ruling!(
        "Stoneforge Mystic",
        "When Stoneforge Mystic's second ability resolves, you may put any Equipment card from your hand onto the battlefield, not just the one you searched for with its first ability. The Equipment is put onto the battlefield unattached."
    );
    let mut t = TestGame::new(2);
    let sm = t.battlefield(P0, "Stoneforge Mystic");
    let hammer = t.hand(P0, "Loxodon Warhammer");
    t.lands(P0, "Plains", 2);
    t.answer_choose(P0, &[obj(hammer)]);
    t.answer_yes(P0, true);
    t.activate(P0, sm, 0, &[]).unwrap();
    t.resolve_all();
    let h = t.named_on_battlefield("Loxodon Warhammer");
    assert_eq!(h.len(), 1);
    assert_eq!(t.g.obj(h[0]).attached_to, None);
}

#[test]
fn godo_adds_a_combat_phase_right_after_this_one() {
    cr!("500.8");
    ruling!(
        "Godo, Bandit Warlord",
        "Unlike many effects that grant additional combat phases, you don't get an additional main phase with Godo, Bandit Warlord's ability. The additional combat phase happens immediately after the first combat phase."
    );
    use mtg_engine::turn::Step;
    let mut t = TestGame::new(2);
    let godo = t.battlefield(P0, "Godo, Bandit Warlord");
    attack_with(&mut t, &[(godo, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!t.obj_now(godo).tapped, "untapped");
    block_and_finish(&mut t, P1, &[]);
    // The next step is the beginning of combat of the added phase, not a main phase.
    let ok = t.g.run_until(1000, |g| g.turn.step != Step::EndOfCombat);
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    let mains = t.g.turn.schedule.iter().filter(|s| s.is_main()).count();
    assert_eq!(mains, 1, "only the normal postcombat main phase");
}

// ---------------------------------------------------------------------------------------
// Attaching: what an Equipment or Aura can legally be attached to (CR 301.5c, 303.4,
// 701.3b)
// ---------------------------------------------------------------------------------------

#[test]
fn one_last_job_attaches_only_where_legal() {
    cr!("303.4g", "301.5e", "702.16d", "702.172a");
    ruling!(
        "One Last Job",
        "The Aura or Equipment must be able to be legally attached to the creature you choose. You can’t return an Aura card to the battlefield if there’s nothing to enchant. However, if you control no creatures (and didn’t bring any back with the first two modes), you could return an Equipment card to the battlefield unattached to anything."
    );
    // An Aura with nothing to enchant stays in the graveyard.
    let mut t = TestGame::new(2);
    let aura = t.graveyard(P0, "Holy Strength");
    let job = t.hand(P0, "One Last Job");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, job).modes(&[2]).target(obj(aura)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    // An Equipment with no creature enters unattached.
    let mut t = TestGame::new(2);
    let sword = t.graveyard(P0, "Bonesplitter");
    let job = t.hand(P0, "One Last Job");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, job).modes(&[2]).target(obj(sword)).go();
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter");
    assert_eq!(s.len(), 1);
    assert_eq!(t.g.obj(s[0]).attached_to, None);
    // With a creature it can't be attached to (protection from artifacts) and one it can,
    // it goes on the one it can: the other can't be chosen.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.graveyard(P0, "Bonesplitter");
    let job = t.hand(P0, "One Last Job");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, job).modes(&[2]).target(obj(sword)).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(chosen)]);
    t.resolve_all();
    t.clear_answers();
    for (_, d) in &t.asked()[from..] {
        if let mtg_engine::decision::Decision::ChooseEntities { candidates, .. } = d {
            assert!(!candidates.contains(&obj(chosen)), "{candidates:?}");
        }
    }
    let s = t.named_on_battlefield("Bonesplitter");
    assert_eq!(t.g.obj(s[0]).attached_to, Some(obj(bears)));
}

#[test]
fn one_last_job_chooses_the_creature_as_it_resolves() {
    cr!("608.2c", "700.2", "301.7", "301.5e");
    ruling!(
        "One Last Job",
        "The creature that the Aura or Equipment card returned by the third mode will enter attached to isn’t a target of that mode and is chosen as One Last Job resolves. Notably, if you return a Vehicle with the second mode, you won’t have the opportunity to crew it before having to choose a creature for the third mode. In most cases, it won’t be a creature at that time."
    );
    // A creature returned by the first mode can be chosen; it has hexproof-like
    // irrelevance: the choice isn't targeted. (Slippery Bogle has hexproof.)
    let mut t = TestGame::new(2);
    let bogle = t.graveyard(P0, "Slippery Bogle");
    let sword = t.graveyard(P0, "Bonesplitter");
    let job = t.hand(P0, "One Last Job");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 5);
    t.cast(P0, job)
        .modes(&[0, 2])
        .targets(&[obj(bogle)])
        .targets(&[obj(sword)])
        .go();
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter")[0];
    let b = t.named_on_battlefield("Slippery Bogle")[0];
    assert_eq!(t.g.obj(s).attached_to, Some(obj(b)));
    // A Vehicle returned by the second mode isn't a creature: the Equipment enters
    // unattached.
    let mut t = TestGame::new(2);
    let copter = t.graveyard(P0, "Smuggler's Copter");
    let sword = t.graveyard(P0, "Bonesplitter");
    let job = t.hand(P0, "One Last Job");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, job)
        .modes(&[1, 2])
        .targets(&[obj(copter)])
        .targets(&[obj(sword)])
        .go();
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter")[0];
    assert_eq!(t.named_on_battlefield("Smuggler's Copter").len(), 1);
    assert_eq!(t.g.obj(s).attached_to, None);
}

#[test]
fn effects_cant_attach_equipment_where_it_cant_legally_be_attached() {
    cr!("701.3b", "702.16d", "301.5c");
    ruling!(
        "Battlefield Improvisation",
        "You can't try to attach an Equipment to the target creature if that Equipment can't legally be attached to it."
    );
    ruling!(
        "Beatrix, Loyal General",
        "You can't use Beatrix's last ability to try to attach an Equipment to a creature if that Equipment can't legally be attached to that creature."
    );
    ruling!(
        "Barret, Avalanche Leader",
        "You can't use Barret's last ability to try to attach an Equipment to a Rebel if that Equipment can't legally be attached to that Rebel."
    );
    ruling!(
        "Yuffie, Materia Hunter",
        "You can't use Yuffie's last ability to try to attach an Equipment to Yuffie if that Equipment can't legally be attached to Yuffie."
    );
    // Battlefield Improvisation on an attacking Tel-Jilad Chosen (protection from
    // artifacts): it gets +2/+2, the Equipment stays where it is.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let holder = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Bonesplitter", holder);
    let loose = t.battlefield(P0, "Leonin Scimitar");
    attack_with(&mut t, &[(chosen, Entity::Player(P1))]);
    let bi = t.hand(P0, "Battlefield Improvisation");
    give_mana_for(&mut t, P0, "Battlefield Improvisation");
    t.answer_choose(P0, &[obj(sword), obj(loose)]);
    t.answer_yes(P0, true);
    t.cast(P0, bi).target(obj(chosen)).go();
    t.resolve_all();
    assert_eq!(t.pt(chosen), (4, 3));
    assert_eq!(attached_to(&t, sword), Some(obj(holder)));
    assert_eq!(attached_to(&t, loose), None);
    // Beatrix targeting Tel-Jilad Chosen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Beatrix, Loyal General");
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let holder = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Bonesplitter", holder);
    t.answer_targets(P0, &[obj(chosen)]);
    t.answer_choose(P0, &[obj(sword)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(holder)));
    // Barret: an Equipment that's a creature (March of the Machines) can't equip the
    // Rebel; it stays unattached.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barret, Avalanche Leader");
    t.battlefield(P0, "March of the Machines");
    let rebel = t.battlefield(P0, "Barret Wallace");
    let sword = t.battlefield(P0, "Bonesplitter");
    assert!(is_creature(&t, sword));
    t.answer_targets(P0, &[obj(sword)]);
    t.answer_targets(P0, &[obj(rebel)]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), None);
    // Yuffie, given protection from artifacts in response to its trigger.
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Bonesplitter");
    let relic = t.battlefield(P1, "Mind Stone");
    t.answer_targets(P0, &[obj(relic)]);
    let yuffie = enter(&mut t, P0, "Yuffie, Materia Hunter");
    assert_eq!(t.stack_len(), 1);
    let blessing = t.hand(P0, "Apostle's Blessing");
    mana(&mut t, P0, ManaType::W, 2);
    // Protection from artifacts (the first of the choices offered).
    t.cast(P0, blessing).target(obj(yuffie)).go();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.resolve();
    t.answer_choose(P0, &[obj(sword)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.obj_now(relic).controller, P0);
    assert_eq!(attached_to(&t, sword), None);
    // ... while a noncreature Equipment does go on Yuffie.
    let mut t2 = TestGame::new(2);
    let sword2 = t2.battlefield(P0, "Bonesplitter");
    let relic2 = t2.battlefield(P1, "Mind Stone");
    t2.answer_targets(P0, &[obj(relic2)]);
    t2.answer_choose(P0, &[obj(sword2)]);
    t2.answer_yes(P0, true);
    let yuffie2 = enter(&mut t2, P0, "Yuffie, Materia Hunter");
    t2.resolve_all();
    assert_eq!(attached_to(&t2, sword2), Some(obj(yuffie2)));
    let _ = yuffie;
}

#[test]
fn cass_cant_return_an_aura_that_cant_enchant_the_target() {
    cr!("303.4i", "702.16b", "702.16c", "613.1e");
    ruling!(
        "Cass, Hand of Vengeance",
        "You can’t return Aura cards that can’t legally be attached to the target creature. For example, if the target creature has protection from red, you can’t return Grim Reaper’s Sprint to the battlefield with Cass’s last ability."
    );
    // A black Aura and a target with protection from black: the Aura stays in the
    // graveyard; the Equipment is still attached.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cass, Hand of Vengeance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let knight = t.battlefield(P0, "White Knight");
    let aura = attach_new(&mut t, P0, "Unholy Strength", bears);
    let sword = attach_new(&mut t, P0, "Bonesplitter", bears);
    t.answer_targets(P0, &[obj(knight)]);
    // Choose to return the Aura card anyway, and the Equipment.
    t.answer_choose_moved(P0, &[obj(aura)]);
    t.answer_choose(P0, &[obj(sword)]);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    let aura_card = t.g.current(aura);
    assert_eq!(t.zone(aura_card), Zone::Graveyard(P0));
    t.resolve_all();
    // The card never left the graveyard (it wasn't returned, then put back by the state-based
    // action for an Aura attached illegally).
    assert!(t.g.is_live(aura_card));
    assert_eq!(t.zone(aura_card), Zone::Graveyard(P0));
    assert_eq!(attached_to(&t, sword), Some(obj(knight)));
    // The example: a creature with protection from red can't get Grim Reaper's Sprint
    // (Cass's ability, from a red source, can't even target it).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cass, Hand of Vengeance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let walker = t.battlefield(P0, "Kor Firewalker");
    attach_new(&mut t, P0, "Grim Reaper's Sprint", bears);
    let from = t.asked().len();
    destroy(&mut t, bears);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered.is_empty());
    assert!(offered.iter().all(|c| !c.contains(&obj(walker))));
    t.resolve_all();
    assert!(attachments_of(&t, walker).is_empty());
    // The example as such, with Cass made black (Darkest Hour: "All creatures are black."):
    // its ability can target the creature with protection from red, and Grim Reaper's
    // Sprint (a red Aura) can't be returned attached to it; the Equipment can.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Darkest Hour");
    t.battlefield(P0, "Cass, Hand of Vengeance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let walker = t.battlefield(P0, "Kor Firewalker");
    let sprint = attach_new(&mut t, P0, "Grim Reaper's Sprint", bears);
    let sword = attach_new(&mut t, P0, "Bonesplitter", bears);
    t.answer_targets(P0, &[obj(walker)]);
    t.answer_choose_moved(P0, &[obj(sprint)]);
    t.answer_choose(P0, &[obj(sword)]);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    let sprint_card = t.g.current(sprint);
    t.resolve_all();
    assert!(t.g.is_live(sprint_card));
    assert_eq!(t.zone(sprint_card), Zone::Graveyard(P0));
    assert_eq!(attached_to(&t, sword), Some(obj(walker)));
}

#[test]
fn reckless_crew_and_goldwardens_gambit_attach_one_equipment_per_token() {
    cr!("701.3a", "608.2c");
    ruling!(
        "Reckless Crew",
        "You can’t have more than one Equipment become attached to the same token."
    );
    ruling!(
        "Goldwardens' Gambit",
        "You choose which Equipment to attach to each token, then attach those Equipment all at once. You can't move the same Equipment around to each of the tokens as the ability resolves."
    );
    use mtg_engine::decision::Decision;
    for (spell, equipment) in [("Reckless Crew", 3), ("Goldwardens' Gambit", 2)] {
        let mut t = TestGame::new(2);
        let eq: Vec<ObjectId> = ["Bonesplitter", "Leonin Scimitar", "Vulshok Morningstar"]
            [..equipment]
            .iter()
            .map(|n| t.battlefield(P0, n))
            .collect();
        let s = t.hand(P0, spell);
        mana(&mut t, P0, ManaType::R, 2);
        mana(&mut t, P0, ManaType::C, 6);
        // Try to put the same Equipment on every token.
        for _ in 0..5 {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(eq[0])]);
        }
        t.cast(P0, s).go();
        let from = t.asked().len();
        t.resolve_all();
        let toks = tokens(&t, P0);
        assert_eq!(toks.len(), if equipment == 3 { 3 } else { 5 }, "{spell}");
        // Each choice is among the Equipment not chosen for an earlier token.
        let offered: Vec<Vec<Entity>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(offered.len(), equipment, "{spell}: {offered:?}");
        for (i, c) in offered.iter().enumerate() {
            assert_eq!(c.len(), equipment - i, "{spell}: {offered:?}");
        }
        assert_eq!(attached_to(&t, eq[0]), Some(obj(toks[0])), "{spell}");
        for tok in &toks {
            assert!(attachments_of(&t, *tok).len() <= 1, "{spell}");
        }
        let attached: Vec<_> = eq.iter().filter_map(|e| attached_to(&t, *e)).collect();
        assert_eq!(
            attached.len(),
            equipment,
            "{spell}: every Equipment got a token"
        );
        let mut distinct = attached.clone();
        distinct.sort_by_key(|e| format!("{e:?}"));
        distinct.dedup();
        assert_eq!(distinct.len(), attached.len());
    }
}

#[test]
fn kor_outfitter_may_target_itself() {
    cr!("115.1", "603.3d");
    ruling!(
        "Kor Outfitter",
        "You may target Kor Outfitter with its ability."
    );
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Bonesplitter");
    let ko = t.enter(P0, "Kor Outfitter");
    t.g.flush_events();
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(sword)]);
    t.answer_targets(P0, &[obj(ko)]);
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().any(|c| c.contains(&obj(ko))), "{offered:?}");
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(ko)));
    assert_eq!(t.pt(ko), (4, 2));
}

#[test]
fn magnetic_snuffler_returns_an_equipment_it_cant_wear_unattached() {
    cr!("301.5c", "301.5e");
    ruling!(
        "Magnetic Snuffler",
        "You may target an Equipment card that can't legally be attached to Magnetic Snuffler with its first ability. If you do, it will enter the battlefield unattached."
    );
    for march in [true, false] {
        let mut t = TestGame::new(2);
        if march {
            t.battlefield(P0, "March of the Machines");
        }
        let sword = t.graveyard(P0, "Bonesplitter");
        t.answer_targets(P0, &[obj(sword)]);
        let sn = enter(&mut t, P0, "Magnetic Snuffler");
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        let s = t.named_on_battlefield("Bonesplitter");
        assert_eq!(s.len(), 1, "{march}");
        let expected = if march { None } else { Some(obj(sn)) };
        assert_eq!(t.g.obj(s[0]).attached_to, expected, "{march}");
    }
}

#[test]
fn armed_and_armored_locks_in_its_vehicles_and_needs_neither() {
    cr!("611.2c", "608.2c", "609.3");
    ruling!(
        "Armed and Armored",
        "The set of Vehicles affected by Armed and Armored is determined as the spell resolves. Vehicles you begin to control later in the turn won't automatically become creatures."
    );
    ruling!(
        "Armed and Armored",
        "You can cast Armed and Armored even if you don't control any Vehicles or Dwarves. Whatever part of the effect that can happen does, and the rest is ignored."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Smuggler's Copter");
    let theirs = t.battlefield(P1, "Cultivator's Caravan");
    let aa = t.hand(P0, "Armed and Armored");
    give_mana_for(&mut t, P0, "Armed and Armored");
    t.cast(P0, aa).go();
    t.resolve_all();
    assert!(is_creature(&t, mine));
    give_control(&mut t, theirs, P0);
    assert!(!is_creature(&t, theirs));
    // No Vehicles, no Dwarves: it can be cast and does nothing.
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Bonesplitter");
    let aa = t.hand(P0, "Armed and Armored");
    give_mana_for(&mut t, P0, "Armed and Armored");
    t.cast(P0, aa).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Armed and Armored"));
    assert_eq!(attached_to(&t, sword), None);
    // A Dwarf but no Vehicles: the Equipment is attached.
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Bonesplitter");
    let dwarf = t.battlefield(P0, "Sram, Senior Edificer");
    let aa = t.hand(P0, "Armed and Armored");
    give_mana_for(&mut t, P0, "Armed and Armored");
    t.answer_choose(P0, &[obj(dwarf)]);
    t.answer_choose(P0, &[obj(sword)]);
    t.cast(P0, aa).go();
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(dwarf)));
}
