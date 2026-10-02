//! Rules prohibitions compiled by the restriction grammar: "can't become untapped",
//! "can't be equipped", "can't be enchanted by other Auras", "can't become suspected",
//! "can't be turned face up", "[cards] in graveyards can't enter the battlefield",
//! "Players can't get counters", "Counters can't be put on ...", "Players can't gain
//! life this turn", "Players can't search libraries this turn", "can't be countered"
//! effects (this turn, target spell, the next spell), targeting restrictions with
//! opponents' sources, and "its activated abilities can't be activated" effects.

use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The candidates offered when `p` casts `card` (its first target).
fn target_candidates(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<Entity> {
    let from = t.asked().len();
    t.cast(p, card).go();
    t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
}

/// Whether `p` may activate any ability of `src` now.
fn can_activate(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == src))
}

#[test]
fn cant_become_untapped() {
    cr!("701.26b");
    compiles("Frozen in Ice");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let aura = t.hand(P0, "Frozen in Ice");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    // Neither an effect nor its controller's untap step untaps it.
    assert!(!t.g.untap(bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn cant_be_equipped_and_cant_be_enchanted_by_other_auras() {
    cr!("301.5c", "303.4d", "704.5m");
    compiles("Goblin Brawler");
    compiles("Consecrate Land");
    let mut t = TestGame::new(2);
    let brawler = t.battlefield(P0, "Goblin Brawler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Short Sword");
    assert!(!mtg_engine::attach::can_attach(&t.g, sword, Entity::Object(brawler)));
    assert!(mtg_engine::attach::can_attach(&t.g, sword, Entity::Object(bears)));

    // Consecrate Land enchants a land; another Aura already on it falls off (SBA).
    let mut t = TestGame::new(2);
    let land = t.lands(P0, "Plains", 1)[0];
    let other = t.battlefield(P0, "Wild Growth");
    t.g.attach(other, Entity::Object(land));
    let consecrate = t.battlefield(P0, "Consecrate Land");
    t.g.attach(consecrate, Entity::Object(land));
    t.settle();
    assert_eq!(t.obj_now(consecrate).attached_to, Some(Entity::Object(land)));
    assert!(!t.on_battlefield(other));
    assert!(t.in_graveyard(P0, "Wild Growth"));
}

#[test]
fn cant_become_suspected() {
    cr!("701.60a");
    compiles("Airtight Alibi");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let alibi = t.battlefield(P0, "Airtight Alibi");
    t.g.attach(alibi, Entity::Object(bears));
    assert!(!mtg_engine::kwa::suspect_detain::suspect(&mut t.g, bears));
    assert!(!t.obj_now(bears).suspected);
    let other = t.battlefield(P0, "Hill Giant");
    assert!(mtg_engine::kwa::suspect_detain::suspect(&mut t.g, other));
}

#[test]
fn opponents_permanents_cant_be_turned_face_up_during_your_turn() {
    cr!("708.8", "701.40b");
    compiles("Karlov Watchdog");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karlov Watchdog");
    // P1 manifests a Hill Giant during their turn.
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.library_top(P1, "Hill Giant");
    t.lands(P1, "Plains", 2);
    let summons = t.hand(P1, "Soul Summons");
    t.cast(P1, summons).go();
    t.resolve_all();
    let m = t.g.current(giant);
    assert!(t.obj_now(m).face_down);
    t.lands(P1, "Mountain", 4);
    let turn_up = Action::Special(SpecialAction::TurnFaceUp { obj: m });
    t.g.turn.priority = Some(P1);
    assert!(t.g.legal_actions(P1).contains(&turn_up));
    // During P0's turn, it can't be turned face up.
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    assert!(!t.g.legal_actions(P1).contains(&turn_up));
}

#[test]
fn creature_cards_in_graveyards_cant_enter_the_battlefield() {
    cr!("614.17d");
    ruling!(
        "Grafdigger's Cage",
        "Look at the card as it exists in your graveyard to determine whether it can enter"
    );
    compiles("Grafdigger's Cage");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grafdigger's Cage");
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let reanimate = t.hand(P0, "Exhume");
    t.cast(P0, reanimate).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.zone(dead), Zone::Graveyard(P0));
    // A creature card from the hand still enters.
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn players_cant_get_counters_and_counters_cant_be_put_on_permanents() {
    cr!("122.1", "614.1");
    compiles("Solemnity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solemnity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.g.add_counters(Entity::Player(P1), "poison", 3, None), 0);
    assert_eq!(t.g.player(P1).counter("poison"), 0);
}

#[test]
fn players_cant_gain_life_or_search_this_turn() {
    cr!("119.7", "701.23");
    compiles("Skullcrack");
    compiles("Shadow of Doubt");
    let mut t = TestGame::new(2);
    let crack = t.hand(P0, "Skullcrack");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, crack).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    t.g.gain_life(P1, 5);
    t.g.gain_life(P0, 5);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    // Next turn, they can.
    t.advance_to(P1, Step::Upkeep);
    t.g.gain_life(P1, 1);
    assert_eq!(t.life(P1), 18);

    let mut t = TestGame::new(2);
    let doubt = t.hand(P0, "Shadow of Doubt");
    t.lands(P0, "Island", 2);
    t.cast(P0, doubt).go();
    t.resolve();
    assert!(t
        .g
        .player_restricted(P1, |r| matches!(r, Restriction::CantSearch(_))));
}

#[test]
fn spells_cant_be_countered_effects() {
    cr!("701.6a", "611.2f");
    compiles("Insist");
    compiles("Vexing Shusher");
    compiles("Domri, Anarch of Bolas");
    // "The next creature spell you cast this turn can't be countered."
    let mut t = TestGame::new(2);
    let insist = t.hand(P0, "Insist");
    t.lands(P0, "Forest", 3);
    t.cast(P0, insist).go();
    t.resolve();
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let counter = t.hand(P1, "Cancel");
    t.lands(P1, "Island", 3);
    t.cast(P1, counter).target(spell).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);

    // "Target spell can't be countered."
    let mut t = TestGame::new(2);
    let shusher = t.battlefield(P0, "Vexing Shusher");
    t.lands(P0, "Mountain", 4);
    let giant = t.hand(P0, "Hill Giant");
    let spell = t.cast(P0, giant).go();
    t.lands(P0, "Forest", 1);
    t.activate(P0, shusher, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    let counter = t.hand(P1, "Cancel");
    t.lands(P1, "Island", 3);
    t.cast(P1, counter).target(spell).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn cant_be_the_target_of_spells_or_abilities_your_opponents_control() {
    cr!("115.4");
    compiles("Fiendslayer Paladin");
    compiles("Shanna, Sisay's Legacy");
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Fiendslayer Paladin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An opponent's black spell can't target it; a blue one can.
    t.lands(P1, "Swamp", 2);
    let doom = t.hand(P1, "Doom Blade");
    let c = target_candidates(&mut t, P1, doom);
    assert!(c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(paladin)));
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Fiendslayer Paladin");
    t.lands(P1, "Island", 1);
    let bounce = t.hand(P1, "Unsummon");
    assert!(target_candidates(&mut t, P1, bounce).contains(&Entity::Object(paladin)));
    // Its controller's own black spell can.
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Fiendslayer Paladin");
    t.lands(P0, "Swamp", 2);
    let own = t.hand(P0, "Doom Blade");
    assert!(target_candidates(&mut t, P0, own).contains(&Entity::Object(paladin)));

    // Shanna: an opponent's ability can't target it; an opponent's spell can.
    let mut t = TestGame::new(2);
    let shanna = t.battlefield(P0, "Shanna, Sisay's Legacy");
    t.battlefield(P0, "Grizzly Bears");
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(t.g.object_untargetable(shanna, P1, Some(pinger)));
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    assert!(target_candidates(&mut t, P1, shock).contains(&Entity::Object(shanna)));
}

#[test]
fn its_activated_abilities_cant_be_activated_this_turn() {
    cr!("602.5");
    compiles("Deadlock Trap");
    let mut t = TestGame::new(2);
    let trap = t.battlefield(P0, "Deadlock Trap");
    t.g.untap(trap);
    t.g.add_counters(Entity::Player(P0), "energy", 1, None);
    let pinger = t.battlefield(P1, "Prodigal Pyromancer");
    let other = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P0, trap, 0, &[Entity::Object(pinger)]).unwrap();
    t.resolve();
    assert!(t.obj_now(pinger).tapped);
    t.g.untap(pinger);
    assert!(!can_activate(&mut t, P1, pinger));
    assert!(can_activate(&mut t, P1, other));
    // Next turn, it can.
    t.advance_to(P1, Step::Upkeep);
    assert!(can_activate(&mut t, P1, pinger));
}
