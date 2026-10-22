//! More rulings on objects that have the activated abilities of other objects
//! (gap-ability-grants, CR 113.10, 613.1f, 201.5b, 607.5, 400.7): Patchwork Crawler, Dark
//! Impostor, Thranduil, Sharkey, Mirran Safehouse, Robaran Mercenaries, Rex, Territory
//! Forge, Hazel's Brewmaster, Trazyn and Conspicuous Snoop.

use crate::r_s01_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn activated(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.clone())
        .collect()
}

fn triggered(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .count()
}

fn has(t: &mut TestGame, id: ObjectId, text: &str) -> bool {
    activated(t, id).iter().any(|a| a.contains(text))
}

fn index_of(t: &mut TestGame, id: ObjectId, text: &str) -> usize {
    let all = activated(t, id);
    all.iter()
        .position(|a| a.contains(text))
        .unwrap_or_else(|| panic!("no activated ability {text:?} among {all:?}"))
}

/// Regenerates `id` with its "Regenerate" ability and checks that the shield works.
fn regenerates(t: &mut TestGame, id: ObjectId) -> bool {
    let i = index_of(t, id, "Regenerate");
    t.activate(P0, id, i, &[]).unwrap();
    t.resolve_all();
    t.g.destroy(id, None);
    t.settle();
    t.on_battlefield(id)
}

// ---------------------------------------------------------------------------
// Patchwork Crawler and Dark Impostor: cards exiled with that specific object
// ---------------------------------------------------------------------------

/// Patchwork Crawler exiles `card` from P0's graveyard with its own ability.
fn crawler_exiles(t: &mut TestGame, crawler: ObjectId, card: ObjectId) {
    t.lands(P0, "Island", 3);
    let i = index_of(t, crawler, "Exile target creature card");
    t.activate(P0, crawler, i, &[Entity::Object(card)]).unwrap();
    t.resolve_all();
}

#[test]
fn each_patchwork_crawler_has_the_abilities_of_its_own_exiled_cards() {
    ruling!(
        "Patchwork Crawler",
        "each will have only the activated abilities of creature cards exiled with that specific Patchwork Crawler"
    );
    ruling!(
        "Patchwork Crawler",
        "treat Patchwork Crawler's version of that ability as though it referenced Patchwork Crawler by name instead"
    );
    ruling!(
        "Patchwork Crawler",
        "Patchwork Crawler gains only activated abilities. It doesn't gain triggered abilities or static abilities."
    );
    supported("Patchwork Crawler");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Patchwork Crawler");
    let b = t.battlefield(P0, "Patchwork Crawler");
    let troll = t.graveyard(P0, "Cudgel Troll");
    let kavu = t.graveyard(P0, "Flametongue Kavu");
    crawler_exiles(&mut t, a, troll);
    crawler_exiles(&mut t, b, kavu);
    assert!(has(&mut t, a, "Regenerate"));
    assert!(!has(&mut t, b, "Regenerate"));
    // Flametongue Kavu's enters ability is a triggered ability: not gained.
    assert_eq!(triggered(&mut t, b), 0);
    t.lands(P0, "Forest", 1);
    assert!(regenerates(&mut t, a), "the shield is the Crawler's");
}

#[test]
fn a_patchwork_crawler_that_returns_is_a_new_object() {
    ruling!(
        "Patchwork Crawler",
        "If it returns to the battlefield, it will be a new Patchwork Crawler with no connection"
    );
    let mut t = TestGame::new(2);
    let crawler = t.battlefield(P0, "Patchwork Crawler");
    let troll = t.graveyard(P0, "Cudgel Troll");
    crawler_exiles(&mut t, crawler, troll);
    assert!(has(&mut t, crawler, "Regenerate"));
    let card = t
        .g
        .move_object(crawler, Zone::Hand(P0), MoveCause::Effect, None)
        .unwrap();
    let back = t
        .g
        .move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    assert!(!has(&mut t, back, "Regenerate"));
    assert!(t.in_exile("Cudgel Troll"));
}

#[test]
fn an_exiled_double_faced_card_has_its_front_faces_abilities() {
    ruling!(
        "Patchwork Crawler",
        "If Patchwork Crawler exiles a double-faced card, it will have the activated abilities of only the front face of that exiled card."
    );
    ruling!(
        "Dark Impostor",
        "it will only have activated abilities of the front face of that exiled card, no matter which face was up on the battlefield"
    );
    supported("Dark Impostor");
    // Ulvenwald Mystics // Ulvenwald Primordials: only the back face has "{G}: Regenerate
    // this creature."
    let name = "Ulvenwald Mystics // Ulvenwald Primordials";
    let mut t = TestGame::new(2);
    let crawler = t.battlefield(P0, "Patchwork Crawler");
    let mystics = t.graveyard(P0, name);
    crawler_exiles(&mut t, crawler, mystics);
    assert!(!has(&mut t, crawler, "Regenerate"));
    // Dark Impostor exiles it from the battlefield with its back face up.
    let impostor = t.battlefield(P0, "Dark Impostor");
    let primordials = t.battlefield(P1, name);
    assert!(mtg_engine::dfc::transform(&mut t.g, primordials));
    assert!(has(&mut t, primordials, "Regenerate"));
    t.lands(P0, "Swamp", 6);
    let i = index_of(&mut t, impostor, "Exile target creature");
    t.activate(P0, impostor, i, &[Entity::Object(primordials)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(primordials)), Zone::Exile);
    assert!(!has(&mut t, impostor, "Regenerate"));
}

#[test]
fn dark_impostor_gains_nothing_from_an_animated_land() {
    ruling!(
        "Dark Impostor",
        "because that card isn't a creature card, Dark Impostor won't have any of that card's activated abilities"
    );
    ruling!(
        "Dark Impostor",
        "treat Dark Impostor's version of that ability as though it referenced Dark Impostor by name instead"
    );
    ruling!(
        "Dark Impostor",
        "Dark Impostor gains only activated abilities. It doesn't gain triggered abilities or static abilities."
    );
    let mut t = TestGame::new(2);
    let impostor = t.battlefield(P0, "Dark Impostor");
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    let i = index_of(&mut t, vault, "becomes a 2/2");
    t.activate(P1, vault, i, &[]).unwrap();
    t.resolve_all();
    t.lands(P0, "Swamp", 18);
    let i = index_of(&mut t, impostor, "Exile target creature");
    t.activate(P0, impostor, i, &[Entity::Object(vault)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Mutavault"));
    assert!(!has(&mut t, impostor, "becomes a 2/2"));
    // A creature: its activated ability (naming itself) is the Impostor's; its triggered
    // ability isn't.
    let troll = t.battlefield(P1, "Cudgel Troll");
    let kavu = t.battlefield(P1, "Flametongue Kavu");
    for c in [troll, kavu] {
        t.g.untap(impostor);
        let i = index_of(&mut t, impostor, "Exile target creature");
        t.activate(P0, impostor, i, &[Entity::Object(c)]).unwrap();
        t.resolve_all();
    }
    assert_eq!(triggered(&mut t, impostor), 0);
    t.lands(P0, "Forest", 1);
    assert!(regenerates(&mut t, impostor));
}

// ---------------------------------------------------------------------------
// Linked abilities gained together (CR 607.5)
// ---------------------------------------------------------------------------

/// A card whose two activated abilities are linked (CR 607.2a).
fn linked_pair(name: &str, type_line: &str) -> mtg_engine::card::CardDef {
    custom_card(
        name,
        type_line,
        "{1}",
        type_line.contains("Creature").then_some((1, 1)),
        &format!(
            "{{T}}: Exile target card from a graveyard.\n{{T}}: You gain 1 life for each card exiled with {name}."
        ),
    )
}

#[test]
fn thranduils_linked_abilities_last_while_the_elf_card_stays_in_the_graveyard() {
    ruling!(
        "Thranduil, the Elvenking",
        "If Thranduil, the Elvenking gains a set of linked activated abilities"
    );
    ruling!(
        "Thranduil, the Elvenking",
        "treat Thranduil, the Elvenking's instance of that ability as though it referenced Thranduil by name instead"
    );
    supported("Thranduil, the Elvenking");
    let mut t = TestGame::new(2);
    let thranduil = t.battlefield(P0, "Thranduil, the Elvenking");
    let elf = t.custom(P0, linked_pair("Archivist Elf", "Creature — Elf"), Zone::Graveyard(P0));
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let i = index_of(&mut t, thranduil, "Exile target card");
    t.activate(P0, thranduil, i, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    t.g.untap(thranduil);
    let i = index_of(&mut t, thranduil, "for each card exiled");
    t.activate(P0, thranduil, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // The Elf card leaves the graveyard and returns: a new object, a new link.
    let gone = t
        .g
        .move_object(elf, Zone::Exile, MoveCause::Effect, None)
        .unwrap();
    t.g.move_object(gone, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.g.untap(thranduil);
    let i = index_of(&mut t, thranduil, "for each card exiled");
    t.activate(P0, thranduil, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn robaran_mercenaries_keeps_links_and_pays_costs_normally() {
    ruling!(
        "Robaran Mercenaries",
        "those two abilities Robaran Mercenaries gains are linked"
    );
    ruling!(
        "Robaran Mercenaries",
        "the ability Robaran Mercenaries has isn't linked to any ability"
    );
    ruling!(
        "Robaran Mercenaries",
        "The costs of activated abilities that Robaran Mercenaries gains must be paid as normal."
    );
    supported("Robaran Mercenaries");
    let mut t = TestGame::new(2);
    let mercs = t.battlefield(P0, "Robaran Mercenaries");
    let legend = t.custom(
        P0,
        linked_pair("Archivist Prime", "Legendary Creature — Human"),
        Zone::Battlefield,
    );
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let i = index_of(&mut t, mercs, "Exile target card");
    t.activate(P0, mercs, i, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    t.g.untap(mercs);
    let i = index_of(&mut t, mercs, "for each card exiled");
    t.activate(P0, mercs, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "the card the Mercenaries exiled");
    let i = index_of(&mut t, legend, "for each card exiled");
    t.activate(P0, legend, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "the legend's own ability isn't linked to it");
    // Linked to a non-activated ability: "As ~ enters, choose a color." / "{T}: Add one
    // mana of the chosen color." The Mercenaries' copy has no choice.
    let chooser = custom_card(
        "Prism Lord",
        "Legendary Creature — Human",
        "{1}",
        Some((1, 1)),
        "As Prism Lord enters, choose a color.\n{T}: Add one mana of the chosen color.",
    );
    let lord = t.custom(P0, chooser, Zone::Battlefield);
    t.g.objects[lord.0 as usize].choices.color = Some(Color::Red);
    t.g.untap(mercs);
    let i = index_of(&mut t, mercs, "chosen color");
    t.activate(P0, mercs, i, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    let i = index_of(&mut t, lord, "chosen color");
    t.activate(P0, lord, i, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    // Kamahl, Fist of Krosa's "{G}: ..." needs green mana.
    t.battlefield(P0, "Kamahl, Fist of Krosa");
    t.g.players[P0.idx()].mana_pool.mana.clear();
    t.lands(P0, "Plains", 1);
    let land = t.battlefield(P1, "Island");
    let i = index_of(&mut t, mercs, "becomes a 1/1");
    assert!(t
        .activate(P0, mercs, i, &[Entity::Object(land)])
        .is_err());
}

// ---------------------------------------------------------------------------
// Lands' abilities: Sharkey, Mirran Safehouse
// ---------------------------------------------------------------------------

#[test]
fn sharkey_gains_lands_abilities_except_mana_abilities() {
    ruling!(
        "Sharkey, Tyrant of the Shire",
        "gains only activated abilities, excluding activated mana abilities"
    );
    ruling!(
        "Sharkey, Tyrant of the Shire",
        "references the card it's printed on by name, treat Sharkey, Tyrant of the Shire's version of that ability"
    );
    let mut t = TestGame::new(2);
    let sharkey = t.battlefield(P0, "Sharkey, Tyrant of the Shire");
    t.battlefield(P1, "Mutavault");
    t.battlefield(P1, "Island");
    assert!(!has(&mut t, sharkey, "Add {C}"));
    assert!(!has(&mut t, sharkey, "Add {U}"));
    t.lands(P0, "Wastes", 1);
    let i = index_of(&mut t, sharkey, "becomes a 2/2");
    t.activate(P0, sharkey, i, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.obj(sharkey).chars.all_creature_types);
    assert_eq!(t.pt(sharkey), (2, 2));
}

#[test]
fn mirran_safehouse_gains_land_cards_mana_abilities_too() {
    ruling!(
        "Mirran Safehouse",
        "Cards with basic land types intrinsically have the appropriate mana ability"
    );
    ruling!(
        "Mirran Safehouse",
        "Mirran Safehouse gains only activated abilities, including activated mana abilites."
    );
    ruling!(
        "Mirran Safehouse",
        "treat Mirran Safehouse's version of that ability as though it referenced Mirran Safehouse by name instead"
    );
    supported("Mirran Safehouse");
    let mut t = TestGame::new(2);
    let safehouse = t.battlefield(P0, "Mirran Safehouse");
    t.graveyard(P1, "Plains");
    t.graveyard(P0, "Mutavault");
    // Lotus Field: hexproof and an enters trigger aren't gained.
    t.graveyard(P0, "Lotus Field");
    let i = index_of(&mut t, safehouse, "Add {W}");
    t.activate(P0, safehouse, i, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::W), 1);
    assert!(has(&mut t, safehouse, "Add {C}"));
    assert_eq!(triggered(&mut t, safehouse), 0);
    t.g.recompute();
    assert!(!t.g.obj(safehouse).chars.has_keyword(KeywordKind::Hexproof));
    // Mutavault's ability makes the Safehouse a 2/2 creature (paid with the {W}).
    let i = index_of(&mut t, safehouse, "becomes a 2/2");
    t.activate(P0, safehouse, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(safehouse), (2, 2));
}

// ---------------------------------------------------------------------------
// Rex, Territory Forge, Trazyn, Hazel's Brewmaster, Conspicuous Snoop
// ---------------------------------------------------------------------------

#[test]
fn rex_has_the_abilities_of_cards_with_brain_counters() {
    ruling!(
        "Rex, Cyber-Hound",
        "treat Rex's version of that ability as though it referenced Rex, Cyber-Hound instead"
    );
    ruling!(
        "Rex, Cyber-Hound",
        "Rex gains only activated abilities. It doesn't gain triggered abilities or static abilities."
    );
    supported("Rex, Cyber-Hound");
    let mut t = TestGame::new(2);
    let rex = t.battlefield(P0, "Rex, Cyber-Hound");
    let cannon = t.exile(P1, "Lux Cannon");
    let mine = t.exile(P1, "Howling Mine");
    // Without a brain counter, nothing.
    assert!(!has(&mut t, rex, "charge counter"));
    for c in [cannon, mine] {
        t.g.add_counters(Entity::Object(c), "brain", 1, None);
    }
    assert!(has(&mut t, rex, "charge counter"));
    // Rex's own two triggered/activated abilities, nothing from Howling Mine.
    assert_eq!(triggered(&mut t, rex), 1);
    let i = index_of(&mut t, rex, "Put a charge counter");
    t.activate(P0, rex, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(rex, "charge"), 1);
}

#[test]
fn territory_forge_has_the_exiled_lands_abilities() {
    ruling!(
        "Territory Forge",
        "treat Territory Forge's version of that ability as though it referenced Territory Forge instead"
    );
    ruling!(
        "Territory Forge",
        "Territory Forge gains only activated abilities. It doesn't gain triggered abilities or static abilities."
    );
    supported("Territory Forge");
    let mut t = TestGame::new(2);
    // Treetop Village: "This land enters tapped." (static), "{T}: Add {G}.", "{1}{G}: This
    // land becomes a 3/3 green Ape creature with trample until end of turn. It's still a
    // land."
    let village = t.battlefield(P1, "Treetop Village");
    t.lands(P0, "Mountain", 5);
    let forge = t.hand(P0, "Territory Forge");
    t.cast(P0, forge).go();
    t.answer_targets(P0, &[Entity::Object(village)]);
    t.resolve_all();
    let forge = t.g.current(forge);
    assert!(t.in_exile("Treetop Village"));
    assert!(has(&mut t, forge, "Add {G}"));
    assert_eq!(triggered(&mut t, forge), 1, "only its own enters trigger");
    // The Ape ability animates Territory Forge.
    t.lands(P0, "Forest", 2);
    let i = index_of(&mut t, forge, "becomes a 3/3");
    t.activate(P0, forge, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(forge), (3, 3));
    t.g.recompute();
    assert!(t.g.obj(forge).chars.has_keyword(KeywordKind::Trample));
}

#[test]
fn trazyn_has_artifact_cards_activated_abilities() {
    ruling!(
        "Trazyn the Infinite",
        "treat Trazyn the Infinite's version of that ability as though it referenced Trazyn the Infinite by name instead"
    );
    ruling!(
        "Trazyn the Infinite",
        "Trazyn the Infinite gains only activated abilities."
    );
    supported("Trazyn the Infinite");
    let mut t = TestGame::new(2);
    let trazyn = t.battlefield(P0, "Trazyn the Infinite");
    t.graveyard(P0, "Lux Cannon");
    t.graveyard(P0, "Howling Mine");
    t.graveyard(P0, "Bonesplitter");
    t.g.recompute();
    assert!(t.g.obj(trazyn).chars.has_keyword(KeywordKind::Equip));
    assert_eq!(triggered(&mut t, trazyn), 0);
    let base = t.pt(trazyn);
    let i = index_of(&mut t, trazyn, "Put a charge counter");
    t.activate(P0, trazyn, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(trazyn, "charge"), 1);
    assert_eq!(t.pt(trazyn), base, "no static +2/+0");
}

#[test]
fn hazels_brewmasters_foods_have_the_exiled_creatures_abilities() {
    ruling!(
        "Hazel's Brewmaster",
        "Hazel's Brewmaster's last ability grants only activated abilities."
    );
    ruling!(
        "Hazel's Brewmaster",
        "so you treat the abilities as though they were printed on the permanent that gained the ability"
    );
    ruling!(
        "Hazel's Brewmaster",
        "Some keyword abilities are activated abilities; those will often have colons in their reminder text."
    );
    supported("Hazel's Brewmaster");
    let mut t = TestGame::new(2);
    let sprite = t.graveyard(P1, "Argothian Sprite");
    t.answer_targets(P0, &[Entity::Object(sprite)]);
    t.enter(P0, "Hazel's Brewmaster");
    t.resolve_all();
    assert!(t.in_exile("Argothian Sprite"));
    let food = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).chars.has_subtype("Food"))
        .expect("a Food");
    // "{7}: Put two +1/+1 counters on this creature" — on the Food.
    assert!(has(&mut t, food, "Put two +1/+1 counters"));
    t.lands(P0, "Forest", 7);
    let i = index_of(&mut t, food, "Put two +1/+1 counters");
    t.activate(P0, food, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(food, "+1/+1"), 2);
    // A non-Food permanent doesn't have it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!has(&mut t, bears, "Put two +1/+1 counters"));
}

#[test]
fn conspicuous_snoop_has_the_top_goblins_abilities() {
    ruling!(
        "Conspicuous Snoop",
        "treat Conspicuous Snoop's instance of that ability as though it referenced Conspicuous Snoop by name"
    );
    supported("Conspicuous Snoop");
    let mut t = TestGame::new(2);
    let snoop = t.battlefield(P0, "Conspicuous Snoop");
    t.library_top(P0, "Prodigal Sorcerer");
    assert!(activated(&mut t, snoop).is_empty(), "not a Goblin card");
    t.library_top(P0, "Goblin Balloon Brigade");
    t.lands(P0, "Mountain", 1);
    let i = index_of(&mut t, snoop, "gains flying");
    t.activate(P0, snoop, i, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.obj(snoop).chars.has_keyword(KeywordKind::Flying));
}

// ---------------------------------------------------------------------------
// Marvin, Murderous Mimic
// ---------------------------------------------------------------------------

#[test]
fn marvin_has_other_named_creatures_abilities() {
    ruling!(
        "Marvin, Murderous Mimic",
        "treat Marvin's version of that ability as though it referenced Marvin instead"
    );
    ruling!(
        "Marvin, Murderous Mimic",
        "Marvin gains only activated abilities. It doesn't gain triggered abilities or static abilities."
    );
    ruling!(
        "Marvin, Murderous Mimic",
        "it doesn't matter if the other creature you control with that ability leaves the battlefield; the ability still resolves"
    );
    supported("Marvin, Murderous Mimic");
    let mut t = TestGame::new(2);
    let marvin = t.battlefield(P0, "Marvin, Murderous Mimic");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.battlefield(P0, "Flametongue Kavu");
    t.battlefield(P1, "Cudgel Troll");
    // Its own creatures only, and not a triggered ability.
    assert!(!has(&mut t, marvin, "Regenerate"));
    assert_eq!(triggered(&mut t, marvin), 0);
    let i = index_of(&mut t, marvin, "deals 1 damage");
    t.activate(P0, marvin, i, &[Entity::Player(P1)]).unwrap();
    // The Sorcerer leaves before the ability resolves.
    t.g.destroy(sorcerer, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!has(&mut t, marvin, "deals 1 damage"));
    assert!(t.obj_now(marvin).tapped, "Marvin tapped for the cost");
    // A creature with the same name doesn't give its abilities; another of its
    // controller's creatures does.
    t.battlefield(P0, "Marvin, Murderous Mimic");
    assert_eq!(activated(&mut t, marvin).len(), 0);
    t.battlefield(P0, "Cudgel Troll");
    assert!(has(&mut t, marvin, "Regenerate"));
    t.lands(P0, "Forest", 1);
    assert!(regenerates(&mut t, marvin));
}

// ---------------------------------------------------------------------------
// Loyalty abilities of other planeswalkers: Kasmina, Nicol Bolas, Dragon-God
// ---------------------------------------------------------------------------

#[test]
fn kasminas_abilities_count_toward_each_planeswalkers_one_activation() {
    ruling!(
        "Kasmina, Enigma Sage",
        "For each planeswalker you control, you may still activate only one of that planeswalker's loyalty abilities per turn."
    );
    supported("Kasmina, Enigma Sage");
    let mut t = TestGame::new(2);
    let kasmina = t.battlefield(P0, "Kasmina, Enigma Sage");
    let jace = t.battlefield(P0, "Jace Beleren");
    // Jace has Kasmina's loyalty abilities in addition to its own; Kasmina doesn't get
    // Jace's.
    assert!(has(&mut t, jace, "Scry 1"));
    assert!(has(&mut t, jace, "Each player draws a card"));
    assert!(!has(&mut t, kasmina, "Each player draws a card"));
    let loyalty = t.counters(jace, "loyalty");
    let i = index_of(&mut t, jace, "Scry 1");
    t.activate(P0, jace, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(jace, "loyalty"), loyalty + 2, "Jace's loyalty pays");
    let i = index_of(&mut t, jace, "Each player draws a card");
    assert!(t.activate(P0, jace, i, &[]).is_err());
    // Kasmina may still activate one of her own.
    let i = index_of(&mut t, kasmina, "Scry 1");
    t.activate(P0, kasmina, i, &[]).unwrap();
}

#[test]
fn kasminas_last_ability_on_another_planeswalker_uses_its_colors() {
    ruling!(
        "Kasmina, Enigma Sage",
        "you search your library for a card that shares a color with that planeswalker, not with Kasmina"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kasmina, Enigma Sage");
    // Jace Beleren is blue only; Kasmina is green and blue.
    let jace = t.battlefield(P0, "Jace Beleren");
    t.g.add_counters(Entity::Object(jace), "loyalty", 10, None);
    let lib = stack_library(&mut t, P0, &["Giant Growth", "Opt", "Island"]);
    let growth = lib[0];
    t.answer_choose(P0, &[Entity::Object(growth)]);
    t.answer_yes(P0, true);
    let i = index_of(&mut t, jace, "Search your library");
    t.activate(P0, jace, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        t.zone(t.g.current(growth)),
        Zone::Library(P0),
        "the green card can't be found"
    );
    assert_ne!(t.zone(t.g.current(lib[1])), Zone::Library(P0), "Opt was");
}

#[test]
fn nicol_bolas_dragon_god_has_other_planeswalkers_loyalty_abilities() {
    ruling!(
        "Nicol Bolas, Dragon-God",
        "treat Nicol Bolas's instance of that ability as though it referenced Nicol Bolas, Dragon-God by name instead"
    );
    ruling!(
        "Nicol Bolas, Dragon-God",
        "Nicol Bolas doesn't gain any static or triggered abilities of other planeswalkers"
    );
    ruling!(
        "Nicol Bolas, Dragon-God",
        "Nicol Bolas doesn't remove loyalty abilities from the other planeswalkers."
    );
    ruling!(
        "Nicol Bolas, Dragon-God",
        "you can still activate only one loyalty ability of Nicol Bolas, Dragon-God during each of your turns"
    );
    let mut t = TestGame::new(2);
    let bolas = t.battlefield(P0, "Nicol Bolas, Dragon-God");
    let ral = t.battlefield(P0, "Ral Zarek");
    // Teferi, Time Raveler's static ability isn't gained.
    t.battlefield(P0, "Teferi, Time Raveler");
    assert!(has(&mut t, bolas, "deals 3 damage"));
    assert!(has(&mut t, ral, "deals 3 damage"));
    let loyalty = (t.counters(bolas, "loyalty"), t.counters(ral, "loyalty"));
    let i = index_of(&mut t, bolas, "deals 3 damage");
    t.activate(P0, bolas, i, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(bolas, "loyalty"), loyalty.0 - 2);
    assert_eq!(t.counters(ral, "loyalty"), loyalty.1);
    let i = index_of(&mut t, bolas, "Return up to one target");
    assert!(t.activate(P0, bolas, i, &[]).is_err());
    // Teferi's static ability: an opponent can still cast instants at instant speed
    // because of Teferi itself, not because of Bolas: Bolas has no static abilities from it.
    t.g.recompute();
    let statics = t
        .g
        .obj(bolas)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Static(_)))
        .count();
    assert_eq!(statics, 1, "only its own");
}
