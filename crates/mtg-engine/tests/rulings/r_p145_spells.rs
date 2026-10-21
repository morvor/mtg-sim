//! Rulings batch P145 — copies of spells (Nivix Guildmage, Jace Reawakened) and the
//! resolution of planeswalker and activated abilities (Tibalt, the Fiend-Blooded,
//! Traveling Minister, Kaito, Cunning Infiltrator, Hard Cover).

use crate::r_p145_common::*;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets, x_of};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0's Nivix Guildmage copies the spell `spell` (its second ability); the copy is on the
/// stack (keeping its targets). Returns the copy.
fn nivix_copies(t: &mut TestGame, spell: ObjectId) -> ObjectId {
    supported("Nivix Guildmage");
    let nivix = t.battlefield(P0, "Nivix Guildmage");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    keep_copy_targets(t, P0);
    t.activate(P0, nivix, 1, &[obj(spell)]).expect("copy");
    t.resolve();
    let copies = spell_copies(t);
    assert_eq!(copies.len(), 1);
    copies[0]
}

fn set_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), n);
}

#[test]
fn nivix_copies_any_instant_or_sorcery_spell_without_casting_it() {
    cr!("707.10", "707.10c", "601.2");
    ruling!(
        "Nivix Guildmage",
        "Nivix Guildmage’s second ability can target (and copy) any instant or sorcery spell you control, not just one with targets."
    );
    ruling!(
        "Nivix Guildmage",
        "When the second ability resolves, it creates a copy of a spell. That copy is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger."
    );
    // Dark Ritual: the copy adds {B}{B}{B} too. Windrider Wizard sees only the cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Windrider Wizard");
    let ritual = cast_new(&mut t, P0, "Dark Ritual", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 1);
    t.resolve();
    nivix_copies(&mut t, ritual);
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 0);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 6);
}

#[test]
fn nivix_copies_keep_x_modes_and_paid_costs() {
    cr!("707.10", "707.10c", "702.33d");
    ruling!(
        "Nivix Guildmage",
        "If the copied spell has an X whose value was determined as it was cast (like Sphinx’s Revelation does), the copy has the same value of X."
    );
    ruling!(
        "Nivix Guildmage",
        "If the copied spell is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. You can’t choose a different one."
    );
    ruling!(
        "Nivix Guildmage",
        "You can’t choose to pay any additional costs for the copy. However, effects based on any additional or alternative costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    // X: Fireball for 2 at P1; the copy's X is 2.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let fireball = t.hand(P0, "Fireball");
    let spell = t.cast(P0, fireball).x(2).target(Entity::Player(P1)).go();
    let copy = nivix_copies(&mut t, spell);
    assert_eq!(x_of(&t, copy), Some(2));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Modal: Izzet Charm's third mode (draw two, then discard two); the copy has it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let charm = t.hand(P0, "Izzet Charm");
    let spell = t.cast(P0, charm).modes(&[2]).go();
    let copy = nivix_copies(&mut t, spell);
    let mode = |id: ObjectId| t.g.obj(id).stack.as_ref().unwrap().chosen[0].mode;
    assert_eq!(mode(copy), Some(2));
    // Kicked Burst Lightning: the copy deals 4 too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Burst Lightning");
    let spell = t
        .cast(P0, bolt)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    let from = n_asked(&t);
    nivix_copies(&mut t, spell);
    assert!(!asked_of(&t, P0, from)
        .iter()
        .any(|d| matches!(d, mtg_engine::decision::Decision::OptionalCost { .. })));
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn jace_reawakened_copies_keep_or_change_targets_and_arent_cast() {
    cr!("707.10", "707.10c", "603.2");
    ruling!(
        "Jace Reawakened",
        "The copy created by Jace Reawakened's last ability will have the same targets as the spell it's copying unless you choose new ones."
    );
    ruling!(
        "Jace Reawakened",
        "The copy made by Jace Reawakened's last ability is created on the stack, so it's not \"cast.\""
    );
    supported("Jace Reawakened");
    for change in [false, true] {
        let mut t = TestGame::new(2);
        let jace = t.battlefield(P0, "Jace Reawakened");
        set_loyalty(&mut t, jace, 6);
        t.battlefield(P0, "Windrider Wizard");
        let bears = t.battlefield(P1, "Grizzly Bears");
        activate_containing(&mut t, P0, jace, "copy it").expect("−6");
        t.resolve_all();
        cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.settle();
        // Jace's trigger and Windrider Wizard's: the copy won't trigger the Wizard again.
        assert_eq!(stacked_triggers(&t), 2);
        assert_eq!(triggers_on_stack(&t, "you may draw a card"), 1);
        if change {
            change_copy_targets(&mut t, P0, &[Some(obj(bears))]);
        } else {
            keep_copy_targets(&mut t, P0);
        }
        t.resolve_all();
        assert_eq!(spell_copies(&t).len(), 0);
        assert_eq!(triggers_on_stack(&t, "you may draw a card"), 0);
        if change {
            assert_eq!(t.life(P1), 18);
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
        } else {
            assert_eq!(t.life(P1), 16);
            assert!(t.on_battlefield(bears));
        }
    }
}

#[test]
fn tibalt_counts_the_hand_on_resolution_and_untaps_all_creatures() {
    cr!("608.2h", "702.10b");
    ruling!(
        "Tibalt, the Fiend-Blooded",
        "The number of cards in the target player's hand is determined when Tibalt's second ability resolves."
    );
    ruling!(
        "Tibalt, the Fiend-Blooded",
        "When Tibalt's third ability resolves, all creatures will become untapped and gain haste, including ones you controlled before the ability resolved."
    );
    supported("Tibalt, the Fiend-Blooded");
    let mut t = TestGame::new(2);
    let tibalt = t.battlefield(P0, "Tibalt, the Fiend-Blooded");
    set_loyalty(&mut t, tibalt, 4);
    fill_hand(&mut t, P1, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, tibalt, "deals damage").expect("−4");
    // In response, P1 draws three more.
    t.g.draw_cards(P1, 3);
    t.resolve();
    assert_eq!(t.life(P1), 15);
    // −6: P0's own tapped, summoning-sick creature untaps and gains haste too.
    let mut t = TestGame::new(2);
    let tibalt = t.battlefield(P0, "Tibalt, the Fiend-Blooded");
    set_loyalty(&mut t, tibalt, 6);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.objects[mine.0 as usize].tapped = true;
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[theirs.0 as usize].tapped = true;
    activate_containing(&mut t, P0, tibalt, "Gain control").expect("−6");
    t.resolve_all();
    for c in [mine, theirs] {
        assert!(!t.obj_now(c).tapped);
        assert!(t.obj_now(c).has_keyword(KeywordKind::Haste));
        assert_eq!(t.obj_now(c).controller, P0);
    }
}

#[test]
fn abilities_with_illegal_targets_do_nothing() {
    cr!("608.2b", "606.4");
    ruling!(
        "Traveling Minister",
        "If the target of Traveling Minister's activated ability isn't legal as it tries to resolve, you won't gain 1 life."
    );
    ruling!(
        "Kaito, Cunning Infiltrator",
        "You don't have to choose a target for Kaito's first loyalty ability. However, if you do and that target is illegal when the ability tries to resolve, it won't resolve and none of its effects will happen."
    );
    supported("Traveling Minister");
    let mut t = TestGame::new(2);
    let minister = t.battlefield(P0, "Traveling Minister");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, minister, 0, &[obj(bears)])
        .expect("activate");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // With a legal target: +1/+0 and 1 life.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[minister.0 as usize].tapped = false;
    t.activate(P0, minister, 0, &[obj(bears)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.pt(bears), (3, 2));
    // Kaito: no target — he still loots; an illegal target — nothing, but the loyalty
    // counter stays.
    supported("Kaito, Cunning Infiltrator");
    let mut t = TestGame::new(2);
    let kaito = t.battlefield(P0, "Kaito, Cunning Infiltrator");
    let loyalty = t.counters(kaito, counters::LOYALTY);
    t.answer(
        P0,
        DecisionKind::Targets,
        mtg_engine::decision::Answer::Entities(vec![]),
    );
    activate_containing(&mut t, P0, kaito, "Draw a card").expect("+1");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 29);
    let mut t = TestGame::new(2);
    let kaito = t.battlefield(P0, "Kaito, Cunning Infiltrator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, kaito, "Draw a card").expect("+1");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.counters(kaito, counters::LOYALTY), loyalty + 1);
}

#[test]
fn hard_cover_granted_ability_resolves_without_its_source() {
    cr!("113.7a");
    ruling!(
        "Hard Cover",
        "Once the ability granted by Hard Cover has been activated, it will resolve even if Hard Cover leaves the battlefield before that ability resolves."
    );
    supported("Hard Cover");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Hard Cover", &[obj(bears)]);
    let cover = t.named_on_battlefield("Hard Cover")[0];
    t.library_top(P0, "Memnite");
    t.activate(P0, bears, 0, &[]).expect("granted ability");
    destroy(&mut t, cover);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.graveyard_size(P0), 2, "Hard Cover and the discarded card");
}

#[test]
fn urzas_tome_choice_comes_after_the_draw_and_discarding_is_allowed() {
    cr!("608.2c", "118.12");
    ruling!(
        "Urza's Tome",
        "You don’t choose whether to discard or exile a card from your graveyard until after you see the card you draw."
    );
    ruling!(
        "Urza's Tome",
        "You can choose to discard a card even if there’s a historic card in your graveyard you could exile."
    );
    supported("Urza's Tome");
    let mut t = TestGame::new(2);
    let tome = t.battlefield(P0, "Urza's Tome");
    t.lands(P0, "Wastes", 3);
    let memnite = t.graveyard(P0, "Memnite");
    t.library_top(P0, "Island");
    let seen = watch(
        &mut t,
        P0,
        |d| {
            matches!(
                d,
                mtg_engine::decision::Decision::YesNo { .. }
                    | mtg_engine::decision::Decision::OptionalCost { .. }
                    | mtg_engine::decision::Decision::ChooseOption { .. }
            )
        },
        |g| g.player(P0).hand.len(),
    );
    t.answer_yes(P0, false);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(false),
    );
    t.activate(P0, tome, 0, &[]).expect("activate");
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.first(), Some(&1), "asked after the draw");
    // Declined to exile Memnite: the Island was discarded.
    assert_eq!(t.zone(memnite), mtg_engine::object::Zone::Graveyard(P0));
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn sacrificed_creatures_power_is_last_known_information() {
    cr!("608.2h", "701.21a");
    ruling!(
        "Greater Good",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how many cards you draw."
    );
    ruling!(
        "Tom, Bert, and William",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how many cards you draw."
    );
    ruling!(
        "Greater Good",
        "If you don't have three cards in hand when instructed to discard three cards, you discard your hand."
    );
    // Greater Good: Grizzly Bears pumped to 4/4: draw four, discard three.
    supported("Greater Good");
    let mut t = TestGame::new(2);
    let gg = t.battlefield(P0, "Greater Good");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p057_common::pump(&mut t, bears, 2, 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, gg, 0, &[]).expect("sacrifice");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 26);
    assert_eq!(t.hand_size(P0), 1);
    // Llanowar Elves (1 power) with an empty hand: draw one, discard it.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let in_hand = t.g.player(P0).hand[0];
    discard(&mut t, P0, in_hand);
    t.answer_choose(P0, &[obj(elves)]);
    t.activate(P0, gg, 0, &[]).expect("sacrifice");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.library_size(P0), 25);
    // Tom, Bert, and William: the same with Elves pumped to 3/3.
    let name = "Tom, Bert, and William";
    supported(name);
    let mut t = TestGame::new(2);
    let trolls = t.battlefield(P0, name);
    t.lands(P0, "Wastes", 1);
    let elves = t.battlefield(P0, "Llanowar Elves");
    crate::r_p057_common::pump(&mut t, elves, 2, 2);
    t.answer_choose(P0, &[obj(elves)]);
    t.activate(P0, trolls, 0, &[]).expect("sacrifice");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 27);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn attunement_discards_all_when_short() {
    cr!("701.9b");
    ruling!(
        "Attunement",
        "If you have less than 4 cards, discard all you have."
    );
    supported("Attunement");
    let mut t = TestGame::new(2);
    let attunement = t.battlefield(P0, "Attunement");
    // Only two cards to draw: Attunement (returned as the cost) and two cards.
    t.g.players[P0.idx()].library.clear();
    t.library_top(P0, "Island");
    t.library_top(P0, "Swamp");
    t.activate(P0, attunement, 0, &[]).expect("activate");
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn monastery_siege_khans_draws_before_discarding() {
    cr!("504.1", "603.2", "608.2c");
    ruling!(
        "Monastery Siege",
        "The “Khans” ability happens after you draw the card you normally draw during your draw step. That card and the additional card will be in your hand when you have to discard a card."
    );
    supported("Monastery Siege");
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    enter(&mut t, P0, "Monastery Siege");
    t.resolve_all();
    t.library_top(P0, "Island");
    t.library_top(P0, "Swamp");
    let seen = watch(&mut t, P0, is_discard, |g| g.player(P0).hand.len());
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![2]);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn sphinxs_tutelage_repeats_only_for_shared_colors_of_nonland_cards() {
    cr!("105.1", "105.2c", "701.17a");
    ruling!(
        "Sphinx's Tutelage",
        "Colorless is not a color, so putting two colorless cards into a graveyard won’t cause the process to repeat."
    );
    ruling!(
        "Sphinx's Tutelage",
        "The process will keep repeating as long as both cards share a color and neither is a land card."
    );
    supported("Sphinx's Tutelage");
    let run = |top: &[&str]| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sphinx's Tutelage");
        stack_library(&mut t, P1, top);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        crate::r_p143_common::draw(&mut t, P0, 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        t.graveyard_size(P1)
    };
    assert_eq!(
        run(&["Ornithopter", "Memnite", "Shock", "Lightning Bolt"]),
        2
    );
    assert_eq!(
        run(&[
            "Shock",
            "Lightning Bolt",
            "Grizzly Bears",
            "Llanowar Elves",
            "Forest",
            "Llanowar Elves",
            "Llanowar Elves"
        ]),
        6
    );
}

#[test]
fn spymasters_vault_conniving_without_discarding_gives_no_counter() {
    cr!("701.50d", "701.50f");
    ruling!(
        "Spymaster's Vault",
        "If no cards are discarded, most likely because that player's hand is empty and an effect says they can't draw cards, the conniving creature does not receive any +1/+1 counters."
    );
    supported("Spymaster's Vault");
    supported("Maralen of the Mornsong");
    for cant_draw in [true, false] {
        let mut t = TestGame::new(2);
        let vault = t.battlefield(P0, "Spymaster's Vault");
        t.lands(P0, "Swamp", 1);
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_p108_common::a_creature_dies(&mut t, P1);
        if cant_draw {
            t.battlefield(P1, "Maralen of the Mornsong");
        }
        t.library_top(P0, "Memnite");
        t.activate(P0, vault, 1, &[obj(bears)]).expect("connive");
        t.resolve_all();
        assert_eq!(
            t.counters(bears, counters::PLUS1),
            u32::from(!cant_draw),
            "can't draw: {cant_draw}"
        );
    }
}

#[test]
fn spymasters_vault_checks_swamps_already_there() {
    cr!("614.1d", "614.12");
    ruling!(
        "Spymaster's Vault",
        "You must already control a Swamp as Spymaster's Vault enters the battlefield for it to enter untapped. If it enters the battlefield at the same time as a Swamp when you control no other Swamps, it will enter tapped."
    );
    let mut t = TestGame::new(2);
    let ids = enter_together(&mut t, P0, &["Spymaster's Vault", "Swamp"]);
    assert!(t.obj_now(ids[0]).tapped);
    t.battlefield(P0, "Swamp");
    let vault = enter(&mut t, P0, "Spymaster's Vault");
    assert!(!t.obj_now(vault).tapped);
}
