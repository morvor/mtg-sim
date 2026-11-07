//! Rulings batch P046 — how much "This spell costs {N} less to cast if ..." reduces a
//! cost (CR 601.2f, 118.7a): once however many times the condition holds, never below the
//! colored part of the cost, and never changing the spell's mana value (CR 202.3).

use crate::r_p046_common::*;
use crate::r_s01_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The cost P0 pays for `name` in a game of `players` set up by `setup` (which returns
/// the spell's targets).
fn cost_of(
    name: &str,
    players: usize,
    setup: impl FnOnce(&mut TestGame) -> Vec<Entity>,
) -> Option<usize> {
    supported(name);
    let mut t = TestGame::new(players);
    let targets = setup(&mut t);
    let s = t.hand(P0, name);
    cost_to_cast(&mut t, P0, s, &targets)
}

/// A target for spells that target a creature: P1's Grizzly Bears.
fn p1_bears(t: &mut TestGame) -> Vec<Entity> {
    vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))]
}

/// A target for counterspells: P1's Shock targeting P0.
fn p1_spell(t: &mut TestGame) -> Vec<Entity> {
    vec![Entity::Object(p1_shocks(t, Entity::Player(P0)))]
}

#[test]
fn a_reduction_for_controlling_one_applies_only_once_with_several() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Cactarantula",
        "Cactarantula still costs only {1} less to cast if you control multiple Deserts."
    );
    assert_eq!(cost_of("Cactarantula", 2, |_| vec![]), Some(6));
    let two_deserts = |t: &mut TestGame| {
        t.lands(P0, "Desert of the Indomitable", 2);
        vec![]
    };
    assert_eq!(cost_of("Cactarantula", 2, two_deserts), Some(5));

    ruling!(
        "Chill of the Grave",
        "It doesn't cost {1} less for each Zombie you control. If you control more than one Zombie, the spell still costs {1} less to cast."
    );
    assert_eq!(cost_of("Chill of the Grave", 2, p1_bears), Some(3));
    let zombies = |t: &mut TestGame| {
        t.battlefield(P0, "Gravedigger");
        t.battlefield(P0, "Gravedigger");
        p1_bears(t)
    };
    assert_eq!(cost_of("Chill of the Grave", 2, zombies), Some(2));

    ruling!(
        "Academy Journeymage",
        "If you control more than one Wizard, Academy Journeymage's cost is reduced by only {1}."
    );
    assert_eq!(cost_of("Academy Journeymage", 2, |_| vec![]), Some(5));
    let wizards = |t: &mut TestGame| {
        t.battlefield(P0, "Youthful Scholar");
        t.battlefield(P0, "Youthful Scholar");
        t.battlefield(P0, "Youthful Scholar");
        vec![]
    };
    assert_eq!(cost_of("Academy Journeymage", 2, wizards), Some(4));

    ruling!(
        "Igneous Elemental",
        "Igneous Elemental costs only {2} less to cast if you have more than one land card in your graveyard."
    );
    assert_eq!(cost_of("Igneous Elemental", 2, |_| vec![]), Some(6));
    let lands = |t: &mut TestGame| {
        t.graveyard(P0, "Forest");
        t.graveyard(P0, "Mountain");
        t.graveyard(P0, "Mountain");
        vec![]
    };
    assert_eq!(cost_of("Igneous Elemental", 2, lands), Some(4));

    ruling!(
        "Winged Words",
        "Winged Words costs only {1} less to cast if you control multiple creatures with flying."
    );
    assert_eq!(cost_of("Winged Words", 2, |_| vec![]), Some(3));
    let flyers = |t: &mut TestGame| {
        t.battlefield(P0, "Serra Angel");
        t.battlefield(P0, "Leonin Skyhunter");
        vec![]
    };
    assert_eq!(cost_of("Winged Words", 2, flyers), Some(2));
}

#[test]
fn geistlight_snare_counts_one_permanent_for_both_reductions() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Geistlight Snare",
        "It also costs {2} less if you control one permanent that's both an enchantment and a Spirit."
    );
    assert_eq!(cost_of("Geistlight Snare", 2, p1_spell), Some(3));
    // Kami of Ancient Law: a Spirit creature; Glorious Anthem: an enchantment.
    let spirit = |t: &mut TestGame| {
        t.battlefield(P0, "Kami of Ancient Law");
        p1_spell(t)
    };
    assert_eq!(cost_of("Geistlight Snare", 2, spirit), Some(2));
    let both = |t: &mut TestGame| {
        t.battlefield(P0, "Kami of Ancient Law");
        t.battlefield(P0, "Glorious Anthem");
        p1_spell(t)
    };
    assert_eq!(cost_of("Geistlight Snare", 2, both), Some(1));
    // Hopeful Eidolon: an Enchantment Creature — Spirit.
    let one = |t: &mut TestGame| {
        t.battlefield(P0, "Hopeful Eidolon");
        p1_spell(t)
    };
    assert_eq!(cost_of("Geistlight Snare", 2, one), Some(1));
}

#[test]
fn hagra_mauling_needs_one_opponent_without_basics_whoever_controls_the_target() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Hagra Mauling // Hagra Broodpit",
        "Hagra Mauling's first ability reduces the cost by only {1}, even if you have two opponents with no basic lands."
    );
    ruling!(
        "Hagra Mauling // Hagra Broodpit",
        "The opponent who controls no basic lands doesn't have to be the controller of the target creature"
    );
    // Both opponents control basic lands: full cost.
    let both_basics = |t: &mut TestGame| {
        t.lands(P1, "Swamp", 1);
        t.lands(P2, "Forest", 1);
        p1_bears(t)
    };
    assert_eq!(
        cost_of("Hagra Mauling // Hagra Broodpit", 3, both_basics),
        Some(4)
    );
    // Neither controls one: still only {1} less.
    assert_eq!(
        cost_of("Hagra Mauling // Hagra Broodpit", 3, p1_bears),
        Some(3)
    );
    // P1 (the target's controller) controls a basic land, P2 doesn't.
    let one_basic = |t: &mut TestGame| {
        t.lands(P1, "Swamp", 1);
        t.lands(P2, "Desert of the Indomitable", 1);
        p1_bears(t)
    };
    assert_eq!(
        cost_of("Hagra Mauling // Hagra Broodpit", 3, one_basic),
        Some(3)
    );
}

/// P1 and P2 each cast two Opts (P1's last one stays on the stack as the target).
fn opponents_cast_two_spells_each(t: &mut TestGame) -> Vec<Entity> {
    let mut last = None;
    for p in [P1, P2] {
        for _ in 0..2 {
            let o = t.hand(p, "Opt");
            t.g.players[p.idx()].mana_pool.add_type(ManaType::U, 1);
            last = Some(t.cast(p, o).go());
        }
    }
    vec![Entity::Object(last.unwrap())]
}

#[test]
fn ertais_scorn_costs_u_less_only_once() {
    cr!("601.2f", "118.7");
    ruling!(
        "Ertai's Scorn",
        "If multiple opponents have each cast two or more spells this turn, Ertai’s Scorn costs only {U} less to cast"
    );
    supported("Opt");
    assert_eq!(cost_of("Ertai's Scorn", 3, p1_spell), Some(3));
    assert_eq!(
        cost_of("Ertai's Scorn", 3, opponents_cast_two_spells_each),
        Some(2)
    );
}

#[test]
fn wolfkin_outcast_costs_less_even_at_night() {
    cr!("601.2f", "731.1");
    ruling!(
        "Wolfkin Outcast // Wedding Crasher",
        "Wolfkin Outcast costs {2} less to cast if you control a Wolf or Werewolf even if it's currently night."
    );
    let night_with = |what: Option<&'static str>| {
        move |t: &mut TestGame| {
            t.g.set_day(false);
            if let Some(w) = what {
                t.battlefield(P0, w);
            }
            vec![]
        }
    };
    let name = "Wolfkin Outcast // Wedding Crasher";
    assert_eq!(cost_of(name, 2, night_with(None)), Some(6));
    assert_eq!(cost_of(name, 2, night_with(Some("Watchwolf"))), Some(4));
    // Lambholt Elder: a Human Werewolf.
    assert_eq!(cost_of(name, 2, night_with(Some("Lambholt Elder // Silverpelt Werewolf"))), Some(4));
}

// ---------------------------------------------------------------------------------------
// Reductions can't reduce the colored part of a cost (CR 118.7a).
// ---------------------------------------------------------------------------------------

/// The mana P0 pays for `name` with `colored` mana of type `ty` only (plus generic in
/// other types), or `None` if it can't be cast with no mana of that type.
fn needs_colored(t: &mut TestGame, name: &str, ty: ManaType, n: u32) {
    // With plenty of mana but none of `ty`, it can't be cast.
    let s = t.hand(P0, name);
    tap_lands(t, P0);
    for other in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G, ManaType::C] {
        if other != ty {
            t.g.players[0].mana_pool.add_type(other, 10);
        }
    }
    t.g.players[0].mana_pool.add_type(ty, n - 1);
    assert!(t.cast(P0, s).try_go().is_err(), "{name}: cast without its colored mana");
    // With exactly `n` of it and nothing else, it can be.
    t.g.players[0].mana_pool = Default::default();
    t.g.players[0].mana_pool.add_type(ty, n);
    t.cast(P0, s).go();
    assert_eq!(pool(t, P0), 0);
}

#[test]
fn ghalta_cant_cost_less_than_gg() {
    cr!("601.2f", "118.7a");
    ruling!("Ghalta, Primal Hunger", "Ghalta's first ability can't reduce its cost below {G}{G}.");
    supported("Ghalta, Primal Hunger");
    let mut t = TestGame::new(2);
    // Total power 24.
    for _ in 0..4 {
        t.battlefield(P0, "Craw Wurm");
    }
    needs_colored(&mut t, "Ghalta, Primal Hunger", ManaType::G, 2);
}

#[test]
fn primeval_protector_cant_cost_less_than_g() {
    cr!("601.2f", "118.7a");
    ruling!("Primeval Protector", "Primeval Protector's ability can't reduce its cost below {G}.");
    supported("Primeval Protector");
    let mut t = TestGame::new(2);
    bears(&mut t, P1, 13);
    needs_colored(&mut t, "Primeval Protector", ManaType::G, 1);
}

#[test]
fn the_lord_of_the_eagles_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "The Lord of the Eagles",
        "The cost reduction ability reduces only the generic mana in the cost to cast The Lord of the Eagles. The colored mana must still be paid."
    );
    supported("The Lord of the Eagles");
    let mut t = TestGame::new(2);
    // Total power of creatures with flying: 20.
    for _ in 0..5 {
        t.battlefield(P0, "Serra Angel");
    }
    needs_colored(&mut t, "The Lord of the Eagles", ManaType::U, 2);
}

#[test]
fn ghalta_counts_negative_power() {
    cr!("601.2f", "613.4c");
    ruling!(
        "Ghalta, Primal Hunger",
        "If a creature's power is somehow less than 0, it subtracts from the total power of your other creatures. If the total power of your creatures is 0 or less, Ghalta's cost remains {10}{G}{G}."
    );
    // Wall of Stone (0/8) with Disfigure twice: -4/4.
    let wall_minus_four = |t: &mut TestGame| {
        let wall = t.battlefield(P0, "Wall of Stone");
        for _ in 0..2 {
            let d = t.hand(P0, "Disfigure");
            t.g.players[0].mana_pool.add_type(ManaType::B, 1);
            t.cast(P0, d).target(wall).go();
            t.resolve_all();
        }
        assert_eq!(t.pt(wall), (-4, 4));
        wall
    };
    // Three Bears and the wall: total power 2.
    let total_two = |t: &mut TestGame| {
        bears(t, P0, 3);
        wall_minus_four(t);
        vec![]
    };
    assert_eq!(cost_of("Ghalta, Primal Hunger", 2, total_two), Some(10));
    // One Bears and the wall: total power -2.
    let total_negative = |t: &mut TestGame| {
        bears(t, P0, 1);
        wall_minus_four(t);
        vec![]
    };
    assert_eq!(cost_of("Ghalta, Primal Hunger", 2, total_negative), Some(12));
}

// ---------------------------------------------------------------------------------------
// Torgaar: "As an additional cost to cast this spell, you may sacrifice any number of
// creatures. This spell costs {2} less to cast for each creature sacrificed this way."
// ---------------------------------------------------------------------------------------

#[test]
fn torgaar_can_sacrifice_more_creatures_than_reduce_its_cost() {
    cr!("601.2b", "601.2f", "118.7a");
    ruling!(
        "Torgaar, Famine Incarnate",
        "Torgaar's first ability can't reduce its cost below {B}{B}. You can sacrifice any number of creatures, even if they won't reduce Torgaar's cost any further."
    );
    supported("Torgaar, Famine Incarnate");
    let mut t = TestGame::new(2);
    let mine = bears(&mut t, P0, 5);
    let torgaar = t.hand(P0, "Torgaar, Famine Incarnate");
    // Five creatures sacrificed: {10} of reduction for a {6} generic part.
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Number(5));
    // No target for its enters trigger ("up to one").
    t.g.players[0].mana_pool.add_type(ManaType::B, 2);
    t.cast(P0, torgaar).go();
    assert_eq!(pool(&t, P0), 0);
    assert!(mine.iter().all(|b| !t.on_battlefield(*b)));
    assert_eq!(t.graveyard_size(P0), 5);
    assert_eq!(t.zone(torgaar), Zone::Stack);
}

// ---------------------------------------------------------------------------------------
// The mana value is that of the mana cost, whatever the total cost (CR 202.3, 601.2f).
// ---------------------------------------------------------------------------------------

/// P0 casts `name` (set up by `setup`, which returns its targets) for `expected` mana,
/// less than its mana cost; its mana value on the stack is still the printed one, and so
/// is that of the permanent it becomes.
fn mana_value_unchanged(
    name: &str,
    expected: usize,
    setup: impl FnOnce(&mut TestGame) -> Vec<Entity>,
) {
    supported(name);
    let mut t = TestGame::new(2);
    let targets = setup(&mut t);
    let s = t.hand(P0, name);
    let mv = printed_mana_value(name);
    assert_eq!(cost_to_cast(&mut t, P0, s, &targets), Some(expected), "{name}");
    assert!((expected as u32) < mv, "{name}: not reduced");
    let spell = t.g.current(s);
    assert_eq!(t.g.mana_value_of(spell), mv, "{name} on the stack");
    t.resolve_all();
    let now = t.g.current(s);
    if t.g.obj(now).zone == Zone::Battlefield {
        assert_eq!(t.g.mana_value_of(now), mv, "{name} on the battlefield");
    }
}

/// A tapped creature of P1's.
fn tapped_p1_creature(t: &mut TestGame) -> Vec<Entity> {
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[b.0 as usize].tapped = true;
    vec![Entity::Object(b)]
}

#[test]
fn reduced_spells_keep_their_mana_value() {
    cr!("202.3", "601.2f");
    ruling!(
        "Fungal Colossus",
        "Fungal Colossus’s mana value doesn’t change no matter what the number of differently named lands you control is."
    );
    mana_value_unchanged("Fungal Colossus", 4, |t| {
        for l in ["Forest", "Island", "Swamp"] {
            t.lands(P0, l, 1);
        }
        vec![]
    });
    ruling!("Avatar of Woe", "The mana value of this card is still 8, even if you only pay {B}{B} to cast it.");
    mana_value_unchanged("Avatar of Woe", 2, |t| {
        for i in 0..10 {
            t.graveyard(if i % 2 == 0 { P0 } else { P1 }, "Grizzly Bears");
        }
        vec![]
    });
    ruling!("Avatar of Might", "The mana value of this card is still 8, even if you only pay {G}{G} to cast it.");
    mana_value_unchanged("Avatar of Might", 2, |t| {
        bears(t, P1, 4);
        vec![]
    });
    ruling!("Avatar of Will", "The mana value of this card is still 8, even if you only pay {U}{U} to cast it.");
    mana_value_unchanged("Avatar of Will", 2, |_| vec![]);
    ruling!(
        "Ghalta, Primal Hunger",
        "Ghalta's mana value remains unchanged, no matter what the total cost to cast it was."
    );
    mana_value_unchanged("Ghalta, Primal Hunger", 6, |t| {
        bears(t, P0, 3);
        vec![]
    });
    ruling!(
        "Ajani's Response",
        "apply any cost reductions (such as that of Ajani's Response's first ability). The mana value of the spell is determined by only its mana cost"
    );
    mana_value_unchanged("Ajani's Response", 2, tapped_p1_creature);
    ruling!(
        "Grounded for Life",
        "apply any cost reductions (such as that of Grounded for Life's first ability). The mana value of the spell remains unchanged"
    );
    mana_value_unchanged("Grounded for Life", 2, tapped_p1_creature);
    ruling!(
        "Mortality Spear",
        "Mortality Spear's mana value is always 4, no matter what the total cost to cast it was."
    );
    mana_value_unchanged("Mortality Spear", 2, |t| {
        t.g.gain_life(P0, 2);
        vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))]
    });
    ruling!(
        "Of One Mind",
        "apply any cost reductions (such as that of Of One Mind). The mana value of the spell remains unchanged"
    );
    mana_value_unchanged("Of One Mind", 1, |t| {
        t.battlefield(P0, "Order of the Sacred Bell");
        t.battlefield(P0, "Grizzly Bears");
        vec![]
    });
    ruling!(
        "Orysa, Tide Choreographer",
        "apply any cost reductions (such as that of Orysa's first ability). The mana value of the spell is determined by only its mana cost"
    );
    mana_value_unchanged("Orysa, Tide Choreographer", 2, |t| {
        t.battlefield(P0, "Wall of Stone");
        t.battlefield(P0, "Grizzly Bears");
        vec![]
    });
    ruling!(
        "Swampsnare Trap",
        "apply any cost reductions (such as that of Swampsnare Trap). The mana value of the spell is determined only by its mana cost"
    );
    mana_value_unchanged("Swampsnare Trap", 2, |t| {
        vec![Entity::Object(t.battlefield(P1, "Serra Angel"))]
    });
    ruling!(
        "Tentative Connection",
        "apply any cost reductions (such as that of Tentative Connection). The mana value of the spell remains unchanged"
    );
    mana_value_unchanged("Tentative Connection", 1, |t| {
        t.battlefield(P0, "Alley Strangler");
        p1_bears(t)
    });
    ruling!(
        "The Lord of the Eagles",
        "apply any cost reductions (such as that of The Lord of the Eagles). The mana value of the spell remains unchanged"
    );
    mana_value_unchanged("The Lord of the Eagles", 5, |t| {
        t.battlefield(P0, "Serra Angel");
        vec![]
    });
    ruling!(
        "Wilt in the Heat",
        "apply any cost reductions (such as that of Wilt in the Heat's first ability). The mana value of the spell is determined by only its mana cost"
    );
    mana_value_unchanged("Wilt in the Heat", 2, |t| {
        let g = t.graveyard(P0, "Grizzly Bears");
        bounce(t, g);
        p1_bears(t)
    });
    ruling!(
        "Mage Duel",
        "Mage Duel’s mana value is always 3, no matter what the total cost to cast it was."
    );
    mana_value_unchanged("Mage Duel", 1, |t| {
        let mine = t.battlefield(P0, "Grizzly Bears");
        let opt = t.hand(P0, "Opt");
        t.g.players[0].mana_pool.add_type(ManaType::U, 1);
        t.cast(P0, opt).go();
        t.resolve_all();
        vec![Entity::Object(mine), p1_bears(t)[0]]
    });
    ruling!(
        "Mascot Interception",
        "Mascot Interception’s mana value is always 4, no matter what the total cost to cast it was."
    );
    mana_value_unchanged("Mascot Interception", 1, |t| {
        vec![Entity::Object(crate::r_p023_common::wolf_token(t, P1))]
    });
}

#[test]
fn fast_forward_keeps_its_mana_value() {
    cr!("202.3", "601.2f");
    ruling!(
        "Fast Forward",
        "apply any cost reductions (such as that of Fast Forward's first ability). The mana value of the spell remains unchanged"
    );
    supported("Fast Forward");
    // P0 attacks both opponents, then casts Fast Forward in the second main phase.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    let s = t.hand(P0, "Fast Forward");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(3));
    assert_eq!(t.g.mana_value_of(t.g.current(s)), 5);
}
