//! Rulings batch P160 — "shade pump" abilities: +X/+X effects whose value and affected set
//! are determined as they resolve (CR 608.2h, 611.2c), sacrificing a creature to its own
//! ability, activation restrictions, and end-step triggers that count the turn's events.

use crate::r_p160_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn osai_vultures_gets_one_counter_however_many_creatures_died() {
    cr!("603.4", "700.4");
    ruling!(
        "Osai Vultures",
        "Only gets one counter per turn, not one per creature."
    );
    supported("Osai Vultures");
    let mut t = TestGame::new(2);
    let vultures = t.battlefield(P0, "Osai Vultures");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(a, None);
    t.g.destroy(b, None);
    t.settle();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(vultures, "carrion"), 1);
}

#[test]
fn resplendent_angel_counts_life_gained_this_turn() {
    cr!("603.4");
    ruling!(
        "Resplendent Angel",
        "Resplendent Angel's triggered ability checks if you gained 5 or more life total during the turn. It doesn't matter if you also lost life or whether your life total is greater than it was at the beginning of the turn. It also doesn't matter whether Resplendent Angel was on the battlefield when any of the life gain happened."
    );
    ruling!(
        "Resplendent Angel",
        "You don't need to have gained 5 life all at once to satisfy Resplendent Angel's triggered ability."
    );
    ruling!(
        "Resplendent Angel",
        "You create only one Angel token, no matter how many times you gained 5 or more life."
    );
    supported("Resplendent Angel");
    let mut t = TestGame::new(2);
    // Before the Angel is on the battlefield: gain 3, then 2, then 5 (10 in all), and lose
    // 15 — the life total ends lower than it began.
    t.g.gain_life(P0, 3);
    t.g.gain_life(P0, 2);
    t.g.gain_life(P0, 5);
    t.g.lose_life(P0, 15);
    t.settle();
    assert_eq!(t.life(P0), 15);
    t.battlefield(P0, "Resplendent Angel");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let angels: Vec<_> = tokens_of(&t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.name.contains("Angel"))
        .collect();
    assert_eq!(angels.len(), 1);
    assert_eq!(t.pt(angels[0]), (4, 4));
    // A turn in which only 4 life was gained creates no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Resplendent Angel");
    t.g.gain_life(P0, 2);
    t.g.gain_life(P0, 2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn elvish_warmaster_pumps_only_the_elves_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Elvish Warmaster",
        "The activated ability affects only Elves you control as the ability resolves. Elves you begin to control later in the turn won't get the bonuses."
    );
    supported("Elvish Warmaster");
    let mut t = TestGame::new(2);
    let warmaster = t.battlefield(P0, "Elvish Warmaster");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 7);
    activate_resolve(&mut t, P0, warmaster, 0, &[]);
    assert_eq!(t.pt(warmaster), (4, 4));
    assert_eq!(t.pt(elf), (3, 3));
    let late = t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.pt(late), (1, 1));
    assert!(!t.obj(late).has_keyword(mtg_engine::keywords::KeywordKind::Deathtouch));
}

#[test]
fn the_bonus_counts_as_the_ability_resolves_and_then_stays_fixed() {
    cr!("608.2h");
    ruling!(
        "Gran Pulse Ochu",
        "The bonus from Gran Pulse Ochu's last ability is determined at the time it resolves. Once the ability resolves, the bonus applied by that instance of the ability doesn't change even if the number of permanent cards in your graveyard changes."
    );
    ruling!(
        "Elder of Laurels",
        "The number of creatures you control is counted as the ability resolves."
    );
    ruling!(
        "Creeping Trailblazer",
        "The size of the bonus is determined as Creeping Trailblazer's last ability begins to resolve; it won't change later in the turn if the number of Elementals you control changes."
    );
    ruling!(
        "Conifer Wurm",
        "The value of X is determined only as Conifer Wurm's ability begins to resolve. It won't change later in the turn if the number of snow permanents you control changes."
    );
    for name in [
        "Gran Pulse Ochu",
        "Elder of Laurels",
        "Creeping Trailblazer",
        "Conifer Wurm",
    ] {
        supported(name);
    }

    // Gran Pulse Ochu: "{8}: Until end of turn, this creature gets +1/+1 for each permanent
    // card in your graveyard." Two permanent cards (and an instant) in the graveyard.
    let mut t = TestGame::new(2);
    let ochu = t.battlefield(P0, "Gran Pulse Ochu");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Wastes", 8);
    t.activate(P0, ochu, 0, &[]).unwrap();
    // A permanent card put into the graveyard before it resolves counts.
    t.graveyard(P0, "Plains");
    t.resolve_all();
    assert_eq!(t.pt(ochu), (4, 4));
    t.graveyard(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(ochu), (4, 4));

    // Elder of Laurels: "{3}{G}: Target creature gets +X/+X until end of turn, where X is
    // the number of creatures you control."
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Elder of Laurels");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    t.activate(P0, elder, 0, &[Entity::Object(bears)]).unwrap();
    t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));

    // Creeping Trailblazer: "{2}{R}{G}: This creature gets +1/+1 until end of turn for each
    // Elemental you control." (Other Elementals get +1/+0 from its first ability.)
    let mut t = TestGame::new(2);
    let trail = t.battlefield(P0, "Creeping Trailblazer");
    t.battlefield(P0, "Air Elemental");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    activate_resolve(&mut t, P0, trail, 0, &[]);
    assert_eq!(t.pt(trail), (4, 4));
    t.battlefield(P0, "Air Elemental");
    t.g.recompute();
    assert_eq!(t.pt(trail), (4, 4));

    // Conifer Wurm: "{3}{G}: ... +X/+X ..., where X is the number of snow permanents you
    // control." The Wurm and four snow lands are five.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Conifer Wurm");
    t.lands(P0, "Snow-Covered Forest", 4);
    activate_resolve(&mut t, P0, wurm, 0, &[]);
    assert_eq!(t.pt(wurm), (9, 9));
    t.battlefield(P0, "Snow-Covered Forest");
    t.g.recompute();
    assert_eq!(t.pt(wurm), (9, 9));
}

#[test]
fn stromkirk_condemned_pumps_only_the_vampires_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Stromkirk Condemned",
        "The set of creatures affected by Stromkirk Condemned's ability is determined as the ability resolves. Vampires you begin to control later in the turn won't get +1/+1."
    );
    supported("Stromkirk Condemned");
    let mut t = TestGame::new(2);
    let condemned = t.battlefield(P0, "Stromkirk Condemned");
    let nighthawk = t.battlefield(P0, "Vampire Nighthawk");
    let card = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(card)]);
    activate_resolve(&mut t, P0, condemned, 0, &[]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.pt(condemned), (3, 3));
    assert_eq!(t.pt(nighthawk), (3, 4));
    let late = t.battlefield(P0, "Vampire Nighthawk");
    assert_eq!(t.pt(late), (2, 3));
}

#[test]
fn x_where_x_is_its_power_is_determined_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Heroes' Bane",
        "The value of X is calculated only once, as Heroes' Bane's last ability resolves."
    );
    ruling!(
        "Electrostatic Pummeler",
        "The value of X is determined as Electrostatic Pummeler's last ability resolves."
    );
    ruling!(
        "Yew Spirit",
        "The value of X is determined when the ability resolves. The bonus won't change later in the turn if the creature's power changes."
    );
    for name in ["Heroes' Bane", "Electrostatic Pummeler", "Yew Spirit", "Giant Growth"] {
        supported(name);
    }

    // Heroes' Bane: "{2}{G}{G}: Put X +1/+1 counters on this creature, where X is its
    // power." Two activations on the stack: the first to resolve doubles 4 to 8, the second
    // sees power 8 and doubles it again.
    let mut t = TestGame::new(2);
    let bane = t.enter(P0, "Heroes' Bane");
    t.settle();
    assert_eq!(t.pt(bane), (4, 4));
    t.lands(P0, "Forest", 8);
    t.activate(P0, bane, 0, &[]).unwrap();
    t.activate(P0, bane, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bane, counters::PLUS1), 16);

    // Electrostatic Pummeler: "Pay {E}{E}{E}: This creature gets +X/+X until end of turn,
    // where X is its power." Giant Growth in response: X is 4.
    let mut t = TestGame::new(2);
    let pummeler = t.enter(P0, "Electrostatic Pummeler");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 3);
    t.activate(P0, pummeler, 0, &[]).unwrap();
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(pummeler)]);
    t.resolve_all();
    assert_eq!(t.pt(pummeler), (8, 8));

    // Yew Spirit: "{2}{G}{G}: This creature gets +X/+X until end of turn, where X is its
    // power." Giant Growth afterwards doesn't change the bonus.
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Yew Spirit");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    activate_resolve(&mut t, P0, spirit, 0, &[]);
    assert_eq!(t.pt(spirit), (6, 6));
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(spirit)]);
    assert_eq!(t.pt(spirit), (9, 9));
}

#[test]
fn thorn_lieutenant_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Thorn Lieutenant",
        "Thorn Lieutenant's triggered ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered."
    );
    supported("Thorn Lieutenant");
    let mut t = TestGame::new(2);
    let thorn = t.battlefield(P0, "Thorn Lieutenant");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(thorn)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger is on top: resolving it creates the token while the Bolt waits.
    t.resolve();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(t.stack_len(), 1);
    // Again, with the Bolt countered in response to the trigger.
    let mut t = TestGame::new(2);
    let thorn = t.battlefield(P0, "Thorn Lieutenant");
    t.set_step(P1, Step::PrecombatMain);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(thorn)]);
    t.settle();
    cast_new(&mut t, P0, "Counterspell", &[Entity::Object(bolt)]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert!(t.on_battlefield(thorn));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn korozda_guildmage_uses_the_sacrificed_creatures_last_known_toughness() {
    cr!("608.2h", "701.21a");
    ruling!(
        "Korozda Guildmage",
        "Use the sacrificed creature’s toughness when it was last on the battlefield to determine the value of X."
    );
    supported("Korozda Guildmage");
    let mut t = TestGame::new(2);
    let guildmage = t.battlefield(P0, "Korozda Guildmage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_resolve(&mut t, P0, guildmage, 1, &[]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tokens_of(&t, P0).len(), 5);
}

#[test]
fn chronatog_can_be_activated_only_once_each_turn() {
    cr!("602.5b");
    ruling!(
        "Chronatog",
        "You can only activate the ability once each turn for each Chronatog."
    );
    supported("Chronatog");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Chronatog");
    let b = t.battlefield(P0, "Chronatog");
    t.activate(P0, a, 0, &[]).unwrap();
    assert!(t.activate(P0, a, 0, &[]).is_err());
    // Another Chronatog's ability can still be activated.
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 5));
    assert_eq!(t.pt(b), (4, 5));
}

#[test]
fn sacrificing_the_creature_to_its_own_ability_gives_no_bonus() {
    cr!("602.2", "701.21a", "608.2b");
    ruling!(
        "Bloodthrone Vampire",
        "You can sacrifice Bloodthrone Vampire to activate its own ability, but it won't be on the battlefield to get the bonus."
    );
    ruling!(
        "Nantuko Husk",
        "You can sacrifice Nantuko Husk itself to activate its own ability. However, the only thing that will do is put Nantuko Husk into the graveyard."
    );
    ruling!(
        "Vampire Aristocrat",
        "You can sacrifice Vampire Aristocrat to activate its own ability, but it won't be around to get the bonus."
    );
    for name in ["Bloodthrone Vampire", "Nantuko Husk", "Vampire Aristocrat"] {
        supported(name);
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        let other = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[Entity::Object(id)]);
        activate_resolve(&mut t, P0, id, 0, &[]);
        assert!(t.in_graveyard(P0, name), "{name}");
        assert_eq!(t.pt(other), (2, 2), "{name}");
        assert!(t.stack.is_empty());
    }
}

#[test]
fn gateway_shade_cant_tap_a_gate_for_mana_and_for_its_ability() {
    cr!("118.3");
    ruling!(
        "Gateway Shade",
        "You can't tap an untapped Gate you control for mana and tap it to activate Gateway Shade's ability at the same time. You must choose one or the other."
    );
    supported("Gateway Shade");
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Gateway Shade");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    // Tapped to pay for Shade's ability, the Gate can't also pay for anything else.
    t.answer_choose(P0, &[Entity::Object(gate)]);
    activate_resolve(&mut t, P0, shade, 1, &[]);
    assert!(t.obj(gate).tapped);
    assert_eq!(t.pt(shade), (3, 3));
    assert!(t.activate(P0, shade, 1, &[]).is_err());
    // Tapped for mana first, the Gate can't pay the Shade's tap cost.
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Gateway Shade");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    t.g.tap_ex(gate, true);
    mana(&mut t, P0, ManaType::W, 1);
    assert!(t.activate(P0, shade, 1, &[]).is_err());
    assert_eq!(t.pt(shade), (1, 1));
}

#[test]
fn avizoa_skips_save_up() {
    cr!("614.10", "502.3");
    ruling!(
        "Avizoa",
        "You skip the next untap step that you are not skipping for any other reason. In other words, the skips save up until you skip as many as required."
    );
    supported("Avizoa");
    let mut t = TestGame::new(2);
    let avizoa = t.battlefield(P0, "Avizoa");
    let land = t.battlefield(P0, "Forest");
    // Activated on P0's turn and again on P1's turn: two skipped untap steps.
    t.activate(P0, avizoa, 0, &[]).unwrap();
    t.resolve_all();
    t.g.tap(land);
    t.advance_to(P1, Step::PrecombatMain);
    t.activate(P0, avizoa, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj(land).tapped, "first untap step skipped");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj(land).tapped, "second untap step skipped");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj(land).tapped, "third untap step happens");
}

#[test]
fn fallen_ideal_grants_the_ability_to_the_creatures_controller() {
    cr!("602.2", "113.10");
    ruling!(
        "Fallen Ideal",
        "The creature's controller (not Fallen Ideal's controller) can activate the \"sacrifice a creature\" ability."
    );
    supported("Fallen Ideal");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fodder = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Fallen Ideal", &[Entity::Object(bears)]);
    assert!(t.activate(P0, bears, 0, &[]).is_err());
    t.answer_choose(P1, &[Entity::Object(fodder)]);
    activate_resolve(&mut t, P1, bears, 0, &[]);
    assert_eq!(t.zone(fodder), Zone::Graveyard(P1));
    assert_eq!(t.pt(bears), (4, 3));
}

#[test]
fn knight_of_dawns_light_adds_one_to_a_for_each_life_gain_event() {
    cr!("614.1a", "119.10");
    ruling!(
        "Knight of Dawn's Light",
        "The middle ability of Knight of Dawn's Light applies just once to each life-gaining event, no matter how much life is gained. If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Knight of Dawn's Light applies only once."
    );
    supported("Knight of Dawn's Light");
    supported("Peach Garden Oath");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight of Dawn's Light");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    // "You gain 2 life for each creature you control": 6, plus 1.
    cast_resolve(&mut t, P0, "Peach Garden Oath", &[]);
    assert_eq!(t.life(P0), 27);
}

#[test]
fn aetherwind_basker_triggers_on_entering_and_on_attacking() {
    cr!("603.2");
    ruling!(
        "Aetherwind Basker",
        "The triggered ability triggers both when Aetherwind Basker enters the battlefield and whenever it attacks. You don't have to choose only one."
    );
    supported("Aetherwind Basker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let basker = t.enter(P0, "Aetherwind Basker");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 2);
    let basker = t.g.current(basker);
    t.g.objects[basker.0 as usize].summoning_sick = false;
    t.attack(&[(basker, Entity::Player(P1))], &[]);
    assert_eq!(t.g.player(P0).counter("energy"), 4);
}
