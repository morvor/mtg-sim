//! What a triggered ability's effect refers to (CR 603.2, 608.2h): the source for self
//! triggers ("attach it to target creature you control"), the triggering creature
//! ("attach ~ to it", "that creature's toughness"), the blocked or blocking creature
//! remembered by a delayed trigger (CR 603.7c), and "that many" (the event's amount).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.hand(p, name);
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(p),
    )
    .expect("failed to enter the battlefield")
}

#[test]
fn equipment_enters_and_attaches_itself() {
    cr!("301.5", "603.6a");
    assert_supported(&["Maul of the Skyclaves"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let maul = enter(&mut t, P0, "Maul of the Skyclaves");
    t.resolve_all();
    assert_eq!(t.obj_now(maul).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn attach_to_the_creature_that_entered() {
    cr!("603.6a", "701.3a");
    assert_supported(&["Stormrider Rig"]);
    let mut t = TestGame::new(2);
    let rig = t.battlefield(P0, "Stormrider Rig");
    t.answer_yes(P0, true);
    let bears = enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.obj_now(rig).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn gain_life_equal_to_that_creatures_toughness() {
    cr!("603.6a", "608.2h");
    assert_supported(&["Verdant Sun's Avatar"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verdant Sun's Avatar");
    enter(&mut t, P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn delayed_trigger_remembers_the_blocked_creature() {
    cr!("603.7a", "603.7c", "509.3b");
    assert_supported(&["Wall of Tears"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Tears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, bears)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    // Still on the battlefield until end of combat.
    assert!(t.on_battlefield(bears));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn blocked_by_a_non_wall_creature() {
    cr!("509.3d", "603.7c");
    assert_supported(&["Infernal Medusa"]);
    let mut t = TestGame::new(2);
    let medusa = t.battlefield(P0, "Infernal Medusa");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(medusa, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, medusa)]),
    );
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn that_many_counters_from_combat_damage() {
    cr!("510.2", "603.2");
    assert_supported(&["Westgate Regent"]);
    let mut t = TestGame::new(2);
    let regent = t.battlefield(P0, "Westgate Regent");
    let (p, _) = t.pt(regent);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(regent, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(regent, "+1/+1"), p as u32);
}

#[test]
fn enchanted_creature_means_the_one_this_aura_enchants() {
    cr!("303.4b", "510.2");
    assert_supported(&["Pollenbright Wings", "Holy Strength"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let wings = t.battlefield(P0, "Pollenbright Wings");
    let strength = t.battlefield(P0, "Holy Strength");
    t.g.attach(wings, Entity::Object(a));
    t.g.attach(strength, Entity::Object(b));
    t.set_step(P0, Step::BeginningOfCombat);
    // Only the creature enchanted by Pollenbright Wings makes Saprolings.
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    let saprolings =
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Saproling"))
            .count();
    assert_eq!(saprolings, 2);
    assert_eq!(t.life(P1), 20 - 2 - 4);
}

#[test]
fn youre_dealt_damage_by_several_sources_triggers_once() {
    cr!("603.2c", "510.2");
    assert_supported(&["Darien, King of Kjeldor"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darien, King of Kjeldor");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.attack(&[(a, Entity::Player(P0)), (b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 16);
    let soldiers = t
        .g
        .battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype("Soldier") && t.g.obj(**id).controller == P0)
        .count();
    // Darien is a Soldier too.
    assert_eq!(soldiers, 1 + 4);
}

#[test]
fn a_source_you_control_deals_damage_to_you() {
    cr!("120.2", "603.2");
    assert_supported(&["Auntie Blyte, Bad Influence"]);
    let mut t = TestGame::new(2);
    let auntie = t.battlefield(P0, "Auntie Blyte, Bad Influence");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.counters(auntie, "+1/+1"), 3);
}

#[test]
fn put_into_your_graveyard_from_your_library() {
    cr!("113.6k", "400.7e");
    assert_supported(&["Narcomoeba"]);
    let mut t = TestGame::new(2);
    t.library_top(P0, "Narcomoeba");
    t.answer_yes(P0, true);
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Narcomoeba").len(), 1);
}

#[test]
fn sheoldred_they_lose_life() {
    cr!("603.2", "121.1");
    assert_supported(&["Sheoldred, the Apocalypse"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sheoldred, the Apocalypse");
    // "Whenever you draw a card, you gain 2 life. Whenever an opponent draws a card, they
    // lose 2 life."
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    t.g.draw_cards(P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn that_player_sacrifices_a_creature_of_their_choice() {
    cr!("701.21a", "510.2");
    assert_supported(&["Demon of Loathing"]);
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Demon of Loathing");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(demon, Entity::Player(P1))], &[]);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(demon));
}

#[test]
fn that_player_mills_that_many_cards() {
    cr!("510.2", "701.17a");
    assert_supported(&["Crosstown Courier"]);
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Crosstown Courier");
    let (p, _) = t.pt(courier);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(courier, Entity::Player(P1))], &[]);
    assert_eq!(t.graveyard_size(P1), p as usize);
}

#[test]
fn sacrifice_unless_you_pay() {
    cr!("118.12a", "603.6a");
    assert_supported(&["Archway Commons"]);
    // Can't pay: sacrificed.
    let mut t = TestGame::new(2);
    let commons = enter(&mut t, P0, "Archway Commons");
    t.resolve_all();
    assert!(!t.on_battlefield(commons));
    // Pays {1}: kept.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.answer(P0, DecisionKind::Any, Answer::Bool(true));
    let commons = enter(&mut t, P0, "Archway Commons");
    t.resolve_all();
    assert!(t.on_battlefield(commons));
}

#[test]
fn upkeep_sacrifice_unless_you_pay() {
    cr!("118.12a", "603.2b");
    assert_supported(&["Phantasmal Forces"]);
    let mut t = TestGame::new(2);
    let forces = t.battlefield(P0, "Phantasmal Forces");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    assert!(!t.on_battlefield(forces));
}

#[test]
fn put_into_a_graveyard_from_anywhere_shuffle_it_in() {
    cr!("113.6k", "400.7e");
    assert_supported(&["Dread"]);
    let mut t = TestGame::new(2);
    let dread = t.hand(P0, "Dread");
    let lib = t.library_size(P0);
    t.g.discard(P0, dread, None);
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Dread"));
    assert_eq!(t.library_size(P0), lib + 1);
}

#[test]
fn that_player_draws_an_additional_card() {
    cr!("504.1", "603.2b");
    assert_supported(&["Well of Ideas"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Well of Ideas");
    let hand = t.hand_size(P1);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    // The normal draw plus one additional card.
    assert_eq!(t.hand_size(P1), hand + 2);
}

#[test]
fn you_may_have_it_deal_damage() {
    cr!("603.6a", "120.3");
    assert_supported(&["Aether Charge"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aether Charge");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Leatherback Baloth");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A non-Beast doesn't trigger it.
    enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn damage_to_that_lands_controller() {
    cr!("603.6a", "201.5c");
    assert_supported(&["Zo-Zu the Punisher"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zo-Zu the Punisher");
    enter(&mut t, P1, "Mountain");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}
