//! Rulings batch P091 — the rest: Plaza of Harmony, Echoing Deeps, Crackling Perimeter,
//! Worms of the Earth, Desert Warfare, The First Tyrannic War and Traveling Chocobo's
//! look-at-the-top and linked-ability rulings.

use crate::r_p091_common::*;
use crate::r_s01_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType;
use mtg_engine::mana_abilities::could_produce;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn plaza_of_harmony_checks_the_gates_current_mana_abilities_not_their_costs() {
    cr!("106.7", "605.1a");
    ruling!(
        "Plaza of Harmony",
        "Plaza of Harmony checks the effects of all mana-producing abilities of Gates you control, but it doesn’t check their costs or legality. For example, it doesn’t check whether a Gate is untapped."
    );
    ruling!(
        "Plaza of Harmony",
        "Any additions to a Gate’s land types or abilities gained by a Gate can affect the types of mana a Gate can produce."
    );
    supported("Plaza of Harmony");
    supported("Prismatic Omen");
    let mut t = TestGame::new(2);
    let plaza = t.battlefield(P0, "Plaza of Harmony");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    t.g.tap(gate);
    t.g.recompute();
    let types = could_produce(&t.g, plaza);
    assert!(types.contains(&ManaType::W) && types.contains(&ManaType::U));
    assert!(!types.contains(&ManaType::G));
    // Prismatic Omen: the Gate is every basic land type too, so it could produce {G}.
    t.battlefield(P0, "Prismatic Omen");
    t.g.recompute();
    let types = could_produce(&t.g, plaza);
    assert!(types.contains(&ManaType::G), "{types:?}");
    assert!(types.contains(&ManaType::R) && types.contains(&ManaType::B));
}

#[test]
fn echoing_deeps_copies_the_printed_land_card_with_its_entry_abilities() {
    cr!("707.2", "707.9", "614.1c", "603.6a");
    ruling!(
        "Echoing Deeps",
        "Any enters-the-battlefield abilities of the copied land card will trigger when Echoing Deeps enters the battlefield. Any \"as [this land] enters the battlefield\" or \"[this land] enters the battlefield with\" abilities of the chosen land card will also work."
    );
    ruling!(
        "Echoing Deeps",
        "Echoing Deeps copies exactly what was printed on the original land card, with the noted exception."
    );
    // Glimmerpost: its ETB trigger triggers; the copy is a Locus and a Cave, and tapped.
    let mut t = TestGame::new(2);
    let post = t.graveyard(P1, "Glimmerpost");
    t.answer_choose(P0, &[obj(post)]);
    let d = put_onto_battlefield(&mut t, P0, "Echoing Deeps");
    assert_eq!(t.obj_now(d).chars.name, "Glimmerpost");
    assert!(t.obj_now(d).chars.has_subtype("Locus"));
    assert!(t.obj_now(d).chars.has_subtype("Cave"));
    assert!(t.obj_now(d).tapped);
    assert_eq!(triggers_of(&t, d), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Gemstone Mine: enters with three mining counters.
    let mut t = TestGame::new(2);
    let mine = t.graveyard(P0, "Gemstone Mine");
    t.answer_choose(P0, &[obj(mine)]);
    let d = put_onto_battlefield(&mut t, P0, "Echoing Deeps");
    assert_eq!(t.obj_now(d).chars.name, "Gemstone Mine");
    assert_eq!(t.counters(d, "mining"), 3);
}

#[test]
fn crackling_perimeter_cant_share_a_gate_with_a_mana_ability() {
    cr!("602.2b", "106.12", "118.3");
    ruling!(
        "Crackling Perimeter",
        "You can't tap an untapped Gate you control for mana and tap it to activate Crackling Perimeter's ability at the same time. You must choose one or the other."
    );
    supported("Crackling Perimeter");
    let mut t = TestGame::new(2);
    let perim = t.battlefield(P0, "Crackling Perimeter");
    let gate = t.battlefield(P0, "Gruul Guildgate");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, gate, "{R}"));
    assert!(t.activate(P0, perim, 0, &[]).is_err());
    // Positive control: with the Gate untapped, the ability works.
    let mut t = TestGame::new(2);
    let perim = t.battlefield(P0, "Crackling Perimeter");
    t.battlefield(P0, "Gruul Guildgate");
    t.activate(P0, perim, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn worms_of_the_earth_lands_cant_enter() {
    cr!("614.17c", "608.3e", "609.3");
    ruling!(
        "Worms of the Earth",
        "If a permanent spell tries to enter as a land during its resolution (for instance, a Clone entering the battlefield as a copy of a Dryad Arbor or an animated Mutavault), it is put into its owner's graveyard instead of entering. It never enters, so abilities that would have triggered on it entering won't trigger."
    );
    ruling!(
        "Worms of the Earth",
        "If you cast a spell or activate an ability that lets you put a land onto the battlefield, the effect fails. You'll still do everything else the spell or ability tells you to do, such as shuffle the library."
    );
    ruling!(
        "Worms of the Earth",
        "Land can't enter, so an effect that \"puts a land onto the battlefield\" won't do anything, and the land will stay where it is."
    );
    supported("Worms of the Earth");
    supported("Kodama's Reach");
    // Clone as a copy of Dryad Arbor: to the graveyard; Soul Warden doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Worms of the Earth");
    let warden = t.battlefield(P0, "Soul Warden");
    let arbor = t.battlefield(P1, "Dryad Arbor");
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(arbor)]);
    t.cast(P0, clone).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Clone"));
    assert_eq!(triggers_of(&t, warden), 0);
    // Positive control: Clone copying Grizzly Bears enters and Soul Warden triggers.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, clone).go();
    t.resolve();
    assert_eq!(triggers_of(&t, warden), 1);
    // Kodama's Reach: the land for the battlefield stays in the library; the other goes to
    // the hand; the library is shuffled.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Worms of the Earth");
    t.lands(P0, "Forest", 3);
    let a = t.library_top(P0, "Plains");
    let b = t.library_top(P0, "Island");
    let reach = t.hand(P0, "Kodama's Reach");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_choose(P0, &[obj(a)]);
    t.cast(P0, reach).go();
    t.resolve_all();
    let on_bf = t.g.find_in_zone(Zone::Battlefield, "Plains").len()
        + t.g.find_in_zone(Zone::Battlefield, "Island").len();
    assert_eq!(on_bf, 0);
    let in_hand = usize::from(t.in_hand(P0, "Plains")) + usize::from(t.in_hand(P0, "Island"));
    assert_eq!(in_hand, 1);
    let in_lib = [a, b]
        .iter()
        .filter(|c| t.zone(**c) == Zone::Library(P0))
        .count();
    assert_eq!(in_lib, 1);
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == P0)));
}

#[test]
fn desert_warfare_returns_only_deserts_still_in_the_graveyard_after_it_entered() {
    cr!("603.7", "400.7", "603.2");
    ruling!(
        "Desert Warfare",
        "Desert Warfare's first ability won't return any Deserts that you sacrificed or that were put into your graveyard from your hand or library before Desert Warfare entered the battlefield."
    );
    ruling!(
        "Desert Warfare",
        "If the Desert leaves your graveyard before the delayed triggered ability created by Desert Warfare's first ability resolves, that Desert won't be put onto the battlefield from its new zone when that ability resolves."
    );
    supported("Desert Warfare");
    // (milled before Warfare entered, exiled from the graveyard before the end step)
    for (before, leaves) in [(true, false), (false, true), (false, false)] {
        let mut t = TestGame::new(2);
        if !before {
            t.battlefield(P0, "Desert Warfare");
        }
        let desert = t.library_top(P0, "Sunscorched Desert");
        t.g.mill(P0, 1);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        if before {
            t.battlefield(P0, "Desert Warfare");
        }
        let in_gy = t.g.current(desert);
        assert_eq!(t.zone(in_gy), Zone::Graveyard(P0));
        if leaves {
            t.g.move_object(in_gy, Zone::Exile, MoveCause::Effect, None);
            t.g.flush_events();
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let returned = !t.named_on_battlefield("Sunscorched Desert").is_empty();
        assert_eq!(returned, !before && !leaves, "before {before}, leaves {leaves}");
        if leaves {
            assert!(t.in_exile("Sunscorched Desert"));
        }
    }
}

#[test]
fn the_first_tyrannic_war_checks_the_mana_cost_of_what_the_creature_copies() {
    cr!("707.2", "202.1", "714.2b");
    ruling!(
        "The First Tyrannic War",
        "If the card you are putting onto the battlefield with the first chapter ability would enter the battlefield as a copy of another permanent, it will get +1/+1 counters this way only if the permanent that it is copying has a mana cost that contains {X}, regardless of the original card's mana cost."
    );
    supported("The First Tyrannic War");
    // (card from hand, copy target, counters expected) with three lands.
    for (name, copy, expected) in [
        ("Hangarback Walker", None, 3u32),
        ("Grizzly Bears", None, 0),
        ("Clone", Some("Hangarback Walker"), 3),
        ("Clone", Some("Grizzly Bears"), 0),
    ] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        let target = copy.map(|c| t.battlefield(P1, c));
        if let Some(target) = target {
            // (Hangarback Walker is 0/0: keep it alive.)
            crate::r_s29_common::put_counters(&mut t, target, "+1/+1", 1);
        }
        let card = t.hand(P0, name);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(card)]);
        if let Some(target) = target {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(target)]);
        }
        put_onto_battlefield(&mut t, P0, "The First Tyrannic War");
        t.resolve_all();
        let entered = t.g.current(card);
        assert_eq!(t.zone(entered), Zone::Battlefield, "{name} {copy:?}");
        assert_eq!(t.counters(entered, "+1/+1"), expected, "{name} {copy:?}");
    }
}

#[test]
fn chocobo_look_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Traveling Chocobo",
        "Traveling Chocobo lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Traveling Chocobo");
    t.g.recompute();
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn chocobo_doubled_hideaway_exiles_two_cards_for_the_linked_ability() {
    cr!("607.2a", "603.2d", "702.75a");
    ruling!(
        "Traveling Chocobo",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers."
    );
    supported("Clive's Hideaway");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Traveling Chocobo");
    let cards = stack_library(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Wastes",
            "Wastes",
            "Wastes",
            "Hill Giant",
            "Wastes",
            "Wastes",
            "Wastes",
        ],
    );
    let hideaway = play_from_hand(&mut t, P0, "Clive's Hideaway");
    assert_eq!(triggers_of(&t, hideaway), 2);
    t.answer_choose(P0, &[obj(cards[0])]);
    t.resolve();
    t.answer_choose(P0, &[obj(cards[4])]);
    t.resolve_all();
    let bears = t.g.current(cards[0]);
    let giant = t.g.current(cards[4]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    // Four legendary creatures; the last ability may play both exiled cards.
    for n in [
        "Rocco, Street Chef",
        "Zimone, All-Questioning",
        "Hazezon, Shaper of Sand",
        "Aesi, Tyrant of Gyre Strait",
    ] {
        t.battlefield(P0, n);
    }
    t.lands(P0, "Wastes", 2);
    t.g.untap(hideaway);
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    crate::r_s06_common::activate_containing(&mut t, P0, hideaway, "{2}").unwrap();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Grizzly Bears").is_empty(), "{}", t.dump_log());
    assert!(!t.named_on_battlefield("Hill Giant").is_empty(), "{}", t.dump_log());
}

#[test]
fn withering_wisps_end_step_intervening_if() {
    cr!("603.4", "513.1a");
    ruling!(
        "Withering Wisps",
        "Withering Wisps's triggered ability has an \"intervening 'if' clause.\" That means (1) the ability won't trigger at all unless there are no creatures on the battlefield when a turn's end step begins, and (2) the ability will do nothing if there are any creatures on the battlefield by the time the ability resolves."
    );
    ruling!(
        "Withering Wisps",
        "If all creatures leave the battlefield during a turn's End step (after any \"at the beginning of the end step\" triggers would have triggered that turn), Withering Wisps will remain on the battlefield because the trigger condition has already passed."
    );
    supported("Withering Wisps");
    // A creature as the end step begins: no trigger; it dies during the end step: Wisps
    // stays.
    let mut t = TestGame::new(2);
    let wisps = t.battlefield(P0, "Withering Wisps");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_of(&t, wisps), 0);
    crate::r_s02_common::destroy(&mut t, bears);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(wisps));
    // No creatures: it triggers; a creature arriving before it resolves: nothing.
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_of(&t, wisps), 1);
    t.battlefield(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.on_battlefield(wisps));
    // Positive control: no creatures at all: sacrificed.
    let mut t = TestGame::new(2);
    let wisps = t.battlefield(P0, "Withering Wisps");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(wisps));
}
