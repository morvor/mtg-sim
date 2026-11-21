//! Rulings batch P179 — "creatures you control get +N/+M until end of turn" (Charge,
//! Inspiring Captain, Dwarven Lightsmith, ...): the affected set is locked in as the
//! spell or ability resolves (CR 611.2c).

use crate::r_p179_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Resolves `name` for P0: an instant or sorcery is cast; a creature is cast and its
/// "enters" trigger resolves. Returns the creature (for creature cards).
fn resolve_pump(t: &mut TestGame, name: &str) -> Option<ObjectId> {
    supported(name);
    let id = cast_targeting(t, P0, name, &[]);
    t.resolve_all();
    t.on_battlefield(id).then_some(id)
}

/// (card, power bonus, toughness bonus): the pump spells and "enters" abilities of the
/// batch. Each case: Grizzly Bears (2/2) on the battlefield gets the bonus; a creature
/// entering afterward, a creature P0 gains control of afterward, and (for the rulings that
/// say so) an artifact that becomes a creature afterward don't.
#[test]
fn team_pumps_affect_only_creatures_there_as_they_resolve() {
    cr!("611.2c");
    ruling!(
        "Charge",
        "Charge affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +1/+1."
    );
    ruling!(
        "Civic Stalwart",
        "Civic Stalwart’s triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +1/+1."
    );
    ruling!(
        "Glorious Charge",
        "Glorious Charge affects only creatures you control at the time it resolves. It won’t affect creatures that come under your control later in the turn. It won’t affect permanents that become creatures later in the turn."
    );
    ruling!(
        "Inspiring Captain",
        "Inspiring Captain’s triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +1/+1."
    );
    ruling!(
        "Inspiring Captain",
        "The set of creatures affected by Inspiring Captain’s ability is determined as the ability resolves. Creatures you begin to control later in the turn won’t get +1/+1."
    );
    ruling!(
        "Righteous Charge",
        "Only creatures you control when Righteous Charge resolves will get +2/+2. Creatures that come under your control later in the turn will not."
    );
    ruling!(
        "Ethereal Guidance",
        "The set of creatures affected by Ethereal Guidance is determined as the spell resolves. Creatures you begin to control later in the turn won’t get +2/+1."
    );
    ruling!(
        "Inspired Charge",
        "The set of creatures affected by Inspired Charge is determined as the spell resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won't get +2/+1."
    );
    ruling!(
        "Warrior's Honor",
        "Warrior’s Honor affects creatures you control that are on the battlefield at the time it resolves. If you put a creature onto the battlefield later in the turn, that creature won’t get the bonus."
    );
    for (name, dp, dt) in [
        ("Charge", 1, 1),
        ("Civic Stalwart", 1, 1),
        ("Glorious Charge", 1, 1),
        ("Inspiring Captain", 1, 1),
        ("Righteous Charge", 2, 2),
        ("Ethereal Guidance", 2, 1),
        ("Inspired Charge", 2, 1),
        ("Warrior's Honor", 1, 1),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let factory = t.battlefield(P0, "Mishra's Factory");
        let stolen = t.battlefield(P1, "Gray Ogre");
        resolve_pump(&mut t, name);
        assert_eq!(t.pt(bears), (2 + dp, 2 + dt), "{name}");
        // Afterward: a creature enters, P0 gains control of P1's Gray Ogre, and Mishra's
        // Factory becomes a 2/2 creature.
        let later = t.enter(P0, "Hill Giant");
        t.settle();
        give_control(&mut t, stolen, P0);
        t.lands(P0, "Wastes", 1);
        activate_containing(&mut t, P0, factory, "becomes").unwrap();
        t.resolve_all();
        assert_eq!(t.pt(later), (3, 3), "{name}: entered later");
        assert_eq!(t.pt(stolen), (2, 2), "{name}: gained control later");
        assert_eq!(t.pt(factory), (2, 2), "{name}: became a creature later");
        assert_eq!(t.pt(bears), (2 + dp, 2 + dt), "{name}");
    }
}

#[test]
fn leonin_armorguard_gets_its_own_bonus() {
    cr!("603.6a", "611.2c");
    ruling!("Leonin Armorguard", "Leonin Armorguard gets this bonus too.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let guard = resolve_pump(&mut t, "Leonin Armorguard").expect("on the battlefield");
    assert_eq!(t.pt(guard), (4, 4));
    assert_eq!(t.pt(bears), (3, 3));
    let ogre = t.battlefield(P0, "Gray Ogre");
    assert_eq!(t.pt(ogre), (2, 2));
}

#[test]
fn dwarven_lightsmith_affects_the_team_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Dwarven Lightsmith",
        "Dwarven Lightsmith’s ability affects only creatures your team controls at the time it resolves. Creatures that come under your team’s control later in the turn won’t get +1/+1."
    );
    let mut t = two_headed_giant();
    let mine = t.battlefield(P0, "Grizzly Bears");
    let teammates = t.battlefield(P1, "Grizzly Bears");
    let opponents = t.battlefield(P2, "Grizzly Bears");
    let stolen = t.battlefield(P3, "Gray Ogre");
    let smith = resolve_pump(&mut t, "Dwarven Lightsmith").expect("on the battlefield");
    assert_eq!(t.pt(smith), (4, 5));
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(teammates), (3, 3));
    assert_eq!(t.pt(opponents), (2, 2));
    // Later: P1 (a teammate) gains control of a creature, and one enters under P1.
    give_control(&mut t, stolen, P1);
    let later = t.enter(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.pt(stolen), (2, 2));
    assert_eq!(t.pt(later), (3, 3));
}
