//! Rulings batch P087 — "each player can't cast more than one [noncreature] spell each
//! turn" (Rule of Law, Arcane Laboratory, Eidolon of Rhetoric, Deafening Silence): the
//! limit looks at the whole turn, counts the hate card's own casting and countered spells,
//! and ends when the permanent leaves (CR 601.3, 611.3a).

use crate::r_p087_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The spell-count hate cards of this batch: (name, whether only noncreature spells count).
const LIMITS: [(&str, bool); 4] = [
    ("Rule of Law", false),
    ("Arcane Laboratory", false),
    ("Eidolon of Rhetoric", false),
    ("Deafening Silence", true),
];

#[test]
fn the_limit_counts_spells_cast_before_it_entered_including_itself() {
    cr!("601.3", "611.3a");
    ruling!(
        "Rule of Law",
        "Rule of Law looks at the entire turn to see if a player has cast a spell, even if Rule of Law wasn’t on the battlefield when that spell was cast. Notably, you can’t cast Rule of Law and then cast another spell during the same turn."
    );
    ruling!(
        "Eidolon of Rhetoric",
        "Eidolon of Rhetoric will look at the entire turn to see if a player has cast a spell yet that turn, even if Eidolon of Rhetoric wasn't on the battlefield when that spell was cast. Specifically, you can't cast Eidolon of Rhetoric and then cast another spell that turn."
    );
    ruling!(
        "Deafening Silence",
        "Deafening Silence will look at the entire turn to see if a player has cast a noncreature spell yet that turn, even if Deafening Silence wasn't on the battlefield when that spell was cast. Notably, you can't cast Deafening Silence and then cast another noncreature spell that turn."
    );
    ruling!(
        "Arcane Laboratory",
        "When Arcane Laboratory enters, it will look back over the turn to see if any spell was cast before it entered the battlefield. It will notice itself being cast."
    );
    for (name, noncreature_only) in LIMITS {
        supported(name);
        // Cast the hate card itself: it's the one spell (it is a noncreature spell, except
        // Eidolon of Rhetoric, a creature spell, which still counts as "a spell").
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, name, &[]);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
        assert!(!could_cast(&mut t, P0, "Lightning Bolt"), "{name}");
        // Deafening Silence counts only noncreature spells: a creature spell is fine.
        assert_eq!(
            could_cast(&mut t, P0, "Grizzly Bears"),
            noncreature_only,
            "{name}"
        );
        // P1 hasn't cast a spell this turn.
        assert!(could_cast(&mut t, P1, "Lightning Bolt"), "{name}");

        // A spell cast earlier in the turn, before the permanent was on the battlefield.
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
        assert!(could_cast(&mut t, P0, "Lightning Bolt"), "{name}");
        t.battlefield(P1, name);
        assert!(!could_cast(&mut t, P0, "Lightning Bolt"), "{name}");
        // The next turn, P0 may cast one again.
        t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
        assert!(could_cast(&mut t, P0, "Lightning Bolt"), "{name}");
    }
}

#[test]
fn a_countered_spell_still_counts() {
    cr!("601.3", "701.6a");
    ruling!(
        "Arcane Laboratory",
        "If a spell is countered, it still counts as the one spell cast."
    );
    ruling!(
        "Deafening Silence",
        "If you cast a noncreature spell that was countered, you can't cast another noncreature spell that turn."
    );
    for name in ["Arcane Laboratory", "Deafening Silence"] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, name);
        let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
        cast_new(&mut t, P1, "Counterspell", &[obj(bolt)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Lightning Bolt"), "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
        assert!(!could_cast(&mut t, P0, "Shock"), "{name}");
    }
}

#[test]
fn the_limit_ends_when_the_permanent_leaves() {
    cr!("611.3a", "601.3");
    ruling!(
        "Arcane Laboratory",
        "When it leaves the battlefield, the effect ends and all players are able to cast spells unrestricted again."
    );
    let mut t = TestGame::new(2);
    let lab = t.battlefield(P1, "Arcane Laboratory");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!could_cast(&mut t, P0, "Lightning Bolt"));
    destroy(&mut t, lab);
    assert!(could_cast(&mut t, P0, "Lightning Bolt"));
    // Unrestricted: a third spell too.
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(could_cast(&mut t, P0, "Shock"));
    assert_eq!(t.life(P1), 15);
}
