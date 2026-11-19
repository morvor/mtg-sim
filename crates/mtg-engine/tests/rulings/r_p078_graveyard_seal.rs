//! Rulings batch P078 — graveyard seals: Head of the Hunt and Malicious Eclipse ("if a
//! creature an opponent controls would die, exile it instead"), Anafenza, the Foremost,
//! Planar Void, Wheel of Sun and Moon, Weathered Runestone, and Last One Standing.

use crate::r_p078_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_cast, create_token, destroy};
use crate::r_s06_common::give_control;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn head_of_the_hunt_leaving_with_the_dying_creatures_still_exiles_them() {
    cr!("614.1a", "614.4", "614.6");
    ruling!(
        "Head of the Hunt",
        "If Head of the Hunt would leave the battlefield at the same time as one or more creatures an opponent controls would die, those creatures will still be exiled."
    );
    supported("Head of the Hunt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Head of the Hunt");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Head of the Hunt"));
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn weathered_runestone_lets_a_card_exiled_from_a_library_be_cast_from_exile() {
    cr!("601.2", "601.3");
    ruling!(
        "Weathered Runestone",
        "If an effect exiles a card from a graveyard or library and allows a player to cast it, the player may do so. The spell is cast from exile."
    );
    supported("Weathered Runestone");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Weathered Runestone");
    // A card with flashback in the graveyard can't be cast while Runestone is around ...
    let looting = t.graveyard(P0, "Faithless Looting");
    t.lands(P0, "Mountain", 3);
    assert!(!can_cast(
        &mut t,
        P0,
        looting,
        CastMethod::Keyword(KeywordKind::Flashback)
    ));
    // ... but a card exiled from the library with permission to play it can.
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    cast_new(&mut t, P0, "Act on Impulse", &[]);
    t.resolve_all();
    let bears =
        t.g.exile
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).chars.name == "Grizzly Bears")
            .expect("Grizzly Bears exiled");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn planar_void_destroyed_by_a_spell_doesnt_exile_that_spell() {
    cr!("603.10", "608.2n");
    ruling!(
        "Planar Void",
        "If spell is used to destroy Planar Void, it goes to the graveyard, then the spell goes to the graveyard. The spell is not exiled, as Planar Void was not on the battlefield when it was put into the graveyard."
    );
    supported("Planar Void");
    let mut t = TestGame::new(2);
    let void = t.battlefield(P0, "Planar Void");
    cast_new(&mut t, P1, "Disenchant", &[obj(void)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Planar Void"));
    assert!(t.in_graveyard(P1, "Disenchant"));
    assert!(t.g.exile.is_empty());
}

#[test]
fn planar_void_ignores_itself_and_the_cards_going_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Planar Void",
        "It will not trigger on itself going to the graveyard, or on any other cards going to the graveyard at the same time. This means that neither itself nor other cards put into the graveyard at the same time as it will get exiled."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Planar Void");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let disk = t.battlefield(P0, "Nevinyrral's Disk");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, disk, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Planar Void"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Nevinyrral's Disk"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.g.exile.is_empty());
    // On its own, a card put into a graveyard while Planar Void is around is exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Planar Void");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn planar_void_lets_other_abilities_see_the_card_reach_the_graveyard() {
    cr!("603.2", "603.3", "700.4");
    ruling!(
        "Planar Void",
        "The card does go to the graveyard before it is exiled, so other triggered abilities will trigger on the card going to the graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Planar Void");
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    destroy(&mut t, bears);
    // Both Planar Void's and Blood Artist's abilities triggered.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn anafenza_lets_madness_and_discard_checks_work() {
    cr!("702.35a", "701.9a", "614.1a");
    ruling!(
        "Anafenza, the Foremost",
        "If your opponent discards a creature card while Anafenza is on the battlefield, abilities that function when a card is discarded (such as madness) still work, even though that card never reaches a graveyard. In addition, spells or abilities that check the characteristics of a discarded card (such as Chandra Ablaze's first ability) can find that card in exile."
    );
    supported("Anafenza, the Foremost");
    supported("Chandra Ablaze");
    // Madness: P1 discards Basking Rootwalla and casts it for its madness cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, the Foremost");
    t.hand(P1, "Basking Rootwalla");
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Mind Rot", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Basking Rootwalla").len(), 1);
    // Chandra Ablaze: the red creature card P1 discards is exiled, and Chandra still finds
    // it red.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, the Foremost");
    let chandra = t.battlefield(P1, "Chandra Ablaze");
    t.hand(P1, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.activate(P1, chandra, 0, &[pl(P0)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn anafenza_stops_nontoken_dies_triggers_but_not_token_ones() {
    cr!("700.4", "603.6c", "111.7");
    ruling!(
        "Anafenza, the Foremost",
        "While Anafenza is on the battlefield, abilities that trigger whenever a nontoken creature your opponent owns dies won't trigger, as that card will never reach that player's graveyard. (Token creatures will still go to the graveyard briefly before ceasing to exist.)"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, the Foremost");
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 0, "no dies trigger for the exiled card");
    assert!(t.in_exile("Grizzly Bears"));
    let token = create_token(&mut t, P1, "Plant");
    t.answer_targets(P0, &[pl(P1)]);
    destroy(&mut t, token);
    assert_eq!(t.stack_len(), 1, "the token died");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn wheel_of_sun_and_moon_ignores_tokens_and_copies_of_spells() {
    cr!("111.7", "707.10a", "614.1a");
    ruling!(
        "Wheel of Sun and Moon",
        "Wheel of Sun and Moon won’t affect tokens that are put into the graveyard from the battlefield or copies of spells that resolve or are countered. They’re not cards, so they’ll go to the graveyard as normal, then cease to exist."
    );
    supported("Wheel of Sun and Moon");
    let mut t = TestGame::new(2);
    let wheel = t.battlefield(P0, "Wheel of Sun and Moon");
    assert!(t.g.attach(wheel, pl(P1)));
    t.battlefield(P0, "Blood Artist");
    // A token dies: it goes to the graveyard (Blood Artist sees it die).
    let token = create_token(&mut t, P1, "Plant");
    t.answer_targets(P0, &[pl(P1)]);
    destroy(&mut t, token);
    assert_eq!(t.stack_len(), 1, "the token died");
    t.resolve_all();
    // A card dies: it goes to the bottom of the library instead.
    let lib = t.library_size(P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(t.library_size(P1), lib + 1);
    assert_eq!(t.stack_len(), 0, "the card never died");
    // A spell and its copy resolve: the two cards go to the library; the copy doesn't.
    let (lib, life) = (t.library_size(P1), t.life(P0));
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    cast_new(&mut t, P1, "Twincast", &[obj(shock)]);
    t.resolve_all();
    assert_eq!(t.life(P0), life - 4, "the copy resolved");
    assert_eq!(t.library_size(P1), lib + 2);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn malicious_eclipse_exiles_creatures_that_join_opponents_later_that_turn() {
    cr!("611.2c", "614.1a");
    ruling!(
        "Malicious Eclipse",
        "While the -2/-2 applies only to creatures that are on the battlefield as Malicious Eclipse resolves, the replacement effect applies to all creatures opponents control that turn, including ones that enter the battlefield after Malicious Eclipse resolves, noncreatures that become creatures after Malicious Eclipse resolves, and creatures that become controlled by opponents after Malicious Eclipse resolves."
    );
    supported("Malicious Eclipse");
    let mut t = TestGame::new(2);
    let early = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    let vault = t.battlefield(P1, "Mutavault");
    cast_new(&mut t, P0, "Malicious Eclipse", &[]);
    t.resolve_all();
    assert_eq!(t.pt(early), (1, 1));
    // A creature that enters later doesn't get -2/-2.
    let late = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
    destroy(&mut t, late);
    assert!(t.in_exile("Grizzly Bears"));
    // A noncreature that becomes a creature.
    t.lands(P1, "Wastes", 1);
    t.activate(P1, vault, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    destroy(&mut t, vault);
    assert!(t.in_exile("Mutavault"));
    // A creature an opponent gains control of.
    give_control(&mut t, mine, P1);
    destroy(&mut t, mine);
    assert_eq!(t.zone(t.g.current(mine)), Zone::Exile);
    // P0's own creatures still die normally.
    let own = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, own);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn last_one_standing_destroys_the_rest_in_the_same_resolution() {
    cr!("608.2", "608.2c", "117.1");
    ruling!(
        "Last One Standing",
        "Players can’t take actions between the time the creature is randomly chosen and the time the rest are destroyed."
    );
    supported("Last One Standing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Last One Standing", &[]);
    let from = t.asked().len();
    t.g.settle();
    t.g.resolve_top();
    // No player gets priority during the resolution (P1 only orders the cards going to
    // the graveyard), and only one creature remains once it's done.
    let priority = deciders_since(&t, from, |d| matches!(d, Decision::Priority { .. }));
    assert!(priority.is_empty());
    let creatures =
        t.g.permanents()
            .filter(|o| o.is(CardType::Creature))
            .count();
    assert_eq!(creatures, 1);
}
