//! Rulings batch P194 — abilities that count the life a player gained or lost this turn:
//! the totals gained and lost are counted separately, not as the net change of the life
//! total (CR 119.3), and the amount is read as the ability resolves (CR 608.2h).

use crate::r_p194_common::*;
use crate::r_s01_common::{attack_with, supported};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn life_gained_and_lost_this_turn_are_counted_separately() {
    cr!("119.3", "608.2h");
    ruling!(
        "Nightmare's Thirst",
        "if you’ve gained 5 life and lost 8 before Nightmare’s Thirst resolves, Nightmare’s Thirst has you gain 1 more life, then it gives the target creature -6/-6."
    );
    ruling!(
        "Essence Pulse",
        "if you lost 3 life and gained 3 life earlier in the turn, you will gain 2 more life from Essence Pulse and each creature will get -5/-5."
    );
    ruling!(
        "Children of Korlis",
        "if you lose 5 life and gain 3 life before activating the ability, the ability will cause you to gain 5 life, not 2."
    );
    ruling!(
        "Voracious Wurm",
        "if you start the turn at 10 life, gain 6 life during the turn, then lose 6 life later that turn, Voracious Wurm will enter the battlefield with six +1/+1 counters."
    );
    ruling!(
        "Wound Reflection",
        "if an opponent lost 4 life and gained 6 life during the turn, that player will have a higher life total than they started the turn with but Wound Reflection's ability will still cause that player to lose 4 life."
    );
    for n in [
        "Nightmare's Thirst",
        "Essence Pulse",
        "Children of Korlis",
        "Voracious Wurm",
        "Wound Reflection",
        "Ancient Brontodon",
    ] {
        supported(n);
    }
    // Nightmare's Thirst: gained 5, lost 8, then gains 1 more: -6/-6 on a 9/9.
    let mut t = TestGame::new(2);
    let big = t.battlefield(P1, "Ancient Brontodon");
    gain(&mut t, P0, 5);
    lose(&mut t, P0, 8);
    let nt = t.hand(P0, "Nightmare's Thirst");
    mana(&mut t, P0, ManaType::B, 1);
    t.cast(P0, nt).target(big).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(big), (3, 3));

    // Essence Pulse: lost 3, gained 3, then gains 2: each creature gets -5/-5.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Ancient Brontodon");
    let theirs = t.battlefield(P1, "Ancient Brontodon");
    lose(&mut t, P0, 3);
    gain(&mut t, P0, 3);
    let ep = t.hand(P0, "Essence Pulse");
    mana(&mut t, P0, ManaType::B, 4);
    t.cast(P0, ep).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.pt(mine), (4, 4));
    assert_eq!(t.pt(theirs), (4, 4));

    // Children of Korlis: lost 5, gained 3, gains 5.
    let mut t = TestGame::new(2);
    let kids = t.battlefield(P0, "Children of Korlis");
    lose(&mut t, P0, 5);
    gain(&mut t, P0, 3);
    t.activate(P0, kids, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);

    // Voracious Wurm: from 10 life, gained 6 then lost 6: six counters.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 10;
    gain(&mut t, P0, 6);
    lose(&mut t, P0, 6);
    assert_eq!(t.life(P0), 10);
    let w = t.hand(P0, "Voracious Wurm");
    mana(&mut t, P0, ManaType::G, 2);
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 6);

    // Wound Reflection: the opponent lost 4 and gained 6; it loses 4 more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wound Reflection");
    lose(&mut t, P1, 4);
    gain(&mut t, P1, 6);
    assert_eq!(t.life(P1), 22);
    end_step(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn bogbeast_counts_life_gained_including_its_own() {
    cr!("119.3", "608.2h");
    ruling!(
        "Blossoming Bogbeast",
        "if you lost 3 life and gained 3 life earlier in the turn, you'll gain 2 more life from Blossoming Bogbeast and creatures you control will get +5/+5."
    );
    supported("Blossoming Bogbeast");
    let mut t = TestGame::new(2);
    let bog = t.battlefield(P0, "Blossoming Bogbeast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    lose(&mut t, P0, 3);
    gain(&mut t, P0, 3);
    attack_with(&mut t, &[(bog, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.pt(bog), (8, 8));
    assert_eq!(t.pt(bears), (7, 7));
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn life_lost_this_turn_is_counted_without_the_life_gained() {
    cr!("119.3", "608.2h");
    ruling!(
        "Rakdos, Lord of Riots",
        "if an opponent loses 5 life and then gains 10 life in a turn, creature spells you cast will cost {5} less to cast."
    );
    ruling!(
        "Wound Reflection",
        "It doesn't matter how the opponent lost life or who caused it. It also doesn't matter if Wound Reflection wasn't on the battlefield at the time some or all of the life was lost."
    );
    supported("Rakdos, Lord of Riots");
    // Rakdos: the opponent lost 5 and gained 10: Ancient Brontodon ({6}{G}{G}) costs
    // {1}{G}{G}.
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Rakdos, Lord of Riots");
        lose(t, P1, 5);
        gain(t, P1, 10);
    };
    assert_total_cost(&setup, P0, "Ancient Brontodon", &[(ManaType::G, 2)], 1);

    // Wound Reflection: 2 damage from a Shock and 3 life paid by the opponent, all
    // before Wound Reflection was on the battlefield.
    let mut t = TestGame::new(2);
    shock(&mut t, P0, pl(P1));
    assert!(t.g.pay_life(P1, 3));
    assert_eq!(t.life(P1), 15);
    t.battlefield(P0, "Wound Reflection");
    end_step(&mut t);
    assert_eq!(t.life(P1), 10);
}

#[test]
fn lifelost_cost_reductions_reduce_only_generic_mana() {
    cr!("118.7a", "601.2f", "202.3");
    ruling!(
        "Rakdos, Lord of Riots",
        "Rakdos's last ability can't reduce the colored mana requirement of a creature spell you cast."
    );
    ruling!(
        "Ezzaroot Channeler",
        "The second ability’s cost reduction can reduce only generic mana in a spell’s total cost. It can’t reduce colored mana requirements."
    );
    ruling!(
        "Ezzaroot Channeler",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions (such as that of Ezzaroot Channeler’s second ability). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Ezzaroot Channeler");
    // Rakdos with the opponent down 10: Brontodon still costs {G}{G}.
    let rakdos = |t: &mut TestGame| {
        t.battlefield(P0, "Rakdos, Lord of Riots");
        lose(t, P1, 10);
    };
    assert!(casts_for(&rakdos, P0, "Ancient Brontodon", &[(ManaType::G, 2)], 0));
    assert!(!casts_for(&rakdos, P0, "Ancient Brontodon", &[(ManaType::G, 1)], 1));
    // Ezzaroot Channeler with 10 life gained: {G}{G} as well; the spell's mana value is
    // still 8.
    let ezza = |t: &mut TestGame| {
        t.battlefield(P0, "Ezzaroot Channeler");
        gain(t, P0, 10);
    };
    assert!(casts_for(&ezza, P0, "Ancient Brontodon", &[(ManaType::G, 2)], 0));
    assert!(!casts_for(&ezza, P0, "Ancient Brontodon", &[(ManaType::G, 1)], 1));
    // 3 life gained: {3}{G}{G}.
    let ezza3 = |t: &mut TestGame| {
        t.battlefield(P0, "Ezzaroot Channeler");
        gain(t, P0, 3);
    };
    assert_total_cost(&ezza3, P0, "Ancient Brontodon", &[(ManaType::G, 2)], 3);
    let mut t = TestGame::new(2);
    ezza(&mut t);
    let b = t.hand(P0, "Ancient Brontodon");
    mana(&mut t, P0, ManaType::G, 2);
    let spell = t.cast(P0, b).go();
    assert_eq!(t.g.mana_value_of(spell), 8);
    // Noncreature spells aren't reduced.
    let ezza_div = |t: &mut TestGame| {
        t.battlefield(P0, "Ezzaroot Channeler");
        gain(t, P0, 10);
    };
    assert_total_cost(&ezza_div, P0, "Divination", &[(ManaType::U, 1)], 2);
}

#[test]
fn rakdos_cast_restriction_and_its_own_cost() {
    cr!("601.3", "601.2f");
    ruling!(
        "Rakdos, Lord of Riots",
        "Rakdos can be put onto the battlefield by another spell or ability even if no opponent has lost life that turn."
    );
    ruling!(
        "Rakdos, Lord of Riots",
        "Rakdos's last ability won't reduce the cost to cast Rakdos itself. It applies only to creature spells you cast once Rakdos is on the battlefield."
    );
    // No opponent lost life: it can't be cast, but it can be put onto the battlefield.
    let mut t = TestGame::new(2);
    let r = t.hand(P0, "Rakdos, Lord of Riots");
    mana(&mut t, P0, ManaType::B, 2);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(t.cast(P0, r).try_go().is_err());
    let r2 = t.enter(P0, "Rakdos, Lord of Riots");
    assert!(t.on_battlefield(r2));
    // In hand, Rakdos reduces nothing: with the opponent down 3, Brontodon costs full.
    let in_hand = |t: &mut TestGame| {
        t.hand(P0, "Rakdos, Lord of Riots");
        lose(t, P1, 3);
    };
    assert_total_cost(&in_hand, P0, "Ancient Brontodon", &[(ManaType::G, 2)], 6);
}

#[test]
fn cryptborn_horror_counts_damage_as_life_lost_but_not_infect_damage() {
    cr!("120.3a", "120.3b", "614.1c");
    ruling!(
        "Cryptborn Horror",
        "Remember, damage dealt to a player causes that player to lose that much life (unless the source has infect)."
    );
    supported("Cryptborn Horror");
    supported("Plague Stinger");
    let mut t = TestGame::new(2);
    let stinger = t.battlefield(P0, "Plague Stinger");
    shock(&mut t, P0, pl(P1));
    damage(&mut t, stinger, pl(P1), 4);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.g.player(P1).counter("poison"), 4);
    let c = t.hand(P0, "Cryptborn Horror");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 2);
}

#[test]
fn rowan_and_will_reduce_generic_by_life_lost_or_gained() {
    cr!("118.7a", "601.2f", "202.3", "608.2h");
    ruling!(
        "Rowan, Scion of War",
        "Rowan's activated ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that cost."
    );
    ruling!(
        "Rowan, Scion of War",
        "if you gained 3 life and lost 3 life earlier in the turn, the cost of black and/or red spells you cast this turn will be reduced by {3}."
    );
    ruling!(
        "Rowan, Scion of War",
        "Rowan's activated ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    ruling!(
        "Will, Scion of Peace",
        "Will's activated ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that cost."
    );
    ruling!(
        "Will, Scion of Peace",
        "if you gained 3 life and lost 3 life earlier in the turn, the cost of white and/or blue spells you cast this turn will be reduced by {3}."
    );
    ruling!(
        "Will, Scion of Peace",
        "Will's activated ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    ruling!(
        "Will, Scion of Peace",
        "The value of X is determined only once, at the time Will, Scion of Peace's activated ability resolves."
    );
    supported("Rowan, Scion of War");
    supported("Will, Scion of Peace");
    // (scion, a spell of its colors with {3} generic, its colored mana, an off-color spell)
    let cases = [
        (
            "Rowan, Scion of War",
            "Gravedigger",
            (ManaType::B, 1),
            "Divination",
        ),
        (
            "Will, Scion of Peace",
            "Air Elemental",
            (ManaType::U, 2),
            "Gravedigger",
        ),
    ];
    for (scion, spell, colored, other) in cases {
        // Gained 3 and lost 3: reduced by {3} (Gravedigger {3}{B} costs {B}; Air
        // Elemental {3}{U}{U} costs {U}{U}).
        let setup = |t: &mut TestGame| {
            let s = t.battlefield(P0, scion);
            gain(t, P0, 3);
            lose(t, P0, 3);
            t.activate(P0, s, 0, &[]).unwrap();
            t.resolve_all();
        };
        assert!(casts_for(&setup, P0, spell, &[colored], 0), "{scion}");
        // Never the colored part.
        assert!(
            !casts_for(&setup, P0, spell, &[(colored.0, colored.1 - 1)], 1),
            "{scion}"
        );
        // Other spells aren't reduced (Divination {2}{U}, Gravedigger {3}{B}).
        let other_color = if other == "Divination" {
            (ManaType::U, 1)
        } else {
            (ManaType::B, 1)
        };
        let full = if other == "Divination" { 2 } else { 3 };
        assert_total_cost(&setup, P0, other, &[other_color], full);
        // The spell's mana value is unchanged.
        let mut t = TestGame::new(2);
        setup(&mut t);
        let c = t.hand(P0, spell);
        mana(&mut t, P0, colored.0, colored.1);
        let s = t.cast(P0, c).go();
        assert_eq!(
            t.g.mana_value_of(s),
            colored.1 + 3,
            "{scion}: mana value"
        );
    }
    // Will's X is locked in as the ability resolves: life gained afterwards doesn't
    // reduce costs further.
    let setup = |t: &mut TestGame| {
        let s = t.battlefield(P0, "Will, Scion of Peace");
        gain(t, P0, 1);
        t.activate(P0, s, 0, &[]).unwrap();
        t.resolve_all();
        gain(t, P0, 5);
    };
    assert_total_cost(&setup, P0, "Air Elemental", &[(ManaType::U, 2)], 2);
}

#[test]
fn heliod_warped_eclipse_reduces_generic_and_flash_is_only_for_spells() {
    cr!("118.7a", "601.2f", "202.3", "602.5d");
    ruling!(
        "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse",
        "That ability can’t reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that cost."
    );
    ruling!(
        "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse",
        "The last ability of Heliod, the Warped Eclipse doesn’t change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    ruling!(
        "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse",
        "The “as though they had flash” effect applies only to casting spells. It does not, for example, change when you may activate abilities that can be activated “only as a sorcery.”"
    );
    supported("Heliod, the Radiant Dawn // Heliod, the Warped Eclipse");
    const HELIOD: &str = "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse";
    // The opponent drew two cards: Gravedigger {3}{B} costs {1}{B}; four cards: {B}, never
    // less.
    for (drawn, generic) in [(2, 1), (4, 0)] {
        let setup = move |t: &mut TestGame| {
            crate::r_p079_common::back_face(t, P0, HELIOD);
            t.g.draw_cards(P1, drawn);
            t.g.flush_events();
        };
        assert_total_cost(&setup, P0, "Gravedigger", &[(ManaType::B, 1)], generic);
        assert!(!casts_for(&setup, P0, "Gravedigger", &[], 4));
        let mut t = TestGame::new(2);
        setup(&mut t);
        let c = t.hand(P0, "Gravedigger");
        mana(&mut t, P0, ManaType::B, 1);
        mana(&mut t, P0, ManaType::C, generic);
        let s = t.cast(P0, c).go();
        assert_eq!(t.g.mana_value_of(s), 4);
    }
    // On the opponent's turn: a sorcery can be cast, but Will's "activate only as a
    // sorcery" ability can't be activated.
    let mut t = TestGame::new(2);
    crate::r_p079_common::back_face(&mut t, P0, HELIOD);
    let will = t.battlefield(P0, "Will, Scion of Peace");
    t.set_step(P1, Step::PrecombatMain);
    let d = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, d).go();
    assert!(t.activate(P0, will, 0, &[]).is_err());
}

#[test]
fn end_step_life_loss_amounts_are_read_as_each_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Archfiend of Despair",
        "if you control two of them and an opponent lost 3 life earlier in the turn, the first ability to resolve would have that player lose 3 life, and the second would have that player lose 6 life."
    );
    ruling!(
        "Defiling Daemogoth",
        "The value of X is calculated at the time Defiling Daemogoth's last ability resolves."
    );
    ruling!(
        "Hope Estheim",
        "The value of X is calculated only once, as Hope's last ability resolves."
    );
    supported("Archfiend of Despair");
    supported("Defiling Daemogoth");
    supported("Hope Estheim");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archfiend of Despair");
    t.battlefield(P0, "Archfiend of Despair");
    lose(&mut t, P1, 3);
    end_step(&mut t);
    assert_eq!(t.life(P1), 20 - 3 - 3 - 6);

    // Life gained in response to the trigger counts.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiling Daemogoth");
    gain(&mut t, P0, 1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    gain(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hope Estheim");
    gain(&mut t, P0, 1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    gain(&mut t, P0, 2);
    let lib = t.library_size(P1);
    t.resolve();
    assert_eq!(t.library_size(P1), lib - 3);
    // Gaining more afterwards doesn't mill more.
    gain(&mut t, P0, 4);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 3);
}

#[test]
fn celestine_targets_with_the_life_gained_when_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "601.2c");
    ruling!(
        "Celestine, the Living Saint",
        "The target creature card is chosen as the triggered ability is put on the stack. Gaining life after that point won't let you choose a target with a higher mana value."
    );
    supported("Celestine, the Living Saint");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestine, the Living Saint");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    gain(&mut t, P0, 2);
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.settle();
    let choices = target_choices(&t, P0, from, "");
    assert_eq!(choices, vec![vec![obj(bears)]]);
    // Gaining life now doesn't let the ability return Hill Giant.
    gain(&mut t, P0, 5);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn warlock_class_checks_deaths_this_turn_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Warlock Class",
        "Warlock Class does not need to have been on the battlefield when the creature died for its first ability."
    );
    ruling!(
        "Warlock Class",
        "Warlock Class's first ability will trigger only once during your end step, no matter how many creatures died this turn. However, if no creatures have died so far this turn as the end step begins, the ability won't trigger at all."
    );
    supported("Warlock Class");
    // Two creatures died before Warlock Class entered: one trigger, 1 life.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, a);
    kill(&mut t, b);
    t.battlefield(P0, "Warlock Class");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // No creature died before the end step began: no trigger, even if one dies in the
    // end step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warlock Class");
    let c = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    kill(&mut t, c);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn postcombat_main_triggers_happen_without_attacks() {
    cr!("505.1");
    ruling!(
        "Neheb, the Eternal",
        "You get a postcombat main phase even if no creatures attacked during a turn. Neheb's last ability will trigger."
    );
    ruling!(
        "Florian, Voldaren Scion",
        "You get a postcombat main phase even if no creatures attacked during a turn. Florian's last ability will trigger."
    );
    supported("Neheb, the Eternal");
    supported("Florian, Voldaren Scion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neheb, the Eternal");
    lose(&mut t, P1, 3);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 3);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Florian, Voldaren Scion");
    lose(&mut t, P1, 2);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 1);
}
