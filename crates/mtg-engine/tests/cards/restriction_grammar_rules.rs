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
    // A search P1 would make this turn finds nothing.
    let wilds = t.battlefield(P1, "Evolving Wilds");
    let forest = t.library_top(P1, "Forest");
    t.activate(P1, wilds, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.zone(forest), Zone::Library(P1));
    assert!(!t.on_battlefield(wilds));
}

#[test]
fn that_player_cant_gain_life_for_the_rest_of_the_game() {
    cr!("119.7");
    compiles("Stigma Lasher");
    let mut t = TestGame::new(2);
    let lasher = t.battlefield(P0, "Stigma Lasher");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(lasher, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.g.gain_life(P1, 3);
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 23);
    // Turns later, still not.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.g.gain_life(P1, 3);
    assert_eq!(t.life(P1), 18);
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
fn creature_spells_you_cast_this_turn_cant_be_countered() {
    cr!("701.6a", "611.2c");
    ruling!("Domri, Anarch of Bolas", "not just the one you spend the mana on");
    ruling!("Domri, Anarch of Bolas", "can still target a creature spell you control");
    let mut t = TestGame::new(2);
    let domri = t.battlefield(P0, "Domri, Anarch of Bolas");
    t.activate(P0, domri, 0, &[]).unwrap();
    t.resolve();
    t.lands(P0, "Forest", 6);
    t.lands(P1, "Island", 12);
    for _ in 0..4 {
        t.library_top(P1, "Island");
        t.library_top(P0, "Island");
    }
    // Two creature spells cast later this turn: neither can be countered, and a spell that
    // counters creature spells can still target one and its other effects happen.
    for name in ["Grizzly Bears", "Centaur Courser"] {
        let c = t.hand(P0, name);
        let spell = t.cast(P0, c).go();
        let deny = t.hand(P1, "Deny Entry");
        let gy = t.graveyard_size(P1);
        t.cast(P1, deny).target(spell).go();
        t.resolve();
        // Deny Entry and the card it discarded after drawing.
        assert_eq!(t.graveyard_size(P1), gy + 2);
        t.resolve();
        assert_eq!(t.named_on_battlefield(name).len(), 1);
    }
    // A noncreature spell can still be countered.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    // Next turn, creature spells can be countered again.
    t.advance_to(P1, Step::Upkeep);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, c).go();
    let deny = t.hand(P1, "Deny Entry");
    t.cast(P1, deny).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
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

#[test]
fn display_of_dominance_modes() {
    cr!("700.2a", "115.4");
    compiles("Display of Dominance");
    // "Destroy target blue or black noncreature permanent": noncreature applies to both
    // colors.
    let mut t = TestGame::new(2);
    let display = t.hand(P0, "Display of Dominance");
    t.lands(P0, "Forest", 2);
    let blue_creature = t.battlefield(P1, "Coral Merfolk");
    let black_creature = t.battlefield(P1, "Vampire Bats");
    let blue_enchantment = t.battlefield(P1, "Propaganda");
    let black_enchantment = t.battlefield(P1, "Bad Moon");
    let red_enchantment = t.battlefield(P1, "Pyrohemia");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    let c = target_candidates(&mut t, P0, display);
    assert!(c.contains(&Entity::Object(blue_enchantment)));
    assert!(c.contains(&Entity::Object(black_enchantment)));
    assert!(!c.contains(&Entity::Object(blue_creature)));
    assert!(!c.contains(&Entity::Object(black_creature)));
    assert!(!c.contains(&Entity::Object(red_enchantment)));

    // "Permanents you control can't be the targets of blue or black spells your opponents
    // control this turn."
    let mut t = TestGame::new(2);
    let display = t.hand(P0, "Display of Dominance");
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, display).modes(&[1]).go();
    t.resolve();
    t.lands(P1, "Swamp", 2);
    let own = t.battlefield(P1, "Hill Giant");
    let blade = t.hand(P1, "Doom Blade");
    let c = target_candidates(&mut t, P1, blade);
    assert!(c.contains(&Entity::Object(own)));
    assert!(!c.contains(&Entity::Object(bears)));
    let mut t2 = TestGame::new(2);
    let display = t2.hand(P0, "Display of Dominance");
    t2.lands(P0, "Forest", 2);
    let bears = t2.battlefield(P0, "Grizzly Bears");
    t2.cast(P0, display).modes(&[1]).go();
    t2.resolve();
    t2.lands(P1, "Mountain", 1);
    let shock = t2.hand(P1, "Shock");
    assert!(target_candidates(&mut t2, P1, shock).contains(&Entity::Object(bears)));
}

#[test]
fn the_next_spell_you_cast_this_turn_cant_be_countered() {
    cr!("611.2f", "701.6a");
    compiles("Mistrise Village");
    let mut t = TestGame::new(2);
    let village = t.battlefield(P0, "Mistrise Village");
    t.g.untap(village);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Island", 6);
    t.activate(P0, village, 1, &[]).unwrap();
    t.resolve();
    // An instant: the next spell of any kind.
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // Only the next one.
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn permanents_cant_phase_in() {
    cr!("702.26a");
    // Disciple of Caelus Nin: "Permanents can't phase in." (its enters ability isn't
    // compiled yet; the static is).
    let def = card("Disciple of Caelus Nin");
    assert!(!def
        .unsupported_text()
        .iter()
        .any(|t| t.contains("can't phase in")));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let disciple = t.battlefield(P0, "Disciple of Caelus Nin");
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![bears]);
    assert!(t.g.obj(bears).phased_out);
    // P1's untap step doesn't phase it in.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.obj(bears).phased_out);
    // Once the Disciple is gone, it phases in during its controller's next untap step.
    t.g.destroy(disciple, None);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.obj(bears).phased_out);
}

#[test]
fn enchanted_permanent_cant_transform() {
    cr!("701.27a");
    compiles("Bound by Moonsilver");
    let mut t = TestGame::new(2);
    let messenger = t.battlefield(P1, "Village Messenger // Moonrise Intruder");
    let other = t.battlefield(P1, "Village Messenger // Moonrise Intruder");
    let aura = t.battlefield(P0, "Bound by Moonsilver");
    t.g.attach(aura, Entity::Object(messenger));
    t.g.recompute();
    assert!(!mtg_engine::transform_rules::can_transform(&t.g, messenger));
    assert!(mtg_engine::transform_rules::can_transform(&t.g, other));
}
