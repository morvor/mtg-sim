//! CR 605.3a, 601.2g–h, 118.3: mana abilities whose costs tap other untapped permanents
//! ("Tap three untapped Elves you control: Add {G}{G}{G}."), discard or exile cards from a
//! hand, or whose mana depends on the object the cost takes ("Sacrifice a land: Add one
//! mana of any type the sacrificed land could produce.", "Exile a creature you control:
//! Add X mana of any one color, where X is 1 plus the exiled creature's mana value.") are
//! planned by the automatic payment (`automatic-mana-payment`), so the spells they pay for
//! are offered and cast.

use mtg_engine::decision::{Action, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn main_phase() -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// Whether casting `card` is among `p`'s legal actions.
fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).contains(&Action::Cast {
        card,
        method: CastMethod::Normal,
    })
}

fn on_stack(t: &TestGame, name: &str) -> bool {
    t.g.stack
        .iter()
        .any(|id| t.g.obj(*id).chars.name.as_str() == name)
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(t.g.current(id)).tapped
}

/// The candidates of each "Choose cards to ..." cost decision asked so far.
fn cost_choices(t: &TestGame, word: &str) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if prompt.contains(word) => Some(candidates),
            _ => None,
        })
        .collect()
}

#[test]
fn heritage_druid_taps_summoning_sick_elves_itself_included() {
    cr!("605.3a", "601.2g", "302.6");
    interp!("automatic-mana-payment");
    ruling!(
        "Heritage Druid",
        "you can tap creatures that haven’t been under your control since your most recent turn began (including Heritage Druid itself) to pay the cost"
    );
    // Heritage Druid ("Tap three untapped Elves you control: Add {G}{G}{G}.") and two
    // Llanowar Elves, all summoning sick: the Elves can't be tapped for their own mana
    // (CR 302.6), but the Druid's cost has no {T} and taps all three.
    let mut t = main_phase();
    let druid = t.battlefield_sick(P0, "Heritage Druid");
    let a = t.battlefield_sick(P0, "Llanowar Elves");
    let b = t.battlefield_sick(P0, "Llanowar Elves");
    let baloth = t.hand(P0, "Leatherback Baloth");
    assert!(offered(&mut t, P0, baloth));
    t.cast(P0, baloth).go();
    assert!(on_stack(&t, "Leatherback Baloth"));
    assert!(tapped(&t, druid) && tapped(&t, a) && tapped(&t, b));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn a_permanent_tapped_for_its_own_mana_isnt_also_tapped_for_another_cost() {
    cr!("118.3", "605.3a", "601.2g");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Heritage Druid and three Llanowar Elves: the Druid's ability taps itself and two
    // Elves ({G}{G}{G}) and the third Elf taps for {G}: {4}. Never {5}: an Elf tapped for
    // its own mana can't also be tapped for the Druid's cost.
    let mut t = main_phase();
    let druid = t.battlefield(P0, "Heritage Druid");
    let elves: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Llanowar Elves"))
        .collect();
    let golem = t.hand(P0, "Precursor Golem");
    let tome = t.hand(P0, "Jayemdae Tome");
    assert!(!offered(&mut t, P0, golem));
    assert!(offered(&mut t, P0, tome));
    t.cast(P0, tome).go();
    assert!(on_stack(&t, "Jayemdae Tome"));
    assert!(tapped(&t, druid));
    assert!(elves.iter().all(|e| tapped(&t, *e)));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn birchlore_rangers_taps_two_elves_itself_included_for_any_color() {
    cr!("605.3a", "601.2g", "302.6");
    interp!("automatic-mana-payment");
    ruling!(
        "Birchlore Rangers",
        "You may tap any untapped permanents you control with the appropriate creature type (including this creature) to pay the cost of the ability"
    );
    let mut t = main_phase();
    let rangers = t.battlefield_sick(P0, "Birchlore Rangers");
    let elf = t.battlefield_sick(P0, "Llanowar Elves");
    let opt = t.hand(P0, "Opt");
    assert!(offered(&mut t, P0, opt));
    t.cast(P0, opt).go();
    assert!(on_stack(&t, "Opt"));
    assert!(tapped(&t, rangers) && tapped(&t, elf));
}

#[test]
fn urza_taps_artifacts_even_new_artifact_creatures_for_blue() {
    cr!("605.3a", "601.2g", "302.6");
    interp!("automatic-mana-payment");
    ruling!(
        "Urza, Lord High Artificer",
        "You can tap any untapped artifact you control to pay the cost of the mana ability, including an artifact creature you haven't controlled continuously since the beginning of your most recent turn"
    );
    // Urza ("Tap an untapped artifact you control: Add {U}.") with two Ornithopters that
    // just arrived: {1}{U}, once per artifact.
    let mut t = main_phase();
    let urza = t.battlefield(P0, "Urza, Lord High Artificer");
    let a = t.battlefield_sick(P0, "Ornithopter");
    let b = t.battlefield_sick(P0, "Ornithopter");
    let golem = t.hand(P0, "Precursor Golem");
    let twice = t.hand(P0, "Think Twice");
    assert!(!offered(&mut t, P0, golem));
    assert!(offered(&mut t, P0, twice));
    t.cast(P0, twice).go();
    assert!(on_stack(&t, "Think Twice"));
    assert!(tapped(&t, a) && tapped(&t, b));
    assert!(!tapped(&t, urza));
}

#[test]
fn skirge_familiar_discards_the_card_the_player_chooses() {
    cr!("605.3a", "601.2g", "118.3", "701.9a");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Skirge Familiar ("Discard a card: Add {B}."): Duress alone in hand can't pay for
    // itself (it's being cast, CR 601.2a); with other cards, the player chooses which
    // one to discard.
    let mut t = main_phase();
    t.battlefield(P0, "Skirge Familiar");
    let duress = t.hand(P0, "Duress");
    assert!(!offered(&mut t, P0, duress));
    let bears = t.hand(P0, "Grizzly Bears");
    let forest = t.hand(P0, "Forest");
    assert!(offered(&mut t, P0, duress));
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.cast(P0, duress).target(P1).go();
    assert!(on_stack(&t, "Duress"));
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.g.current(bears), bears);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // The player was asked, among both other cards.
    let asked = cost_choices(&t, "discard");
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(
        asked[0].contains(&Entity::Object(bears)) && asked[0].contains(&Entity::Object(forest))
    );
}

#[test]
fn cadaverous_bloom_exiles_another_card_from_hand() {
    cr!("605.3a", "601.2g", "118.3");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Cadaverous Bloom ("Exile a card from your hand: Add {B}{B} or {G}{G}."): Sign in
    // Blood ({B}{B}) alone in hand isn't offered; with a Grizzly Bears, the Bears pays.
    let mut t = main_phase();
    t.battlefield(P0, "Cadaverous Bloom");
    let sign = t.hand(P0, "Sign in Blood");
    assert!(!offered(&mut t, P0, sign));
    t.hand(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, sign));
    t.cast(P0, sign).target(P1).go();
    assert!(on_stack(&t, "Sign in Blood"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn squandered_resources_sacrifices_the_land_whose_mana_was_planned() {
    cr!("605.3a", "601.2g", "106.7");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    ruling!(
        "Squandered Resources",
        "Can sacrifice a land which can’t produce mana, but you don’t get any mana from this ability."
    );
    // Squandered Resources ("Sacrifice a land: Add one mana of any type the sacrificed land
    // could produce."): with tapped lands only, Opt ({U}) is paid by sacrificing the
    // Island, not the Forest or Maze of Ith (no mana).
    let mut t = main_phase();
    t.battlefield(P0, "Squandered Resources");
    let maze = t.battlefield(P0, "Maze of Ith");
    let forest = t.battlefield(P0, "Forest");
    t.g.objects[forest.0 as usize].tapped = true;
    let opt = t.hand(P0, "Opt");
    assert!(!offered(&mut t, P0, opt));
    let island = t.battlefield(P0, "Island");
    t.g.objects[island.0 as usize].tapped = true;
    assert!(offered(&mut t, P0, opt));
    t.cast(P0, opt).go();
    assert!(on_stack(&t, "Opt"));
    assert!(!t.on_battlefield(island));
    assert!(t.on_battlefield(forest) && t.on_battlefield(maze));
    // Each land pays for one activation: the Forest's {G} and the Island's {U}.
    let mut t = main_phase();
    t.battlefield(P0, "Squandered Resources");
    for land in ["Forest", "Island"] {
        let l = t.battlefield(P0, land);
        t.g.objects[l.0 as usize].tapped = true;
    }
    let gu = t.hand(P0, "Simic Signet");
    let gu_creature = t.hand(P0, "Coiling Oracle");
    assert!(offered(&mut t, P0, gu));
    t.cast(P0, gu_creature).go();
    assert!(on_stack(&t, "Coiling Oracle"));
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert!(t.named_on_battlefield("Island").is_empty());
}

#[test]
fn food_chain_adds_one_plus_the_exiled_creatures_mana_value() {
    cr!("605.3a", "601.2g", "106.6", "400.7j");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    interp!("castability-is-optimistic");
    // Food Chain ("Exile a creature you control: Add X mana of any one color, where X is 1
    // plus the exiled creature's mana value. Spend this mana only to cast creature
    // spells."), activated by hand: Grizzly Bears (mana value 2) makes three.
    let mut t = main_phase();
    let chain = t.battlefield(P0, "Food Chain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, chain, 0, &[]).expect("activate Food Chain");
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
    // Planned: Memnite (mana value 0) makes only one; the Bears pays for Centaur Courser
    // ({2}{G}), but not for a noncreature spell (CR 106.6; the list of castable spells
    // ignores the restriction, `castability-is-optimistic`, and the cast is reversed).
    let mut t = main_phase();
    t.battlefield(P0, "Food Chain");
    let memnite = t.battlefield(P0, "Memnite");
    let courser = t.hand(P0, "Centaur Courser");
    let twice = t.hand(P0, "Think Twice");
    assert!(!offered(&mut t, P0, courser));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, courser));
    assert!(offered(&mut t, P0, twice));
    assert!(t.cast(P0, twice).try_go().is_err());
    assert!(t.on_battlefield(bears) && t.in_hand(P0, "Think Twice"));
    t.cast(P0, courser).go();
    assert!(on_stack(&t, "Centaur Courser"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(memnite));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn food_chain_mana_pays_a_creature_spells_kicker() {
    cr!("605.3a", "601.2f", "106.6", "702.33d");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    ruling!(
        "Food Chain",
        "This mana may be used on additional costs to cast the spell, such as Kicker."
    );
    // Exiling Hill Giant (mana value 4) makes five: Gnarlid Colony ({1}{G}) with its
    // kicker ({2}{G}).
    let mut t = main_phase();
    t.battlefield(P0, "Food Chain");
    t.battlefield(P0, "Hill Giant");
    let colony = t.hand(P0, "Gnarlid Colony");
    t.cast(P0, colony).kicked(true).go();
    assert!(on_stack(&t, "Gnarlid Colony"));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve();
    let colony = t.named_on_battlefield("Gnarlid Colony")[0];
    assert_eq!(t.counters(colony, "+1/+1"), 2);
}

#[test]
fn creatures_druids_and_foods_tapped_for_mana_by_other_permanents() {
    cr!("605.3a", "601.2g", "302.6");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    ruling!(
        "The Cabbage Merchant",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token."
    );
    // Seton, Krosan Protector ("Tap an untapped Druid you control: Add {G}."), a Druid
    // itself but already tapped, taps a summoning-sick Llanowar Elves (an Elf Druid).
    let mut t = main_phase();
    let seton = t.battlefield(P0, "Seton, Krosan Protector");
    t.g.objects[seton.0 as usize].tapped = true;
    let elf = t.battlefield_sick(P0, "Llanowar Elves");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(elf).go();
    assert!(on_stack(&t, "Giant Growth"));
    assert!(tapped(&t, elf));
    // Supportive Parents ("Tap two untapped creatures you control: Add one mana of any
    // color."), already tapped, taps two new Grizzly Bears for {U}.
    let mut t = main_phase();
    let parents = t.battlefield(P0, "Supportive Parents");
    t.g.objects[parents.0 as usize].tapped = true;
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Grizzly Bears");
    let opt = t.hand(P0, "Opt");
    assert!(offered(&mut t, P0, opt));
    t.cast(P0, opt).go();
    assert!(on_stack(&t, "Opt"));
    assert!(tapped(&t, a) && tapped(&t, b));
    // The Cabbage Merchant ("Tap two untapped Foods you control: Add one mana of any
    // color.") taps two Gingerbrutes (Artifact Creature — Food Golem).
    let mut t = main_phase();
    t.battlefield(P0, "The Cabbage Merchant");
    let a = t.battlefield_sick(P0, "Gingerbrute");
    let b = t.battlefield_sick(P0, "Gingerbrute");
    let opt = t.hand(P0, "Opt");
    assert!(offered(&mut t, P0, opt));
    t.cast(P0, opt).go();
    assert!(on_stack(&t, "Opt"));
    assert!(tapped(&t, a) && tapped(&t, b));
}
