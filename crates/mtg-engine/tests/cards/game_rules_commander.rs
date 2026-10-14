//! Commander rules in Oracle text (CR 903): "commander" as a noun and an adjective
//! ("target commander", "commander creatures", "except for commanders"), the command zone
//! as a zone ("from the command zone", "to the command zone"), an ability that functions
//! in the command zone, a flashback cost reduction "this way", "your commander" in a
//! mana restriction, and commander casts counted for another player.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::game::Variant;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use mtg_engine::types::counters;

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

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Makes `id` one of `p`'s commanders (CR 903.3).
fn make_commander(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[p.idx()].commander_names.push(name);
    t.g.recompute();
}

fn commander_in_zone(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    make_commander(t, p, id);
    id
}

fn commander_on_battlefield(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    make_commander(t, p, id);
    id
}

#[test]
fn slash_the_ranks_spares_commanders() {
    cr!("903.3");
    assert_supported(&["Slash the Ranks"]);
    let mut t = commander_game();
    let cmdr = commander_on_battlefield(&mut t, P1, "Esior, Wardwing Familiar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Slash the Ranks");
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.on_battlefield(cmdr));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn gray_harbor_merfolk_counts_a_commander_creature_you_control() {
    cr!("903.3");
    assert_supported(&["Gray Harbor Merfolk"]);
    let mut t = commander_game();
    let m = t.battlefield(P0, "Gray Harbor Merfolk");
    assert_eq!(t.pt(m), (0, 3));
    // An opponent's commander doesn't count.
    commander_on_battlefield(&mut t, P1, "Esior, Wardwing Familiar");
    assert_eq!(t.pt(m), (0, 3));
    commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(m), (2, 3));
}

#[test]
fn witchs_clinic_targets_only_commanders() {
    cr!("903.3", "115.1");
    assert_supported(&["Witch's Clinic"]);
    let mut t = commander_game();
    let clinic = t.battlefield(P0, "Witch's Clinic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cmdr = commander_on_battlefield(&mut t, P0, "Esior, Wardwing Familiar");
    t.lands(P0, "Plains", 2);
    t.activate(P0, clinic, 1, &[Entity::Object(cmdr)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(cmdr).chars.has_keyword(KeywordKind::Lifelink));
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Lifelink));
}

#[test]
fn sanctum_of_eternity_returns_a_commander_from_the_battlefield() {
    cr!("903.3", "903.9b");
    assert_supported(&["Sanctum of Eternity"]);
    let mut t = commander_game();
    let s = t.battlefield(P0, "Sanctum of Eternity");
    let cmdr = commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    // CR 903.9b: the owner may put it into the command zone instead; they don't.
    t.answer_yes(P0, false);
    t.activate(P0, s, 1, &[Entity::Object(cmdr)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn leadership_vacuum_returns_commanders_to_the_command_zone() {
    cr!("903.3", "408.1");
    assert_supported(&["Leadership Vacuum"]);
    let mut t = commander_game();
    let cmdr = commander_on_battlefield(&mut t, P1, "Esior, Wardwing Familiar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let lv = t.hand(P0, "Leadership Vacuum");
    t.cast(P0, lv).target(P1).go();
    t.resolve();
    assert_eq!(t.zone(t.g.current(cmdr)), Zone::Command);
    assert!(t.on_battlefield(bears));
}

#[test]
fn hellkite_courser_puts_a_commander_onto_the_battlefield_until_end_step() {
    cr!("903.3", "603.7");
    assert_supported(&["Hellkite Courser"]);
    let mut t = commander_game();
    let cmdr = commander_in_zone(&mut t, P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Hellkite Courser");
    t.resolve_all();
    let on_bf = t.g.current(cmdr);
    assert!(t.on_battlefield(on_bf));
    assert!(t.obj_now(on_bf).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(cmdr)), Zone::Command);
}

#[test]
fn campfire_puts_commanders_from_the_command_zone_and_graveyard_into_hand() {
    cr!("903.3", "903.9b");
    assert_supported(&["Campfire"]);
    let mut t = commander_game();
    commander_in_zone(&mut t, P0, "Esior, Wardwing Familiar");
    let g = t.graveyard(P0, "Grizzly Bears");
    make_commander(&mut t, P0, g);
    t.graveyard(P0, "Hill Giant");
    let c = t.battlefield(P0, "Campfire");
    t.lands(P0, "Mountain", 2);
    // CR 903.9b: the owner may put them into the command zone instead; they don't.
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.activate(P0, c, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Esior, Wardwing Familiar"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn derevi_enters_from_the_command_zone() {
    cr!("113.6", "903.3");
    assert_supported(&["Derevi, Empyrial Tactician"]);
    let mut t = commander_game();
    let d = commander_in_zone(&mut t, P0, "Derevi, Empyrial Tactician");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Derevi, Empyrial Tactician").len(), 1);
}

#[test]
fn commanders_insight_counts_the_target_players_commander_casts() {
    cr!("903.8");
    assert_supported(&["Commander's Insight"]);
    let mut t = commander_game();
    commander_in_zone(&mut t, P1, "Esior, Wardwing Familiar");
    t.g.players[1]
        .commander_casts
        .insert("Esior, Wardwing Familiar".into(), 2);
    t.lands(P0, "Island", 4);
    let ci = t.hand(P0, "Commander's Insight");
    let before = t.hand_size(P1);
    t.cast(P0, ci).x(1).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), before + 3);
}

#[test]
fn visions_of_dread_costs_less_only_with_flashback() {
    cr!("702.34a", "601.2f");
    assert_supported(&["Visions of Dread", "Visions of Glory"]);
    let mut t = commander_game();
    // A commander with mana value 4 in the command zone: flashback {8}{B}{B} costs
    // {4}{B}{B}.
    commander_in_zone(&mut t, P0, "Hill Giant");
    t.graveyard(P1, "Grizzly Bears");
    let v = t.graveyard(P0, "Visions of Dread");
    t.lands(P0, "Swamp", 5);
    let r = t
        .cast(P0, v)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .try_go();
    assert!(r.is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, v)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_exile("Visions of Dread"));
}

#[test]
fn jeweled_lotus_mana_only_casts_your_commander() {
    cr!("106.6", "903.3");
    assert_supported(&["Jeweled Lotus"]);
    let mut t = commander_game();
    let lotus = t.battlefield(P0, "Jeweled Lotus");
    let cmdr = commander_in_zone(&mut t, P0, "Hill Giant");
    let other = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, lotus, 0, &[]).unwrap();
    // The three mana can't pay for another spell.
    assert!(t.cast(P0, other).try_go().is_err());
    t.cast(P0, cmdr).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn esior_taxes_spells_targeting_your_commanders() {
    cr!("601.2f", "903.3");
    assert_supported(&["Esior, Wardwing Familiar"]);
    let mut t = commander_game();
    let esior = commander_on_battlefield(&mut t, P0, "Esior, Wardwing Familiar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    // Bolting a noncommander costs {R}; bolting the commander costs {3}{R}.
    assert!(t.cast(P1, bolt).target(esior).try_go().is_err());
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn myth_unbound_offsets_part_of_the_commander_tax() {
    cr!("903.8", "601.2f");
    assert_supported(&["Myth Unbound"]);
    let mut t = commander_game();
    t.battlefield(P0, "Myth Unbound");
    let giant = commander_in_zone(&mut t, P0, "Hill Giant");
    t.g.players[0]
        .commander_casts
        .insert("Hill Giant".into(), 2);
    // {3}{R} + {4} tax - {2} = six mana.
    t.lands(P0, "Mountain", 5);
    assert!(t.cast(P0, giant).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    t.cast(P0, giant).go();
}

#[test]
fn geode_golem_casts_your_commander_paying_only_the_tax() {
    cr!("903.8", "118.9d");
    assert_supported(&["Geode Golem"]);
    let mut t = commander_game();
    let golem = t.battlefield(P0, "Geode Golem");
    let giant = commander_in_zone(&mut t, P0, "Hill Giant");
    t.g.players[0]
        .commander_casts
        .insert("Hill Giant".into(), 1);
    // The {2} tax only.
    let lands = t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(golem, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(giant)));
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn banes_contingency_draws_when_the_spell_targets_your_commander() {
    cr!("608.2c", "701.6a");
    assert_supported(&["Bane's Contingency"]);
    let mut t = commander_game();
    let cmdr = commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(cmdr).go();
    t.lands(P0, "Island", 3);
    let bane = t.hand(P0, "Bane's Contingency");
    let before = t.hand_size(P0);
    t.cast(P0, bane).target(spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(cmdr));
    // Bane's Contingency left the hand; a card was drawn.
    assert_eq!(t.hand_size(P0), before);
    // Targeting a noncommander: only countered.
    let mut t = commander_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(bears).go();
    t.lands(P0, "Island", 3);
    let bane = t.hand(P0, "Bane's Contingency");
    let before = t.hand_size(P0);
    t.cast(P0, bane).target(spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), before - 1);
}

#[test]
fn tevesh_szat_draws_another_card_for_a_sacrificed_commander() {
    cr!("903.3", "608.2h");
    assert_supported(&["Tevesh Szat, Doom of Fools"]);
    let mut t = commander_game();
    let tevesh = t.battlefield(P0, "Tevesh Szat, Doom of Fools");
    let cmdr = commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    let before = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(cmdr)]);
    // CR 903.9a: it stays in the graveyard.
    t.answer_yes(P0, false);
    t.activate(P0, tevesh, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(cmdr));
    assert_eq!(t.hand_size(P0), before + 3);
    // A noncommander: two cards.
    let mut t = commander_game();
    let tevesh = t.battlefield(P0, "Tevesh Szat, Doom of Fools");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, tevesh, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 2);
}

#[test]
fn drillworks_mole_counters_up_to_one_commander_creature() {
    cr!("903.3");
    assert_supported(&["Drillworks Mole"]);
    let mut t = commander_game();
    let mole = t.battlefield(P0, "Drillworks Mole");
    let cmdr = commander_on_battlefield(&mut t, P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    // A noncommander creature isn't a legal target: no target is chosen ("up to one").
    t.activate(P0, mole, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(mole, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    t.g.untap(mole);
    t.activate(P0, mole, 0, &[Entity::Object(cmdr)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(mole, counters::PLUS1), 2);
    assert_eq!(t.counters(cmdr, counters::PLUS1), 1);
}

#[test]
fn forge_of_heroes_counters_a_commander_that_entered_this_turn() {
    cr!("903.3");
    assert_supported(&["Forge of Heroes"]);
    let mut t = commander_game();
    let forge = t.battlefield(P0, "Forge of Heroes");
    let cmdr = t.enter(P0, "Hill Giant");
    make_commander(&mut t, P0, cmdr);
    t.activate(P0, forge, 1, &[Entity::Object(cmdr)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(cmdr, counters::PLUS1), 1);
    assert_eq!(t.counters(cmdr, "loyalty"), 0);
}

#[test]
fn dungeon_delver_gives_commander_creatures_hama_pashars_ability() {
    cr!("903.3", "603.2d");
    assert_supported(&["Dungeon Delver"]);
    use mtg_engine::ability::{Effect, KeywordAction, PlayerRef, Sel, Value};
    fn venture(t: &mut TestGame) {
        let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
        t.g.exec(
            &Effect::KeywordAction {
                action: KeywordAction::Venture,
                who: PlayerRef::You,
                what: Sel::None,
                n: Value::c(1),
            },
            &mut ctx,
        );
        t.g.flush_events();
        t.resolve_all();
    }
    fn treasures(t: &TestGame) -> usize {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Treasure"))
            .count()
    }
    for commander in [true, false] {
        let mut t = commander_game();
        // A Background: it grants the ability to a commander creature you own.
        t.battlefield(P0, "Dungeon Delver");
        let giant = t.battlefield(P0, "Hill Giant");
        if commander {
            make_commander(&mut t, P0, giant);
        }
        t.custom(
            P0,
            (*card("Lost Mine of Phandelver")).clone(),
            Zone::Outside(P0),
        );
        // Cave Entrance, then Mine Tunnels ("Create a Treasure token.").
        venture(&mut t);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        venture(&mut t);
        assert_eq!(treasures(&t), if commander { 2 } else { 1 });
    }
}
