//! Damage, destruction, exile, bounce and removal spells compiled from oracle text
//! (patterns in `src/oracle/patterns/damage_removal.rs`).

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn removal_cards_compile() {
    assert_compiles(&[
        // Adjective lists in object phrases.
        "Divine Verdict",
        "Hamato Ninpō",
        "Evaporate",
        // "It/They can't be regenerated."
        "Wrath of God",
        "Terminate",
        "Sever Soul",
        // Multi-part and counted-target damage.
        "Psionic Blast",
        "Chandra's Outrage",
        "Seismic Wave",
        "Furious Reprisal",
        "Arc Lightning",
        "Electrolyze",
        "Rumbling Rockslide",
        "Self-Destruct",
        "Wave of Reckoning",
        "Simoon",
        // Exile instead of dying.
        "Puncturing Blow",
        "Anger of the Gods",
        // Prevention.
        "Master Healer",
        "Healing Salve",
        "Impractical Joke",
        // Exile until / flicker.
        "Banisher Priest",
        "Momentary Blink",
        "Turn to Mist",
        // Graveyards, bounce, edicts, pairs, delayed removal.
        "Bojuka Bog",
        "Selesnya Sanctuary",
        "Diabolic Edict",
        "Spiteful Blow",
        "Cinder Wall",
        // Delayed sacrifice.
        "Slave of Bolas",
        "Spinal Embrace",
        // "Can't be regenerated this turn."
        "Engulfing Flames",
        "Gravebind",
        "Jaya Ballard, Task Mage",
        "Lim-Dûl's Cohort",
    ]);
}

// ---------------------------------------------------------------------------
// Targets and filters
// ---------------------------------------------------------------------------

#[test]
fn divine_verdict_targets_only_attacking_or_blocking_creatures() {
    cr!("601.2c", "115.1");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Plains", 4);
    let verdict = t.hand(P1, "Divine Verdict");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::DeclareAttackers);
    t.cast(P1, verdict).target(attacker).go();
    // Only the attacking creature was a legal choice.
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if p == P1 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands.last().unwrap(), &vec![Entity::Object(attacker)]);
    t.resolve();
    assert!(!t.on_battlefield(attacker));
    assert!(t.on_battlefield(idle));
}

#[test]
fn evaporate_hits_white_and_or_blue_creatures() {
    cr!("120.3e", "704.5g");
    let mut t = TestGame::new(2);
    // Savannah Lions (white 2/1), Merfolk of the Pearl Trident (blue 1/1), Grizzly Bears
    // (green 2/2).
    let lions = t.battlefield(P1, "Savannah Lions");
    let merfolk = t.battlefield(P1, "Merfolk of the Pearl Trident");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let e = t.hand(P0, "Evaporate");
    t.cast(P0, e).go();
    t.resolve();
    assert!(!t.on_battlefield(lions));
    assert!(!t.on_battlefield(merfolk));
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------
// Destroy / regeneration
// ---------------------------------------------------------------------------

#[test]
fn murder_is_stopped_by_regeneration_but_wrath_of_god_is_not() {
    cr!("701.19a", "701.19c");
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Plains", 4);
    // Regeneration shield.
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(boa).go();
    t.resolve();
    assert!(t.on_battlefield(boa), "regenerated");
    assert!(t.obj_now(boa).tapped);
    // A new shield, then Wrath of God: "They can't be regenerated."
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    assert!(!t.on_battlefield(boa));
    assert!(t.in_graveyard(P1, "River Boa"));
}

#[test]
fn sever_soul_ignores_regeneration_and_gains_life_equal_to_toughness() {
    cr!("701.19c", "608.2h");
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    t.lands(P0, "Swamp", 5);
    let s = t.hand(P0, "Sever Soul");
    t.cast(P0, s).target(boa).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "River Boa"));
    // River Boa is 2/1: its last known toughness is 1.
    assert_eq!(t.life(P0), 21);
}

#[test]
fn spiteful_blow_destroys_a_creature_and_a_land() {
    cr!("701.8a", "608.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Swamp", 6);
    let s = t.hand(P0, "Spiteful Blow");
    t.cast(P0, s).target(bears).target(land).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(land));
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

#[test]
fn psionic_blast_deals_damage_to_any_target_and_to_you() {
    cr!("120.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let b = t.hand(P0, "Psionic Blast");
    t.cast(P0, b).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn chandras_outrage_also_damages_the_creatures_controller() {
    cr!("120.3a", "120.3e", "704.5g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Chandra's Outrage");
    t.cast(P0, c).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn seismic_wave_damages_nonartifact_creatures_the_target_opponent_controls() {
    cr!("115.1", "120.3e");
    let mut t = TestGame::new(2);
    let their_elves = t.battlefield(P1, "Llanowar Elves");
    let their_memnite = t.battlefield(P1, "Memnite");
    let my_elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Mountain", 3);
    let w = t.hand(P0, "Seismic Wave");
    // Slot 0: any target; slot 1: target opponent.
    t.cast(P0, w).target(P1).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(their_elves));
    assert!(t.on_battlefield(their_memnite), "artifact creature spared");
    assert!(t.on_battlefield(my_elves));
}

#[test]
fn simoon_only_damages_creatures_the_target_opponent_controls() {
    cr!("115.1", "120.3e", "704.5g");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Simoon");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(mine));
}

#[test]
fn furious_reprisal_deals_damage_to_each_of_two_targets() {
    cr!("115.1", "120.3e");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let f = t.hand(P0, "Furious Reprisal");
    t.cast(P0, f)
        .targets(&[Entity::Object(a), Entity::Player(P1)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn arc_lightning_divides_damage_as_chosen() {
    cr!("601.2d", "120.3e");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P0, "Arc Lightning");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, a)
        .targets(&[Entity::Object(bears), Entity::Player(P1)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn divided_damage_assigned_to_an_illegal_target_is_not_dealt() {
    cr!("608.2b", "601.2d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P0, "Arc Lightning");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, a)
        .targets(&[Entity::Object(bears), Entity::Player(P1)])
        .go();
    // The creature leaves before Arc Lightning resolves; the player still gets only
    // the 1 damage assigned to them.
    t.g.destroy(bears, None);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn rumbling_rockslide_counts_lands_you_control() {
    cr!("608.2h", "120.3e");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 5);
    let r = t.hand(P0, "Rumbling Rockslide");
    t.cast(P0, r).target(wurm).go();
    t.resolve();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 5);
}

#[test]
fn self_destruct_uses_the_targeted_creatures_power() {
    cr!("120.3a", "120.3e", "704.5g");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Self-Destruct");
    t.cast(P0, s).target(giant).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn wave_of_reckoning_makes_each_creature_damage_itself() {
    cr!("120.3e", "704.5g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let walker = t.battlefield(P1, "Phyrexian Walker");
    let mastodon = t.battlefield(P0, "Siege Mastodon");
    t.lands(P0, "Plains", 5);
    let w = t.hand(P0, "Wave of Reckoning");
    t.cast(P0, w).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(walker));
    assert!(t.on_battlefield(mastodon));
    assert_eq!(t.obj_now(mastodon).damage, 3);
}

// ---------------------------------------------------------------------------
// "If that creature would die this turn, exile it instead."
// ---------------------------------------------------------------------------

#[test]
fn puncturing_blow_exiles_the_creature_if_it_dies_this_turn() {
    cr!("614.1a", "614.6", "700.4");
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    // Four for Puncturing Blow, one for Shock.
    t.lands(P0, "Mountain", 5);
    let p = t.hand(P0, "Puncturing Blow");
    t.cast(P0, p).target(dreadmaw).go();
    t.resolve();
    // 5 damage to a 6/6: it survives for now.
    assert!(t.on_battlefield(dreadmaw));
    // Later this turn it dies: it's exiled instead.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(dreadmaw).go();
    t.resolve();
    assert!(!t.on_battlefield(dreadmaw));
    assert!(t.in_exile("Colossal Dreadmaw"));
    assert!(!t.in_graveyard(P1, "Colossal Dreadmaw"));
}

#[test]
fn anger_of_the_gods_exiles_only_creatures_it_dealt_damage() {
    cr!("614.1a", "120.3e");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mastodon = t.battlefield(P1, "Siege Mastodon");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Swamp", 3);
    let a = t.hand(P0, "Anger of the Gods");
    t.cast(P0, a).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    // Siege Mastodon (3/5) was dealt damage this way and dies later this turn.
    let later = t.battlefield(P1, "Grizzly Bears");
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(mastodon).go();
    t.resolve();
    assert!(t.in_exile("Siege Mastodon"));
    // A creature that wasn't dealt damage this way goes to the graveyard as usual.
    t.g.destroy(later, None);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let _ = bears;
}

// ---------------------------------------------------------------------------
// Prevention
// ---------------------------------------------------------------------------

#[test]
fn master_healer_prevents_the_next_four_damage() {
    cr!("615.7", "615.1a");
    let mut t = TestGame::new(2);
    let healer = t.battlefield(P1, "Master Healer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P1, healer, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    t.lands(P0, "Mountain", 2);
    let b1 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears), "3 of the 4 prevented");
    assert_eq!(t.obj_now(bears).damage, 0);
    // One point of the shield is left.
    let b2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b2).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn impractical_joke_damage_cant_be_prevented() {
    cr!("615.12");
    let mut t = TestGame::new(2);
    let healer = t.battlefield(P1, "Master Healer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P1, healer, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    t.lands(P0, "Mountain", 1);
    let j = t.hand(P0, "Impractical Joke");
    t.cast(P0, j).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------
// Exile until ~ leaves the battlefield (CR 610.3) and flicker
// ---------------------------------------------------------------------------

#[test]
fn banisher_priest_exiles_until_it_leaves_the_battlefield() {
    cr!("610.3", "610.3c", "400.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let priest = t.enter(P0, "Banisher Priest");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_exile("Grizzly Bears"));
    // The Priest dies: the creature returns immediately, under its owner's control.
    t.g.destroy(priest, None);
    t.settle();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P1);
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn banisher_priest_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let priest = t.enter(P0, "Banisher Priest");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(priest, None);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn momentary_blink_returns_a_new_object_that_triggers_again() {
    cr!("400.7", "603.6a");
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    t.lands(P0, "Plains", 2);
    let hand_before = t.hand_size(P0);
    let blink = t.hand(P0, "Momentary Blink");
    t.cast(P0, blink).target(visionary).go();
    t.resolve_all();
    let now = t.named_on_battlefield("Elvish Visionary");
    assert_eq!(now.len(), 1);
    assert_ne!(now[0], visionary, "a new object");
    // Its enters trigger drew a card (Momentary Blink itself left the hand).
    assert_eq!(t.hand_size(P0), hand_before + 1);
}

#[test]
fn turn_to_mist_returns_the_creature_at_the_next_end_step() {
    cr!("603.7a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let m = t.hand(P0, "Turn to Mist");
    t.cast(P0, m).target(bears).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P1);
}

// ---------------------------------------------------------------------------
// Graveyards, bounce, edicts, delayed removal
// ---------------------------------------------------------------------------

#[test]
fn bojuka_bog_exiles_target_players_graveyard() {
    cr!("406.1", "115.1");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Bojuka Bog");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn selesnya_sanctuary_returns_a_land_you_control() {
    cr!("608.2d");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let sanctuary = t.enter(P0, "Selesnya Sanctuary");
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.on_battlefield(sanctuary));
}

#[test]
fn diabolic_edict_lets_the_player_choose_what_to_sacrifice() {
    cr!("701.21a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P1, &[Entity::Object(elves)]);
    let e = t.hand(P0, "Diabolic Edict");
    t.cast(P0, e).target(P1).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
}

#[test]
fn cinder_wall_is_destroyed_at_end_of_combat_after_blocking() {
    cr!("603.7a", "511.2");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Phyrexian Walker");
    let wall = t.battlefield(P1, "Cinder Wall");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(walker, Entity::Player(P1))], &[(wall, walker)]);
    // The 0/3 Walker dealt no damage; the wall's delayed trigger is waiting.
    assert!(t.on_battlefield(wall));
    t.resolve_all();
    assert!(!t.on_battlefield(wall));
    assert!(t.in_graveyard(P1, "Cinder Wall"));
}

#[test]
fn slave_of_bolas_sacrifices_the_stolen_creature_at_the_next_end_step() {
    cr!("603.7a", "701.21a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Slave of Bolas");
    t.cast(P0, s).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn spinal_embrace_gains_life_equal_to_the_sacrificed_creatures_toughness() {
    cr!("603.7a", "608.2h");
    let mut t = TestGame::new(2);
    let mastodon = t.battlefield(P1, "Siege Mastodon");
    t.lands(P0, "Island", 5);
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    let s = t.hand(P0, "Spinal Embrace");
    t.cast(P0, s).target(mastodon).go();
    t.resolve();
    assert_eq!(t.obj_now(mastodon).controller, P0);
    assert_eq!(t.life(P0), 20, "no life yet");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Siege Mastodon"));
    assert_eq!(t.life(P0), 25, "Siege Mastodon's toughness is 5");
}

#[test]
fn spinal_embrace_gains_nothing_if_the_creature_is_gone() {
    cr!("603.7c", "701.21a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    let s = t.hand(P0, "Spinal Embrace");
    t.cast(P0, s).target(bears).go();
    t.resolve();
    // The creature leaves before the delayed trigger resolves: nothing is sacrificed.
    t.g.destroy(bears, None);
    t.settle();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------
// "Can't be regenerated this turn" (CR 701.19c)
// ---------------------------------------------------------------------------

#[test]
fn engulfing_flames_stops_regeneration_from_lethal_damage() {
    cr!("701.19c", "704.5g");
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    t.lands(P0, "Mountain", 1);
    let f = t.hand(P0, "Engulfing Flames");
    t.cast(P0, f).target(boa).go();
    t.resolve();
    assert!(!t.on_battlefield(boa), "the shield doesn't apply");
    assert!(t.in_graveyard(P1, "River Boa"));
}

#[test]
fn gravebind_stops_regeneration_for_later_destruction_this_turn() {
    cr!("701.19c", "701.19a");
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    t.lands(P0, "Swamp", 5);
    let g = t.hand(P0, "Gravebind");
    t.cast(P0, g).target(boa).go();
    t.resolve();
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(boa).go();
    t.resolve();
    assert!(!t.on_battlefield(boa));
}

#[test]
fn regeneration_still_works_without_the_restriction() {
    cr!("701.19a");
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    let other = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve();
    t.lands(P0, "Swamp", 5);
    // Gravebind on the other Boa doesn't affect this one.
    let g = t.hand(P0, "Gravebind");
    t.cast(P0, g).target(other).go();
    t.resolve();
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(boa).go();
    t.resolve();
    assert!(t.on_battlefield(boa), "regenerated");
}

// ---------------------------------------------------------------------------
// "Target player or planeswalker and each creature that player or that
// planeswalker's controller controls"
// ---------------------------------------------------------------------------

#[test]
fn bonfire_of_the_damned_hits_the_player_and_their_creatures() {
    cr!("115.1", "120.3");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Siege Mastodon");
    t.lands(P0, "Mountain", 5);
    let b = t.hand(P0, "Bonfire of the Damned");
    t.cast(P0, b).target(P1).x(2).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(theirs));
    assert_eq!(t.obj_now(big).damage, 2);
    assert!(t.on_battlefield(mine));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn bonfire_of_the_damned_on_a_planeswalker_hits_its_controllers_creatures() {
    cr!("115.1", "120.3c");
    let mut t = TestGame::new(2);
    let jace = t.enter(P1, "Jace Beleren");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let b = t.hand(P0, "Bonfire of the Damned");
    t.cast(P0, b).target(jace).x(2).go();
    t.resolve();
    assert_eq!(t.counters(jace, "loyalty"), 1);
    assert_eq!(t.life(P1), 20, "the player isn't dealt damage");
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(mine));
}

// ---------------------------------------------------------------------------
// Exile "edicts": a player exiles cards or permanents of their choice
// ---------------------------------------------------------------------------

#[test]
fn exile_edict_cards_compile() {
    assert_compiles(&[
        "Unscrupulous Agent",
        "Vessel of Malignity",
        "Yarok's Fenlurker",
        "Scrabbling Claws",
        "Debt to the Kami",
    ]);
}

#[test]
fn unscrupulous_agent_makes_the_opponent_exile_a_card_of_their_choice() {
    cr!("608.2d", "406.1");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bears = t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.enter(P0, "Unscrupulous Agent");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
}

#[test]
fn yaroks_fenlurker_makes_each_opponent_exile_a_card() {
    cr!("608.2d");
    let mut t = TestGame::new(3);
    t.hand(P1, "Lightning Bolt");
    t.hand(P2, "Grizzly Bears");
    let mine = t.hand(P0, "Shock");
    let h1 = t.hand_size(P1);
    let h2 = t.hand_size(P2);
    t.enter(P0, "Yarok's Fenlurker");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 - 1);
    assert_eq!(t.hand_size(P2), h2 - 1);
    assert_eq!(t.zone(mine), Zone::Hand(P0));
}

#[test]
fn debt_to_the_kami_exiles_a_creature_the_opponent_chooses() {
    cr!("608.2d", "700.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.answer_choose(P1, &[Entity::Object(elves)]);
    let d = t.hand(P0, "Debt to the Kami");
    t.cast(P0, d).modes(&[0]).target(P1).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
    assert!(t.in_exile("Llanowar Elves"));
}

#[test]
fn end_the_festivities_hits_opponents_and_their_creatures_and_planeswalkers() {
    cr!("120.3", "120.3c");
    assert_compiles(&["End the Festivities", "Fulminous Forte"]);
    let mut t = TestGame::new(2);
    let jace = t.enter(P1, "Jace Beleren");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let land = t.battlefield(P1, "Dryad Arbor");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Mountain", 1);
    let e = t.hand(P0, "End the Festivities");
    t.cast(P0, e).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(jace, "loyalty"), 2);
    assert!(!t.on_battlefield(elves));
    assert!(!t.on_battlefield(land), "a land creature is a creature");
    assert!(t.on_battlefield(mine));
}

#[test]
fn ratchet_bomb_destroys_permanents_with_mana_value_equal_to_its_counters() {
    cr!("608.2h", "113.7a");
    assert_compiles(&["Ratchet Bomb", "Powder Keg", "Wave of Terror", "Pernicious Deed"]);
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Ratchet Bomb");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let forest = t.battlefield(P1, "Forest");
    for _ in 0..2 {
        t.activate(P0, bomb, 0, &[]).unwrap();
        t.resolve();
        t.g.objects[bomb.0 as usize].tapped = false;
    }
    assert_eq!(t.counters(bomb, "charge"), 2);
    // Sacrificed as a cost: the number of counters is its last known information.
    t.activate(P0, bomb, 1, &[]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(elves));
    assert!(t.on_battlefield(forest));
}

#[test]
fn pernicious_deed_destroys_artifacts_creatures_and_enchantments() {
    cr!("107.3a", "701.8a");
    let mut t = TestGame::new(2);
    let deed = t.battlefield(P0, "Pernicious Deed");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let myr = t.battlefield(P1, "Ornithopter");
    let cop = t.battlefield(P1, "Circle of Protection: Red");
    let mastodon = t.battlefield(P1, "Siege Mastodon");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Swamp", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, deed, 0, &[]).unwrap();
    t.resolve();
    for o in [bears, myr, cop] {
        assert!(!t.on_battlefield(o));
    }
    assert!(t.on_battlefield(mastodon), "mana value 5");
    assert!(t.on_battlefield(forest));
}

// ---------------------------------------------------------------------------
// "Other target" after an earlier target, and replacement effects that aren't
// regeneration
// ---------------------------------------------------------------------------

/// The candidates offered by `p`'s ChooseTargets decisions, in order.
fn target_candidates(t: &TestGame, p: PlayerId) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .collect()
}

#[test]
fn trick_shots_other_target_must_be_a_different_token() {
    cr!("115.3", "601.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let token_of = |t: &mut TestGame| {
        let spec = mtg_engine::replacement::TokenCreate {
            chars: t.g.obj(bears).copiable.clone(),
            card: t.g.obj(bears).card.clone(),
            tapped: false,
            attacking: None,
            copy_of: Some(bears),
            copy_exceptions: vec![],
        };
        t.g.create_tokens(P1, spec, 1, None)[0]
    };
    let a = token_of(&mut t);
    let b = token_of(&mut t);
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Trick Shot");
    // Choosing the same token for "up to one other target creature token" isn't allowed.
    t.cast(P0, s).target(a).target(a).go();
    let cands = target_candidates(&t, P0);
    assert_eq!(cands.len(), 2);
    assert!(cands[0].contains(&Entity::Object(a)));
    assert_eq!(cands[1], vec![Entity::Object(b)]);
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(bears));
}

#[test]
fn relic_crushs_second_target_is_another_permanent() {
    cr!("115.3", "701.8a");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let memnite = t.battlefield(P1, "Memnite");
    t.lands(P0, "Forest", 5);
    let r = t.hand(P0, "Relic Crush");
    t.cast(P0, r).target(thopter).target(memnite).go();
    let cands = target_candidates(&t, P0);
    assert_eq!(cands.len(), 2);
    assert!(!cands[1].contains(&Entity::Object(thopter)));
    t.resolve();
    assert!(!t.on_battlefield(thopter));
    assert!(!t.on_battlefield(memnite));
}

#[test]
fn cant_be_regenerated_leaves_other_destruction_replacements_alone() {
    cr!("701.19c", "614.1a");
    use mtg_engine::ability::*;
    use mtg_engine::card::CardDef;
    use mtg_engine::object::Characteristics;
    use mtg_engine::types::CardType;
    let mut t = TestGame::new(2);
    // A 2/2 with "If this creature would be destroyed, exile it instead." — a
    // replacement effect that isn't regeneration.
    let mut chars = Characteristics {
        name: "Exile Instead".into(),
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    chars.card_types.insert(CardType::Creature);
    chars.abilities.push(AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Destroy(Filter::Source),
                action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
                self_replacement: false,
                optional: false,
            },
        ))),
        "If this creature would be destroyed, exile it instead.",
    ));
    let odd = t.custom(P1, CardDef::custom(chars), Zone::Battlefield);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    // "They can't be regenerated" stops only regeneration shields and effects.
    assert!(!t.on_battlefield(odd));
    assert!(t.in_exile("Exile Instead"));
    assert!(!t.in_graveyard(P1, "Exile Instead"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    let _ = elves;
}
