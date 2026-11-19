//! Rulings batch P191 — what spells and abilities do as they resolve: zone changes making
//! new objects (CR 400.7), copies and their copiable values (CR 707), "destroy" and what
//! counts as destroyed (CR 701.8), {X} of cards not on the stack (CR 107.3f), searches
//! for names (CR 201.2, 701.23), mass effects whose set is fixed (CR 611.2c), choices in
//! turn order (CR 101.4), mana abilities (CR 605), and tokens' names (CR 111.4).

use crate::r_p003_common::put;
use crate::r_p022_common::choose_color;
use crate::r_p076_common::mana;
use crate::r_p191_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported, with_subtype};
use crate::r_s05_common::enter;
use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_resolve(t: &mut TestGame, card: ObjectId) {
    t.cast(P0, card).go();
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Another Round
// ---------------------------------------------------------------------------------------

#[test]
fn another_round_with_x_zero_blinks_once() {
    cr!("107.3", "608.2c");
    ruling!(
        "Another Round",
        "If X is 0, you’ll perform the process just once when Another Round resolves."
    );
    supported("Another Round");
    for x in [0u32, 1] {
        let mut t = TestGame::new(2);
        let elf = t.battlefield(P0, "Elvish Visionary");
        let c = t.hand(P0, "Another Round");
        mana(&mut t, P0, ManaType::W, 1);
        mana(&mut t, P0, ManaType::C, 2 + 2 * x);
        for _ in 0..=x {
            t.answer_choose_moved(P0, &[obj(elf)]);
        }
        let hand = t.hand_size(P0);
        t.cast(P0, c).x(x as i64).go();
        t.resolve_all();
        // One Elvish Visionary draw per time it entered (minus the spell itself).
        assert_eq!(t.hand_size(P0), hand - 1 + 1 + x as usize, "X = {x}");
        assert!(t.on_battlefield(elf));
    }
}

#[test]
fn another_round_returns_a_new_object() {
    cr!("400.7", "303.4d", "301.5c", "704.5m", "704.5n");
    ruling!(
        "Another Round",
        "Once an exiled creature returns, it’s considered a new object with no relation to the object that it was. Auras attached to the exiled creature will be put into their owners’ graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    let equip = attach_new(&mut t, P0, "Bonesplitter", bears);
    put(&mut t, bears, "+1/+1", 2);
    let c = t.hand(P0, "Another Round");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, c).x(0).go();
    t.resolve_all();
    let now = t.g.current(bears);
    assert_ne!(now, bears);
    assert!(t.on_battlefield(now));
    assert_eq!(t.counters(now, "+1/+1"), 0);
    assert_eq!(t.pt(now), (2, 2));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(t.zone(aura), Zone::Graveyard(P0));
    assert!(t.on_battlefield(equip));
    assert_eq!(attached_to(&t, equip), None);
}

// ---------------------------------------------------------------------------------------
// Hour of Eternity
// ---------------------------------------------------------------------------------------

/// P0 casts Hour of Eternity exiling the given creature cards from their graveyard; it
/// resolves (with its tokens' triggers). Returns the tokens.
fn hour_of_eternity(t: &mut TestGame, cards: &[ObjectId]) -> Vec<ObjectId> {
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    let c = t.hand(P0, "Hour of Eternity");
    let x = cards.len() as u32;
    mana(t, P0, ManaType::U, 3);
    mana(t, P0, ManaType::C, 2 * x);
    let targets: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_targets(P0, &targets);
    t.cast(P0, c).x(x as i64).go();
    t.resolve_all();
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| !before.contains(id))
        .collect()
}

#[test]
fn hour_of_eternity_tokens_have_the_cards_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Hour of Eternity",
        "If a card copied by one of the tokens had any “when [this permanent] enters the battlefield” abilities, then the token also has those abilities and will trigger them when it's created. Similarly, any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities that the token has copied will also work."
    );
    supported("Hour of Eternity");
    let mut t = TestGame::new(2);
    let elf = t.graveyard(P0, "Elvish Visionary");
    let deity = t.graveyard(P0, "Deity of Scars");
    let hand = t.hand_size(P0);
    let tokens = hour_of_eternity(&mut t, &[elf, deity]);
    assert_eq!(tokens.len(), 2);
    // Elvish Visionary's copy drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
    // Deity of Scars' copy entered with two -1/-1 counters: a 4/4 with them is 2/2.
    let d = *tokens
        .iter()
        .find(|id| t.obj(**id).chars.name == "Deity of Scars")
        .unwrap();
    assert_eq!(t.counters(d, "-1/-1"), 2);
    assert_eq!(t.pt(d), (2, 2));
}

#[test]
fn hour_of_eternity_tokens_copiable_values_are_black_zombie_four_four() {
    cr!("707.9b", "707.2", "707.3");
    ruling!(
        "Hour of Eternity",
        "The tokens are Zombies instead of their other types (unlike Zombies created by an eternalize ability) and are black instead of their other colors. Their power and toughness are 4/4. These are copiable values of the tokens that other effects may copy."
    );
    let mut t = TestGame::new(2);
    let elf = t.graveyard(P0, "Elvish Visionary");
    let tokens = hour_of_eternity(&mut t, &[elf]);
    assert_eq!(tokens.len(), 1);
    let tok = tokens[0];
    let check = |t: &TestGame, id: ObjectId| {
        let o = t.obj_now(id);
        assert_eq!(o.chars.name, "Elvish Visionary");
        assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
        assert!(o.chars.subtypes.iter().any(|s| s == "Zombie"));
        assert!(!o.chars.subtypes.iter().any(|s| s == "Elf"));
        assert!(o.is(CardType::Creature));
        assert_eq!((o.power(), o.toughness()), (4, 4));
    };
    check(&t, tok);
    // A Clone copying the token copies those values.
    t.answer_choose(P0, &[obj(tok)]);
    let clone = enter(&mut t, P0, "Clone");
    t.resolve_all();
    check(&t, clone);
}

// ---------------------------------------------------------------------------------------
// Doppelgang
// ---------------------------------------------------------------------------------------

#[test]
fn doppelgang_copies_copiable_values_only() {
    cr!("707.2", "707.3", "707.4");
    ruling!(
        "Doppelgang",
        "If the copied permanent is copying something else, then the tokens enter the battlefield as whatever that permanent copied."
    );
    ruling!(
        "Doppelgang",
        "The tokens copy exactly what was printed on the original permanent and nothing else (unless that permanent is itself copying something else; see below). They don't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Doppelgang");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    let clone = enter(&mut t, P1, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    put(&mut t, giant, "+1/+1", 1);
    attach_new(&mut t, P1, "Holy Strength", giant);
    crate::r_p003_common::giant_growth(&mut t, giant);
    assert_eq!(t.pt(giant), (8, 9));
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    let c = t.hand(P0, "Doppelgang");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 6);
    t.answer_targets(P0, &[obj(clone), obj(giant)]);
    t.cast(P0, c).x(2).go();
    t.resolve_all();
    let tokens: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| !before.contains(id) && t.obj(*id).controller == P0)
            .collect();
    assert_eq!(tokens.len(), 4);
    let named = |n: &str| {
        tokens
            .iter()
            .copied()
            .filter(|id| t.obj(*id).chars.name == n)
            .collect::<Vec<_>>()
    };
    assert_eq!(named("Grizzly Bears").len(), 2);
    let giants = named("Hill Giant");
    assert_eq!(giants.len(), 2);
    for g in giants {
        assert_eq!(t.pt(g), (3, 3));
        assert!(!t.obj(g).tapped);
        assert_eq!(t.counters(g, "+1/+1"), 0);
    }
}

// ---------------------------------------------------------------------------------------
// Death Begets Life, Gaze of Granite, Immortal Servitude, Time Wipe
// ---------------------------------------------------------------------------------------

#[test]
fn death_begets_life_counts_destroyed_permanents_wherever_they_go() {
    cr!("701.8a", "701.8b", "702.12b");
    ruling!(
        "Death Begets Life",
        "If a permanent destroyed this way goes to a zone other than a graveyard (for example, if it has a finality counter on it), it still counts as being destroyed and you’ll still draw a card."
    );
    ruling!(
        "Death Begets Life",
        "Permanents with indestructible aren’t destroyed this way."
    );
    supported("Death Begets Life");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, "finality", 1);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Glorious Anthem");
    let myr = t.battlefield(P0, "Darksteel Myr");
    let c = t.hand(P0, "Death Begets Life");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 5);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, c);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    assert!(t.on_battlefield(myr));
    // Three destroyed (the Myr isn't): three cards.
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}

#[test]
fn x_in_the_mana_cost_of_a_card_not_on_the_stack_is_zero() {
    cr!("107.3f", "202.3e");
    ruling!(
        "Gaze of Granite",
        "If a permanent has {X} in its mana cost, the value of that X is 0."
    );
    ruling!(
        "Immortal Servitude",
        "If a creature card in a graveyard has X in its mana cost, that X is considered to be 0."
    );
    supported("Gaze of Granite");
    supported("Immortal Servitude");
    // Gaze of Granite with X = 0 destroys Hangarback Walker ({X}{X}).
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Hangarback Walker");
    put(&mut t, walker, "+1/+1", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Gaze of Granite");
    mana(&mut t, P0, ManaType::B, 2);
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, c).x(0).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hangarback Walker"));
    assert!(t.on_battlefield(bears));
    // Immortal Servitude: Rocco, Cabaretti Caterer ({X}{R}{G}{W}) has mana value 3 in the
    // graveyard; Broodlord ({X}{3}{G}) has 4.
    let all = [
        "Rocco, Cabaretti Caterer",
        "Broodlord",
        "Hill Giant",
        "Memnite",
    ];
    for (x, returned) in [
        (3u32, vec!["Rocco, Cabaretti Caterer"]),
        (4, vec!["Broodlord", "Hill Giant"]),
        (0, vec!["Memnite"]),
    ] {
        let mut t = TestGame::new(2);
        for n in all {
            t.graveyard(P0, n);
        }
        let c = t.hand(P0, "Immortal Servitude");
        mana(&mut t, P0, ManaType::W, 3);
        mana(&mut t, P0, ManaType::C, x);
        t.cast(P0, c).x(x as i64).go();
        t.resolve_all();
        for n in all {
            assert_eq!(
                t.named_on_battlefield(n).len(),
                usize::from(returned.contains(&n)),
                "X = {x}: {n}"
            );
        }
    }
}

#[test]
fn time_wipe_without_a_creature_to_return_destroys_all_creatures() {
    cr!("608.2c", "701.8a");
    ruling!(
        "Time Wipe",
        "If you control no creatures, you simply destroy all creatures."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Time Wipe");
    mana(&mut t, P0, ManaType::W, 2);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    cast_resolve(&mut t, c);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Time Wipe"));
}

// ---------------------------------------------------------------------------------------
// Titanic Ultimatum
// ---------------------------------------------------------------------------------------

#[test]
fn titanic_ultimatum_redundant_keywords_and_a_fixed_set() {
    cr!("702.2f", "702.7c", "702.15f", "702.19e", "611.2c");
    ruling!(
        "Titanic Ultimatum",
        "If a creature you control already has lifelink, Titanic Ultimatum will give it a second instance of lifelink; however, multiple instances of lifelink on the same creature are redundant. You still only gain life equal to the amount of damage dealt, regardless of how many instances of lifelink the creature has."
    );
    ruling!(
        "Titanic Ultimatum",
        "Multiple instances of first strike and trample are redundant."
    );
    ruling!(
        "Titanic Ultimatum",
        "The set of creatures affected is fixed as Titanic Ultimatum resolves."
    );
    supported("Titanic Ultimatum");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let knight = t.battlefield(P0, "White Knight");
    let c = t.hand(P0, "Titanic Ultimatum");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::G, 3);
    mana(&mut t, P0, ManaType::W, 2);
    cast_resolve(&mut t, c);
    assert_eq!(t.pt(hawk), (7, 8));
    assert_eq!(t.pt(knight), (7, 7));
    // A creature that comes later doesn't get the bonus.
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
    // Each attacker deals its damage once (first strike twice over is still first
    // strike), and the Nighthawk's lifelink gains 7 once.
    let blocker = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(hawk, pl(P1)), (knight, pl(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(blocker, knight)]);
    // The White Knight tramples over the Hill Giant (3 toughness): 4 to P1.
    assert_eq!(t.life(P1), 20 - 7 - 4);
    // Lifelink from both creatures (the White Knight's is from the Ultimatum): 7 + 7.
    assert_eq!(t.life(P0), 20 + 7 + 7);
    assert!(!t.on_battlefield(blocker));
}

// ---------------------------------------------------------------------------------------
// Clarion Ultimatum
// ---------------------------------------------------------------------------------------

#[test]
fn clarion_ultimatum_searches_for_current_names() {
    cr!("201.2", "707.2", "708.2a", "701.23a", "305.7");
    ruling!(
        "Clarion Ultimatum",
        "If you choose a permanent whose name has been changed by an effect (for example, a Clone that's copying another creature), you'll search for a card with the new name, not one with the original name. Note that changing a land's subtype doesn't change its name."
    );
    supported("Clarion Ultimatum");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let clone = enter(&mut t, P0, "Clone");
    t.resolve_all();
    let forest = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Spreading Seas", forest);
    assert!(t.obj(forest).chars.subtypes.iter().any(|s| s == "Island"));
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Clone", "Forest", "Island"]);
    let c = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(clone), obj(forest)]);
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    cast_resolve(&mut t, c);
    let bf = |id: ObjectId| t.on_battlefield(id);
    assert!(bf(lib[0]), "a Grizzly Bears was found");
    assert!(!bf(lib[1]), "no Clone");
    assert!(bf(lib[2]), "a Forest was found");
    assert!(!bf(lib[3]), "no Island");
    let _ = from;
}

#[test]
fn clarion_ultimatum_face_down_permanent_has_no_name_to_find() {
    cr!("708.2a", "201.2a", "701.23a");
    ruling!(
        "Clarion Ultimatum",
        "If you choose a permanent with no name, such as a face-down creature, no card in your library can be found with that name."
    );
    let mut t = TestGame::new(2);
    let fd = crate::r_s11_common::manifest_card(&mut t, P0, "Grizzly Bears");
    let lib = stack_library(&mut t, P0, &["Grizzly Bears"]);
    let c = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    t.answer_choose(P0, &[obj(fd)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    cast_resolve(&mut t, c);
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
}

#[test]
fn clarion_ultimatum_only_finds_permanent_cards() {
    cr!("701.23a", "110.4", "305.7");
    ruling!(
        "Clarion Ultimatum",
        "If you find a card that isn't a permanent card while searching"
    );
    // A permanent named like an instant (a test card named Lightning Bolt): the
    // Lightning Bolt found stays in the library.
    let mut t = TestGame::new(2);
    let named = t.custom(
        P0,
        crate::r_s01_common::custom_card("Lightning Bolt", "Artifact", "{1}", None, ""),
        Zone::Battlefield,
    );
    let lib = stack_library(&mut t, P0, &["Lightning Bolt"]);
    let c = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    t.answer_choose(P0, &[obj(named)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    cast_resolve(&mut t, c);
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
}

#[test]
fn clarion_ultimatum_auras_enter_attached_to_permanents_already_there() {
    cr!("303.4f", "303.4g", "614.12");
    ruling!(
        "Clarion Ultimatum",
        "The cards all enter at the same time. If one of the cards that's entering is an Aura, it must enter attached to a permanent already on the battlefield. It can't enter attached to another permanent entering via Clarion Ultimatum. If an Aura can't enter this way, it remains in your library."
    );
    // Holy Strength on the Bears: one more Holy Strength and Bears found. With no other
    // creature on the battlefield, the found Holy Strength enters attached to the Bears
    // already there (not the new one).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hs = attach_new(&mut t, P0, "Holy Strength", bears);
    let lib = stack_library(&mut t, P0, &["Holy Strength", "Grizzly Bears"]);
    let c = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    t.answer_choose(P0, &[obj(bears), obj(hs)]);
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    cast_resolve(&mut t, c);
    let new_bears = t.g.current(lib[1]);
    assert!(t.on_battlefield(new_bears));
    let new_hs = t.g.current(lib[0]);
    assert!(t.on_battlefield(new_hs));
    assert_eq!(attached_to(&t, new_hs), Some(obj(bears)));
    // Pacifism chosen with no creature to enchant: the Pacifism found stays.
    let mut t = TestGame::new(2);
    let pac = t.battlefield(P0, "Pacifism");
    let lib = stack_library(&mut t, P0, &["Pacifism", "Grizzly Bears"]);
    let land = t.battlefield(P0, "Plains");
    let c = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    t.answer_choose(P0, &[obj(pac), obj(land)]);
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    cast_resolve(&mut t, c);
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
}

#[test]
fn clarion_ultimatum_five_different_permanents_names_may_repeat() {
    cr!("701.23a", "608.2c");
    ruling!(
        "Clarion Ultimatum",
        "The five permanents you choose must all be different. However, some of them may have the same name as one another. For example, you may choose two Empyrial Archangels and three Plains. If you do, you search your library for up to two more Empyrial Archangels and up to three more Plains and put them onto the battlefield tapped."
    );
    let mut t = TestGame::new(2);
    let mut chosen = vec![];
    for _ in 0..2 {
        chosen.push(obj(t.battlefield(P0, "Empyrial Archangel")));
    }
    for _ in 0..3 {
        chosen.push(obj(t.battlefield(P0, "Plains")));
    }
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Empyrial Archangel",
            "Empyrial Archangel",
            "Empyrial Archangel",
            "Plains",
            "Plains",
            "Plains",
            "Plains",
        ],
    );
    let c = t.hand(P0, "Clarion Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 3);
    mana(&mut t, P0, ManaType::U, 2);
    let from = t.asked().len();
    t.answer_choose(P0, &chosen);
    for _ in 0..5 {
        t.answer_yes(P0, true);
    }
    cast_resolve(&mut t, c);
    // The choice was of five different permanents.
    let five = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .unwrap();
    assert_eq!(five.1, 5);
    let on = |name: &str| {
        lib.iter()
            .filter(|id| t.on_battlefield(**id) && t.obj_now(**id).chars.name == name)
            .count()
    };
    assert_eq!(on("Empyrial Archangel"), 2);
    assert_eq!(on("Plains"), 3);
    for id in &lib {
        if t.on_battlefield(*id) {
            assert!(t.obj_now(*id).tapped);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Genesis Ultimatum
// ---------------------------------------------------------------------------------------

#[test]
fn genesis_ultimatum_auras_attach_without_targeting() {
    cr!("303.4f", "303.4g", "702.11b");
    ruling!(
        "Genesis Ultimatum",
        "If an Aura is put onto the battlefield without being cast, the Aura’s controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, you can’t choose to put it onto the battlefield; in this case, it would be put into your hand along with any other cards that weren’t put onto the battlefield this way."
    );
    supported("Genesis Ultimatum");
    // Pacifism onto P1's hexproof creature.
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P1, "Gladecover Scout");
    let lib = stack_library(&mut t, P0, &["Pacifism", "Grizzly Bears"]);
    let c = t.hand(P0, "Genesis Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::U, 3);
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    t.answer_choose(P0, &[obj(lion)]);
    cast_resolve(&mut t, c);
    let pac = t.g.current(lib[0]);
    assert!(t.on_battlefield(pac));
    assert_eq!(attached_to(&t, pac), Some(obj(lion)));
    // No creature to enchant: the Pacifism goes to the hand.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Pacifism", "Forest"]);
    let c = t.hand(P0, "Genesis Ultimatum");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::U, 3);
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    cast_resolve(&mut t, c);
    assert!(t.in_hand(P0, "Pacifism"));
    assert!(t.on_battlefield(lib[1]) || t.named_on_battlefield("Forest").len() == 1);
}

// ---------------------------------------------------------------------------------------
// Harness Infinity, Reap Intellect
// ---------------------------------------------------------------------------------------

#[test]
fn harness_infinity_exchanges_even_with_an_empty_zone() {
    cr!("701.12a", "514.1");
    ruling!(
        "Harness Infinity",
        "If either your graveyard or your hand has no cards in it, the exchange still happens. If one of those zones has any cards in it, you will move them to the other zone."
    );
    ruling!(
        "Harness Infinity",
        "If you have more cards in hand than your maximum hand size (usually seven) during the cleanup step of your turn, you will need to discard cards down to your maximum hand size."
    );
    supported("Harness Infinity");
    // Empty graveyard: the hand goes to the graveyard.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let c = t.hand(P0, "Harness Infinity");
    mana(&mut t, P0, ManaType::B, 3);
    mana(&mut t, P0, ManaType::G, 3);
    mana(&mut t, P0, ManaType::C, 1);
    cast_resolve(&mut t, c);
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Empty hand, ten cards in the graveyard: ten in hand, then discard to seven.
    let mut t = TestGame::new(2);
    for _ in 0..10 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let c = t.hand(P0, "Harness Infinity");
    mana(&mut t, P0, ManaType::B, 3);
    mana(&mut t, P0, ManaType::G, 3);
    mana(&mut t, P0, ManaType::C, 1);
    cast_resolve(&mut t, c);
    assert_eq!(t.hand_size(P0), 10);
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t.in_exile("Harness Infinity"));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn reap_intellect_searches_only_for_what_it_exiled_and_nobody_acts_meanwhile() {
    cr!("608.2", "701.23a", "117.1");
    ruling!(
        "Reap Intellect",
        "If you don’t exile any cards from the player’s hand, you don’t search that player’s library."
    );
    ruling!(
        "Reap Intellect",
        "Once Reap Intellect begins to resolve, no player can cast spells or activate abilities until it has completely resolved. For example, the opponent can’t cast any cards that would be exiled after their hand is revealed."
    );
    supported("Reap Intellect");
    for exile_one in [false, true] {
        let mut t = TestGame::new(2);
        t.hand(P1, "Shock");
        t.hand(P1, "Cancel");
        t.graveyard(P1, "Shock");
        t.library_top(P1, "Shock");
        let c = t.hand(P0, "Reap Intellect");
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::B, 1);
        mana(&mut t, P0, ManaType::C, 3);
        t.answer_targets(P0, &[pl(P1)]);
        t.cast(P0, c).x(1).go();
        t.settle();
        let shock = t.g.find_in_zone(Zone::Hand(P1), "Shock")[0];
        if exile_one {
            t.answer_choose(P0, &[obj(shock)]);
            t.answer_choose_moved(P0, &[]);
        } else {
            t.answer_choose(P0, &[]);
        }
        let from = t.asked().len();
        t.g.resolve_top();
        // No one got priority while it resolved.
        assert!(t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
        let shocks_left = [Zone::Hand(P1), Zone::Graveyard(P1), Zone::Library(P1)]
            .iter()
            .map(|z| t.g.find_in_zone(*z, "Shock").len())
            .sum::<usize>();
        if exile_one {
            // The Shock in hand is exiled; the search may find the others.
            assert!(shocks_left <= 2);
            assert!(!t.in_hand(P1, "Shock"));
        } else {
            assert_eq!(shocks_left, 3);
            assert!(t.in_hand(P1, "Cancel"));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Selective Obliteration
// ---------------------------------------------------------------------------------------

#[test]
fn selective_obliteration_colors_chosen_in_turn_order_for_own_permanents() {
    cr!("101.4", "105.2c", "105.2b", "105.3");
    ruling!(
        "Selective Obliteration",
        "Lands have no mana cost, so they are colorless unless an effect states otherwise. The colors of mana that a land can produce do not affect that land's color."
    );
    ruling!(
        "Selective Obliteration",
        "No matter what color any player chooses, all multicolored permanents will be exiled."
    );
    ruling!(
        "Selective Obliteration",
        "Players decide and announce in turn order, starting with the active player, which color they are choosing. Players later in the turn order will know which colors were chosen by players before them."
    );
    ruling!(
        "Selective Obliteration",
        "The colors chosen by players affect only whether or not permanents they control will be exiled. That choice does not affect whether particular permanents controlled by other players will be exiled."
    );
    supported("Selective Obliteration");
    let mut t = TestGame::new(2);
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    let my_goblin = t.battlefield(P0, "Raging Goblin");
    let my_forest = t.battlefield(P0, "Forest");
    let my_gold = t.battlefield(P0, "Tithe Drinker");
    let my_myr = t.battlefield(P0, "Darksteel Myr");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    let their_goblin = t.battlefield(P1, "Raging Goblin");
    let their_island = t.battlefield(P1, "Island");
    let c = t.hand(P0, "Selective Obliteration");
    mana(&mut t, P0, ManaType::C, 5);
    choose_color(&mut t, P0, Color::Green);
    choose_color(&mut t, P1, Color::Red);
    // What P1 can know as P1 chooses: the colors chosen so far for the spell (shown to
    // every player, CR 607.2d).
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose a color"),
        |g| {
            g.objects
                .iter()
                .filter(|o| o.chars.name == "Selective Obliteration")
                .flat_map(|o| o.choices.by_player.iter().map(|(p, c)| (*p, c.color)))
                .collect::<Vec<_>>()
        },
    );
    let from = t.asked().len();
    t.cast(P0, c).go();
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose a color"))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    assert_eq!(*seen.lock().unwrap(), vec![vec![(P0, Some(Color::Green))]]);
    assert!(t.on_battlefield(my_bears));
    assert!(!t.on_battlefield(my_goblin));
    assert!(t.on_battlefield(my_forest));
    assert!(!t.on_battlefield(my_gold));
    assert!(t.on_battlefield(my_myr));
    assert!(!t.on_battlefield(their_bears));
    assert!(t.on_battlefield(their_goblin));
    assert!(t.on_battlefield(their_island));
}

// ---------------------------------------------------------------------------------------
// Congregation at Dawn, Enter the Infinite
// ---------------------------------------------------------------------------------------

#[test]
fn congregation_at_dawn_order_on_top_isnt_revealed() {
    cr!("701.23a", "401.2", "701.20a");
    ruling!(
        "Congregation at Dawn",
        "The order in which the cards are placed on top of the library isn’t revealed."
    );
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Forest"]);
    let c = t.hand(P0, "Congregation at Dawn");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::W, 1);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    cast_resolve(&mut t, c);
    let top: Vec<ObjectId> =
        t.g.player(P0)
            .library
            .iter()
            .rev()
            .take(2)
            .copied()
            .collect();
    let mut names: Vec<String> = top
        .iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
    // The opponent saw the cards revealed but can't see which is where now.
    for id in top {
        assert!(!mtg_engine::player_control::can_see(&t.g, P1, id));
    }
}

#[test]
fn enter_the_infinite_reveals_each_new_top_card_before_drawing_it() {
    cr!("401.5", "121.2");
    ruling!(
        "Enter the Infinite",
        "If you are playing with the top card of your library revealed (due to Garruk's Horde, for example), you'll reveal each card before you draw it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    let first = mtg_engine::zones::revealed_top(&t.g, P0).expect("top revealed");
    let c = t.hand(P0, "Enter the Infinite");
    mana(&mut t, P0, ManaType::U, 4);
    mana(&mut t, P0, ManaType::C, 8);
    let from = t.g.turn_events.len();
    let lib = t.library_size(P0);
    cast_resolve(&mut t, c);
    let revealed: Vec<ObjectId> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Custom {
                name, obj: Some(o), ..
            } if name == mtg_engine::zones::TOP_REVEALED => Some(*o),
            _ => None,
        })
        .chain(std::iter::once(first))
        .collect();
    // Every card drawn was revealed on top first.
    let drawn: Vec<ObjectId> =
        t.g.player(P0)
            .hand
            .iter()
            .copied()
            .filter(|id| t.obj(*id).chars.name == "Filler")
            .collect();
    assert!(drawn.len() >= lib - 1);
    for d in drawn {
        assert!(
            revealed.iter().any(|r| t.g.current(*r) == d),
            "drawn card wasn't revealed"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Goblin Offensive, Oona, Tariel, Valleymaker
// ---------------------------------------------------------------------------------------

#[test]
fn goblin_offensive_tokens_are_named_goblin_token() {
    cr!("111.4");
    ruling!(
        "Goblin Offensive",
        "The tokens are named \"Goblin Token\" and have the creature type \"Goblin.\""
    );
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Goblin Offensive");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, c).x(2).go();
    t.resolve_all();
    let goblins = with_subtype(&t, P0, "Goblin");
    assert_eq!(goblins.len(), 2);
    for g in goblins {
        assert_eq!(t.obj(g).chars.name, "Goblin Token");
    }
}

#[test]
fn oona_tokens_enter_under_your_control() {
    cr!("111.2", "701.13a");
    ruling!(
        "Oona, Queen of the Fae",
        "The tokens enter under your control."
    );
    supported("Oona, Queen of the Fae");
    let mut t = TestGame::new(2);
    let oona = t.battlefield(P0, "Oona, Queen of the Fae");
    stack_library(&mut t, P1, &["Cancel", "Divination", "Shock"]);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    // "How will you pay {U/B}?": either way.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    choose_color(&mut t, P0, Color::Blue);
    t.activate(P0, oona, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    // Two Faerie Rogue tokens (Cancel and Divination are blue) besides Oona.
    let rogues = with_subtype(&t, P0, "Rogue");
    assert_eq!(rogues.len(), 2);
    assert!(rogues.iter().all(|r| t.obj(*r).is_token()));
    assert!(with_subtype(&t, P1, "Faerie").is_empty());
}

#[test]
fn tariel_targets_only_the_opponent_and_picks_at_random() {
    cr!("115.1", "608.2b");
    ruling!(
        "Tariel, Reckoner of Souls",
        "The creature card is chosen at random from among creature cards in the target opponent's graveyard as the ability resolves. Players can respond to the ability knowing the targeted player, but they won't know which creature card is about to enter."
    );
    ruling!(
        "Tariel, Reckoner of Souls",
        "The only target of this ability is the opponent."
    );
    let mut t = TestGame::new(2);
    let tariel = t.battlefield(P0, "Tariel, Reckoner of Souls");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Shock");
    let from = t.asked().len();
    t.activate(P0, tariel, 0, &[pl(P1)]).unwrap();
    let ab = t.g.stack[0];
    assert_eq!(targets_of(&t, ab), vec![pl(P1)]);
    for c in target_choices(&t, from) {
        assert!(c.iter().all(|e| matches!(e, Entity::Player(_))));
    }
    // Nothing about the card is chosen until it resolves, and no one chooses it then.
    assert_eq!(t.graveyard_size(P1), 3);
    let from = t.asked().len();
    t.resolve_all();
    assert!(entity_choices(&t, from).is_empty());
    let mine: Vec<ObjectId> = crate::r_s01_common::creatures(&t, P0)
        .into_iter()
        .filter(|id| *id != tariel)
        .collect();
    assert_eq!(mine.len(), 1);
    assert_eq!(t.graveyard_size(P1), 2);
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn valleymaker_mana_ability_chooses_a_player_without_the_stack() {
    cr!("605.1a", "605.3a", "106.4", "115.10");
    ruling!(
        "Valleymaker",
        "The second ability is a mana ability. It doesn't target a player and it doesn't use the stack. If the player who adds {G}{G}{G} can't spend all of it before the phase ends, the remaining mana will be lost at the end of the current step (or phase)."
    );
    supported("Valleymaker");
    let mut t = TestGame::new(2);
    let vm = t.battlefield(P0, "Valleymaker");
    let forest = t.battlefield(P0, "Forest");
    // P1 has hexproof: still chosen, since it isn't targeted.
    t.battlefield(P1, "Leyline of Sanctity");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose(P0, &[pl(P1)]);
    let r = crate::r_s06_common::activate_containing(&mut t, P0, vm, "Forest");
    assert!(r.is_ok(), "{r:?}");
    assert!(target_choices(&t, from).is_empty());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P1).mana_pool.total(), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
}
