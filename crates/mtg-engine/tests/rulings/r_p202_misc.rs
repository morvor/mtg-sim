//! Rulings batch P202 — bestow (CR 702.103) on Eidolon of Countless Battles, changeling
//! (CR 702.73) on Formless Genesis, and instants and sorceries whose "Whenever ... this
//! turn, ..." creates a delayed triggered ability (CR 603.7b).

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_bestowed_eidolon_counts_as_an_aura_not_a_creature() {
    cr!("702.103b", "613.4c");
    ruling!(
        "Eidolon of Countless Battles",
        "A permanent with bestow is either a creature or an Aura, not both (although it's an enchantment either way). It will contribute just +1/+1 toward the bonus given by Eidolon of Countless Battles."
    );
    supported("Eidolon of Countless Battles");
    // "This creature and enchanted creature each get +1/+1 for each creature you control
    // and +1/+1 for each Aura you control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let c = t.hand(P0, "Eidolon of Countless Battles");
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    let eidolon = t.named_on_battlefield("Eidolon of Countless Battles")[0];
    let o = t.obj_now(eidolon);
    assert!(o.chars.has_subtype("Aura") && !o.is(CardType::Creature));
    assert!(o.is(CardType::Enchantment));
    // One creature (the Bears) and one Aura (the Eidolon): +2/+2.
    assert_eq!(t.pt(bears), (4, 4));

    // Unattached, it's a creature and not an Aura: it counts once, as a creature.
    destroy(&mut t, bears);
    let o = t.obj_now(eidolon);
    assert!(o.is(CardType::Creature) && !o.chars.has_subtype("Aura"));
    assert_eq!(t.pt(eidolon), (1, 1));
}

#[test]
fn formless_genesis_is_every_creature_type_in_every_zone() {
    cr!("702.73a", "604.3", "205.3m");
    ruling!(
        "Formless Genesis",
        "Changeling is a characteristic-defining ability. It functions in all zones."
    );
    ruling!(
        "Formless Genesis",
        "The subtype Shapeshifter that appears on the type line is mostly there to reinforce the flavor. A spell or token with changeling is just as much an Elf, a Dwarf, a Sliver, a Goat, a Coward, and a Zombie as it is a Shapeshifter."
    );
    // (Formless Genesis's changeling compiles; its token-making effect doesn't.)
    let types = ["Shapeshifter", "Elf", "Dwarf", "Sliver", "Goat", "Coward", "Zombie"];
    let mut t = TestGame::new(2);
    let in_library = t.library_top(P0, "Formless Genesis");
    let in_graveyard = t.graveyard(P0, "Formless Genesis");
    let in_exile = t.exile(P0, "Formless Genesis");
    let in_hand = t.hand(P0, "Formless Genesis");
    t.g.recompute();
    for id in [in_library, in_graveyard, in_exile, in_hand] {
        let o = t.obj(id);
        for ty in types {
            assert!(o.chars.has_subtype(ty), "{ty} in {:?}", o.zone);
        }
    }
    // As a spell, too.
    t.lands(P0, "Forest", 3);
    let spell = t.cast(P0, in_hand).go();
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    for ty in types {
        assert!(t.obj(spell).chars.has_subtype(ty));
    }
    assert!(!t.obj(spell).chars.has_subtype("Aura"));
}

// An instant or sorcery whose text is "Whenever ... this turn, ..." creates a delayed
// triggered ability as it resolves (CR 603.7b).

#[test]
fn glimpse_of_natures_trigger_resolves_before_the_spell_even_if_its_countered() {
    cr!("603.7b", "603.3");
    ruling!(
        "Glimpse of Nature",
        "The delayed triggered ability created by Glimpse of Nature resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Glimpse of Nature");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Glimpse of Nature");
    let glimpse = t.hand(P0, "Glimpse of Nature");
    t.cast(P0, glimpse).go();
    t.resolve_all();
    // A noncreature spell doesn't trigger it.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "delayed trigger"), 0);
    t.resolve_all();
    // A creature spell does; the trigger is above the spell.
    give_mana_for(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, bears).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "delayed trigger"), 1);
    assert_ne!(*t.g.stack.last().unwrap(), spell);
    // P1 counters the creature spell; the trigger still resolves.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(triggers_on_stack(&t, "delayed trigger"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // It lasts only this turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    give_mana_for(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "delayed trigger"), 0);
}

#[test]
fn theoretical_duplication_copies_only_opponents_nontoken_creatures() {
    cr!("603.7b", "707.2");
    supported("Theoretical Duplication");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Theoretical Duplication");
    let c = t.hand(P0, "Theoretical Duplication");
    t.cast(P0, c).go();
    t.resolve_all();
    t.enter(P1, "Hill Giant");
    t.enter(P0, "Grizzly Bears");
    create_token(&mut t, P1, "Soldier");
    t.resolve_all();
    let giants = t.named_on_battlefield("Hill Giant");
    assert_eq!(giants.len(), 2);
    let copy = giants.iter().find(|g| t.obj(**g).controller == P0).unwrap();
    assert!(t.obj(*copy).is_token());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Soldier").len(), 1);
}

#[test]
fn battle_cry_gives_each_blocker_plus_zero_plus_one() {
    cr!("603.7b", "509.1");
    supported("Battle Cry");
    let mut t = TestGame::new(2);
    let blocker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, mtg_engine::turn::Step::BeginningOfCombat);
    give_mana_for(&mut t, P0, "Battle Cry");
    let c = t.hand(P0, "Battle Cry");
    t.cast(P0, c).go();
    t.resolve_all();
    t.attack(&[(attacker, Entity::Player(P0))], &[(blocker, attacker)]);
    // The attacking Bears' 2 damage doesn't kill the 2/3 blocker.
    assert!(t.on_battlefield(blocker));
    assert_eq!(t.pt(blocker), (2, 3));
    assert_eq!(t.pt(idle), (2, 2));
}
