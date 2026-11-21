//! Rulings batch P164 — Adventure synergies (Mysterious Pathlighter; Edgewall Innkeeper,
//! Garenbrig Squire and Wandermare's cast triggers) and basic-land synergies (Imperiosaur's
//! "Spend only mana produced by basic lands"; Hermit Druid).

use crate::r_p164_common::*;
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn mysterious_pathlighter_adds_a_counter_to_creatures_with_an_adventure() {
    cr!("614.1c", "715.2a");
    ruling!(
        "Mysterious Pathlighter",
        "A creature with an Adventure you control that would enter the battlefield with no +1/+1 counters on it enters with one +1/+1 counter on it."
    );
    supported("Mysterious Pathlighter");
    supported("Bonecrusher Giant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mysterious Pathlighter");
    t.lands(P0, "Mountain", 3);
    let giant = t.hand(P0, "Bonecrusher Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    let giant = t.named_on_battlefield("Bonecrusher Giant")[0];
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert_eq!(t.pt(giant), (5, 4));
    // A creature without an Adventure gets none.
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn adventure_cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a", "715.2a");
    ruling!(
        "Edgewall Innkeeper",
        "Edgewall Innkeeper's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Garenbrig Squire",
        "Garenbrig Squire's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Wandermare",
        "Wandermare's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Edgewall Innkeeper");
    supported("Garenbrig Squire");
    supported("Wandermare");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewall Innkeeper");
    let squire = t.battlefield(P0, "Garenbrig Squire");
    let mare = t.battlefield(P0, "Wandermare");
    t.lands(P0, "Mountain", 3);
    let giant = t.hand(P0, "Bonecrusher Giant");
    let spell = t.cast(P0, giant).go();
    t.settle();
    // Three triggers above the spell.
    assert_eq!(t.stack_len(), 4);
    assert_eq!(*t.g.stack.first().unwrap(), spell);
    // P1 counters the Giant in response.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bonecrusher Giant"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(squire), (3, 3));
    assert_eq!(t.counters(mare, counters::PLUS1), 1);
}

#[test]
fn imperiosaur_needs_mana_produced_by_basic_lands() {
    cr!("609.4b", "106.12a", "605.1b", "605.4a");
    ruling!(
        "Imperiosaur",
        "An effect that allows you to spend mana as though it were mana of any type to cast Imperiosaur won't allow you to spend mana from a source other than a basic land to cast it."
    );
    ruling!(
        "Imperiosaur",
        "If a basic land gains a mana ability, mana produced with that ability can be spent to cast Imperiosaur."
    );
    ruling!(
        "Imperiosaur",
        "If a triggered mana ability produces mana when a basic land is tapped for mana, that additional mana is produced by the source of the triggered mana ability, not by that basic land."
    );
    supported("Imperiosaur");
    supported("Vizier of the Menagerie");
    supported("Chromatic Lantern");
    supported("Wild Growth");
    // Vizier of the Menagerie ("You can spend mana of any type to cast creature spells"):
    // four Plains pay for it, but Llanowar Elves' mana can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Menagerie");
    t.lands(P0, "Plains", 3);
    t.battlefield(P0, "Llanowar Elves");
    let dino = t.hand(P0, "Imperiosaur");
    assert!(t.cast(P0, dino).try_go().is_err());
    t.lands(P0, "Plains", 1);
    let dino = t.g.current(dino);
    assert!(t.cast(P0, dino).try_go().is_ok());
    // Plains with Chromatic Lantern's "{T}: Add one mana of any color.": {G}{G} from
    // basic lands.
    let mut t = TestGame::new(2);
    let lantern = t.battlefield(P0, "Chromatic Lantern");
    t.g.objects[lantern.0 as usize].tapped = true;
    t.lands(P0, "Plains", 4);
    let dino = t.hand(P0, "Imperiosaur");
    assert!(t.cast(P0, dino).try_go().is_ok());
    // Wild Growth's extra {G} is produced by Wild Growth: three Forests aren't enough.
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 3);
    attach_new(&mut t, P0, "Wild Growth", forests[0]);
    let dino = t.hand(P0, "Imperiosaur");
    assert!(t.cast(P0, dino).try_go().is_err());
    // A four-mana green creature without the restriction can use it.
    let unicorn = t.hand(P0, "Prized Unicorn");
    assert!(t.cast(P0, unicorn).try_go().is_ok());
}

#[test]
fn hermit_druid_with_no_basic_land_mills_the_whole_library() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Hermit Druid",
        "If you reveal all of the cards in your library without revealing any basic land cards, you’ll put all of the revealed cards into your graveyard."
    );
    supported("Hermit Druid");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Hermit Druid");
    set_library(&mut t, P0, &["Grizzly Bears", "Lightning Bolt", "Mutavault"]);
    t.lands(P0, "Forest", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, druid, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.hand_size(P0), hand);
}
