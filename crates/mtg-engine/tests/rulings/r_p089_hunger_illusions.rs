//! Rulings batch P089 — "hunger" (whenever a creature dealt damage by this creature this
//! turn dies) and Illusions (when this creature becomes the target of a spell or
//! ability, sacrifice it), plus Eternal Scourge.

use crate::r_p089_common::*;
use crate::r_s01_common::triggers_on_stack;
use crate::r_s06_common::damage;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Hunger
// ---------------------------------------------------------------------------------------

#[test]
fn damaged_earlier_this_turn_then_killed_by_something_else() {
    cr!("603.2", "603.10a", "700.4");
    ruling!(
        "Predator Ooze",
        "Each time a creature dies, check whether Predator Ooze had dealt any damage to it at any time during that turn."
    );
    ruling!(
        "Sengir Vampire",
        "Each time a creature is put into a graveyard from the battlefield, check whether Sengir Vampire had dealt any damage to it at any time during that turn."
    );
    ruling!(
        "Sengir Vampire",
        "If Sengir Vampire deals nonlethal damage to a creature and then a different effect or damage source causes that creature to be put into a graveyard later in the turn"
    );
    ruling!(
        "Blood Cultist",
        "Each time a creature is put into a graveyard from the battlefield, check whether Blood Cultist had dealt any damage to it at any time during that turn."
    );
    for name in ["Predator Ooze", "Sengir Vampire", "Blood Cultist"] {
        supported(name);
        let mut t = TestGame::new(2);
        let hunter = t.battlefield(P0, name);
        // Nonlethal damage to P1's creature and to P0's own.
        let theirs = t.battlefield(P1, "Hill Giant");
        let mine = t.battlefield(P0, "Grizzly Bears");
        let untouched = t.battlefield(P1, "Grizzly Bears");
        damage(&mut t, hunter, 1, theirs);
        damage(&mut t, hunter, 1, mine);
        assert_eq!(t.counters(hunter, "+1/+1"), 0, "{name}");
        // Later, other sources kill them (and a creature it didn't damage).
        cast_and_resolve(&mut t, P0, "Shock", &[obj(theirs)]);
        cast_and_resolve(&mut t, P1, "Shock", &[obj(mine)]);
        cast_and_resolve(&mut t, P1, "Shock", &[obj(untouched)]);
        assert!(!t.on_battlefield(theirs) && !t.on_battlefield(mine));
        assert!(!t.on_battlefield(untouched));
        assert_eq!(t.counters(hunter, "+1/+1"), 2, "{name}");
    }
}

#[test]
fn blood_cultist_counts_combat_damage() {
    cr!("510.2", "603.2");
    ruling!(
        "Blood Cultist",
        "(This includes combat damage.) If so, Blood Cultist's second ability will trigger."
    );
    supported("Blood Cultist");
    let mut t = TestGame::new(2);
    let cultist = t.battlefield(P0, "Blood Cultist");
    put_counters(&mut t, cultist, "+1/+1", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(cultist, player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(bears, cultist)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.counters(cultist, "+1/+1"), 3);
}

#[test]
fn dying_at_the_same_time_triggers_but_does_nothing() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Blood Cultist",
        "If Blood Cultist and a creature it dealt damage to are both put into a graveyard at the same time, Blood Cultist's ability will trigger, but it will do nothing when it resolves."
    );
    ruling!(
        "Sengir Vampire",
        "If Sengir Vampire and a creature it dealt damage to are both put into a graveyard at the same time, Sengir Vampire's ability will trigger, but it will do nothing when it resolves."
    );
    for name in ["Blood Cultist", "Sengir Vampire"] {
        supported(name);
        let mut t = TestGame::new(2);
        let hunter = t.battlefield(P0, name);
        let a = t.battlefield(P1, "Hill Giant");
        let b = t.battlefield(P1, "Serra Angel");
        damage(&mut t, hunter, 1, a);
        damage(&mut t, hunter, 1, b);
        cast_new(&mut t, P0, "Wrath of God", &[]);
        t.resolve();
        assert!(!t.on_battlefield(hunter));
        // One trigger for each damaged creature.
        assert_eq!(triggers_on_stack(&t, "dealt damage by"), 2, "{name}");
        t.resolve_all();
        let gone = t.g.current(hunter);
        assert_eq!(t.g.obj(gone).zone, Zone::Graveyard(P0));
        assert_eq!(t.counters(hunter, "+1/+1"), 0, "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Illusions
// ---------------------------------------------------------------------------------------

const ILLUSIONS: [&str; 7] = [
    "Frost Walker",
    "Illusionary Servant",
    "Phantasmal Abomination",
    "Phantom Beast",
    "Gossamer Phantasm",
    "Skulking Ghost",
    "Phantasmal Shieldback",
];

#[test]
fn targeted_illusion_is_sacrificed_first_and_the_spell_fizzles() {
    cr!("603.3", "608.2b", "701.21a");
    ruling!(
        "Frost Walker",
        "Unless the spell or ability has another target, it won’t resolve as it tries to resolve for having no legal targets."
    );
    ruling!(
        "Illusionary Servant",
        "If Illusionary Servant becomes the target of a spell or ability, Illusionary Servant’s ability triggers and is put on the stack on top of that spell or ability."
    );
    ruling!(
        "Phantasmal Abomination",
        "If Phantasmal Abomination becomes the target of a spell or ability, Phantasmal Abomination’s ability triggers and goes on the stack on top of that spell or ability."
    );
    ruling!(
        "Phantom Beast",
        "If Phantom Beast becomes the target of a spell or ability, Phantom Beast’s ability triggers and goes on the stack on top of that spell or ability."
    );
    ruling!(
        "Gossamer Phantasm",
        "If the spell that targets Gossamer Phantasm has no other targets, it won't resolve"
    );
    ruling!(
        "Skulking Ghost",
        "If the spell that targets Skulking Ghost has no other targets, it won't resolve"
    );
    ruling!(
        "Phantasmal Shieldback",
        "Unless the spell or ability targeting Phantasmal Shieldback has one or more other targets, it will then be removed from the stack when it tries to resolve. None of its effects will happen."
    );
    for name in ILLUSIONS {
        supported(name);
        // A spell with a single target: the trigger goes on top, resolves first, and the
        // spell does nothing.
        let mut t = TestGame::new(2);
        let ill = t.battlefield(P1, name);
        let shock = cast_new(&mut t, P0, "Shock", &[obj(ill)]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}");
        assert_ne!(*t.g.stack.last().unwrap(), shock, "{name}: trigger on top");
        t.resolve();
        assert!(!t.on_battlefield(ill), "{name}: sacrificed");
        assert!(t.g.stack.contains(&shock));
        t.resolve_all();
        assert_eq!(t.life(P1), 20);
        assert!(t.in_graveyard(P0, "Shock"));
        // An ability with a single target: likewise.
        let mut t = TestGame::new(2);
        let ill = t.battlefield(P1, name);
        let tim = t.battlefield(P0, "Prodigal Sorcerer");
        t.activate(P0, tim, 0, &[obj(ill)]).unwrap();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}");
        t.resolve_all();
        assert!(!t.on_battlefield(ill), "{name}");
        // A spell with another target still affects that one: Agony Warp gives the
        // Bears -3/-0 even though the Illusion is gone.
        let mut t = TestGame::new(2);
        let ill = t.battlefield(P1, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, "Agony Warp", &[obj(bears), obj(ill)]);
        t.resolve_all();
        assert!(!t.on_battlefield(ill), "{name}");
        assert_eq!(t.pt(bears), (-1, 2), "{name}");
    }
}

#[test]
fn illusion_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Gossamer Phantasm",
        "Gossamer Phantasm's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Skulking Ghost",
        "Skulking Ghost's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Phantasmal Shieldback",
        "Phantasmal Shieldback's first ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered or otherwise leaves the stack."
    );
    for name in ["Gossamer Phantasm", "Skulking Ghost", "Phantasmal Shieldback"] {
        supported(name);
        let mut t = TestGame::new(2);
        let ill = t.battlefield(P1, name);
        let shock = cast_new(&mut t, P0, "Shock", &[obj(ill)]);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        // P1 counters the Shock above the trigger.
        cast_new(&mut t, P1, "Counterspell", &[obj(shock)]);
        t.resolve();
        assert!(!t.g.stack.contains(&shock));
        let hand = t.hand_size(P1);
        t.resolve_all();
        assert!(!t.on_battlefield(ill), "{name}");
        if name == "Phantasmal Shieldback" {
            // Its dies trigger draws a card.
            assert_eq!(t.hand_size(P1), hand + 1);
        }
    }
}

#[test]
fn eternal_scourge_triggers_only_on_the_battlefield() {
    cr!("603.6d", "113.6");
    ruling!(
        "Eternal Scourge",
        "Eternal Scourge's triggered ability only triggers while it's on the battlefield. For example, Convolute cast targeting Eternal Scourge won't cause its ability to trigger."
    );
    supported("Eternal Scourge");
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Eternal Scourge", &[]);
    cast_new(&mut t, P1, "Convolute", &[obj(spell)]);
    t.settle();
    // Only Convolute and the Scourge spell: nothing triggered.
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Eternal Scourge"));
    assert!(!t.in_exile("Eternal Scourge"));
}

#[test]
fn eternal_scourge_is_exiled_even_if_the_spell_is_countered() {
    cr!("603.3");
    ruling!(
        "Eternal Scourge",
        "You'll exile Eternal Scourge even if the spell or ability that targets it is countered."
    );
    supported("Eternal Scourge");
    let mut t = TestGame::new(2);
    let scourge = t.battlefield(P0, "Eternal Scourge");
    let shock = cast_new(&mut t, P1, "Shock", &[obj(scourge)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_new(&mut t, P0, "Counterspell", &[obj(shock)]);
    t.resolve_all();
    assert!(t.in_exile("Eternal Scourge"));
    // P0's own spell doesn't make it trigger ("an opponent controls").
    let mut t = TestGame::new(2);
    let scourge = t.battlefield(P0, "Eternal Scourge");
    cast_new(&mut t, P0, "Giant Growth", &[obj(scourge)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn eternal_scourge_cast_from_exile_follows_normal_rules() {
    cr!("601.3", "307.1", "601.2f");
    ruling!(
        "Eternal Scourge",
        "You must follow the normal timing permissions and restrictions and pay its cost to cast Eternal Scourge from exile."
    );
    supported("Eternal Scourge");
    let mut t = TestGame::new(2);
    let scourge = t.exile(P0, "Eternal Scourge");
    // Without mana: can't.
    assert!(t.cast(P0, scourge).try_go().is_err());
    t.lands(P0, "Wastes", 3);
    // At instant speed (P1's turn): can't.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P0, scourge).try_go().is_err());
    // In P0's main phase with the mana: can, paying it.
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, scourge).try_go().is_ok());
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
}
