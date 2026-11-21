//! Card-flow grammar IV (`cards2-zones-dig`), review: behaviour tests for cards the item
//! compiled that had no test of their own (a running mana-value total over chosen cards,
//! per-card name searches across zones, "until they have exiled cards with total mana
//! value 5 or greater" with a cast permission, "for each of those permanents, search for
//! a card with the same name", "shares a creature type with enchanted creature",
//! "with different names" across "and/or").

use mtg_engine::object::CastMethod;
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

/// Replaces `p`'s library with the named cards; the last one named ends up on top.
fn library(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.0 as usize].library.clear();
    names.iter().map(|n| t.library_top(p, n)).collect()
}

fn sorted(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    let mut v: Vec<String> = ids
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// `p`'s library, top first.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// The names of the permanents `p` controls among those named.
fn controlled(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.obj_now(*o).controller == p)
        .collect()
}

fn smelting_vat_setup(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    let vat = t.battlefield(P0, "Smelting Vat");
    let millstone = t.battlefield(P0, "Millstone");
    // The cost: sacrifice Millstone (mana value 2).
    t.answer_choose(P0, &[Entity::Object(millstone)]);
    t.lands(P0, "Plains", 1);
    // Top first: Mind Stone (2), Sol Ring (1), Mana Vault (1), Ornithopter (0, a creature),
    // Thran Dynamo (4), Forest, Island, Darksteel Relic (0); Plains isn't revealed.
    let ids = library(
        t,
        P0,
        &[
            "Plains",
            "Darksteel Relic",
            "Island",
            "Forest",
            "Thran Dynamo",
            "Ornithopter",
            "Mana Vault",
            "Sol Ring",
            "Mind Stone",
        ],
    );
    (vat, ids)
}

#[test]
fn smelting_vat_two_artifacts_with_total_mana_value_up_to_the_sacrificed_ones() {
    cr!("202.3", "701.20a");
    assert_supported("Smelting Vat");
    let mut t = TestGame::new(2);
    let (vat, ids) = smelting_vat_setup(&mut t);
    // Sol Ring and Mana Vault: total 1 + 1 = 2, Millstone's mana value.
    t.answer_choose(P0, &[Entity::Object(ids[7]), Entity::Object(ids[6])]);
    t.activate(P0, vat, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Millstone"));
    assert_eq!(controlled(&t, P0, "Sol Ring").len(), 1, "{}", t.dump_log());
    assert_eq!(controlled(&t, P0, "Mana Vault").len(), 1);
    // The unrevealed Plains is now on top; the six others are beneath it.
    let lib = library_names(&t, P0);
    assert_eq!(lib[0], "Plains");
    let mut rest = lib[1..].to_vec();
    rest.sort();
    assert_eq!(
        rest,
        [
            "Darksteel Relic",
            "Forest",
            "Island",
            "Mind Stone",
            "Ornithopter",
            "Thran Dynamo"
        ]
    );
}

#[test]
fn smelting_vat_cant_exceed_the_sacrificed_artifacts_mana_value() {
    cr!("202.3");
    let mut t = TestGame::new(2);
    let (vat, ids) = smelting_vat_setup(&mut t);
    // Mind Stone and Sol Ring total 3, more than Millstone's 2.
    t.answer_choose(P0, &[Entity::Object(ids[8]), Entity::Object(ids[7])]);
    t.activate(P0, vat, 0, &[]).unwrap();
    t.resolve();
    let total: i64 = ["Mind Stone", "Sol Ring", "Mana Vault", "Thran Dynamo"]
        .iter()
        .flat_map(|n| controlled(&t, P0, n))
        .map(|o| t.obj_now(o).chars.mana_value() as i64)
        .sum();
    assert!(total <= 2, "{}", t.dump_log());
    assert!(controlled(&t, P0, "Thran Dynamo").is_empty());
    assert!(controlled(&t, P0, "Ornithopter").is_empty());
    assert!(!(controlled(&t, P0, "Mind Stone").len() == 1
        && controlled(&t, P0, "Sol Ring").len() == 1));
}

#[test]
fn reap_intellect_exiles_the_chosen_cards_and_every_copy() {
    cr!("701.23b", "201.2");
    assert_supported("Reap Intellect");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    let shock = t.hand(P1, "Shock");
    t.hand(P1, "Shock");
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    t.graveyard(P1, "Shock");
    library(&mut t, P1, &["Opt", "Shock"]);
    let spell = t.hand(P0, "Reap Intellect");
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.cast(P0, spell).target(Entity::Player(P1)).x(1).go();
    t.resolve();
    assert_eq!(
        sorted(&t, &t.g.exile.clone()),
        ["Shock", "Shock", "Shock", "Shock"],
        "{}",
        t.dump_log()
    );
    // X = 1: the Lightning Bolt stays.
    assert_eq!(
        sorted(&t, &t.g.player(P1).hand.clone()),
        ["Forest", "Lightning Bolt"]
    );
    assert_eq!(library_names(&t, P1), ["Opt"]);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn invasive_surgery_with_delirium_exiles_every_copy_of_the_countered_sorcery() {
    cr!("701.6a", "701.23b");
    assert_supported("Invasive Surgery");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    // Four card types in P0's graveyard.
    for n in ["Forest", "Shock", "Grizzly Bears", "Sol Ring"] {
        t.graveyard(P0, n);
    }
    t.lands(P1, "Island", 3);
    t.hand(P1, "Divination");
    t.graveyard(P1, "Divination");
    library(&mut t, P1, &["Opt", "Divination"]);
    let div = t.hand(P1, "Divination");
    t.cast(P1, div).go();
    let spell = t.g.stack[0];
    t.lands(P0, "Island", 1);
    let surgery = t.hand(P0, "Invasive Surgery");
    t.cast(P0, surgery).target(spell).go();
    t.resolve_all();
    // The copies in hand, graveyard and library are exiled, and so is the countered spell,
    // which is in the graveyard by then.
    assert_eq!(
        sorted(&t, &t.g.exile.clone()),
        ["Divination", "Divination", "Divination", "Divination"],
        "{}",
        t.dump_log()
    );
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(library_names(&t, P1), ["Opt"]);

    // Without delirium: only countered.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.graveyard(P0, "Forest");
    t.lands(P1, "Island", 3);
    t.hand(P1, "Divination");
    let div = t.hand(P1, "Divination");
    t.cast(P1, div).go();
    let spell = t.g.stack[0];
    t.lands(P0, "Island", 1);
    let surgery = t.hand(P0, "Invasive Surgery");
    t.cast(P0, surgery).target(spell).go();
    t.resolve_all();
    assert!(t.g.exile.is_empty());
    assert!(t.in_graveyard(P1, "Divination"));
    assert!(t.in_hand(P1, "Divination"));
}

#[test]
fn invasive_surgery_counts_the_countered_sorcery_in_your_graveyard() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Invasive Surgery",
        "delirium ability isn't checked until after the sorcery spell has been countered"
    );
    // Three card types in P0's graveyard; the countered sorcery (P0's own) is the fourth.
    let mut t = TestGame::new(2);
    for n in ["Forest", "Shock", "Grizzly Bears"] {
        t.graveyard(P0, n);
    }
    t.lands(P0, "Island", 4);
    library(&mut t, P0, &["Opt", "Divination"]);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    let spell = t.g.stack[0];
    let surgery = t.hand(P0, "Invasive Surgery");
    t.cast(P0, surgery).target(spell).go();
    t.resolve_all();
    assert_eq!(
        sorted(&t, &t.g.exile.clone()),
        ["Divination", "Divination"],
        "{}",
        t.dump_log()
    );
    assert_eq!(library_names(&t, P0), ["Opt"]);
}

#[test]
fn dream_harvest_exiles_until_total_mana_value_five_and_casts_free() {
    cr!("701.20a", "202.3");
    assert_supported("Dream Harvest");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    // Top first: Forest (0), Shock (1), Divination (3), Grizzly Bears (2) — 6 — then
    // Lightning Bolt.
    library(
        &mut t,
        P1,
        &[
            "Lightning Bolt",
            "Grizzly Bears",
            "Divination",
            "Shock",
            "Forest",
        ],
    );
    let spell = t.hand(P0, "Dream Harvest");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(
        sorted(&t, &t.g.exile.clone()),
        ["Divination", "Forest", "Grizzly Bears", "Shock"]
    );
    assert_eq!(library_names(&t, P1), ["Lightning Bolt"]);
    // Cast the exiled Shock without paying its mana cost (all lands are tapped).
    let shock = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Shock")[0];
    t.cast(P0, shock)
        .method(CastMethod::Free)
        .target(Entity::Player(P1))
        .go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // The permission lasts until end of turn only.
    let bears = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Grizzly Bears")[0];
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t
        .cast(P0, bears)
        .method(CastMethod::Free)
        .try_go()
        .is_err());
}

#[test]
fn clarion_ultimatum_searches_for_a_card_named_like_each_chosen_permanent() {
    cr!("701.23a", "201.2");
    assert_supported("Clarion Ultimatum");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    library(&mut t, P0, &["Shock", "Grizzly Bears", "Llanowar Elves", "Serra Angel"]);
    let chosen: Vec<Entity> = vec![Entity::Object(bears), Entity::Object(elves)]
        .into_iter()
        .chain(
            t.named_on_battlefield("Island")
                .into_iter()
                .chain(t.named_on_battlefield("Plains"))
                .take(3)
                .map(Entity::Object),
        )
        .collect();
    t.answer_choose(P0, &chosen);
    let spell = t.hand(P0, "Clarion Ultimatum");
    t.cast(P0, spell).go();
    t.resolve();
    let new_bears: Vec<ObjectId> = controlled(&t, P0, "Grizzly Bears")
        .into_iter()
        .filter(|o| *o != bears)
        .collect();
    assert_eq!(new_bears.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(new_bears[0]).tapped);
    assert_eq!(controlled(&t, P0, "Llanowar Elves").len(), 2);
    // No permanent named Serra Angel was chosen.
    assert!(controlled(&t, P0, "Serra Angel").is_empty());
    let mut lib = library_names(&t, P0);
    lib.sort();
    assert_eq!(lib, ["Serra Angel", "Shock"]);
}

#[test]
fn call_to_the_kindred_puts_a_creature_sharing_a_type_with_the_enchanted_one() {
    cr!("205.3m", "701.20e");
    assert_supported("Call to the Kindred");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let aura = t.battlefield(P0, "Call to the Kindred");
    assert!(t.g.attach(aura, Entity::Object(elves)));
    // Top first: Grizzly Bears, Elvish Mystic, Shock, Forest, Island; then Plains.
    let ids = library(
        &mut t,
        P0,
        &[
            "Plains",
            "Island",
            "Forest",
            "Shock",
            "Elvish Mystic",
            "Grizzly Bears",
        ],
    );
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    // The Bears (a Bear) can't be put onto the battlefield; Elvish Mystic (an Elf) can.
    t.answer_choose(P0, &[Entity::Object(ids[4])]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controlled(&t, P0, "Elvish Mystic").len(), 1, "{}", t.dump_log());
    assert!(controlled(&t, P0, "Grizzly Bears").is_empty());
    let lib = library_names(&t, P0);
    assert_eq!(lib[0], "Plains");
    let mut rest = lib[1..].to_vec();
    rest.sort();
    assert_eq!(rest, ["Forest", "Grizzly Bears", "Island", "Shock"]);
}

#[test]
fn call_to_the_kindred_cant_put_a_creature_of_another_type() {
    cr!("205.3m");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let aura = t.battlefield(P0, "Call to the Kindred");
    assert!(t.g.attach(aura, Entity::Object(elves)));
    let ids = library(&mut t, P0, &["Plains", "Grizzly Bears", "Serra Angel"]);
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ids[1])]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(controlled(&t, P0, "Grizzly Bears").is_empty());
    assert!(controlled(&t, P0, "Serra Angel").is_empty());
    assert_eq!(t.library_size(P0), 3);
}

#[test]
fn ryan_sinclair_casts_a_spell_with_mana_value_up_to_its_power() {
    cr!("701.20a", "305.9");
    assert_supported("Ryan Sinclair");
    // Top first: Forest, Shock (1 <= 2): cast it.
    let mut t = TestGame::new(2);
    let ryan = t.battlefield(P0, "Ryan Sinclair");
    library(&mut t, P0, &["Island", "Shock", "Forest"]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(ryan, Entity::Player(P1))], &[]);
    t.resolve_all();
    // Shock (2) and Ryan's combat damage (2). The exiled Forest is a land, which can't be
    // cast (CR 305.9): only "the exiled card" (the nonland one) may be cast.
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Shock"));
    assert!(controlled(&t, P0, "Forest").is_empty());
    assert_eq!(library_names(&t, P0), ["Island", "Forest"]);

    // Top first: Serra Angel (5 > 2): it can't be cast and goes to the bottom.
    let mut t = TestGame::new(2);
    let ryan = t.battlefield(P0, "Ryan Sinclair");
    library(&mut t, P0, &["Island", "Serra Angel"]);
    t.answer_yes(P0, true);
    t.attack(&[(ryan, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(controlled(&t, P0, "Serra Angel").is_empty());
    assert_eq!(library_names(&t, P0), ["Island", "Serra Angel"]);
}

#[test]
fn reach_the_horizon_finds_lands_with_different_names() {
    cr!("701.23a", "201.2");
    assert_supported("Reach the Horizon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let ids = library(&mut t, P0, &["Shock", "Island", "Forest", "Forest"]);
    let spell = t.hand(P0, "Reach the Horizon");
    // Two Forests share a name: not allowed together.
    t.answer_choose(P0, &[Entity::Object(ids[2]), Entity::Object(ids[3])]);
    t.cast(P0, spell).go();
    t.resolve();
    let found = t.library_size(P0);
    assert!(found >= 2, "{}", t.dump_log());
    assert!(controlled(&t, P0, "Forest").len() <= 5);

    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let ids = library(&mut t, P0, &["Shock", "Island", "Forest", "Forest"]);
    let spell = t.hand(P0, "Reach the Horizon");
    t.answer_choose(P0, &[Entity::Object(ids[1]), Entity::Object(ids[3])]);
    t.cast(P0, spell).go();
    t.resolve();
    let island = controlled(&t, P0, "Island");
    assert_eq!(island.len(), 1);
    assert!(t.obj_now(island[0]).tapped);
    assert_eq!(controlled(&t, P0, "Forest").len(), 5);
    let mut lib = library_names(&t, P0);
    lib.sort();
    assert_eq!(lib, ["Forest", "Shock"]);
}

#[test]
fn defense_of_the_heart_sacrifices_and_puts_two_creatures_onto_the_battlefield() {
    cr!("603.4", "701.23a");
    assert_supported("Defense of the Heart");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defense of the Heart");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    let ids = library(&mut t, P0, &["Shock", "Serra Angel", "Llanowar Elves", "Forest"]);
    t.answer_choose(P0, &[Entity::Object(ids[1]), Entity::Object(ids[2])]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Defense of the Heart"));
    assert_eq!(controlled(&t, P0, "Serra Angel").len(), 1, "{}", t.dump_log());
    assert_eq!(controlled(&t, P0, "Llanowar Elves").len(), 1);

    // Two creatures: no trigger (intervening if, CR 603.4).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defense of the Heart");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Defense of the Heart").len(), 1);
}

#[test]
fn aethermages_touch_creature_returns_to_hand_at_end_step() {
    cr!("701.20a", "513.1a");
    assert_supported("Aethermage's Touch");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    let ids = library(&mut t, P0, &["Plains", "Island", "Shock", "Serra Angel", "Forest"]);
    t.answer_choose(P0, &[Entity::Object(ids[3])]);
    let spell = t.hand(P0, "Aethermage's Touch");
    t.cast(P0, spell).go();
    t.resolve();
    let angel = controlled(&t, P0, "Serra Angel");
    assert_eq!(angel.len(), 1, "{}", t.dump_log());
    let lib = library_names(&t, P0);
    assert_eq!(lib[0], "Plains");
    let mut rest = lib[1..].to_vec();
    rest.sort();
    assert_eq!(rest, ["Forest", "Island", "Shock"]);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Serra Angel"));
}
