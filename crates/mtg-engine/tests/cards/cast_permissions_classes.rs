//! The permission grammar for cards with qualities in a zone: resolved effects' permissions
//! for a while ("You may cast Zombie spells from your graveyard this turn", "a creature
//! spell" — one), static permissions ("You may play lands and cast Insect spells from your
//! graveyard", "You may cast spells from among cards exiled with ~", "once during each of
//! your turns ... by exiling three other cards from your graveyard in addition to paying
//! its other costs"), and timing permissions ("as though they had flash").

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let c = t.g.current(card);
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, CastMethod::Normal).is_ok();
    if ok {
        t.resolve_all();
    }
    ok
}

#[test]
fn liliana_casts_zombie_spells_from_your_graveyard_this_turn() {
    cr!("601.3", "611.2a");
    assert_supported("Liliana, Untouched by Death");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let liliana = t.battlefield(P0, "Liliana, Untouched by Death");
    let ghoul = t.graveyard(P0, "Diregraf Ghoul");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let ghoul2 = t.graveyard(P0, "Diregraf Ghoul");
    // Not before the ability resolves.
    assert!(!can_cast(&mut t, P0, ghoul));
    t.activate(P0, liliana, 2, &[]).unwrap();
    t.resolve_all();
    // A Zombie, as many as you like; not another creature.
    assert!(can_cast(&mut t, P0, ghoul));
    assert!(can_cast(&mut t, P0, ghoul2));
    assert!(!can_cast(&mut t, P0, bears));
    assert_eq!(t.named_on_battlefield("Diregraf Ghoul").len(), 2);
}

#[test]
fn liliana_permission_ends_with_the_turn() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let liliana = t.battlefield(P0, "Liliana, Untouched by Death");
    let ghoul = t.graveyard(P0, "Diregraf Ghoul");
    t.activate(P0, liliana, 2, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, ghoul));
}

#[test]
fn chainer_casts_one_creature_spell_from_your_graveyard() {
    cr!("601.3");
    ruling!(
        "Chainer, Nightmare Adept",
        "The ability creates a permission for you to cast a creature card from your graveyard later in the turn."
    );
    assert_supported("Chainer, Nightmare Adept");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let chainer = t.battlefield(P0, "Chainer, Nightmare Adept");
    t.hand(P0, "Island");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let bears2 = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, chainer, 0, &[]).unwrap();
    t.resolve_all();
    assert!(can_cast(&mut t, P0, bears));
    // One creature spell for the permission.
    assert!(!can_cast(&mut t, P0, bears2));
    assert_eq!(t.zone(t.g.current(bears2)), Zone::Graveyard(P0));
}

#[test]
fn zask_plays_lands_and_casts_insect_spells_from_your_graveyard() {
    cr!("305.1", "601.3");
    assert_supported("Zask, Skittering Swarmlord");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.battlefield(P0, "Zask, Skittering Swarmlord");
    let insect = t.graveyard(P0, "Giant Mantis");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let forest = t.graveyard(P0, "Forest");
    assert!(!can_cast(&mut t, P0, bears));
    t.play_land(P0, forest)
        .expect("play a land from the graveyard");
    assert!(can_cast(&mut t, P0, insect));
    assert_eq!(t.named_on_battlefield("Giant Mantis").len(), 1);
}

#[test]
fn their_number_is_legion_can_be_cast_from_your_graveyard() {
    cr!("601.3");
    assert_supported("Their Number Is Legion");
    ruling!(
        "Their Number Is Legion",
        "You must still pay its normal cost and follow all normal timing rules"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let legion = t.graveyard(P0, "Their Number Is Legion");
    t.cast(P0, legion).x(2).go();
    t.resolve_all();
    let necrons =
        t.g.battlefield
            .iter()
            .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Necron"))
            .count();
    assert_eq!(necrons, 2, "{}", t.dump_log());
    // It exiles itself as it resolves.
    assert_eq!(t.zone(t.g.current(legion)), Zone::Exile);
}

#[test]
fn eternal_scourge_can_be_cast_from_exile() {
    cr!("601.3");
    assert_supported("Eternal Scourge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let scourge = t.exile(P0, "Eternal Scourge");
    assert!(can_cast(&mut t, P0, scourge));
    assert_eq!(t.named_on_battlefield("Eternal Scourge").len(), 1);
}

#[test]
fn kotis_casts_a_creature_spell_from_your_graveyard_by_exiling_three_other_cards() {
    cr!("601.2b", "601.2h", "601.3");
    // (Its other ability is another item's.)
    assert!(card("Kotis, Sibsig Champion")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("once during each of your turns")));
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.battlefield(P0, "Kotis, Sibsig Champion");
    let bears = t.graveyard(P0, "Grizzly Bears");
    // Two other cards: not enough to pay the additional cost.
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Island");
    assert!(!can_cast(&mut t, P0, bears));
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Island");
    assert!(can_cast(&mut t, P0, bears));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Three other cards were exiled to pay for it.
    assert_eq!(t.graveyard_size(P0), 1);
    // Once each turn.
    let bears2 = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Island");
    assert!(!can_cast(&mut t, P0, bears2));
}

#[test]
fn rona_casts_spells_from_among_cards_exiled_with_it() {
    cr!("607.2a");
    assert_supported("Rona, Disciple of Gix");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Island", 1);
    let relic = t.graveyard(P0, "Bonesplitter");
    let rona = t.hand(P0, "Rona, Disciple of Gix");
    t.answer_targets(P0, &[Entity::Object(relic)]);
    t.cast(P0, rona).go();
    t.resolve_all();
    let relic_x = t.g.current(relic);
    assert_eq!(t.zone(relic_x), Zone::Exile);
    // A card exiled some other way isn't one exiled with Rona.
    let other = t.exile(P0, "Bonesplitter");
    assert!(!can_cast(&mut t, P0, other));
    assert!(can_cast(&mut t, P0, relic));
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
}

#[test]
fn serpents_soul_jar_casts_one_creature_spell_from_among_the_exiled_cards() {
    cr!("607.2a", "611.2a");
    assert_supported("Serpent's Soul-Jar");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.battlefield(P0, "Serpent's Soul-Jar");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let elf2 = t.battlefield(P0, "Llanowar Elves");
    t.g.destroy(elf, None);
    t.g.destroy(elf2, None);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(elf)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(elf2)), Zone::Exile);
    // Not before the ability resolves.
    assert!(!can_cast(&mut t, P0, elf));
    let jar = t.named_on_battlefield("Serpent's Soul-Jar")[0];
    t.activate(P0, jar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(can_cast(&mut t, P0, elf));
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    // A creature spell: one.
    assert!(!can_cast(&mut t, P0, elf2));
}

#[test]
fn valley_floodcaller_casts_noncreature_spells_as_though_they_had_flash() {
    cr!("601.3b");
    assert_supported("Valley Floodcaller");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.battlefield(P0, "Valley Floodcaller");
    let sorcery = t.hand(P0, "Lava Spike");
    let bears = t.hand(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    // A creature spell isn't a noncreature spell.
    assert!(t.g.cast_spell(P0, bears, CastMethod::Normal).is_err());
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.cast_spell(P0, sorcery, CastMethod::Normal)
        .expect("a sorcery during the opponent's upkeep");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn bards_company_has_flash_timing_only_while_you_control_a_human() {
    cr!("601.3b");
    assert_supported("Bard's Company");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Island", 1);
    let company = t.hand(P0, "Bard's Company");
    t.advance_to(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, company, CastMethod::Normal).is_err());
    t.battlefield(P0, "Benalish Knight");
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, company, CastMethod::Normal)
        .expect("cast as though it had flash");
}

#[test]
fn teferi_time_raveler_casts_sorceries_as_though_they_had_flash_until_your_next_turn() {
    cr!("601.3b", "611.2a");
    assert_supported("Teferi, Time Raveler");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let teferi = t.battlefield(P0, "Teferi, Time Raveler");
    let spike = t.hand(P0, "Lava Spike");
    let spike2 = t.hand(P0, "Lava Spike");
    t.activate(P0, teferi, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, spike, CastMethod::Normal)
        .expect("a sorcery in the opponent's turn");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // It ends as your next turn begins.
    t.advance_to(P0, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, spike2, CastMethod::Normal).is_err());
}

#[test]
fn daxos_casts_the_exiled_card_spending_mana_as_though_any_color() {
    cr!("609.4b", "611.2a");
    assert_supported("Daxos of Meletis");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let daxos = t.battlefield(P0, "Daxos of Meletis");
    let opt = t.library_top(P1, "Divination");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(daxos, Entity::Player(P1))], &[]);
    t.resolve_all();
    let opt_x = t.g.current(opt);
    assert_eq!(t.zone(opt_x), Zone::Exile);
    // Divination's mana value: 3 life gained.
    assert_eq!(t.life(P0), 23);
    // Not enough mana: Divination costs {2}{U}, only two Mountains.
    assert!(!can_cast(&mut t, P0, opt));
    t.lands(P0, "Mountain", 1);
    t.advance_to(P0, Step::PostcombatMain);
    let hand = t.hand_size(P0);
    assert!(can_cast(&mut t, P0, opt));
    assert_eq!(t.hand_size(P0), hand + 2);
}
