//! The zone-move grammar (`oracle/patterns/zone_move_grammar.rs`): "return/put [objects]
//! [from zone] to [zone] [modifiers]" with zone-qualified objects, history qualifiers,
//! modifiers, multi-target lists, owner-side moves and bounces of your own permanents.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// The card's ability containing `fragment` compiled (other abilities may not have).
fn assert_compiles(name: &str, fragment: &str) {
    let c = card(name);
    let bad: Vec<&str> = c
        .unsupported_text()
        .into_iter()
        .filter(|u| u.to_lowercase().contains(&fragment.to_lowercase()))
        .collect();
    assert!(bad.is_empty(), "{name}: {bad:?}");
    let text: String = c
        .faces
        .iter()
        .map(|f| f.chars.rules_text.to_lowercase())
        .collect();
    assert!(
        text.contains(&fragment.to_lowercase()),
        "{name} has no text {fragment:?}"
    );
}

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Links `card` to `source` as a card its abilities exiled (CR 607.2a).
fn link(t: &mut TestGame, source: ObjectId, card: ObjectId) {
    t.g.objects[source.0 as usize]
        .linked
        .entry(0)
        .or_default()
        .push(card);
}

// ---------------------------------------------------------------------------
// "[cards] exiled with ~"
// ---------------------------------------------------------------------------

#[test]
fn detention_sphere_returns_the_exiled_cards_under_their_owners_control() {
    cr!("607.2a", "110.2a");
    assert_supported("Detention Sphere");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    let sphere = t.hand(P0, "Detention Sphere");
    t.cast(P0, sphere).go();
    t.resolve(); // the Sphere enters
    t.answer_targets(P0, &[o(bears)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let sphere = t.named_on_battlefield("Detention Sphere")[0];
    t.g.destroy(sphere, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    // Under its owner's control, not the Sphere's controller's.
    assert_eq!(t.obj(back[0]).controller, P1);
}

#[test]
fn skyship_weatherlight_puts_a_random_card_exiled_with_it_into_its_owners_hand() {
    cr!("607.2a");
    assert_supported("Skyship Weatherlight");
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Skyship Weatherlight");
    let a = t.exile(P0, "Grizzly Bears");
    let b = t.exile(P0, "Hill Giant");
    // A card exiled some other way can't be chosen.
    t.exile(P0, "Llanowar Elves");
    link(&mut t, ship, a);
    link(&mut t, ship, b);
    t.lands(P0, "Wastes", 4);
    t.activate(P0, ship, 0, &[]).expect("activation");
    t.resolve_all();
    let returned = ["Grizzly Bears", "Hill Giant"]
        .iter()
        .filter(|n| t.in_hand(P0, n))
        .count();
    assert_eq!(returned, 1, "{}", t.dump_log());
    assert!(t.in_exile("Llanowar Elves"));
}

#[test]
fn myr_welder_has_the_activated_abilities_of_cards_it_exiled() {
    cr!("607.2a", "613.1f");
    ruling!("Myr Welder", "Myr Welder has only the activated abilities of cards it exiles");
    assert_supported("Myr Welder");
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Myr Welder");
    let icy = t.graveyard(P1, "Icy Manipulator");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let activated = |t: &TestGame| {
        t.obj(welder)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .count()
    };
    assert_eq!(activated(&t), 1);
    t.activate(P0, welder, 0, &[o(icy)]).expect("imprint");
    t.resolve_all();
    assert!(t.in_exile("Icy Manipulator"));
    t.g.recompute();
    assert_eq!(activated(&t), 2);
    // Use the gained "{1}, {T}: Tap target artifact, creature, or land."
    t.g.untap(welder);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, welder, 1, &[o(bears)]).expect("gained ability");
    t.resolve_all();
    assert!(t.obj(bears).tapped);
}

// ---------------------------------------------------------------------------
// History qualifiers
// ---------------------------------------------------------------------------

#[test]
fn no_rest_for_the_wicked_returns_only_creatures_that_died_this_turn() {
    cr!("400.7");
    ruling!("No Rest for the Wicked", "It doesn’t matter who controlled the creature cards");
    assert_supported("No Rest for the Wicked");
    let mut t = TestGame::new(2);
    let nrftw = t.battlefield(P0, "No Rest for the Wicked");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    // P0's creature controlled by P1 when it died still counts.
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.objects[giant.0 as usize].controller = P1;
    t.g.destroy(giant, None);
    // Discarded, not put there from the battlefield.
    let elves = t.hand(P0, "Llanowar Elves");
    t.g.discard(P0, elves, None);
    t.settle();
    t.activate(P0, nrftw, 0, &[]).expect("activation");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn shadow_of_the_grave_returns_cards_you_discarded_this_turn() {
    cr!("701.9a");
    ruling!("Shadow of the Grave", "returns to your hand all cards that you discarded for any reason");
    assert_supported("Shadow of the Grave");
    let mut t = TestGame::new(2);
    let giant = t.hand(P0, "Hill Giant");
    t.g.discard(P0, giant, None);
    // Put into the graveyard another way.
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let shadow = t.hand(P0, "Shadow of the Grave");
    t.cast(P0, shadow).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn restore_the_peace_returns_each_creature_that_dealt_damage_this_turn() {
    cr!("400.3");
    ruling!("Restore the Peace", "Only creatures on the battlefield will be returned");
    assert_supported("Restore the Peace");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).expect("ping");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 2);
    let peace = t.hand(P0, "Restore the Peace");
    t.cast(P0, peace).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Prodigal Sorcerer"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn tato_farmer_puts_a_milled_land_onto_the_battlefield_tapped() {
    cr!("701.17a");
    ruling!("Tato Farmer", "without using the word \"mill,\"");
    assert_compiles("Tato Farmer", "that was milled this turn");
    let mut t = TestGame::new(2);
    let farmer = t.battlefield(P0, "Tato Farmer");
    t.library_top(P1, "Mountain");
    let milled = t.g.mill(P1, 1);
    assert_eq!(milled.len(), 1);
    let mountain = t.g.current(milled[0]);
    // A land put into a graveyard without milling it isn't a legal target.
    let forest = t.graveyard(P1, "Forest");
    assert!(t.activate(P0, farmer, 0, &[o(forest)]).is_err() || !t.on_battlefield(forest));
    t.clear_answers();
    t.g.stack.clear();
    t.g.untap(farmer);
    t.activate(P0, farmer, 0, &[o(mountain)]).expect("activation");
    t.resolve_all();
    let m = t.named_on_battlefield("Mountain");
    assert_eq!(m.len(), 1);
    assert_eq!(t.obj(m[0]).controller, P0);
    assert!(t.obj(m[0]).tapped);
    assert!(t.in_graveyard(P1, "Forest"));
}

// ---------------------------------------------------------------------------
// Modifiers
// ---------------------------------------------------------------------------

#[test]
fn perennation_returns_a_permanent_card_with_keyword_counters() {
    cr!("122.6", "122.1b");
    assert_supported("Perennation");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 4);
    let p = t.hand(P0, "Perennation");
    t.cast(P0, p).target(bears).go();
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "hexproof"), 1);
    assert_eq!(t.counters(back[0], "indestructible"), 1);
    assert!(t.obj(back[0]).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn storm_of_souls_returns_creatures_as_1_1_spirits_with_flying() {
    cr!("611.2e");
    assert_supported("Storm of Souls");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 6);
    let storm = t.hand(P0, "Storm of Souls");
    t.cast(P0, storm).go();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(t.pt(giant[0]), (1, 1));
    assert!(t.obj(giant[0]).has_keyword(KeywordKind::Flying));
    assert!(t.obj(giant[0]).chars.has_subtype("Spirit"));
    assert!(t.obj(giant[0]).chars.has_subtype("Giant"));
    // Only your graveyard.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_exile("Storm of Souls"));
}

#[test]
fn torrent_elemental_returns_from_exile_tapped() {
    cr!("113.6m");
    assert_supported("Torrent Elemental");
    let mut t = TestGame::new(2);
    let te = t.exile(P0, "Torrent Elemental");
    t.lands(P0, "Swamp", 5);
    t.activate(P0, te, 0, &[]).expect("activation from exile");
    t.resolve_all();
    let e = t.named_on_battlefield("Torrent Elemental");
    assert_eq!(e.len(), 1);
    assert!(t.obj(e[0]).tapped);
}

#[test]
fn talon_gates_of_madara_is_put_onto_the_battlefield_from_your_hand() {
    cr!("113.6m");
    ruling!("Talon Gates of Madara", "If Talon Gates of Madara isn't still in your hand");
    assert_supported("Talon Gates of Madara");
    let mut t = TestGame::new(2);
    let gates = t.hand(P0, "Talon Gates of Madara");
    t.lands(P0, "Wastes", 4);
    // Its third activated ability (after two mana abilities).
    t.activate(P0, gates, 2, &[]).expect("activation from hand");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Talon Gates of Madara").len(), 1);
    // It can't be activated while it's on the battlefield.
    let gates = t.named_on_battlefield("Talon Gates of Madara")[0];
    t.lands(P0, "Wastes", 4);
    assert!(t.activate(P0, gates, 2, &[]).is_err());
}

// ---------------------------------------------------------------------------
// Multi-target returns
// ---------------------------------------------------------------------------

#[test]
fn pull_from_the_deep_returns_an_instant_and_a_sorcery() {
    cr!("601.2c");
    assert_supported("Pull from the Deep");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let div = t.graveyard(P0, "Divination");
    t.graveyard(P0, "Shock");
    t.lands(P0, "Island", 4);
    let pull = t.hand(P0, "Pull from the Deep");
    t.cast(P0, pull).target(bolt).target(div).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(t.in_hand(P0, "Divination"));
    assert!(t.in_graveyard(P0, "Shock"));
    assert!(t.in_exile("Pull from the Deep"));
}

#[test]
fn churning_eddy_returns_a_creature_and_a_land_to_their_owners_hands() {
    cr!("400.3");
    assert_supported("Churning Eddy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Island", 4);
    let eddy = t.hand(P0, "Churning Eddy");
    t.cast(P0, eddy).target(bears).target(forest).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Forest"));
}

// ---------------------------------------------------------------------------
// Owner-side moves
// ---------------------------------------------------------------------------

#[test]
fn hurkyls_recall_returns_artifacts_the_player_owns_whoever_controls_them() {
    cr!("400.3");
    ruling!("Hurkyl's Recall", "Retrieves all artifacts owned by the target player regardless of who controls them");
    assert_supported("Hurkyl's Recall");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    // P1's artifact controlled by P0.
    t.g.objects[thopter.0 as usize].controller = P0;
    t.battlefield(P1, "Ornithopter");
    // P0's own artifact stays.
    t.battlefield(P0, "Millstone");
    t.lands(P0, "Island", 2);
    let recall = t.hand(P0, "Hurkyl's Recall");
    t.cast(P0, recall).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 0);
    assert_eq!(
        t.g.player(P1)
            .hand
            .iter()
            .filter(|c| t.obj(**c).chars.name == "Ornithopter")
            .count(),
        2
    );
    assert_eq!(t.named_on_battlefield("Millstone").len(), 1);
}

#[test]
fn balthor_returns_each_players_black_and_red_creature_cards() {
    cr!("110.2a", "608.2e");
    assert_supported("Balthor the Defiled");
    let mut t = TestGame::new(2);
    let balthor = t.battlefield(P0, "Balthor the Defiled");
    t.graveyard(P0, "Vampire Nighthawk");
    t.graveyard(P0, "Goblin Piker");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, balthor, 0, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Vampire Nighthawk").len(), 1);
    assert_eq!(t.named_on_battlefield("Goblin Piker").len(), 1);
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(t.obj(giant[0]).controller, P1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn whelming_wave_spares_the_listed_creature_types() {
    cr!("400.3");
    assert_supported("Whelming Wave");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Lorthos, the Tidemaker");
    t.lands(P0, "Island", 4);
    let wave = t.hand(P0, "Whelming Wave");
    t.cast(P0, wave).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Lorthos, the Tidemaker").len(), 1);
}

#[test]
fn charmed_griffin_lets_each_other_player_put_an_artifact_or_enchantment_onto_the_battlefield() {
    cr!("110.2a");
    assert_supported("Charmed Griffin");
    let mut t = TestGame::new(2);
    let thopter = t.hand(P1, "Ornithopter");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[o(thopter)]);
    t.enter(P0, "Charmed Griffin");
    t.resolve_all();
    let o2 = t.named_on_battlefield("Ornithopter");
    assert_eq!(o2.len(), 1);
    assert_eq!(t.obj(o2[0]).controller, P1);
}

// ---------------------------------------------------------------------------
// Bounces of your own permanents and chosen cards
// ---------------------------------------------------------------------------

#[test]
fn yaroks_wavecrasher_returns_another_creature_you_control() {
    cr!("608.2c");
    ruling!("Yarok's Wavecrasher", "you simply don't return anything");
    assert_supported("Yarok's Wavecrasher");
    let mut t = TestGame::new(2);
    // With no other creature, nothing happens.
    t.enter(P0, "Yarok's Wavecrasher");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Yarok's Wavecrasher").len(), 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[o(bears)]);
    t.enter(P0, "Yarok's Wavecrasher");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Yarok's Wavecrasher").len(), 2);
}

#[test]
fn kazandu_stomper_returns_up_to_two_lands() {
    cr!("608.2c");
    assert_supported("Kazandu Stomper");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 3);
    t.answer_choose(P0, &[o(lands[0]), o(lands[1])]);
    t.enter(P0, "Kazandu Stomper");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn make_a_wish_returns_two_cards_at_random() {
    cr!("608.2c");
    ruling!("Make a Wish", "If you only have one card in your graveyard");
    assert_supported("Make a Wish");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 4);
    let wish = t.hand(P0, "Make a Wish");
    t.cast(P0, wish).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Make a Wish itself and one of the three remain.
    assert_eq!(t.graveyard_size(P0), 2);
    // One card: that one.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let wish = t.hand(P0, "Make a Wish");
    t.cast(P0, wish).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn grim_captains_call_returns_one_card_of_each_type_in_turn() {
    cr!("608.2c");
    ruling!("Grim Captain's Call", "you just continue to the next listed type");
    assert_supported("Grim Captain's Call");
    let mut t = TestGame::new(2);
    let pirate = t.graveyard(P0, "Fathom Fleet Captain");
    let vampire = t.graveyard(P0, "Vampire Nighthawk");
    let dino = t.graveyard(P0, "Ancient Brontodon");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[o(pirate)]);
    t.answer_choose(P0, &[o(vampire)]);
    t.answer_choose(P0, &[o(dino)]);
    t.lands(P0, "Swamp", 3);
    let call = t.hand(P0, "Grim Captain's Call");
    t.cast(P0, call).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Fathom Fleet Captain"));
    assert!(t.in_hand(P0, "Vampire Nighthawk"));
    assert!(t.in_hand(P0, "Ancient Brontodon"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn estrid_returns_non_aura_enchantments_then_auras() {
    cr!("303.4f");
    ruling!("Estrid, the Masked", "If an Aura card can't enchant anything, it remains in your graveyard");
    assert_compiles("Estrid, the Masked", "then do the same for Aura cards");
    let mut t = TestGame::new(2);
    let estrid = t.battlefield(P0, "Estrid, the Masked");
    t.g.objects[estrid.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    t.graveyard(P0, "Glorious Anthem");
    t.graveyard(P0, "Holy Strength");
    // No creature to enchant: the Aura stays.
    let i = t
        .obj(estrid)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("do the same"))
        .expect("-7 ability");
    t.activate(P0, estrid, i, &[]).expect("-7");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 1);
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn mausoleum_turnkey_lets_an_opponent_choose_the_target() {
    cr!("601.7");
    assert_supported("Mausoleum Turnkey");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P1, &[o(giant)]);
    t.enter(P0, "Mausoleum Turnkey");
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn pulse_of_the_fields_returns_to_hand_if_an_opponent_has_more_life() {
    cr!("608.2c");
    assert_supported("Pulse of the Fields");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 30;
    t.lands(P0, "Plains", 3);
    let pulse = t.hand(P0, "Pulse of the Fields");
    t.cast(P0, pulse).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert!(t.in_hand(P0, "Pulse of the Fields"));
    // Not if no opponent has more life.
    t.g.players[1].life = 25;
    t.lands(P0, "Plains", 3);
    let pulse = t.g.player(P0).hand[0];
    t.cast(P0, pulse).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    assert!(t.in_graveyard(P0, "Pulse of the Fields"));
}

#[test]
fn zone_move_texts_compile() {
    for (name, text) in [
        ("Crop Sigil", "return up to one target creature card and up to one target land card"),
        ("Reconstruct History", "up to one target planeswalker card from your graveyard"),
        ("Relive the Past", "they are 5/5 elemental creatures"),
        ("Gelatinous Cube", "put target creature card with mana value x exiled with"),
        ("Bane Alley Broker", "return a card exiled with"),
        ("Wall of Mourning", "put a card exiled with"),
        ("Ghost Vacuum", "each of them is a 1/1 spirit"),
        ("Parallax Wave", "each player returns to the battlefield all cards they own exiled with it"),
        ("Second Sunrise", "that were put there from the battlefield this turn"),
        ("Thrilling Encore", "all creature cards in all graveyards"),
        ("Soulquake", "all creature cards in graveyards to their owners' hands"),
        ("Gorex, the Tombshell", "choose a card at random exiled with"),
        ("Omenpath Journey", "choose a card at random exiled with"),
        ("Tasigur, the Golden Fang", "of an opponent's choice"),
    ] {
        assert_compiles(name, text);
    }
}

// ---------------------------------------------------------------------------
// Follow-ups to moves
// ---------------------------------------------------------------------------

#[test]
fn barrel_down_sokenzan_counts_the_mountains_returned_this_way() {
    cr!("608.2c");
    assert_supported("Barrel Down Sokenzan");
    let mut t = TestGame::new(2);
    let m = t.lands(P0, "Mountain", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    // Two Mountains returned: 4 damage.
    t.answer_choose(P0, &[o(m[0]), o(m[1])]);
    let spell = t.hand(P0, "Barrel Down Sokenzan");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.named_on_battlefield("Mountain").len(), 1);
    assert!(t.in_graveyard(P1, "Hill Giant"), "{}", t.dump_log());
}

#[test]
fn aberrant_return_puts_creatures_onto_the_battlefield_with_a_minus_counter() {
    cr!("122.6", "110.2a");
    assert_supported("Aberrant Return");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 6);
    let spell = t.hand(P0, "Aberrant Return");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj(g[0]).controller, P0);
    assert_eq!(t.counters(g[0], "-1/-1"), 1);
    assert_eq!(t.pt(g[0]), (2, 2));
}

#[test]
fn coiling_oracle_puts_a_land_onto_the_battlefield_and_anything_else_into_your_hand() {
    cr!("701.20a");
    assert_supported("Coiling Oracle");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.enter(P0, "Coiling Oracle");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    t.library_top(P0, "Grizzly Bears");
    t.enter(P0, "Coiling Oracle");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn venser_s_diffusion_returns_a_suspended_card() {
    cr!("702.62b");
    assert_supported("Venser's Diffusion");
    let mut t = TestGame::new(2);
    let bolt = t.exile(P1, "Rift Bolt");
    t.g.objects[bolt.0 as usize]
        .counters
        .insert("time".into(), 1);
    // An exiled card that isn't suspended can't be chosen.
    t.exile(P1, "Lightning Bolt");
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Venser's Diffusion");
    t.cast(P0, spell).target(bolt).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Rift Bolt"));
    // A nonland permanent.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Venser's Diffusion");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn keen_eyed_curator_grows_with_four_card_types_exiled_with_it() {
    cr!("607.2a", "205.2a");
    assert_supported("Keen-Eyed Curator");
    let mut t = TestGame::new(2);
    let curator = t.battlefield(P0, "Keen-Eyed Curator");
    let cards = [
        t.graveyard(P1, "Grizzly Bears"),
        t.graveyard(P1, "Forest"),
        t.graveyard(P1, "Lightning Bolt"),
    ];
    for c in cards {
        t.lands(P0, "Wastes", 1);
        t.activate(P0, curator, 0, &[o(c)]).expect("exile");
        t.resolve_all();
    }
    assert_eq!(t.pt(curator), (3, 3));
    let div = t.graveyard(P1, "Divination");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, curator, 0, &[o(div)]).expect("exile");
    t.resolve_all();
    assert_eq!(t.pt(curator), (7, 7));
    assert!(t.obj(curator).has_keyword(KeywordKind::Trample));
}

#[test]
fn scrap_mastery_swaps_each_players_artifacts_at_the_same_time() {
    cr!("608.2e", "110.2a");
    assert_supported("Scrap Mastery");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P1, "Memnite");
    t.battlefield(P0, "Millstone");
    t.battlefield(P1, "Mind Stone");
    // A creature that isn't an artifact stays.
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Scrap Mastery");
    t.cast(P0, spell).go();
    t.resolve_all();
    let thopter = t.named_on_battlefield("Ornithopter");
    let memnite = t.named_on_battlefield("Memnite");
    assert_eq!((thopter.len(), memnite.len()), (1, 1));
    assert_eq!(t.obj(thopter[0]).controller, P0);
    assert_eq!(t.obj(memnite[0]).controller, P1);
    assert!(t.in_graveyard(P0, "Millstone"));
    assert!(t.in_graveyard(P1, "Mind Stone"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn swift_warkite_returns_the_creature_at_the_next_end_step() {
    cr!("603.7");
    assert_supported("Swift Warkite");
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.enter(P0, "Swift Warkite");
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert!(t.obj(b[0]).has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn muse_vessel_lets_you_play_a_card_exiled_with_it() {
    cr!("607.2a");
    assert_supported("Muse Vessel");
    let mut t = TestGame::new(2);
    let vessel = t.battlefield(P0, "Muse Vessel");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, vessel, 0, &[Entity::Player(P1)]).expect("exile");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    let exiled = t.g.current(bolt);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[o(exiled)]);
    t.activate(P0, vessel, 1, &[]).expect("choose");
    t.resolve_all();
    // P0 may now cast P1's Lightning Bolt from exile.
    t.lands(P0, "Mountain", 1);
    t.cast(P0, exiled).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sink_into_stupor_returns_only_an_opponents_spell_or_nonland_permanent() {
    cr!("400.3", "115.1");
    assert_compiles(
        "Sink into Stupor // Soporific Springs",
        "return target spell or nonland permanent an opponent controls",
    );
    let mut t = TestGame::new(2);
    // Your own spell and permanent aren't legal targets.
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let own_bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, own_bolt).target(Entity::Player(P1)).go();
    t.lands(P0, "Island", 3);
    let sink = t.hand(P0, "Sink into Stupor // Soporific Springs");
    let spell = t.g.stack.last().copied().expect("bolt on the stack");
    assert!(t.cast(P0, sink).target(spell).try_go().is_err());
    t.clear_answers();
    t.resolve_all();
    // An opponent's spell.
    t.lands(P1, "Mountain", 1);
    let their_bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, their_bolt).target(Entity::Player(P0)).go();
    let spell = t.g.stack.last().copied().expect("bolt on the stack");
    let sink = t.g.player(P0).hand[0];
    t.cast(P0, sink).target(spell).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
}
