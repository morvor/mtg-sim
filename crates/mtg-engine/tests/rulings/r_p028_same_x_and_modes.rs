//! Rulings batch P028 — a copy of an instant or sorcery spell has the value of X chosen as
//! the original was cast (CR 707.10, 107.3) and the same mode or modes (CR 707.10, 700.2):
//! nothing is chosen again for it, whatever makes the copy (a spell, an activated ability,
//! an enters trigger, a cast trigger, a delayed triggered ability). One table of copying
//! cards drives both checks.

use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::chosen_modes;
use crate::r_s17_common::enter_transformed;
use crate::r_s19_common::add_lore;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// What sets up the copying card before the original spell is cast, and what (after the
/// original is cast) puts the copy effect on the stack and queues P0's answers for it.
struct Copier {
    name: &'static str,
    setup: fn(&mut TestGame),
    start: fn(&mut TestGame, ObjectId),
}

/// The original spell: Blaze ("Blaze deals X damage to any target.") with X = 2 at P1, or
/// Izzet Charm choosing its second mode ("Izzet Charm deals 2 damage to target creature.")
/// at P1's Hill Giant (3/3).
#[derive(Clone, Copy, PartialEq)]
enum Original {
    BlazeX2,
    CharmMode1,
}

/// Runs `c` against `orig`: the copies have the original's X or modes (no mode is chosen
/// for them) and do what the original does.
fn copy_keeps_choices(c: &Copier, orig: Original) {
    supported(c.name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    (c.setup)(&mut t);
    let spell = match orig {
        Original::BlazeX2 => {
            t.lands(P0, "Mountain", 1);
            t.lands(P0, "Wastes", 2);
            let card = t.hand(P0, "Blaze");
            t.cast(P0, card).x(2).target(P1).go()
        }
        Original::CharmMode1 => {
            lands_for_cost(&mut t, P0, "Izzet Charm");
            let card = t.hand(P0, "Izzet Charm");
            t.cast(P0, card).modes(&[1]).target(giant).go()
        }
    };
    (c.start)(&mut t, spell);
    let from = t.asked().len();
    let copies = resolve_until_copies(&mut t);
    assert!(!copies.is_empty(), "{}: no copy was made", c.name);
    for copy in &copies {
        match orig {
            Original::BlazeX2 => assert_eq!(x_of(&t, *copy), Some(2), "{}", c.name),
            Original::CharmMode1 => {
                assert_eq!(chosen_modes(&t, *copy), vec![1], "{}", c.name);
                assert_eq!(targets_of(&t, *copy), vec![obj(giant)], "{}", c.name);
            }
        }
    }
    let modes_asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count();
    assert_eq!(modes_asked, 0, "{}: a mode was chosen for the copy", c.name);
    t.resolve_all();
    match orig {
        Original::BlazeX2 => {
            assert_eq!(t.life(P1), 20 - 2 * (1 + copies.len() as i32), "{}", c.name)
        }
        // 2 damage from each: the 3/3 Giant dies only because the copy dealt 2 too.
        Original::CharmMode1 => assert!(!t.on_battlefield(giant), "{}", c.name),
    }
}

// --- The copying cards -------------------------------------------------------------------

fn nothing(_: &mut TestGame) {}

fn keep(t: &mut TestGame, _: ObjectId) {
    keep_copy_targets(t, P0);
}

/// "Copy target instant or sorcery spell. You may choose new targets for the copy."
const REVERBERATE: Copier = Copier {
    name: "Reverberate",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Reverberate", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "You may choose new targets for target instant or sorcery spell. Then copy that spell.
/// You may choose new targets for the copy."
const WILD_RICOCHET: Copier = Copier {
    name: "Wild Ricochet",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Wild Ricochet", &[obj(s)]);
        t.answer_yes(P0, false);
        t.answer_yes(P0, false);
    },
};

/// "• Copy target instant or sorcery spell. You may choose new targets for the copy."
const INSIDIOUS_WILL: Copier = Copier {
    name: "Insidious Will",
    setup: nothing,
    start: |t, s| {
        lands_for_cost(t, P0, "Insidious Will");
        let card = t.hand(P0, "Insidious Will");
        t.cast(P0, card).modes(&[2]).target(s).go();
        keep_copy_targets(t, P0);
    },
};

/// "Copy any number of target instant and/or sorcery spells."
const DISPLAY_OF_POWER: Copier = Copier {
    name: "Display of Power",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Display of Power", &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "When this creature enters, copy target instant or sorcery spell."
const DUALCASTER_MAGE: Copier = Copier {
    name: "Dualcaster Mage",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Dualcaster Mage", &[]);
        t.answer_targets(P0, &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// "When Naru Meha enters, copy target instant or sorcery spell you control."
const NARU_MEHA: Copier = Copier {
    name: "Naru Meha, Master Wizard",
    setup: nothing,
    start: |t, s| {
        cast_new(t, P0, "Naru Meha, Master Wizard", &[]);
        t.answer_targets(P0, &[obj(s)]);
        keep_copy_targets(t, P0);
    },
};

/// Level 2: "{U}{U}, {T}: Copy target instant or sorcery spell."
const ECHO_MAGE: Copier = Copier {
    name: "Echo Mage",
    setup: |t| {
        let mage = t.battlefield(P0, "Echo Mage");
        t.g.add_counters(obj(mage), "level", 2, None);
        t.g.recompute();
    },
    start: |t, s| {
        t.lands(P0, "Island", 2);
        let mage = t.named_on_battlefield("Echo Mage")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, mage, "Copy target").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "{2}{U}, {T}: The controller of target instant or sorcery spell copies it."
const MELETIS_CHARLATAN: Copier = Copier {
    name: "Meletis Charlatan",
    setup: |t| {
        t.battlefield(P0, "Meletis Charlatan");
    },
    start: |t, s| {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 2);
        let c = t.named_on_battlefield("Meletis Charlatan")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, c, "copies it").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "{3}, {T}: Copy target instant or sorcery spell you control."
const LITHOFORM_ENGINE: Copier = Copier {
    name: "Lithoform Engine",
    setup: |t| {
        t.battlefield(P0, "Lithoform Engine");
    },
    start: |t, s| {
        t.lands(P0, "Wastes", 3);
        let e = t.named_on_battlefield("Lithoform Engine")[0];
        t.answer_targets(P0, &[obj(s)]);
        activate_containing(t, P0, e, "instant or sorcery spell").unwrap();
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast your first instant or sorcery spell each turn, copy that spell."
const DOUBLE_VISION: Copier = Copier {
    name: "Double Vision",
    setup: |t| {
        t.battlefield(P0, "Double Vision");
    },
    start: keep,
};

/// "Whenever you cast an instant or sorcery spell, you may pay {3}. If you do, copy that
/// spell."
const MIRARI: Copier = Copier {
    name: "Mirari",
    setup: |t| {
        t.battlefield(P0, "Mirari");
        t.lands(P0, "Wastes", 3);
    },
    start: |t, _| {
        t.answer_yes(P0, true);
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast an instant or sorcery spell, you may copy that spell."
const SWARM_INTELLIGENCE: Copier = Copier {
    name: "Swarm Intelligence",
    setup: |t| {
        t.battlefield(P0, "Swarm Intelligence");
    },
    start: |t, _| {
        t.answer_yes(P0, true);
        keep_copy_targets(t, P0);
    },
};

/// III — "Until end of turn, whenever you cast an instant or sorcery spell, copy it."
const MIRARI_CONJECTURE: Copier = Copier {
    name: "The Mirari Conjecture",
    setup: |t| {
        let saga = t.battlefield(P0, "The Mirari Conjecture");
        add_lore(t, saga, 3);
        t.resolve_all();
    },
    start: keep,
};

/// "Whenever you cast your first instant spell each turn, if Kalamax is tapped, copy that
/// spell."
const KALAMAX: Copier = Copier {
    name: "Kalamax, the Stormsire",
    setup: |t| {
        let k = t.battlefield(P0, "Kalamax, the Stormsire");
        t.g.tap(k);
    },
    start: keep,
};

/// "Whenever you cast an artifact, instant, or sorcery spell, copy that spell. ... This
/// ability triggers only once each turn."
const JIN_GITAXIAS: Copier = Copier {
    name: "Jin-Gitaxias, Progress Tyrant",
    setup: |t| {
        t.battlefield(P0, "Jin-Gitaxias, Progress Tyrant");
    },
    start: keep,
};

/// "Whenever you cast a multicolored instant or sorcery spell, you may pay {1}. If you do,
/// copy that spell."
const CLOVEN_CASTING: Copier = Copier {
    name: "Cloven Casting",
    setup: |t| {
        t.battlefield(P0, "Cloven Casting");
        t.lands(P0, "Wastes", 1);
    },
    start: |t, _| {
        t.answer_yes(P0, true);
        keep_copy_targets(t, P0);
    },
};

/// "Whenever you cast an instant or sorcery spell, copy it for each other instant and
/// sorcery spell you've cast before it this turn." (One spell, Opt, was cast before.)
const THOUSAND_YEAR_STORM: Copier = Copier {
    name: "Thousand-Year Storm",
    setup: |t| {
        t.battlefield(P0, "Thousand-Year Storm");
        cast_new(t, P0, "Opt", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "Until end of turn, whenever a player casts an instant or sorcery spell, that player
/// copies it and may choose new targets for the copy."
const BONUS_ROUND: Copier = Copier {
    name: "Bonus Round",
    setup: |t| {
        cast_new(t, P0, "Bonus Round", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "−2: When you next cast an instant or sorcery spell this turn, copy that spell."
const CHANDRA: Copier = Copier {
    name: "Chandra, the Firebrand",
    setup: |t| {
        let c = t.battlefield(P0, "Chandra, the Firebrand");
        activate_containing(t, P0, c, "When you next cast").unwrap();
        t.resolve_all();
    },
    start: keep,
};

/// "When you next cast an instant or sorcery spell this turn, copy that spell."
const HOWL_OF_THE_HORDE: Copier = Copier {
    name: "Howl of the Horde",
    setup: |t| {
        cast_new(t, P0, "Howl of the Horde", &[]);
        t.resolve_all();
    },
    start: keep,
};

/// "When you next cast an instant or sorcery spell this turn, copy that spell X times."
/// (X = 1.)
const STORM_KINGS_THUNDER: Copier = Copier {
    name: "Storm King's Thunder",
    setup: |t| {
        t.lands(P0, "Mountain", 3);
        t.lands(P0, "Wastes", 1);
        let c = t.hand(P0, "Storm King's Thunder");
        t.cast(P0, c).x(1).go();
        t.resolve_all();
    },
    start: keep,
};

/// "• When you cast your next instant or sorcery spell this turn, copy that spell."
const TWINFERNO: Copier = Copier {
    name: "Twinferno",
    setup: |t| {
        lands_for_cost(t, P0, "Twinferno");
        let c = t.hand(P0, "Twinferno");
        t.cast(P0, c).modes(&[0]).go();
        t.resolve_all();
    },
    start: keep,
};

/// "Whenever Najal attacks, you may pay {2}. If you do, when you next cast an instant or
/// sorcery spell this turn, copy it." (The original is cast in the declare attackers
/// step.)
const NAJAL: Copier = Copier {
    name: "Najal, the Storm Runner",
    setup: |t| {
        let n = t.battlefield(P0, "Najal, the Storm Runner");
        t.lands(P0, "Wastes", 2);
        t.answer_yes(P0, true);
        attack_with(t, &[(n, Entity::Player(P1))]);
        t.resolve_all();
    },
    start: keep,
};

// --- The value of X -----------------------------------------------------------------------

#[test]
fn copies_have_the_same_value_of_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Display of Power",
        "If a spell that's copied has an X whose value was determined as it was cast, the copy has the same value of X."
    );
    copy_keeps_choices(&DISPLAY_OF_POWER, Original::BlazeX2);
    ruling!(
        "Chandra, the Firebrand",
        "If the spell Chandra copies has an X whose value was determined as it was cast (like Consume Spirit does), the copy has the same value of X."
    );
    copy_keeps_choices(&CHANDRA, Original::BlazeX2);
    ruling!(
        "Reverberate",
        "If the spell Reverberate copies has an X whose value was determined as it was cast (like Earthquake does), the copy has the same value of X."
    );
    copy_keeps_choices(&REVERBERATE, Original::BlazeX2);
    ruling!(
        "Storm King's Thunder",
        "If the spell Storm King's Thunder copies has an X whose value was determined as it was cast (like Fireball does), the copy has the same value of X."
    );
    copy_keeps_choices(&STORM_KINGS_THUNDER, Original::BlazeX2);
    ruling!(
        "Wild Ricochet",
        "If the spell Wild Ricochet copies has an X whose value was determined as it was cast (like Volcanic Geyser does), the copy has the same value of X."
    );
    copy_keeps_choices(&WILD_RICOCHET, Original::BlazeX2);
    ruling!(
        "Howl of the Horde",
        "If the spell being copied has an X whose value was determined as it was cast (like Crater's Claws has), each copy will have the same value of X."
    );
    copy_keeps_choices(&HOWL_OF_THE_HORDE, Original::BlazeX2);
    ruling!(
        "Meletis Charlatan",
        "If the spell being copied has an X whose value was determined as it was cast (like Curse of the Swine has), the copy has the same value of X."
    );
    copy_keeps_choices(&MELETIS_CHARLATAN, Original::BlazeX2);
    ruling!(
        "Dualcaster Mage",
        "If the spell has an X whose value was determined as it was cast (like Earthquake does), the copy has the same value of X."
    );
    copy_keeps_choices(&DUALCASTER_MAGE, Original::BlazeX2);
    ruling!(
        "Insidious Will",
        "If the spell that's copied has an X whose value was determined as it was cast (like Eliminate the Competition does), the copy will have the same value of X."
    );
    copy_keeps_choices(&INSIDIOUS_WILL, Original::BlazeX2);
    ruling!(
        "Naru Meha, Master Wizard",
        "If the spell that's copied has an X whose value was determined as it was cast (like Jaya's Immolating Inferno does), the copy will have the same value of X."
    );
    copy_keeps_choices(&NARU_MEHA, Original::BlazeX2);
    ruling!(
        "The Mirari Conjecture",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Jaya’s Immolating Inferno does), the copy will have the same value of X."
    );
    copy_keeps_choices(&MIRARI_CONJECTURE, Original::BlazeX2);
    ruling!(
        "Swarm Intelligence",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Torment of Hailfire does), the copy will have the same value of X."
    );
    copy_keeps_choices(&SWARM_INTELLIGENCE, Original::BlazeX2);
    ruling!(
        "Echo Mage",
        "If the spell the ability copies has an X whose value was determined as it was cast (like Earthquake does), each copy has the same value of X as the original spell."
    );
    copy_keeps_choices(&ECHO_MAGE, Original::BlazeX2);
}

// --- The modes ------------------------------------------------------------------------------

#[test]
fn copies_have_the_same_modes() {
    cr!("707.10", "700.2");
    ruling!(
        "Cloven Casting",
        "If a modal spell is copied, the copy will have the same mode as the original."
    );
    copy_keeps_choices(&CLOVEN_CASTING, Original::CharmMode1);
    ruling!(
        "Display of Power",
        "If a spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&DISPLAY_OF_POWER, Original::CharmMode1);
    ruling!(
        "Chandra, the Firebrand",
        "If the spell Chandra copies is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    copy_keeps_choices(&CHANDRA, Original::CharmMode1);
    ruling!(
        "Reverberate",
        "If the spell Reverberate copies is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    copy_keeps_choices(&REVERBERATE, Original::CharmMode1);
    ruling!(
        "Storm King's Thunder",
        "If the spell Storm King's Thunder's ability copies is modal (that is, it has a bulleted list of choices), the copies will have the same mode(s). You can't choose different ones."
    );
    copy_keeps_choices(&STORM_KINGS_THUNDER, Original::CharmMode1);
    ruling!(
        "Wild Ricochet",
        "If the spell Wild Ricochet copies is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    copy_keeps_choices(&WILD_RICOCHET, Original::CharmMode1);
    ruling!(
        "Howl of the Horde",
        "If the spell being copied is modal (that is, it says \"Choose one —\" or the like), each copy will have the same mode. You can't choose a different one."
    );
    copy_keeps_choices(&HOWL_OF_THE_HORDE, Original::CharmMode1);
    ruling!(
        "Meletis Charlatan",
        "If the spell being copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. Its controller can't choose a different one."
    );
    copy_keeps_choices(&MELETIS_CHARLATAN, Original::CharmMode1);
    ruling!(
        "Dualcaster Mage",
        "If the spell is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    copy_keeps_choices(&DUALCASTER_MAGE, Original::CharmMode1);
    ruling!(
        "Thousand-Year Storm",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copies will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&THOUSAND_YEAR_STORM, Original::CharmMode1);
    ruling!(
        "Kalamax, the Stormsire",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy created by Kalamax's first ability will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&KALAMAX, Original::CharmMode1);
    ruling!(
        "Najal, the Storm Runner",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy created by Najal's delayed triggered ability will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&NAJAL, Original::CharmMode1);
    ruling!(
        "Twinferno",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copy created by Twinferno's delayed triggered ability will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&TWINFERNO, Original::CharmMode1);
    ruling!(
        "Jin-Gitaxias, Progress Tyrant",
        "If the spell that's copied is modal (that is, it says “Choose one —” or the like), the copy created by Jin-Gitaxias's first ability will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&JIN_GITAXIAS, Original::CharmMode1);
    ruling!(
        "Double Vision",
        "If the spell that's copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    copy_keeps_choices(&DOUBLE_VISION, Original::CharmMode1);
    ruling!(
        "Insidious Will",
        "If the spell that's copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. A different mode cannot be chosen."
    );
    copy_keeps_choices(&INSIDIOUS_WILL, Original::CharmMode1);
    ruling!(
        "Bonus Round",
        "If the spell that’s copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. Its controller can’t choose a different one."
    );
    copy_keeps_choices(&BONUS_ROUND, Original::CharmMode1);
    ruling!(
        "Echo Mage",
        "If the spell the ability copies is modal (that is, it says “Choose one —” or the like), each copy will have the same mode as the original spell. You can’t choose a different one."
    );
    copy_keeps_choices(&ECHO_MAGE, Original::CharmMode1);
    // Lithoform Engine's spell-copying ability (its ability-copying ability and permanent
    // spells are in `r_p028_interactions.rs`).
    copy_keeps_choices(&LITHOFORM_ENGINE, Original::CharmMode1);
    copy_keeps_choices(&MIRARI, Original::CharmMode1);
}

#[test]
fn melek_s_copy_of_a_spell_cast_from_the_library_keeps_x_and_modes() {
    cr!("707.10", "107.3", "700.2", "601.2");
    ruling!(
        "Melek, Izzet Paragon",
        "If the spell being copied has an X whose value was determined as it was cast (like Debt to the Deathless has), the copy has the same value of X."
    );
    ruling!(
        "Melek, Izzet Paragon",
        "If the spell being copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. You can’t choose a different one."
    );
    supported("Melek, Izzet Paragon");
    // "You may cast instant and sorcery spells from the top of your library. Whenever you
    // cast an instant or sorcery spell from your library, copy it."
    for orig in [Original::BlazeX2, Original::CharmMode1] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        t.battlefield(P0, "Melek, Izzet Paragon");
        let spell = match orig {
            Original::BlazeX2 => {
                t.lands(P0, "Mountain", 1);
                t.lands(P0, "Wastes", 2);
                let card = t.library_top(P0, "Blaze");
                t.cast(P0, card).x(2).target(P1).go()
            }
            Original::CharmMode1 => {
                lands_for_cost(&mut t, P0, "Izzet Charm");
                let card = t.library_top(P0, "Izzet Charm");
                t.cast(P0, card).modes(&[1]).target(giant).go()
            }
        };
        keep_copy_targets(&mut t, P0);
        let copies = resolve_until_copies(&mut t);
        assert_eq!(copies.len(), 1);
        match orig {
            Original::BlazeX2 => assert_eq!(x_of(&t, copies[0]), x_of(&t, spell)),
            Original::CharmMode1 => assert_eq!(chosen_modes(&t, copies[0]), vec![1]),
        }
        t.resolve_all();
        match orig {
            Original::BlazeX2 => assert_eq!(t.life(P1), 16),
            Original::CharmMode1 => assert!(!t.on_battlefield(giant)),
        }
    }
}

#[test]
fn primal_wellspring_s_copy_has_the_same_mode() {
    cr!("707.10", "700.2", "106.6");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "If the copied spell is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    supported("Primal Amulet // Primal Wellspring");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let well = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    // {R} from the Wellspring, {U} from an Island.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    assert!(tap_for_mana(&mut t, P0, well, "Add one mana"));
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Izzet Charm");
    t.cast(P0, card).modes(&[1]).target(giant).go();
    keep_copy_targets(&mut t, P0);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(chosen_modes(&t, copies[0]), vec![1]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}
