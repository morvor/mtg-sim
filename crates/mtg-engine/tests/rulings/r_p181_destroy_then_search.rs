//! Rulings batch P181 — "destroy target land/creature. Its controller may search their
//! library for a basic land card": the search happens even if the target isn't destroyed
//! (indestructible), and doesn't if the target is illegal (the spell or ability doesn't
//! resolve, CR 608.2b).

use crate::r_p181_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[derive(Clone, Copy)]
enum How {
    /// A spell cast from P0's hand.
    Spell,
    /// A creature whose enters trigger targets.
    Etb,
    /// A land's activated ability (the text it contains).
    Activate(&'static str),
}

/// A spell or ability that destroys (or exiles) its target, then lets players search.
#[derive(Clone, Copy)]
struct Case {
    card: &'static str,
    how: How,
    /// The target, controlled by P1.
    target: &'static str,
    /// Which players search: (P0, P1).
    searchers: (bool, bool),
    /// P0 draws a card.
    draws: bool,
}

const fn case(card: &'static str, how: How, target: &'static str, searchers: (bool, bool)) -> Case {
    Case {
        card,
        how,
        target,
        searchers,
        draws: false,
    }
}

/// Runs `c` against a target that is either made illegal before it resolves (returned to
/// its owner's hand) or indestructible. Returns the game and the target.
fn run(c: Case, illegal: bool) -> (TestGame, ObjectId) {
    supported(c.card);
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest", "Forest", "Forest"]);
    set_library(&mut t, P1, &["Plains", "Plains", "Plains"]);
    let target = t.battlefield(P1, c.target);
    let p0_forest = t.g.player(P0).library[0];
    let p1_plains: Vec<Entity> = t.g.player(P1).library[..2]
        .iter()
        .map(|o| obj(*o))
        .collect();
    // Every player who may search does, and tries to find two cards (one is all most of
    // these allow).
    t.answer_yes(P1, true);
    t.answer_choose(P1, &p1_plains);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(p0_forest)]);
    lots(&mut t, P0);
    match c.how {
        How::Spell => {
            let card = t.hand(P0, c.card);
            t.cast(P0, card).target(target).go();
        }
        How::Etb => {
            t.answer_targets(P0, &[obj(target)]);
            t.enter(P0, c.card);
            t.settle();
        }
        How::Activate(text) => {
            let land = t.battlefield(P0, c.card);
            t.answer_targets(P0, &[obj(target)]);
            activate_containing(&mut t, P0, land, text).unwrap();
        }
    }
    assert_eq!(t.stack_len(), 1, "{}: it's on the stack", c.card);
    if illegal {
        bounce(&mut t, target);
    }
    t.resolve_all();
    (t, target)
}

/// The lands P1 put onto the battlefield (Plains) and P0 (Forests).
fn found(t: &TestGame) -> (usize, usize) {
    (
        controlled(t, P0, "Forest").len(),
        controlled(t, P1, "Plains").len(),
    )
}

#[test]
fn an_illegal_target_means_no_one_searches() {
    cr!("608.2b", "701.23a");
    ruling!(
        "Magmatic Hellkite",
        "If the target land is an illegal target as Magmatic Hellkite’s last ability tries to resolve, it won’t resolve and none of its effects will happen. No player will search their library for a basic land card."
    );
    ruling!(
        "Sandworm",
        "If the target land is an illegal target when Sandworm's last ability tries to resolve, it won't resolve and none of its effects will happen. The target land's controller won't search for a basic land card."
    );
    ruling!(
        "Deathsprout",
        "If the target creature is an illegal target by the time Deathsprout tries to resolve, the spell doesn’t resolve. You don’t search for a land card."
    );
    ruling!(
        "Path to Exile",
        "If the target creature is an illegal target by the time Path to Exile tries to resolve, it won't resolve and none of its effects will happen. The creature's controller won't search for a basic land card."
    );
    ruling!(
        "Erode",
        "If the target is illegal as Erode tries to resolve, it won't resolve and none of its effects will happen. No player will search their library."
    );
    ruling!(
        "Cleansing Wildfire",
        "If the target land is an illegal target by the time Cleansing Wildfire tries to resolve, the spell doesn’t resolve. No player searches for a basic land card and you don’t draw a card."
    );
    ruling!(
        "Field of Ruin",
        "If the target land is an illegal target by the time Field of Ruin's ability tries to resolve, the ability doesn't resolve. No players search for a basic land card or shuffles."
    );
    ruling!(
        "Geomancer's Gambit",
        "If the target land is an illegal target by the time Geomancer’s Gambit tries to resolve, the spell doesn’t resolve. Its controller doesn’t search for a basic land card and you don’t draw a card."
    );
    ruling!(
        "Frenzied Tilling",
        "If the target land is an illegal target when Frenzied Tilling tries to resolve, the spell won’t resolve and none of its effects will happen. You won’t search your library for a basic land card."
    );
    ruling!(
        "White Orchid Phantom",
        "If the target land is an illegal target when White Orchid Phantom's last ability tries to resolve, it won't resolve and none of its effects will happen. That land's controller won't get to search for a basic land card."
    );
    ruling!(
        "Dire-Strain Rampage",
        "If the target permanent became an illegal target by the time Dire-Strain Rampage tried to resolve, the spell won't resolve and none of its effects will happen. No player will search their library for any cards."
    );
    ruling!(
        "Demolition Field",
        "If the targeted land is an illegal target by the time Demolition Field's ability resolves, it won't resolve and none of its effects will happen. Neither player will get to search for a basic land card."
    );
    ruling!(
        "Sundering Eruption // Volcanic Fissure",
        "If the target land is an illegal target when Sundering Eruption tries to resolve, it won't resolve and none of its effects will happen. That land's controller won't get to search for a basic land card, and creatures without flying will still be able to block this turn."
    );
    let cases = [
        case(
            "Magmatic Hellkite",
            How::Etb,
            "Darksteel Citadel",
            (false, true),
        ),
        case("Sandworm", How::Etb, "Darksteel Citadel", (false, true)),
        case(
            "White Orchid Phantom",
            How::Etb,
            "Darksteel Citadel",
            (false, true),
        ),
        case("Deathsprout", How::Spell, "Darksteel Myr", (true, false)),
        case("Path to Exile", How::Spell, "Darksteel Myr", (false, true)),
        case("Erode", How::Spell, "Darksteel Myr", (false, true)),
        Case {
            draws: true,
            ..case(
                "Cleansing Wildfire",
                How::Spell,
                "Darksteel Citadel",
                (false, true),
            )
        },
        Case {
            draws: true,
            ..case(
                "Geomancer's Gambit",
                How::Spell,
                "Darksteel Citadel",
                (false, true),
            )
        },
        case(
            "Frenzied Tilling",
            How::Spell,
            "Darksteel Citadel",
            (true, false),
        ),
        case(
            "Sundering Eruption // Volcanic Fissure",
            How::Spell,
            "Darksteel Citadel",
            (false, true),
        ),
        case(
            "Dire-Strain Rampage",
            How::Spell,
            "Darksteel Citadel",
            (false, true),
        ),
        case(
            "Field of Ruin",
            How::Activate("Destroy target"),
            "Darksteel Citadel",
            (true, true),
        ),
        case(
            "Demolition Field",
            How::Activate("Destroy target"),
            "Darksteel Citadel",
            (true, true),
        ),
    ];
    for c in cases {
        let (mut t, target) = run(c, true);
        assert_eq!(t.zone(t.g.current(target)), Zone::Hand(P1), "{}", c.card);
        assert_eq!(searches(&t, P0) + searches(&t, P1), 0, "{}", c.card);
        assert_eq!(shuffles(&t, P0) + shuffles(&t, P1), 0, "{}", c.card);
        assert_eq!(found(&t), (0, 0), "{}", c.card);
        assert_eq!(draws(&t, P0), 0, "{}: no card drawn", c.card);
        if c.card.starts_with("Sundering") {
            assert!(
                bears_can_block(&mut t),
                "creatures without flying can still block"
            );
        }
    }
}

/// Whether a new Grizzly Bears of P1 could block.
fn bears_can_block(t: &mut TestGame) -> bool {
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    t.g.can_block_at_all(bears)
}

#[test]
fn the_search_happens_even_if_the_target_isnt_destroyed() {
    cr!("608.2c", "702.12b", "701.23a");
    ruling!(
        "Magmatic Hellkite",
        "As long as Magmatic Hellkite’s last ability resolves, the target land’s controller gets to search for a basic land card even if the target land wasn’t destroyed by the ability. This may happen because the land has indestructible."
    );
    ruling!(
        "Sandworm",
        "If Sandworm's last ability resolves but doesn't destroy the target land (probably because the land had indestructible), the target land's controller still gets to search for a basic land card."
    );
    ruling!(
        "Deathsprout",
        "If the target is legal but not destroyed (most likely because it has indestructible), you do search."
    );
    ruling!(
        "Cleansing Wildfire",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does search for a basic land card and you do draw a card."
    );
    ruling!(
        "Field of Ruin",
        "If the target is legal but not destroyed (most likely because it has indestructible), each player does search and shuffle."
    );
    ruling!(
        "Geomancer's Gambit",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does search and you do draw."
    );
    ruling!(
        "White Orchid Phantom",
        "If the target land isn't destroyed by White Orchid Phantom's last ability (perhaps because the land has indestructible), its controller still gets to search for a basic land card."
    );
    ruling!(
        "Sundering Eruption // Volcanic Fissure",
        "If the target land isn't destroyed by Sundering Eruption's effect (perhaps because the land has indestructible), its controller still gets to search for a basic land card, and creatures without flying still can't block this turn."
    );
    ruling!(
        "Dire-Strain Rampage",
        "However, if the target is still legal but wasn't destroyed, perhaps because it was indestructible, its controller may still search their library for a basic land card (even if the target was a land)."
    );
    ruling!(
        "Demolition Field",
        "The target land's controller gets to search for a basic land card even if that land wasn't destroyed by Demolition Field's ability. This may happen because the land has indestructible."
    );
    let cases = [
        case(
            "Magmatic Hellkite",
            How::Etb,
            "Darksteel Citadel",
            (false, true),
        ),
        case("Sandworm", How::Etb, "Darksteel Citadel", (false, true)),
        case(
            "White Orchid Phantom",
            How::Etb,
            "Darksteel Citadel",
            (false, true),
        ),
        case("Deathsprout", How::Spell, "Darksteel Myr", (true, false)),
        Case {
            draws: true,
            ..case(
                "Cleansing Wildfire",
                How::Spell,
                "Darksteel Citadel",
                (false, true),
            )
        },
        Case {
            draws: true,
            ..case(
                "Geomancer's Gambit",
                How::Spell,
                "Darksteel Citadel",
                (false, true),
            )
        },
        case(
            "Sundering Eruption // Volcanic Fissure",
            How::Spell,
            "Darksteel Citadel",
            (false, true),
        ),
        // A land that wasn't destroyed: "Otherwise", one basic land card.
        case(
            "Dire-Strain Rampage",
            How::Spell,
            "Darksteel Citadel",
            (false, true),
        ),
        case(
            "Field of Ruin",
            How::Activate("Destroy target"),
            "Darksteel Citadel",
            (true, true),
        ),
        case(
            "Demolition Field",
            How::Activate("Destroy target"),
            "Darksteel Citadel",
            (true, true),
        ),
    ];
    for c in cases {
        let (mut t, target) = run(c, false);
        assert!(t.on_battlefield(target), "{}: indestructible", c.card);
        let (p0, p1) = c.searchers;
        assert_eq!(searches(&t, P0), p0 as usize, "{}", c.card);
        assert_eq!(searches(&t, P1), p1 as usize, "{}", c.card);
        assert_eq!(shuffles(&t, P0), p0 as usize, "{}", c.card);
        assert_eq!(shuffles(&t, P1), p1 as usize, "{}", c.card);
        // One basic land card each (P1 tried to find two).
        assert_eq!(found(&t), (p0 as usize, p1 as usize), "{}", c.card);
        assert_eq!(draws(&t, P0), c.draws as usize, "{}", c.card);
        if c.card.starts_with("Sundering") {
            assert!(
                !bears_can_block(&mut t),
                "creatures without flying can't block"
            );
        }
    }
}

/// P0's Grizzly Bears equipped with Sword of Hearth and Home deals combat damage to P1;
/// the trigger targets `target` (a creature P0 owns). `before` runs with it on the stack.
fn sword_hits(t: &mut TestGame, target: ObjectId, before: impl FnOnce(&mut TestGame)) {
    use mtg_engine::turn::Step;
    supported("Sword of Hearth and Home");
    let sword = t.battlefield(P0, "Sword of Hearth and Home");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(sword, Entity::Object(bears));
    t.answer_targets(P0, &[obj(target)]);
    attack_with(t, &[(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the sword's trigger");
    before(t);
    t.resolve_all();
}

#[test]
fn sword_of_hearth_and_home_illegal_target_and_token_target() {
    cr!("608.2b", "111.7", "111.8", "701.23a");
    ruling!(
        "Sword of Hearth and Home",
        "If you choose a target creature you own and that creature isn't a legal target as the triggered ability tries to resolve, that ability won't resolve and none of its effects will happen. You won't search for a basic land card."
    );
    ruling!(
        "Sword of Hearth and Home",
        "If the creature you own is a token, it won't return to the battlefield. It will stay in exile and cease to exist after the ability finishes resolving."
    );
    // The target leaves before the trigger resolves: no search.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest", "Forest"]);
    let elf = t.battlefield(P0, "Llanowar Elves");
    sword_hits(&mut t, elf, |t| bounce(t, elf));
    assert_eq!(searches(&t, P0), 0);
    assert!(controlled(&t, P0, "Forest").is_empty());
    // A token: exiled, and gone; the land is still found.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest", "Forest"]);
    let forest = t.g.player(P0).library[0];
    t.answer_choose(P0, &[obj(forest)]);
    let token = create_token(&mut t, P0, "Soldier");
    sword_hits(&mut t, token, |_| {});
    assert_eq!(searches(&t, P0), 1);
    assert_eq!(controlled(&t, P0, "Forest").len(), 1);
    assert!(
        !t.g.permanents().any(|o| o.is_token()),
        "the token didn't return"
    );
    assert!(
        !t.g.exile.iter().any(|o| t.g.obj(*o).is_token()),
        "it ceased to exist"
    );
}

#[test]
fn emergency_eject_illegal_target_creates_no_lander() {
    cr!("608.2b", "111.10u");
    ruling!(
        "Emergency Eject",
        "If the target permanent is an illegal target as Emergency Eject tries to resolve, it won’t resolve and none of its effects will happen. The permanent’s controller won’t create a Lander token."
    );
    // The same, said of Lander-making spells and abilities with targets in general, on
    // Beamsaw Prospector (whose own Lander needs no target): Emergency Eject targets one.
    ruling!(
        "Beamsaw Prospector",
        "Some spells and abilities that create Lander tokens require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won’t resolve. The Lander token won’t be created."
    );
    supported("Emergency Eject");
    supported("Beamsaw Prospector");
    for illegal in [true, false] {
        let mut t = TestGame::new(2);
        lots(&mut t, P0);
        let prospector = t.battlefield(P1, "Beamsaw Prospector");
        let eject = t.hand(P0, "Emergency Eject");
        t.cast(P0, eject).target(prospector).go();
        if illegal {
            // Returned to its owner's hand: it doesn't die either.
            bounce(&mut t, prospector);
        }
        t.resolve_all();
        let landers = crate::r_s01_common::with_subtype(&t, P1, "Lander").len();
        // Destroyed: Emergency Eject's Lander and the dying Prospector's.
        assert_eq!(landers, if illegal { 0 } else { 2 });
    }
}

#[test]
fn price_of_freedom_search_is_optional_and_so_is_the_shuffle() {
    cr!("701.23a", "701.24a");
    ruling!(
        "Price of Freedom",
        "The controller of the destroyed artifact or land isn't required to search their library. If that player doesn't, the player won't shuffle their library."
    );
    supported("Price of Freedom");
    for accept in [true, false] {
        let mut t = TestGame::new(2);
        set_library(&mut t, P1, &["Plains", "Plains"]);
        let plains = t.g.player(P1).library[0];
        let target = t.battlefield(P1, "Mind Stone");
        lots(&mut t, P0);
        t.answer_yes(P1, accept);
        t.answer_choose(P1, &[obj(plains)]);
        let price = t.hand(P0, "Price of Freedom");
        t.cast(P0, price).target(target).go();
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Mind Stone"));
        assert_eq!(searches(&t, P1), accept as usize);
        assert_eq!(shuffles(&t, P1), accept as usize);
        assert_eq!(controlled(&t, P1, "Plains").len(), accept as usize);
    }
}

#[test]
fn escape_tunnel_checks_power_on_resolution_only() {
    use mtg_engine::turn::Step;
    cr!("608.2b", "509.1b", "509.1h");
    ruling!(
        "Escape Tunnel",
        "If the target creature's power is increased to 3 or greater after Escape Tunnel's last ability is activated but before it resolves, the ability doesn't resolve. However, if instead the creature's power is increased to 3 or greater after the ability resolves, it still can't be blocked that turn."
    );
    ruling!(
        "Escape Tunnel",
        "Once a creature with power 2 or less has been blocked, Escape Tunnel's last ability won't cause it to become unblocked."
    );
    supported("Escape Tunnel");
    for pump_first in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Grizzly Bears");
        let tunnel = t.battlefield(P0, "Escape Tunnel");
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, tunnel, "can't be blocked").unwrap();
        if pump_first {
            pump(&mut t, bears, 1, 0);
        }
        t.resolve_all();
        if !pump_first {
            pump(&mut t, bears, 1, 0);
        }
        assert_eq!(t.pt(bears).0, 3);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert_eq!(
            can_block(&mut t, wall, bears),
            pump_first,
            "pump first: {pump_first}"
        );
    }
    // Blocked first, then the ability: it stays blocked.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Grizzly Bears");
    let tunnel = t.battlefield(P0, "Escape Tunnel");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(wall, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, tunnel, "can't be blocked").unwrap();
    t.resolve_all();
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "no damage to P1");
}
