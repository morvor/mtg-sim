//! Rulings batch P174 — costs that tap untapped permanents you control (not with {T}):
//! summoning sickness doesn't restrict the permanents tapped (CR 302.6), only a {T} in the
//! same cost restricts the source itself (CR 302.6, 107.5); such costs can include the
//! source unless they say "another"; abilities that work only from the graveyard
//! (CR 113.6m, 602.1); additional costs of spells (CR 601.2b, 601.2h).

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s24_common::tapped;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[derive(Clone, Copy, PartialEq)]
enum Src {
    /// On the battlefield; `true`: it's in the cost too (summoning sick, tapped by it).
    Battlefield { taps_itself: bool },
    Graveyard,
}

struct TapCase {
    card: &'static str,
    /// A substring of the activated ability's text.
    needle: &'static str,
    src: Src,
    /// Permanents tapped for the cost: they all came under P0's control this turn.
    helpers: &'static [&'static str],
    lands: &'static [&'static str],
    /// The cost has {T}: the source itself must have been under P0's control since the
    /// turn began.
    tap_symbol: bool,
    /// Extra setup; returns the ability's targets (one per target slot).
    setup: fn(&mut TestGame) -> Vec<Entity>,
    /// The ability's outcome (after everything resolved).
    check: fn(&mut TestGame, ObjectId) -> bool,
}

fn no_setup(_: &mut TestGame) -> Vec<Entity> {
    vec![]
}

fn opp_bears(t: &mut TestGame) -> Vec<Entity> {
    vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))]
}

/// The source can't be blocked: blocked by an opponent's Bears, it deals its combat damage
/// to the player anyway.
fn unblockable(t: &mut TestGame, src: ObjectId) -> bool {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let power = t.pt(src).0;
    crate::r_s01_common::attack_with(t, &[(src, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(t, P1, &[(bears, src)]);
    t.life(P1) == 20 - power
}

fn mana_added(t: &mut TestGame, _: ObjectId) -> bool {
    t.g.player(P0).mana_pool.total() > 0
}

/// Sets up the case with the source summoning sick or not and activates the ability.
fn attempt(c: &TapCase, source_sick: bool) -> (TestGame, ObjectId, Vec<ObjectId>, bool) {
    supported(c.card);
    let mut t = TestGame::new(2);
    let src = match c.src {
        Src::Graveyard => t.graveyard(P0, c.card),
        Src::Battlefield { .. } if source_sick => t.battlefield_sick(P0, c.card),
        Src::Battlefield { .. } => t.battlefield(P0, c.card),
    };
    let helpers: Vec<ObjectId> = c
        .helpers
        .iter()
        .map(|h| t.battlefield_sick(P0, h))
        .collect();
    for l in c.lands {
        t.battlefield(P0, l);
    }
    let targets = (c.setup)(&mut t);
    for e in &targets {
        t.answer_targets(P0, &[*e]);
    }
    let ok = activate_containing(&mut t, P0, src, c.needle).is_ok();
    (t, src, helpers, ok)
}

/// The cost can be paid by tapping permanents that came under P0's control this turn
/// (and the source too when it's in the cost); with {T} in the cost, a summoning-sick
/// source can't activate the ability.
fn run(c: TapCase) {
    let taps_itself = c.src == Src::Battlefield { taps_itself: true };
    let (mut t, src, helpers, ok) = attempt(&c, taps_itself);
    assert!(ok, "{}: activation failed\n{}", c.card, t.dump_log());
    t.resolve_all();
    for h in &helpers {
        assert!(tapped(&t, *h), "{}: helper untapped", c.card);
    }
    if taps_itself {
        assert!(tapped(&t, src), "{}: source untapped", c.card);
    }
    assert!((c.check)(&mut t, src), "{}: no effect\n{}", c.card, t.dump_log());
    if c.tap_symbol {
        let (_, _, _, ok) = attempt(&c, true);
        assert!(!ok, "{}: summoning-sick source activated {{T}}", c.card);
    }
}

// --- Any untapped permanents, even ones that came under your control this turn -----------

#[test]
fn kumena_taps_another_new_merfolk() {
    cr!("302.6");
    ruling!(
        "Kumena, Tyrant of Orazca",
        "To activate Kumena's abilities, you may tap any untapped Merfolk you control, including one you haven't controlled continuously since the beginning of your most recent turn."
    );
    run(TapCase {
        card: "Kumena, Tyrant of Orazca",
        needle: "Tap another untapped Merfolk",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Maritime Guard"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: unblockable,
    });
    // The second ability can tap Kumena itself, summoning sick.
    run(TapCase {
        card: "Kumena, Tyrant of Orazca",
        needle: "Tap three untapped Merfolk",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Maritime Guard", "Vodalian Soldiers"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.hand_size(P0) == 1,
    });
}

#[test]
fn veteran_warleader_taps_another_new_ally() {
    cr!("302.6");
    ruling!(
        "Veteran Warleader",
        "To activate the last ability, you can tap any other untapped Ally you control, including one that hasn't been under your control since the beginning of your most recent turn."
    );
    run(TapCase {
        card: "Veteran Warleader",
        needle: "Tap another untapped Ally",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Expedition Envoy"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, src| {
            t.g.recompute();
            [
                KeywordKind::FirstStrike,
                KeywordKind::Vigilance,
                KeywordKind::Trample,
            ]
            .iter()
            .any(|k| has_kw(t, src, *k))
        },
    });
}

#[test]
fn deathless_ancient_taps_new_vampires_from_the_graveyard() {
    cr!("302.6", "113.6m");
    ruling!(
        "Deathless Ancient",
        "To activate the last ability, you may tap any untapped Vampires you control, including ones you haven't controlled continuously since the beginning of your most recent turn."
    );
    run(TapCase {
        card: "Deathless Ancient",
        needle: "Tap three untapped Vampires",
        src: Src::Graveyard,
        helpers: &["Vampire Noble", "Vampire Noble", "Highborn Vampire"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.in_hand(P0, "Deathless Ancient"),
    });
}

#[test]
fn skirsdag_high_priest_taps_two_new_creatures() {
    cr!("302.6");
    ruling!(
        "Skirsdag High Priest",
        "the two other creatures you tap to activate its ability aren't required to have been under your control continuously"
    );
    run(TapCase {
        card: "Skirsdag High Priest",
        needle: "Tap two untapped creatures",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears", "Grizzly Bears"],
        lands: &[],
        tap_symbol: true,
        setup: |t| {
            // Morbid: a creature died this turn.
            let b = t.battlefield(P1, "Grizzly Bears");
            crate::r_s02_common::destroy(t, b);
            vec![]
        },
        check: |t, _| crate::r_s01_common::tokens(t, P0).len() == 1,
    });
}

#[test]
fn myr_turbine_taps_new_myr() {
    cr!("302.6");
    ruling!(
        "Myr Turbine",
        "You can tap a Myr that hasn’t been under your control since your most recent turn began to pay for the cost of the second activated ability."
    );
    run(TapCase {
        card: "Myr Turbine",
        needle: "Tap five untapped Myr",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Alpha Myr"; 5],
        lands: &[],
        tap_symbol: false,
        setup: |t| {
            t.library_top(P0, "Omega Myr");
            vec![]
        },
        check: |t, _| !t.named_on_battlefield("Omega Myr").is_empty(),
    });
}

#[test]
fn springleaf_drum_taps_a_new_creature() {
    cr!("302.6", "605.1a");
    ruling!(
        "Springleaf Drum",
        "You can tap a creature that hasn't been under your control since your most recent turn began to activate Springleaf Drum's ability."
    );
    run(TapCase {
        card: "Springleaf Drum",
        needle: "Tap an untapped creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: mana_added,
    });
}

#[test]
fn black_oak_of_odunos_taps_a_new_creature() {
    cr!("302.6");
    ruling!(
        "Black Oak of Odunos",
        "You can tap a creature that hasn't been under your control since your most recent turn began to activate the last ability."
    );
    run(TapCase {
        card: "Black Oak of Odunos",
        needle: "Tap another untapped creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears"],
        lands: &["Swamp"],
        tap_symbol: false,
        setup: no_setup,
        check: |t, src| t.pt(src) == (1, 6),
    });
}

#[test]
fn shadow_stinger_and_sure_footed_infiltrator_tap_another_new_rogue() {
    cr!("302.6");
    ruling!(
        "Shadow Stinger",
        "to pay the cost of Shadow Stinger's first ability."
    );
    ruling!(
        "Sure-Footed Infiltrator",
        "to pay the cost of Sure-Footed Infiltrator's first ability."
    );
    run(TapCase {
        card: "Shadow Stinger",
        needle: "Tap another untapped Rogue",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Triton Shorethief"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, src| {
            t.g.recompute();
            has_kw(t, src, KeywordKind::Deathtouch)
        },
    });
    run(TapCase {
        card: "Sure-Footed Infiltrator",
        needle: "Tap another untapped Rogue",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Triton Shorethief"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: unblockable,
    });
}

#[test]
fn gallows_at_willow_hill_taps_three_new_humans() {
    cr!("302.6");
    ruling!(
        "Gallows at Willow Hill",
        "You can tap any three untapped Humans you control, including ones you haven’t controlled continuously"
    );
    run(TapCase {
        card: "Gallows at Willow Hill",
        needle: "Tap three untapped Humans",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Alaborn Trooper", "Alaborn Trooper", "Femeref Scouts"],
        lands: &["Wastes", "Wastes", "Wastes"],
        tap_symbol: false,
        setup: opp_bears,
        check: |t, _| t.in_graveyard(P1, "Grizzly Bears"),
    });
}

#[test]
fn cryptbreaker_taps_new_zombies_including_itself() {
    cr!("302.6");
    ruling!(
        "Cryptbreaker",
        "to pay the cost of Cryptbreaker’s second activated ability. This includes Cryptbreaker itself."
    );
    run(TapCase {
        card: "Cryptbreaker",
        needle: "Tap three untapped Zombies",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Warpath Ghoul", "Those Who Serve"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.hand_size(P0) == 1 && t.life(P0) == 19,
    });
}

#[test]
fn artifacts_and_or_creatures_including_the_source() {
    cr!("302.6");
    ruling!(
        "Adaptive Gemguard",
        "including Adaptive Gemguard itself and other creatures you haven't controlled continuously"
    );
    ruling!(
        "Goldfury Strider",
        "including Goldfury Strider and other creatures you haven't controlled continuously"
    );
    ruling!(
        "Sunshot Militia",
        "to pay the cost of Sunshot Militia's activated ability."
    );
    run(TapCase {
        card: "Adaptive Gemguard",
        needle: "Tap two untapped artifacts",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Ornithopter"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, src| t.counters(src, "+1/+1") == 1,
    });
    run(TapCase {
        card: "Goldfury Strider",
        needle: "Tap two untapped artifacts",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: |t| vec![Entity::Object(t.battlefield(P0, "Llanowar Elves"))],
        check: |t, _| t.pt(t.named_on_battlefield("Llanowar Elves")[0]) == (3, 1),
    });
    run(TapCase {
        card: "Sunshot Militia",
        needle: "Tap two untapped artifacts",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Ornithopter", "Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: |t| {
            // The Militia is tapped already: the two new permanents pay.
            let m = t.named_on_battlefield("Sunshot Militia")[0];
            t.g.objects[m.0 as usize].tapped = true;
            vec![]
        },
        check: |t, _| t.life(P1) == 19,
    });
}

#[test]
fn arachnus_spinner_can_tap_itself() {
    cr!("302.6");
    ruling!(
        "Arachnus Spinner",
        "You can even tap Arachnus Spinner itself to pay this cost."
    );
    run(TapCase {
        card: "Arachnus Spinner",
        needle: "Tap an untapped Spider",
        src: Src::Battlefield { taps_itself: true },
        helpers: &[],
        lands: &[],
        tap_symbol: false,
        setup: |t| {
            t.library_top(P0, "Arachnus Web");
            opp_bears(t)
        },
        check: |t, _| !t.named_on_battlefield("Arachnus Web").is_empty(),
    });
}

#[test]
fn grand_architect_taps_itself_for_mana() {
    cr!("302.6", "605.1a");
    ruling!(
        "Grand Architect",
        "to pay the cost of Grand Architect's mana ability. This includes Grand Architect itself."
    );
    run(TapCase {
        card: "Grand Architect",
        needle: "Tap an untapped blue creature",
        src: Src::Battlefield { taps_itself: true },
        helpers: &[],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.g.player(P0).mana_pool.total() == 2,
    });
}

#[test]
fn opposition_taps_a_new_creature() {
    cr!("302.6");
    ruling!(
        "Opposition",
        "You can tap any untapped creature you control to activate Opposition’s ability, even one you haven’t controlled continuously"
    );
    run(TapCase {
        card: "Opposition",
        needle: "Tap an untapped creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: opp_bears,
        check: |t, _| {
            let theirs = t.named_on_battlefield("Grizzly Bears");
            theirs
                .iter()
                .filter(|b| t.obj_now(**b).controller == P1)
                .all(|b| tapped(t, *b))
        },
    });
}

#[test]
fn cryptic_gateway_taps_new_creatures() {
    cr!("302.6");
    ruling!(
        "Cryptic Gateway",
        "You can tap any untapped creatures you control to activate Cryptic Gateway’s ability, even ones you haven’t controlled continuously"
    );
    run(TapCase {
        card: "Cryptic Gateway",
        needle: "Tap two untapped creatures",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears", "Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: |t| {
            t.hand(P0, "Runeclaw Bear");
            t.answer_yes(P0, true);
            vec![]
        },
        check: |t, _| !t.named_on_battlefield("Runeclaw Bear").is_empty(),
    });
}

#[test]
fn leyline_dowser_taps_a_new_legendary_creature() {
    cr!("302.6");
    ruling!(
        "Leyline Dowser",
        "You can tap any untapped legendary creature you control, including one you haven’t controlled continuously"
    );
    run(TapCase {
        card: "Leyline Dowser",
        needle: "Tap an untapped legendary creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Isamaru, Hound of Konda"],
        lands: &[],
        tap_symbol: false,
        setup: |t| {
            let d = t.named_on_battlefield("Leyline Dowser")[0];
            t.g.objects[d.0 as usize].tapped = true;
            vec![]
        },
        check: |t, src| !tapped(t, src),
    });
}

#[test]
fn dwarven_bloodboiler_can_tap_itself() {
    cr!("302.6");
    ruling!("Dwarven Bloodboiler", "Yes, it can tap itself.");
    run(TapCase {
        card: "Dwarven Bloodboiler",
        needle: "Tap an untapped Dwarf",
        src: Src::Battlefield { taps_itself: true },
        helpers: &[],
        lands: &[],
        tap_symbol: false,
        setup: opp_bears,
        check: |t, _| t.pt(t.named_on_battlefield("Grizzly Bears")[0]) == (4, 2),
    });
}

#[test]
fn tyvar_activates_its_first_ability_while_tapped() {
    cr!("602.5a", "302.6");
    ruling!(
        "Tyvar, the Pummeler",
        "You can activate Tyvar's first ability even if Tyvar is already tapped."
    );
    run(TapCase {
        card: "Tyvar, the Pummeler",
        needle: "Tap another untapped creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: |t| {
            let ty = t.named_on_battlefield("Tyvar, the Pummeler")[0];
            t.g.objects[ty.0 as usize].tapped = true;
            vec![]
        },
        check: |t, src| {
            t.g.recompute();
            has_kw(t, src, KeywordKind::Indestructible)
        },
    });
}

// --- {T} in the cost: the source itself must have been under your control ----------------

#[test]
fn tap_symbol_sources_need_continuous_control_but_the_tapped_creatures_dont() {
    cr!("302.6", "107.5");
    ruling!(
        "Devout Chaplain",
        "to pay the cost of Devout Chaplain’s activated ability. You must have controlled Devout Chaplain continuously"
    );
    ruling!(
        "Chosen of Markov",
        "to pay the cost of Chosen of Markov's activated ability. You must have controlled Chosen of Markov continuously"
    );
    ruling!(
        "Citanul Stalwart",
        "However, you must have controlled Citanul Stalwart continuously"
    );
    ruling!(
        "Selesnya Evangel",
        "You must have controlled Selesnya Evangel continuously"
    );
    ruling!(
        "Jaspera Sentinel",
        "You must have controlled Jaspera Sentinel continuously"
    );
    ruling!("Loam Dryad", "You must have controlled Loam Dryad continuously");
    ruling!(
        "Saruli Caretaker",
        "You must have controlled Saruli Caretaker continuously"
    );
    ruling!(
        "Town Gossipmonger",
        "You must have controlled Town Gossipmonger continuously"
    );
    ruling!(
        "Gene Pollinator",
        "You must have controlled Gene Pollinator continuously"
    );
    let mana = |card: &'static str, helper: &'static [&'static str]| TapCase {
        card,
        needle: "Tap an untapped",
        src: Src::Battlefield { taps_itself: false },
        helpers: helper,
        lands: &[],
        tap_symbol: true,
        setup: no_setup,
        check: mana_added,
    };
    run(mana("Citanul Stalwart", &["Ornithopter"]));
    run(mana("Jaspera Sentinel", &["Grizzly Bears"]));
    run(mana("Loam Dryad", &["Grizzly Bears"]));
    run(mana("Saruli Caretaker", &["Grizzly Bears"]));
    run(mana("Gene Pollinator", &["Grizzly Bears"]));
    run(TapCase {
        card: "Devout Chaplain",
        needle: "Tap two untapped Humans",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Alaborn Trooper", "Femeref Scouts"],
        lands: &[],
        tap_symbol: true,
        setup: |t| vec![Entity::Object(t.battlefield(P1, "Ornithopter"))],
        check: |t, _| t.in_exile("Ornithopter"),
    });
    run(TapCase {
        card: "Selesnya Evangel",
        needle: "Tap an untapped creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Grizzly Bears"],
        lands: &["Forest"],
        tap_symbol: true,
        setup: no_setup,
        check: |t, _| crate::r_s01_common::tokens(t, P0).len() == 1,
    });
    for (card, helper) in [
        ("Chosen of Markov", "Vampire Noble"),
        ("Town Gossipmonger", "Grizzly Bears"),
    ] {
        let c = TapCase {
            card,
            needle: "Transform",
            src: Src::Battlefield { taps_itself: false },
            helpers: if helper == "Vampire Noble" {
                &["Vampire Noble"]
            } else {
                &["Grizzly Bears"]
            },
            lands: &[],
            tap_symbol: true,
            setup: no_setup,
            check: |t, src| t.obj_now(src).face == mtg_engine::object::FaceState::Back,
        };
        run(c);
    }
}

#[test]
fn aryel_taps_new_knights_but_needs_continuous_control_itself() {
    cr!("302.6", "107.5", "107.3");
    ruling!(
        "Aryel, Knight of Windgrace",
        "You can tap any untapped Knights you control, including ones you haven't controlled continuously since the beginning of your most recent turn, to pay the cost of Aryel's last ability."
    );
    for sick in [false, true] {
        let mut t = TestGame::new(2);
        let aryel = if sick {
            t.battlefield_sick(P0, "Aryel, Knight of Windgrace")
        } else {
            t.battlefield(P0, "Aryel, Knight of Windgrace")
        };
        let k1 = t.battlefield_sick(P0, "Knight of New Benalia");
        let k2 = t.battlefield_sick(P0, "Trokin High Guard");
        t.battlefield(P0, "Swamp");
        let victim = t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
        t.answer_targets(P0, &[Entity::Object(victim)]);
        let r = activate_containing(&mut t, P0, aryel, "Tap X untapped Knights");
        assert_eq!(r.is_ok(), !sick, "{r:?}");
        if !sick {
            t.resolve_all();
            assert!(tapped(&t, k1) && tapped(&t, k2));
            assert!(!t.on_battlefield(victim));
        }
    }
}

// --- Abilities that work only from the graveyard -------------------------------------------

#[test]
fn gangrenous_goliath_activates_only_from_the_graveyard() {
    cr!("113.6m", "602.1");
    ruling!(
        "Gangrenous Goliath",
        "The ability can only be activated while this card is in your graveyard."
    );
    let clerics = ["Cabal Evangel", "Cabal Evangel", "Keepers of the Faith"];
    // On the battlefield: no.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gangrenous Goliath");
    for c in clerics {
        t.battlefield(P0, c);
    }
    assert!(activate_containing(&mut t, P0, g, "Tap three untapped Clerics").is_err());
    // In the graveyard, tapping Clerics that came under P0's control this turn: yes.
    run(TapCase {
        card: "Gangrenous Goliath",
        needle: "Tap three untapped Clerics",
        src: Src::Graveyard,
        helpers: &["Cabal Evangel", "Cabal Evangel", "Keepers of the Faith"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.in_hand(P0, "Gangrenous Goliath"),
    });
}

#[test]
fn embalmers_tools_doesnt_let_you_activate_other_graveyard_abilities() {
    cr!("113.6", "602.1");
    ruling!(
        "Embalmer's Tools",
        "You can’t normally activate abilities of cards in graveyards."
    );
    supported("Embalmer's Tools");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Embalmer's Tools");
    // "{T}: Prodigal Pyromancer deals 1 damage to any target." — a battlefield ability.
    let p = t.graveyard(P0, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, p, "deals 1 damage").is_err());
    assert_eq!(t.life(P1), 20);
    // Deathless Ancient's graveyard ability is activatable (it costs no mana to reduce).
    let a = t.graveyard(P0, "Deathless Ancient");
    for _ in 0..3 {
        t.battlefield_sick(P0, "Vampire Noble");
    }
    assert!(activate_containing(&mut t, P0, a, "Tap three untapped Vampires").is_ok());
    t.resolve_all();
    assert!(t.in_hand(P0, "Deathless Ancient"));
}

// --- Additional costs of spells -----------------------------------------------------------

#[test]
fn guardian_of_the_great_door_taps_new_creatures() {
    cr!("302.6", "601.2b", "601.2h");
    ruling!(
        "Guardian of the Great Door",
        "You can tap any four untapped artifacts, creatures, and/or lands you control, including creatures you haven't controlled continuously"
    );
    supported("Guardian of the Great Door");
    let mut t = TestGame::new(2);
    let helpers: Vec<ObjectId> = (0..4)
        .map(|_| t.battlefield_sick(P0, "Grizzly Bears"))
        .collect();
    t.lands(P0, "Plains", 2);
    // The two Plains pay {W}{W}; the four new Bears pay the additional cost.
    t.answer_choose(
        P0,
        &helpers
            .iter()
            .map(|h| Entity::Object(*h))
            .collect::<Vec<_>>(),
    );
    let g = t.hand(P0, "Guardian of the Great Door");
    assert!(t.cast(P0, g).try_go().is_ok(), "{}", t.dump_log());
    t.resolve_all();
    assert!(helpers.iter().all(|h| tapped(&t, *h)));
    assert!(!t.named_on_battlefield("Guardian of the Great Door").is_empty());
}

#[test]
fn swallow_whole_taps_exactly_one_new_creature() {
    cr!("302.6", "601.2b", "601.2h");
    ruling!(
        "Swallow Whole",
        "You can tap any untapped creature you control, including a creature you haven’t controlled continuously since the beginning of your most recent turn, to pay the additional cost."
    );
    ruling!(
        "Swallow Whole",
        "You must tap exactly one creature to cast Swallow Whole; you can’t cast it without tapping a creature, and you can’t tap additional creatures."
    );
    supported("Swallow Whole");
    // No untapped creature: it can't be cast.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[victim.0 as usize].tapped = true;
    t.battlefield(P0, "Plains");
    let s = t.hand(P0, "Swallow Whole");
    assert!(t.cast(P0, s).target(victim).try_go().is_err());
    // Two new creatures: exactly one is tapped, and it gets the counter.
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    assert!(t.cast(P0, s).target(victim).try_go().is_ok(), "{}", t.dump_log());
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let tapped_ones: Vec<ObjectId> = [a, b].into_iter().filter(|x| tapped(&t, *x)).collect();
    assert_eq!(tapped_ones.len(), 1);
    assert_eq!(t.counters(tapped_ones[0], "+1/+1"), 1);
}

// --- Attacking and blocking costs ---------------------------------------------------------

#[test]
fn hollow_warrior_cant_tap_itself_to_attack() {
    cr!("508.1h", "508.1g");
    ruling!(
        "Hollow Warrior",
        "This card can’t tap itself to allow itself to attack or block."
    );
    supported("Hollow Warrior");
    let mut t = TestGame::new(2);
    let hw = t.battlefield(P0, "Hollow Warrior");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(hw, Entity::Player(P1))]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "no other creature to tap: it can't attack");
    assert_eq!(t.zone(hw), Zone::Battlefield);
    // With another untapped creature to tap, it attacks.
    let mut t = TestGame::new(2);
    let hw = t.battlefield(P0, "Hollow Warrior");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(hw, Entity::Player(P1))]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert!(tapped(&t, bears), "the Bears paid the attack cost");
    assert_eq!(t.life(P1), 16);
}
