//! Rulings batch S18 — adventurer cards (CR 715) outside the stack: going on an adventure
//! (exiled as it resolves, then cast from exile), cards exiled some other way, adventurer
//! cards in other zones, copies, and additional land plays.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, target_candidates};
use crate::r_s08_common::legal_cast_methods;
use crate::r_s18_common::*;
use mtg_engine::card::card;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The name of `name`'s creature (or other permanent) half.
fn primary(name: &str) -> String {
    card(name).faces[0].chars.name.to_string()
}

/// P0 casts the real card `name` as its Adventure with the targets `setup` returns.
fn cast_as_adventure(t: &mut TestGame, name: &str, setup: fn(&mut TestGame) -> Vec<Entity>) -> ObjectId {
    let targets = setup(t);
    rainbow_lands(t, P0, 3);
    let c = t.hand(P0, name);
    t.cast(P0, c).method(ADVENTURE).targets(&targets).go()
}

// ---------------------------------------------------------------------------------------
// Going on an adventure.
// ---------------------------------------------------------------------------------------

/// Checks that `name`, cast as an Adventure by P0 (targets from `setup`), is exiled as it
/// resolves and then only P0 may cast it, only as the permanent; that countered, it goes
/// to the graveyard; and, if `illegal` is given (making the targets illegal), that it
/// goes to the graveyard when it fails to resolve.
fn goes_on_an_adventure_only_by_resolving(
    name: &str,
    setup: fn(&mut TestGame) -> Vec<Entity>,
    illegal: Option<fn(&mut TestGame)>,
) {
    supported(name);
    let creature = primary(name);
    let mut t = TestGame::new(2);
    cast_as_adventure(&mut t, name, setup);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    let exiled = exiled_named(&t, &creature);
    assert_eq!(exiled.len(), 1);
    rainbow_lands(&mut t, P0, 3);
    assert_eq!(
        legal_cast_methods(&mut t, P0, exiled[0]),
        vec![CastMethod::Normal],
        "{name}"
    );
    rainbow_lands(&mut t, P1, 3);
    assert!(legal_cast_methods(&mut t, P1, exiled[0]).is_empty());
    t.cast(P0, exiled[0]).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(&creature).len(), 1);
    // Countered: it goes to its owner's graveyard, and can't be cast from there.
    let mut t = TestGame::new(2);
    let spell = cast_as_adventure(&mut t, name, setup);
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, &creature));
    assert!(exiled_named(&t, &creature).is_empty());
    let gy = t.g.player(P0).graveyard[0];
    rainbow_lands(&mut t, P0, 3);
    assert!(legal_cast_methods(&mut t, P0, gy).is_empty());
    // Its targets became illegal: it doesn't resolve, and goes to the graveyard.
    if let Some(illegal) = illegal {
        let mut t = TestGame::new(2);
        cast_as_adventure(&mut t, name, setup);
        illegal(&mut t);
        t.resolve_all();
        assert!(t.in_graveyard(P0, &creature));
        assert!(exiled_named(&t, &creature).is_empty());
    }
}

#[test]
fn an_adventure_is_exiled_only_if_it_resolves() {
    cr!("715.3d", "608.2b", "701.6a");
    ruling!(
        "Ettercap // Web Shot",
        "If a spell is cast as an Adventure, its controller exiles it instead of putting it into its owner’s graveyard as it resolves. For as long as it remains exiled, that player may cast it as a permanent spell. If an Adventure spell leaves the stack in any way other than resolving (most likely by being countered or by failing to resolve because its targets have all become illegal), that card won’t be exiled and the spell’s controller won’t be able to cast it as a permanent later."
    );
    ruling!(
        "Bilbo Baggins, Burglar // Take a Glance",
        "If a spell is cast as an Adventure, its controller exiles it instead of putting it into its owner's graveyard as it resolves. For as long as it remains exiled, that player may play it using its primary characteristics. If an Adventure spell leaves the stack in any way other than resolving (most likely by being countered or by failing to resolve because its targets have all become illegal), that card won't be exiled and the spell's controller won't be able to cast it as a permanent later."
    );
    ruling!(
        "Murderous Rider // Swift End",
        "If a spell is cast as an Adventure, its controller exiles it instead of putting it into its owner's graveyard as it resolves. For as long as it remains exiled, that player may play it using its primary characteristics. If an Adventure spell leaves the stack in any way other than resolving (most likely by being countered or by failing to resolve because its targets have all become illegal), that card won't be exiled and the spell's controller won't be able to play that card from exile later."
    );
    // Web Shot: "Destroy target creature with flying."
    goes_on_an_adventure_only_by_resolving(
        "Ettercap // Web Shot",
        |t| vec![Entity::Object(t.battlefield(P1, "Serra Angel"))],
        Some(|t| {
            let angel = t.named_on_battlefield("Serra Angel")[0];
            crate::r_s02_common::destroy(t, angel);
        }),
    );
    // Take a Glance: "Scry 2."
    goes_on_an_adventure_only_by_resolving("Bilbo Baggins, Burglar // Take a Glance", |_| vec![], None);
    // Swift End: "Destroy target creature or planeswalker. You lose 2 life."
    goes_on_an_adventure_only_by_resolving(
        "Murderous Rider // Swift End",
        |t| vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))],
        Some(|t| {
            let bears = t.named_on_battlefield("Grizzly Bears")[0];
            crate::r_s05_common::move_to(t, bears, Zone::Hand(P1));
        }),
    );
}

/// Checks that `name` exiled other than by resolving as an Adventure can't be cast from
/// exile: from the graveyard by Coffin Purge, and as an Adventure spell countered by
/// Dissipate ("If that spell is countered this way, exile it instead").
fn exiled_another_way(name: &str, setup: fn(&mut TestGame) -> Vec<Entity>) {
    supported(name);
    supported("Dissipate");
    let creature = primary(name);
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, name);
    t.lands(P1, "Swamp", 1);
    let purge = t.hand(P1, "Coffin Purge");
    t.cast(P1, purge).target(gy).go();
    t.resolve_all();
    let exiled = exiled_named(&t, &creature);
    assert_eq!(exiled.len(), 1);
    rainbow_lands(&mut t, P0, 3);
    assert!(legal_cast_methods(&mut t, P0, exiled[0]).is_empty());
    let mut t = TestGame::new(2);
    let spell = cast_as_adventure(&mut t, name, setup);
    t.lands(P1, "Island", 3);
    let dissipate = t.hand(P1, "Dissipate");
    t.cast(P1, dissipate).target(spell).go();
    t.resolve_all();
    let exiled = exiled_named(&t, &creature);
    assert_eq!(exiled.len(), 1);
    rainbow_lands(&mut t, P0, 3);
    assert!(legal_cast_methods(&mut t, P0, exiled[0]).is_empty(), "{name}");
}

#[test]
fn an_adventurer_card_exiled_another_way_cant_be_cast_from_exile() {
    cr!("715.3d");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "If an adventurer card ends up in exile for any other reason than by exiling itself while resolving, it won't give you permission to cast it as a permanent spell."
    );
    ruling!(
        "Sea Hag // Aquatic Ingress",
        "If an adventurer card ends up in exile for any other reason than by exiling itself while resolving, it won’t give you permission to cast it as a permanent spell."
    );
    ruling!(
        "Brazen Borrower // Petty Theft",
        "If an adventurer card ends up in exile for any other reason than by exiling itself while resolving, it won't give you permission to play it with its primary characteristics."
    );
    exiled_another_way("Bonecrusher Giant // Stomp", |_| vec![Entity::Player(P1)]);
    exiled_another_way("Sea Hag // Aquatic Ingress", |_| vec![]);
    exiled_another_way("Brazen Borrower // Petty Theft", |t| {
        vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))]
    });
}

// ---------------------------------------------------------------------------------------
// Timing for the permanent cast from exile.
// ---------------------------------------------------------------------------------------

/// After `name`'s Adventure resolved in P0's main phase, the card can be cast from exile
/// only at sorcery speed: in P0's main phase with an empty stack.
fn timing_from_exile(name: &str, setup: fn(&mut TestGame) -> Vec<Entity>) {
    supported(name);
    let creature = primary(name);
    let mut t = TestGame::new(2);
    cast_as_adventure(&mut t, name, setup);
    t.resolve_all();
    let exiled = exiled_named(&t, &creature)[0];
    rainbow_lands(&mut t, P0, 2);
    assert_eq!(legal_cast_methods(&mut t, P0, exiled), vec![CastMethod::Normal]);
    // Not while a spell is on the stack.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    assert!(legal_cast_methods(&mut t, P0, exiled).is_empty(), "{name}");
    t.resolve_all();
    // Not in combat, nor during the opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_cast_methods(&mut t, P0, exiled).is_empty());
    t.set_step(P1, Step::PrecombatMain);
    assert!(legal_cast_methods(&mut t, P0, exiled).is_empty());
    t.set_step(P0, Step::PostcombatMain);
    assert_eq!(legal_cast_methods(&mut t, P0, exiled), vec![CastMethod::Normal]);
}

#[test]
fn the_permanent_cast_from_exile_follows_the_normal_timing_rules() {
    cr!("715.3d", "302.1");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "You must still follow any timing restrictions and permissions for the permanent spell you cast from exile. Normally, you'll be able to cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Two-Headed Hunter // Twice the Rage",
        "You must still follow any timing restrictions and permissions for the permanent spell you cast from exile. Normally, you’ll be able to cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Young Red Dragon // Bathe in Gold",
        "You must still follow any relevant timing rules for the permanent spell you cast from exile. Normally, you’ll be able to cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Karvanista, Loyal Lupari // Lupari Shield",
        "You must still follow any relevant timing rules for the permanent spell you cast from exile. Normally, you'll be able to cast it only during your main phase while the stack is empty."
    );
    timing_from_exile("Bonecrusher Giant // Stomp", |_| vec![Entity::Player(P1)]);
    timing_from_exile("Two-Headed Hunter // Twice the Rage", |t| {
        vec![Entity::Object(t.battlefield(P0, "Grizzly Bears"))]
    });
    timing_from_exile("Young Red Dragon // Bathe in Gold", |_| vec![]);
    timing_from_exile("Karvanista, Loyal Lupari // Lupari Shield", |_| vec![]);
}

#[test]
fn a_land_played_from_exile_needs_a_land_play() {
    cr!("715.3d", "305.1", "305.2b");
    ruling!(
        "Lindblum, Industrial Regency // Mage Siege",
        "You must still follow any timing restrictions and permissions for the card you play from exile. In the case of any of the five lands in this release, you'll be able to play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    supported("Lindblum, Industrial Regency // Mage Siege");
    // Mage Siege ({2}{R} instant): "Create a 0/1 black Wizard creature token ..."
    let mut t = TestGame::new(2);
    cast_as_adventure(&mut t, "Lindblum, Industrial Regency // Mage Siege", |_| vec![]);
    t.resolve_all();
    let lindblum = exiled_named(&t, "Lindblum, Industrial Regency")[0];
    assert!(can_play_land(&mut t, P0, lindblum));
    // Not with a spell on the stack, in combat, or during the opponent's turn.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    assert!(!can_play_land(&mut t, P0, lindblum));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, lindblum));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, lindblum));
    // Not after playing a land this turn.
    t.set_step(P0, Step::PostcombatMain);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("land play");
    assert!(!can_play_land(&mut t, P0, lindblum));
    // With an available land play, it's played as a land.
    let mut t = TestGame::new(2);
    cast_as_adventure(&mut t, "Lindblum, Industrial Regency // Mage Siege", |_| vec![]);
    t.resolve_all();
    let lindblum = exiled_named(&t, "Lindblum, Industrial Regency")[0];
    t.play_land(P0, lindblum).expect("play Lindblum");
    assert!(t.on_battlefield(lindblum));
    assert!(t.obj_now(lindblum).tapped);
}

// ---------------------------------------------------------------------------------------
// Outside the stack, an adventurer card has only its normal characteristics.
// ---------------------------------------------------------------------------------------

/// `name` in P0's graveyard has only its normal characteristics (a permanent card with its
/// mana value `mv`): Archaeomancer ("return target instant or sorcery card from your
/// graveyard to your hand") can't target it, only the Shock beside it.
fn permanent_card_in_the_graveyard(name: &str, mv: u32) {
    supported(name);
    supported("Archaeomancer");
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, name);
    let shock = t.graveyard(P0, "Shock");
    t.g.recompute();
    let chars = t.obj(c).chars.clone();
    assert_eq!(chars.name, primary(name));
    assert!(!chars.is(CardType::Instant) && !chars.is(CardType::Sorcery));
    assert!(chars.is_creature() || chars.is(CardType::Land));
    assert_eq!(t.g.mana_value_of(c), mv);
    let from = t.asked().len();
    t.enter(P0, "Archaeomancer");
    t.g.flush_events();
    t.resolve_all();
    assert!(target_candidates(&t, P0, from)
        .iter()
        .all(|v| !v.contains(&Entity::Object(c))));
    assert_eq!(t.zone(shock), Zone::Hand(P0));
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    // Alone in the graveyard, it isn't a legal target: the ability does nothing.
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, name);
    t.enter(P0, "Archaeomancer");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
}

#[test]
fn outside_the_stack_an_adventurer_card_is_a_permanent_card() {
    cr!("715.4", "202.3");
    ruling!(
        "Two-Headed Hunter // Twice the Rage",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it’s in your graveyard, Questing Druid is a green creature card whose mana value is 2."
    );
    ruling!(
        "Young Red Dragon // Bathe in Gold",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it’s in your graveyard, Altar of Bhaal is an artifact card whose mana value is 2."
    );
    ruling!(
        "Twining Twins // Swift Spiral",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it's in your graveyard, Questing Druid is a green creature card whose mana value is 2."
    );
    ruling!(
        "Bilbo Baggins, Burglar // Take a Glance",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it's in your graveyard, Bilbo, Luckwearer is a blue creature card whose mana value is 2."
    );
    ruling!(
        "Lindblum, Industrial Regency // Mage Siege",
        "An adventurer card uses only its non-Adventure characteristics in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases."
    );
    permanent_card_in_the_graveyard("Two-Headed Hunter // Twice the Rage", 5);
    permanent_card_in_the_graveyard("Young Red Dragon // Bathe in Gold", 4);
    permanent_card_in_the_graveyard("Twining Twins // Swift Spiral", 4);
    permanent_card_in_the_graveyard("Bilbo Baggins, Burglar // Take a Glance", 3);
    // Lindblum is a colorless Town land card with mana value 0.
    permanent_card_in_the_graveyard("Lindblum, Industrial Regency // Mage Siege", 0);
    let mut t = TestGame::new(2);
    let l = t.graveyard(P0, "Lindblum, Industrial Regency // Mage Siege");
    t.g.recompute();
    assert!(t.obj(l).chars.colors.is_colorless());
    assert!(t.obj(l).chars.has_subtype("Town"));
}

// ---------------------------------------------------------------------------------------
// Copies of adventurer permanents; additional land plays.
// ---------------------------------------------------------------------------------------

#[test]
fn a_copy_of_an_adventurer_has_an_adventure_until_it_changes_zones() {
    cr!("715.2b", "707.2", "400.7");
    ruling!(
        "Two-Headed Hunter // Twice the Rage",
        "If an object becomes a copy of an object that has an Adventure, the copy also has an Adventure. If it changes zones, it will either cease to exist (if it’s a token) or cease to be a copy (if it’s a nontoken permanent), and so you won’t be able to cast it as an Adventure."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P1, "Two-Headed Hunter // Twice the Rage");
    t.answer_choose(P0, &[Entity::Object(hunter)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.g.recompute();
    assert_eq!(t.obj_now(clone).chars.name, "Two-Headed Hunter");
    assert!(has_adventure(&mut t, clone));
    // Returned to hand, it's a Clone card: no Adventure to cast.
    let in_hand = crate::r_s05_common::move_to(&mut t, clone, Zone::Hand(P0)).unwrap();
    assert_eq!(t.obj(in_hand).chars.name, "Clone");
    assert!(!has_adventure(&mut t, in_hand));
    rainbow_lands(&mut t, P0, 3);
    assert_eq!(
        legal_cast_methods(&mut t, P0, in_hand),
        vec![CastMethod::Normal]
    );
}

#[test]
fn additional_land_plays_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Beanstalk Wurm // Plant Beans",
        "The effect that allows you to play an additional land that turn is cumulative with other effects that do so."
    );
    supported("Beanstalk Wurm // Plant Beans");
    supported("Explore");
    // Plant Beans ("You may play an additional land this turn.") and Explore ("You may
    // play an additional land this turn. Draw a card."): three lands this turn.
    let mut t = TestGame::new(2);
    cast_as_adventure(&mut t, "Beanstalk Wurm // Plant Beans", |_| vec![]);
    t.resolve_all();
    let explore = t.hand(P0, "Explore");
    t.cast(P0, explore).go();
    t.resolve_all();
    for i in 0..3 {
        let land = t.hand(P0, "Forest");
        assert!(t.play_land(P0, land).is_ok(), "land {i}");
    }
    let fourth = t.hand(P0, "Forest");
    assert!(t.play_land(P0, fourth).is_err());
}

#[test]
fn a_milled_adventurer_card_is_a_creature_card() {
    cr!("715.4", "701.17c");
    ruling!(
        "Colossal Badger // Dig Deep",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it's in your graveyard, Altar of Bhaal is an artifact card whose mana value is 2."
    );
    permanent_card_in_the_graveyard("Colossal Badger // Dig Deep", 6);
    // Dig Deep ({1}{G} sorcery): "Choose target creature. Mill four cards, then put a
    // +1/+1 counter on that creature for each creature card milled this way." The milled
    // Two-Headed Hunter is a creature card in the graveyard, like the Grizzly Bears; the
    // Shock and the Forest aren't. The creature cards already in the graveyards weren't
    // milled this way.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Serra Angel");
    t.graveyard(P1, "Grizzly Bears");
    stack_library(
        &mut t,
        P0,
        &[
            "Two-Headed Hunter // Twice the Rage",
            "Grizzly Bears",
            "Shock",
            "Forest",
        ],
    );
    let spell = cast_as_adventure(&mut t, "Colossal Badger // Dig Deep", |t| {
        vec![Entity::Object(t.battlefield(P0, "Hill Giant"))]
    });
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.obj(spell).chars.name, "Dig Deep");
    t.resolve_all();
    assert_eq!(t.counters(giant, mtg_engine::types::counters::PLUS1), 2);
    assert_eq!(t.graveyard_size(P0), 5);
    assert_eq!(exiled_named(&t, "Colossal Badger").len(), 1);
}
