//! Double-faced cards put onto the battlefield transformed (CR 712.14a): "When ~ dies,
//! return it to the battlefield transformed", delayed returns, returns from the
//! graveyard, and "exile ~, then return her to the battlefield transformed".

use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn murder(t: &mut TestGame, p: PlayerId, victim: ObjectId) {
    t.lands(p, "Swamp", 3);
    let spell = t.hand(p, "Murder");
    t.cast(p, spell).target(victim).go();
    t.resolve_all();
}

fn one_named(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "expected one {name} on the battlefield");
    v[0]
}

#[test]
fn dies_trigger_returns_the_card_transformed_under_your_control() {
    cr!("712.14a", "400.7e");
    let mut t = TestGame::new(2);
    let hand = t.battlefield(P0, "Harvest Hand // Scrounged Scythe");
    murder(&mut t, P1, hand);
    assert!(!t.in_graveyard(P0, "Harvest Hand"));
    let scythe = one_named(&t, "Scrounged Scythe");
    let o = t.obj_now(scythe);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.controller, P0);
    assert!(o.chars.has_subtype("Equipment"));
    assert!(!o.is(types::CardType::Creature));
}

#[test]
fn tapped_and_transformed_with_counters() {
    cr!("712.14a", "122.6");
    let mut t = TestGame::new(2);
    let taq = t.battlefield(P0, "Ojer Taq, Deepest Foundation // Temple of Civilization");
    murder(&mut t, P1, taq);
    let temple = one_named(&t, "Temple of Civilization");
    assert!(t.obj_now(temple).tapped);
    assert_eq!(t.obj_now(temple).controller, P0);

    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "Ojer Pakpatiq, Deepest Epoch // Temple of Cyclical Time");
    murder(&mut t, P1, god);
    let temple = one_named(&t, "Temple of Cyclical Time");
    assert!(t.obj_now(temple).tapped);
    assert_eq!(t.counters(temple, "time"), 3);
}

#[test]
fn delayed_return_at_the_beginning_of_the_next_end_step() {
    cr!("603.7a", "712.14a");
    let mut t = TestGame::new(2);
    let cathar = t.battlefield(P0, "Loyal Cathar");
    murder(&mut t, P1, cathar);
    // Still in the graveyard until the end step.
    assert!(t.in_graveyard(P0, "Loyal Cathar"));
    assert!(t.named_on_battlefield("Unhallowed Cathar").is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = one_named(&t, "Unhallowed Cathar");
    assert_eq!(t.obj_now(back).controller, P0);
    assert_eq!(t.pt(back), (2, 1));
    assert!(!t.in_graveyard(P0, "Loyal Cathar"));
}

#[test]
fn delayed_return_does_nothing_if_the_card_left_the_graveyard() {
    cr!("603.7c", "400.7");
    let mut t = TestGame::new(2);
    let cathar = t.battlefield(P0, "Loyal Cathar");
    murder(&mut t, P1, cathar);
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Loyal Cathar")[0];
    // The card leaves the graveyard and comes back: it's a new object.
    let exiled = t
        .g
        .move_object(card, Zone::Exile, events::MoveCause::Effect, None)
        .unwrap();
    t.g.move_object(
        exiled,
        Zone::Graveyard(P0),
        events::MoveCause::Effect,
        None,
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Unhallowed Cathar").is_empty());
    assert!(t.in_graveyard(P0, "Loyal Cathar"));
}

#[test]
fn when_you_sacrifice_it_returns_transformed_under_its_owners_control() {
    // The sacrificed permanent's own "when you sacrifice ~" ability looks back in time.
    cr!("712.14a", "603.7a", "603.10a");
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Viscera Seer");
    let egg = t.battlefield(P0, "Biolume Egg // Biolume Serpent");
    t.answer_choose(P0, &[Entity::Object(egg)]);
    t.activate(P0, seer, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Biolume Egg"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let serpent = one_named(&t, "Biolume Serpent");
    assert_eq!(t.obj_now(serpent).controller, P0);
    assert_eq!(t.pt(serpent), (4, 4));
}

#[test]
fn return_this_card_from_your_graveyard_transformed() {
    cr!("712.14a", "602.5d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 4);
    let garland = t.graveyard(P0, "Garland, Knight of Cornelia // Chaos, the Endless");
    t.activate(P0, garland, 0, &[]).unwrap();
    t.resolve_all();
    let chaos = one_named(&t, "Chaos, the Endless");
    assert_eq!(t.obj_now(chaos).face, FaceState::Back);
    assert_eq!(t.pt(chaos), (5, 5));

    // Only as a sorcery.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let awake = t.graveyard(P0, "Startled Awake // Persistent Nightmare");
    t.set_step(P0, Step::Upkeep);
    assert!(t.activate(P0, awake, 0, &[]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, awake, 0, &[]).unwrap();
    t.resolve_all();
    let nightmare = one_named(&t, "Persistent Nightmare");
    assert_eq!(t.obj_now(nightmare).controller, P0);
}

#[test]
fn exile_her_then_return_her_transformed() {
    cr!("712.14a", "400.7j", "306.5b");
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "will enter with loyalty counters as normal"
    );
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "unless a spell or ability instructs you to put it onto the battlefield transformed, in which case it enters with its back face up"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana, Heretical Healer // Liliana, Defiant Necromancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    murder(&mut t, P1, bears);
    let walker = one_named(&t, "Liliana, Defiant Necromancer");
    assert_eq!(t.obj_now(walker).face, FaceState::Back);
    assert_eq!(t.counters(walker, "loyalty"), 3);
    // "If you do": a 2/2 Zombie.
    let zombies: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Zombie") && o.controller == P0)
        .map(|o| o.id)
        .collect();
    assert_eq!(zombies.len(), 1);
    assert_eq!(t.pt(zombies[0]), (2, 2));
}

#[test]
fn only_the_first_of_several_liliana_triggers_makes_a_zombie() {
    cr!("400.7", "603.10a");
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "only the first ability to resolve will create a Zombie token"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana, Heretical Healer // Liliana, Defiant Necromancer");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, Step::PrecombatMain);
    let pyro = t.hand(P1, "Pyroclasm");
    t.cast(P1, pyro).go();
    t.resolve_all();
    // Liliana (2/3) survives Pyroclasm; both Bears die: two triggers, one Zombie.
    let walker = one_named(&t, "Liliana, Defiant Necromancer");
    assert_eq!(t.counters(walker, "loyalty"), 3);
    let zombies = t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Zombie"))
        .count();
    assert_eq!(zombies, 1);
}

#[test]
fn liliana_dying_with_the_other_creature_makes_no_zombie() {
    cr!("400.7", "603.10a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana, Heretical Healer // Liliana, Defiant Necromancer");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 4);
    t.set_step(P1, Step::PrecombatMain);
    let wipe = t.hand(P1, "Damnation");
    t.cast(P1, wipe).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Liliana, Defiant Necromancer").is_empty());
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.chars.has_subtype("Zombie"))
            .count(),
        0
    );
    assert!(t.in_graveyard(P0, "Liliana, Heretical Healer"));
}

#[test]
fn nissa_transforms_with_her_seventh_land() {
    cr!("712.14a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nissa, Vastwood Seer // Nissa, Sage Animist");
    t.lands(P0, "Forest", 5);
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    // Six lands: nothing.
    assert_eq!(t.named_on_battlefield("Nissa, Vastwood Seer").len(), 1);
    let land = t.hand(P0, "Forest");
    t.g.players[0].lands_played_this_turn = 0;
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    let walker = one_named(&t, "Nissa, Sage Animist");
    assert_eq!(t.counters(walker, "loyalty"), 3);
}

#[test]
fn aura_returns_itself_transformed_when_the_enchanted_creature_dies() {
    cr!("400.7f", "712.14a", "704.5m");
    ruling!(
        "Skin Invasion // Skin Shedder",
        "The controller of Skin Invasion, not the controller of the enchanted creature, returns Skin Invasion to the battlefield transformed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let aura = t.hand(P0, "Skin Invasion // Skin Shedder");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert_eq!(t.g.attachments_of(Entity::Object(bears)).len(), 1);
    murder(&mut t, P0, bears);
    let shedder = one_named(&t, "Skin Shedder");
    assert_eq!(t.obj_now(shedder).controller, P0);
    assert_eq!(t.pt(shedder), (3, 4));
    assert!(!t.in_graveyard(P0, "Skin Invasion"));
}

#[test]
fn journey_to_eternity_returns_the_creature_then_itself() {
    cr!("400.7f", "712.14a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let aura = t.hand(P0, "Journey to Eternity // Atzal, Cave of Eternity");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    murder(&mut t, P1, bears);
    let back = one_named(&t, "Grizzly Bears");
    assert_eq!(t.obj_now(back).controller, P0);
    let atzal = one_named(&t, "Atzal, Cave of Eternity");
    assert!(t.obj_now(atzal).is(types::CardType::Land));
}

#[test]
fn journey_to_eternity_and_its_creature_destroyed_together_both_return() {
    cr!("400.7f");
    ruling!(
        "Journey to Eternity // Atzal, Cave of Eternity",
        "If Journey to Eternity and the enchanted creature are both put into graveyards at the same time, Journey to Eternity's ability will return both to the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let aura = t.hand(P0, "Journey to Eternity // Atzal, Cave of Eternity");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 6);
    let wipe = t.hand(P1, "Planar Cleansing");
    t.cast(P1, wipe).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Atzal, Cave of Eternity").len(), 1);
}

#[test]
fn aura_back_face_returns_attached_to_target_opponent() {
    cr!("712.14a", "303.4");
    let mut t = TestGame::new(2);
    let witch = t.battlefield(P0, "Accursed Witch // Infectious Curse");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    murder(&mut t, P1, witch);
    let curse = one_named(&t, "Infectious Curse");
    let o = t.obj_now(curse);
    assert_eq!(o.controller, P0);
    assert_eq!(o.attached_to, Some(Entity::Player(P1)));
}

#[test]
fn aura_back_face_returns_attached_to_target_creature() {
    cr!("712.14a", "303.4");
    let mut t = TestGame::new(2);
    let strangler = t.battlefield(P0, "Vengeful Strangler // Strangling Grasp");
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(victim)]);
    murder(&mut t, P1, strangler);
    let grasp = one_named(&t, "Strangling Grasp");
    assert_eq!(t.obj_now(grasp).controller, P0);
    assert_eq!(t.obj_now(grasp).attached_to, Some(Entity::Object(victim)));
}

#[test]
fn a_stolen_loyal_cathar_returns_under_your_control() {
    cr!("603.7a", "712.14a");
    ruling!(
        "Loyal Cathar // Unhallowed Cathar",
        "If you gain control of a Loyal Cathar owned by another player and it dies, it will be put into to its owner's graveyard and then return to the battlefield under your control."
    );
    ruling!(
        "Loyal Cathar // Unhallowed Cathar",
        "It doesn't return to the battlefield and then transform."
    );
    let mut t = TestGame::new(2);
    let cathar = t.battlefield(P1, "Loyal Cathar // Unhallowed Cathar");
    t.lands(P0, "Island", 5);
    let steal = t.hand(P0, "Mind Control");
    t.cast(P0, steal).target(cathar).go();
    t.resolve_all();
    assert_eq!(t.obj_now(cathar).controller, P0);
    murder(&mut t, P1, cathar);
    assert!(t.in_graveyard(P1, "Loyal Cathar"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = one_named(&t, "Unhallowed Cathar");
    assert_eq!(t.obj_now(back).controller, P0);
    assert_eq!(t.obj_now(back).owner, P1);
    assert_eq!(t.obj_now(back).face, FaceState::Back);
}

const GALLEON: &str = "Conqueror's Galleon // Conqueror's Foothold";

/// Conqueror's Galleon, made a 5/5 artifact creature by Ensoul Artifact.
fn animated_galleon(t: &mut TestGame) -> ObjectId {
    let galleon = t.battlefield(P0, GALLEON);
    t.lands(P0, "Island", 2);
    let aura = t.hand(P0, "Ensoul Artifact");
    t.cast(P0, aura).target(galleon).go();
    t.resolve_all();
    assert_eq!(t.pt(galleon), (5, 5));
    galleon
}

/// Declares the Galleon as an attacker and resolves its attack trigger.
fn galleon_attacks(t: &mut TestGame) -> ObjectId {
    let galleon = animated_galleon(t);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(galleon, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    galleon
}

#[test]
fn exiled_at_end_of_combat_then_returned_transformed() {
    cr!("603.7a", "712.14a", "400.7");
    ruling!(
        "Conqueror's Galleon // Conqueror's Foothold",
        "Conqueror's Galleon is exiled and then returned to battlefield transformed. It will be considered a new object entering the battlefield. Notably, it will return to the battlefield untapped."
    );
    let mut t = TestGame::new(2);
    let galleon = galleon_attacks(&mut t);
    // The attack trigger has resolved: nothing happens until end of combat.
    assert_eq!(t.g.current(galleon), galleon);
    assert!(t.on_battlefield(galleon));
    assert_eq!(t.obj_now(galleon).face, FaceState::Front);
    assert!(t.obj_now(galleon).tapped);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    let foothold = one_named(&t, "Conqueror's Foothold");
    assert_ne!(foothold, galleon);
    let o = t.obj_now(foothold);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.controller, P0);
    assert!(!o.tapped);
    assert!(o.is(types::CardType::Land));
    assert!(t.g.find_in_zone(Zone::Exile, "Conqueror's Galleon").is_empty());
}

#[test]
fn a_galleon_that_dies_in_combat_isnt_exiled_or_returned() {
    cr!("603.7c", "400.7");
    ruling!(
        "Conqueror's Galleon // Conqueror's Foothold",
        "Conqueror's Galleon won't be exiled if it doesn't survive combat."
    );
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    let galleon = animated_galleon(&mut t);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(galleon, Entity::Player(P1))], &[(dreadmaw, galleon)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Conqueror's Galleon"));
    assert!(t.named_on_battlefield("Conqueror's Foothold").is_empty());
    assert!(t.g.find_in_zone(Zone::Exile, "Conqueror's Galleon").is_empty());
}
