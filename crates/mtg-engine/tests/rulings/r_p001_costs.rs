//! Rulings batch P001 — cost reductions that reduce only generic mana (CR 118.7a), the
//! order of the total cost (CR 601.2f), the mana value of a spell whose cost was reduced
//! (CR 202.3), what counts toward "for each [card] in your graveyard", and casting a
//! spell as one uninterrupted process (CR 601.2).

use crate::r_p001_common::*;
use crate::r_p006_common::choices_since;
use crate::r_s01_common::{give_mana_for, supported};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// A cost-reduction case: the card, what gives the reduction, the lands that pay exactly
/// the colored mana left, and the card's mana value.
struct Reduced {
    card: &'static str,
    setup: fn(&mut TestGame),
    colored: &'static [&'static str],
    mana_value: u32,
}

fn creatures_6(t: &mut TestGame) {
    t.lands(P0, "Grizzly Bears", 6);
}
fn enchantments_7(t: &mut TestGame) {
    t.lands(P0, "Tainted Remedy", 7);
}
fn spells_in_graveyard_9(t: &mut TestGame) {
    graveyard_n(t, P0, "Shock", 9);
}
fn creature_cards_8(t: &mut TestGame) {
    graveyard_n(t, P0, "Grizzly Bears", 8);
}
fn artifact_creature_cards_5(t: &mut TestGame) {
    graveyard_n(t, P0, "Ornithopter", 5);
}

const COLORED_ONLY: &[Reduced] = &[
    Reduced {
        card: "Argivian Phalanx",
        setup: creatures_6,
        colored: &["Plains"],
        mana_value: 6,
    },
    Reduced {
        card: "Brine Giant",
        setup: enchantments_7,
        colored: &["Island"],
        mana_value: 7,
    },
    Reduced {
        card: "Cryptic Serpent",
        setup: spells_in_graveyard_9,
        colored: &["Island", "Island"],
        mana_value: 7,
    },
    Reduced {
        card: "Ghoultree",
        setup: creature_cards_8,
        colored: &["Forest"],
        mana_value: 8,
    },
    Reduced {
        card: "Chitin Gravestalker",
        setup: artifact_creature_cards_5,
        colored: &["Swamp"],
        mana_value: 6,
    },
    Reduced {
        card: "Writhing Necromass",
        setup: creature_cards_8,
        colored: &["Swamp"],
        mana_value: 7,
    },
];

#[test]
fn cost_reductions_for_each_dont_reduce_colored_mana() {
    cr!("118.7a", "601.2f", "202.3");
    ruling!(
        "Argivian Phalanx",
        "Argivian Phalanx's first ability can't reduce the {W} in its cost."
    );
    ruling!(
        "Brine Giant",
        "The cost reduction ability reduces only the generic mana in Brine Giant's cost. The colored mana must still be paid."
    );
    ruling!(
        "Cryptic Serpent",
        "Cryptic Serpent's ability can't reduce its cost to less than {U}{U}."
    );
    ruling!(
        "Cryptic Serpent",
        "Cryptic Serpent's ability doesn't change its mana cost or mana value. It just reduces the cost to cast the spell."
    );
    ruling!(
        "Ghoultree",
        "Ghoultree’s ability can’t reduce the total cost to cast the spell below {G}."
    );
    ruling!(
        "Chitin Gravestalker",
        "This card’s first ability can’t reduce colored mana in its cost, so the cost can’t be reduced below {B} this way."
    );
    ruling!(
        "Writhing Necromass",
        "Writhing Necromass’s first ability can’t reduce the {B} in its cost."
    );
    for case in COLORED_ONLY {
        supported(case.card);
        // More reduction than the generic mana, and plenty of colorless mana: the colored
        // mana still has to be paid.
        let mut t = TestGame::new(2);
        (case.setup)(&mut t);
        t.lands(P0, "Wastes", 10);
        let card = t.hand(P0, case.card);
        assert!(!castable(&mut t, P0, card), "{}", case.card);
        // Only the colored mana: it's cast for exactly that, and its mana value (in hand
        // and on the stack) is unchanged.
        let mut t = TestGame::new(2);
        (case.setup)(&mut t);
        for land in case.colored {
            t.lands(P0, land, 1);
        }
        let card = t.hand(P0, case.card);
        assert_eq!(mv(&t, card), case.mana_value);
        assert!(castable(&mut t, P0, card), "{}", case.card);
        let spell = t.cast(P0, card).go();
        assert_eq!(untapped_lands(&t, P0), 0);
        assert_eq!(mv(&t, spell), case.mana_value, "{}", case.card);
        t.resolve_all();
        assert!(t.on_battlefield(spell), "{}", case.card);
    }
}

#[test]
fn yavimaya_sojourners_domain_reduction_leaves_the_green_mana() {
    cr!("118.7a", "601.2f", "305.6");
    ruling!(
        "Yavimaya Sojourner",
        "Yavimaya Sojourner’s first ability can’t reduce the {G} in its cost."
    );
    supported("Yavimaya Sojourner");
    // Five basic land types: {7}{G} costs {2}{G}. With the Forest tapped (it still counts
    // for domain), four other mana can't pay it; with the Forest untapped it's cast.
    for forest_untapped in [false, true] {
        let mut t = TestGame::new(2);
        for land in ["Plains", "Island", "Swamp", "Mountain"] {
            t.lands(P0, land, 1);
        }
        let forest = t.battlefield(P0, "Forest");
        if !forest_untapped {
            t.g.tap(forest);
        }
        let card = t.hand(P0, "Yavimaya Sojourner");
        assert_eq!(castable(&mut t, P0, card), forest_untapped);
        if forest_untapped {
            let spell = t.cast(P0, card).go();
            assert!(t.obj_now(forest).tapped);
            assert_eq!(untapped_lands(&t, P0), 2);
            assert_eq!(mv(&t, spell), 8);
        }
    }
}

/// A total-cost case: a card, its reduction (more than its generic mana), the colored
/// lands, and its mana value.
const WITH_SPHERE: &[Reduced] = &[
    Reduced {
        card: "Argivian Phalanx",
        setup: creatures_6,
        colored: &["Plains"],
        mana_value: 6,
    },
    Reduced {
        card: "Brine Giant",
        setup: enchantments_7,
        colored: &["Island"],
        mana_value: 7,
    },
    Reduced {
        card: "Furygale Flocking",
        setup: spells_in_graveyard_9,
        colored: &["Mountain", "Mountain"],
        mana_value: 10,
    },
    Reduced {
        card: "Writhing Necromass",
        setup: creature_cards_8,
        colored: &["Swamp"],
        mana_value: 7,
    },
];

#[test]
fn cost_increases_apply_before_cost_reductions() {
    cr!("601.2f", "202.3");
    ruling!(
        "Argivian Phalanx",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Argivian Phalanx). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    ruling!(
        "Brine Giant",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Brine Giant). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Furygale Flocking",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Furygale Flocking's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    ruling!(
        "Writhing Necromass",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions (such as that of Writhing Necromass). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Sphere of Resistance");
    for case in WITH_SPHERE {
        supported(case.card);
        // Sphere of Resistance's {1} is added first and then reduced away with the rest of
        // the generic mana: only the colored mana is paid.
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Sphere of Resistance");
        (case.setup)(&mut t);
        for land in case.colored {
            t.lands(P0, land, 1);
        }
        let card = t.hand(P0, case.card);
        assert!(castable(&mut t, P0, card), "{}", case.card);
        let spell = t.cast(P0, card).go();
        assert_eq!(untapped_lands(&t, P0), 0);
        assert_eq!(mv(&t, spell), case.mana_value, "{}", case.card);
    }
}

#[test]
fn yavimaya_sojourner_with_a_cost_increase() {
    cr!("601.2f", "202.3");
    ruling!(
        "Yavimaya Sojourner",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions (such as that of Yavimaya Sojourner). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Yavimaya Sojourner");
    // {7}{G} + {1} − 5 = {3}{G}: four of the five lands.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, land, 1);
    }
    let card = t.hand(P0, "Yavimaya Sojourner");
    let spell = t.cast(P0, card).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    assert_eq!(mv(&t, spell), 8);
}

#[test]
fn herald_of_slaanesh_reduces_only_generic_mana() {
    cr!("118.7a", "601.2f");
    ruling!(
        "Herald of Slaanesh",
        "Herald of Slaanesh's first ability only reduces the generic mana component of costs. It cannot reduce the cost of colored mana components."
    );
    supported("Herald of Slaanesh");
    // Another Herald of Slaanesh ({2}{R}, a Demon) costs {R}: one Mountain casts it, any
    // amount of colorless mana doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Slaanesh");
    t.lands(P0, "Wastes", 5);
    let second = t.hand(P0, "Herald of Slaanesh");
    assert!(!castable(&mut t, P0, second));
    let mountain = t.battlefield(P0, "Mountain");
    assert!(castable(&mut t, P0, second));
    t.cast(P0, second).go();
    assert!(t.obj_now(mountain).tapped);
    assert_eq!(untapped_lands(&t, P0), 5);
}

#[test]
fn a_split_card_counts_once_for_cryptic_serpent() {
    cr!("709.4", "118.7a");
    ruling!(
        "Cryptic Serpent",
        "A split card only counts once for Cryptic Serpent's ability, even if it's both an instant and a sorcery."
    );
    supported("Cryptic Serpent");
    // Commit (an instant) // Memory (a sorcery) and three Shocks: four cards, so {5}{U}{U}
    // costs {1}{U}{U}.
    for (islands, ok) in [(2, false), (3, true)] {
        let mut t = TestGame::new(2);
        let split = t.graveyard(P0, "Commit // Memory");
        assert!(t.obj_now(split).is(CardType::Instant) && t.obj_now(split).is(CardType::Sorcery));
        graveyard_n(&mut t, P0, "Shock", 3);
        t.lands(P0, "Island", islands);
        let card = t.hand(P0, "Cryptic Serpent");
        assert_eq!(castable(&mut t, P0, card), ok, "{islands} Islands");
    }
}

#[test]
fn an_artifact_creature_card_counts_once_for_chitin_gravestalker() {
    cr!("118.7a");
    ruling!(
        "Chitin Gravestalker",
        "If a card in your graveyard is an artifact creature card, count it only once when determining the cost reduction for this card’s first ability."
    );
    supported("Chitin Gravestalker");
    // Three Ornithopters: {5}{B} costs {2}{B} (not {B}).
    for (wastes, ok) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        graveyard_n(&mut t, P0, "Ornithopter", 3);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Wastes", wastes);
        let card = t.hand(P0, "Chitin Gravestalker");
        assert_eq!(castable(&mut t, P0, card), ok, "{wastes} Wastes");
    }
}

#[test]
fn thwart_the_graves_first_target_neednt_be_a_party_type() {
    cr!("115.1", "700.8");
    ruling!(
        "Thwart the Grave",
        "Thwart the Grave's first target doesn't have to be one of the party creature types."
    );
    supported("Thwart the Grave");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let cleric = t.graveyard(P0, "Disciple of Law");
    give_mana_for(&mut t, P0, "Thwart the Grave");
    let card = t.hand(P0, "Thwart the Grave");
    t.cast(P0, card).target(bears).target(cleric).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(cleric));
}

#[test]
fn mycosynth_golems_reduction_is_locked_in_before_it_is_sacrificed_for_mana() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Mycosynth Golem",
        "If Mycosynth Golem is sacrificed as part of the cost to cast a spell, the cost reduction will already be locked in, and the cost won’t be increased."
    );
    supported("Mycosynth Golem");
    supported("Ashnod's Altar");
    supported("Juggernaut");
    // Juggernaut ({4}) has affinity for artifacts from the Golem: two artifacts make it
    // {2}. The Golem is then sacrificed to Ashnod's Altar for {C}{C}, which pays it.
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Mycosynth Golem");
    t.battlefield(P0, "Ashnod's Altar");
    let card = t.hand(P0, "Juggernaut");
    t.answer_choose(P0, &[obj(golem)]);
    let spell = t.cast(P0, card).go();
    assert!(t.in_graveyard(P0, "Mycosynth Golem"));
    assert!(t.g.player(P0).mana_pool.is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(spell));
}

/// Casts `name` for P0 (with the mana for its cost) and returns the decisions asked of
/// each player while it was being cast.
fn cast_and_watch(t: &mut TestGame, name: &str, targets: &[Entity]) -> (usize, usize) {
    let card = t.hand(P0, name);
    let from = t.asked().len();
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, card).go();
    let asked = t.asked()[from..].to_vec();
    (
        asked.iter().filter(|(p, _)| *p == P0).count(),
        asked.iter().filter(|(p, _)| *p != P0).count(),
    )
}

#[test]
fn nobody_can_act_while_a_spell_is_being_cast() {
    cr!("601.2", "601.2f", "601.2h", "117.1");
    ruling!(
        "Bone Splinters",
        "Once you begin to cast Bone Splinters, no player may take actions until you’re done. Notably, opponents can’t try to remove the creature you wish to sacrifice."
    );
    ruling!(
        "Demon of Catastrophes",
        "Players can respond only after Demon of Catastrophes has been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to prevent you from casting this spell."
    );
    ruling!(
        "Brine Giant",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to change by how much Brine Giant's cost is reduced."
    );
    ruling!(
        "Ghoultree",
        "Once you begin to cast Ghoultree, no player may take actions until you’re done. Notably, opponents can’t try to remove creature cards from your graveyard to change Ghoultree’s cost."
    );
    // Sacrificing: the creature is gone by the time anyone gets priority.
    for name in ["Bone Splinters", "Demon of Catastrophes"] {
        supported(name);
        let mut t = TestGame::new(2);
        let fodder = t.battlefield(P0, "Grizzly Bears");
        let victim = t.battlefield(P1, "Hill Giant");
        give_mana_for(&mut t, P0, name);
        t.answer_choose(P0, &[obj(fodder)]);
        let targets: &[Entity] = if name == "Bone Splinters" {
            &[obj(victim)]
        } else {
            &[]
        };
        let (mine, theirs) = cast_and_watch(&mut t, name, targets);
        assert!(mine > 0);
        assert_eq!(theirs, 0, "{name}: P1 was asked something while it was cast");
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        if name == "Bone Splinters" {
            assert!(t.in_graveyard(P1, "Hill Giant"));
        } else {
            assert_eq!(t.named_on_battlefield(name).len(), 1);
        }
    }
    // Cost reductions: the number of enchantments / creature cards is what it was as the
    // total cost was determined; P1 can't act before it's paid.
    for name in ["Brine Giant", "Ghoultree"] {
        supported(name);
        let mut t = TestGame::new(2);
        if name == "Brine Giant" {
            t.lands(P0, "Tainted Remedy", 3);
            t.lands(P0, "Island", 4);
        } else {
            graveyard_n(&mut t, P0, "Grizzly Bears", 3);
            t.lands(P0, "Forest", 5);
        }
        let (_, theirs) = cast_and_watch(&mut t, name, &[]);
        assert_eq!(theirs, 0, "{name}: P1 was asked something while it was cast");
        assert_eq!(untapped_lands(&t, P0), 0);
        assert_eq!(t.stack_len(), 1);
    }
}

#[test]
fn the_sacrifice_is_exactly_one_creature() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Demon of Catastrophes",
        "You must sacrifice exactly one creature to cast Demon of Catastrophes; you can’t cast it without sacrificing a creature, and you can’t sacrifice additional creatures."
    );
    ruling!(
        "Bone Splinters",
        "You must sacrifice exactly one creature to cast this spell; you can’t cast it without sacrificing a creature, and you can’t sacrifice additional creatures."
    );
    for name in [
        "Demon of Catastrophes",
        "Bone Splinters",
        "Blood for Bones",
        "Collateral Damage",
    ] {
        supported(name);
        let targets: Vec<Entity> = match name {
            "Bone Splinters" | "Collateral Damage" => vec![pl(P1)],
            _ => vec![],
        };
        // Without a creature, it can't be cast.
        let mut t = TestGame::new(2);
        give_mana_for(&mut t, P0, name);
        let card = t.hand(P0, name);
        let victim = t.battlefield(P1, "Hill Giant");
        t.g.recompute();
        assert!(!castable(&mut t, P0, card), "{name}");
        // With two, exactly one is chosen (an answer naming both isn't taken).
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        assert!(castable(&mut t, P0, card), "{name}");
        let from = t.asked().len();
        t.answer_choose(P0, &[obj(a), obj(b)]);
        let tgt = if name == "Bone Splinters" {
            vec![obj(victim)]
        } else {
            targets
        };
        for e in &tgt {
            t.answer_targets(P0, &[*e]);
        }
        t.cast(P0, card).go();
        let sac = choices_since(&t, P0, from);
        assert!(sac.iter().any(|(c, min, max)| c.contains(&obj(a)) && *min == 1 && *max == 1));
        let gone = [a, b].iter().filter(|c| !t.on_battlefield(**c)).count();
        assert_eq!(gone, 1, "{name}");
    }
}
