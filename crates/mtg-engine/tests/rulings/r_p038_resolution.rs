//! Rulings batch P038 — what happens during one resolution: triggers wait until the
//! spell or ability has finished resolving (CR 603.3, 608.2), state-based actions aren't
//! checked mid-resolution (CR 704.3), choices made as it resolves, replacement effects
//! changing what "this way" finds (CR 608.2c, 614).

use crate::r_p038_common::*;
use crate::r_s01_common::{stack_library, supported, watch};
use crate::r_s02_common::target_candidates;
use crate::r_s24_common::choose_creature_type;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn stitchers_apprentice_creates_then_sacrifices_without_a_gap() {
    cr!("608.2c", "117.3");
    ruling!(
        "Stitcher's Apprentice",
        "You create a token and sacrifice a creature all while the activated ability is resolving."
    );
    supported("Stitcher's Apprentice");
    let mut t = TestGame::new(2);
    let app = t.battlefield(P0, "Stitcher's Apprentice");
    mana_all(&mut t, P0, 1);
    // When the sacrifice is chosen, the new token is already there.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            g.permanents()
                .filter(|o| o.is_token() && o.chars.has_subtype("Homunculus"))
                .count()
        },
    );
    t.activate(P0, app, 0, &[]).unwrap();
    let from = t.asked().len();
    resolve_top_only(&mut t);
    assert_eq!(seen.lock().unwrap().first(), Some(&1));
    // No player got priority during the resolution.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.settle();
    assert_eq!(creature_count(&t, P0), 1, "one of the two was sacrificed");
}

#[test]
fn stitcher_geralf_finds_nothing_milled_if_rest_in_peace_exiles_it() {
    cr!("608.2c", "614.1a", "704.5f");
    ruling!(
        "Stitcher Geralf",
        "If the cards each player would put into their graveyard are exiled because of a replacement effect, those cards won't be in the graveyards to be exiled by the next part of the ability. In this case, the value of X will be 0."
    );
    supported("Stitcher Geralf");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    stack_library(&mut t, P0, &["Craw Wurm", "Craw Wurm", "Craw Wurm"]);
    stack_library(&mut t, P1, &["Craw Wurm", "Craw Wurm", "Craw Wurm"]);
    let geralf = t.battlefield(P0, "Stitcher Geralf");
    mana_all(&mut t, P0, 2);
    t.activate(P0, geralf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0) + t.graveyard_size(P1), 0);
    // The 0/0 Zombie died and ceased to exist.
    assert!(tokens_of(&t, P0, "Zombie").is_empty());
    assert_eq!(creature_count(&t, P0), 1);
}

#[test]
fn stitcher_geralf_totals_only_the_cards_its_exile_instruction_exiled() {
    cr!("608.2c", "608.2h");
    // (Regression: "the cards exiled this way" after a mill meant every milled card.)
    let mut t = TestGame::new(2);
    let mine = stack_library(&mut t, P0, &["Craw Wurm", "Grizzly Bears", "Craw Wurm"]);
    let theirs = stack_library(&mut t, P1, &["Craw Wurm", "Hill Giant", "Craw Wurm"]);
    let geralf = t.battlefield(P0, "Stitcher Geralf");
    mana_all(&mut t, P0, 2);
    t.activate(P0, geralf, 0, &[]).unwrap();
    t.answer_choose_moved(P0, &[obj(mine[1]), obj(theirs[1])]);
    t.resolve_all();
    let zombie = tokens_of(&t, P0, "Zombie");
    assert_eq!(zombie.len(), 1);
    assert_eq!(t.pt(zombie[0]), (5, 5), "Grizzly Bears and Hill Giant");
    assert_eq!(t.graveyard_size(P0) + t.graveyard_size(P1), 4);
}

#[test]
fn shamanic_revelation_gains_life_before_draw_triggers_go_on_the_stack() {
    cr!("603.3", "608.2c");
    ruling!(
        "Shamanic Revelation",
        "After you draw cards while Shamanic Revelation is resolving, nothing else can happen before you gain the appropriate amount of life."
    );
    supported("Psychosis Crawler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Psychosis Crawler");
    t.hand(P0, "Grizzly Bears"); // (the Crawler's toughness is the hand size)
    cast_only(&mut t, P0, "Shamanic Revelation", &[]);
    resolve_top_only(&mut t);
    assert_eq!(t.life(P0), 24, "one creature with power 4 or greater");
    assert_eq!(t.life(P1), 20, "the draw triggers haven't resolved");
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 2, "two draws, two triggers");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn aerial_assault_dies_triggers_wait_until_after_the_life_gain() {
    cr!("603.3", "608.2c");
    ruling!(
        "Aerial Assault",
        "Any abilities that trigger as the target creature is destroyed won't resolve until after you've gained life."
    );
    supported("Perilous Myr");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    let myr = t.battlefield(P1, "Perilous Myr");
    tap(&mut t, myr);
    cast_only(&mut t, P0, "Aerial Assault", &[obj(myr)]);
    resolve_top_only(&mut t);
    assert!(t.in_graveyard(P1, "Perilous Myr"));
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.stack_len(), 0);
    t.answer_targets(P1, &[pl(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn phyrexian_rebirths_horror_can_be_targeted_by_dies_triggers() {
    cr!("603.3", "603.3d");
    ruling!(
        "Phyrexian Rebirth",
        "Any triggered abilities that trigger because a creature was destroyed won't be put onto the stack until after Phyrexian Rebirth finishes resolving. This means the Horror creature token can be targeted by those abilities, if applicable."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Perilous Myr");
    t.battlefield(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Phyrexian Rebirth", &[]);
    resolve_top_only(&mut t);
    let horror = tokens_of(&t, P0, "Horror");
    assert_eq!(horror.len(), 1);
    assert_eq!(t.pt(horror[0]), (2, 2));
    assert_eq!(t.stack_len(), 0);
    let from = t.asked().len();
    t.answer_targets(P1, &[obj(horror[0])]);
    t.settle();
    assert!(target_candidates(&t, P1, from)[0].contains(&obj(horror[0])));
    t.resolve_all();
    assert!(tokens_of(&t, P0, "Horror").is_empty(), "2 damage killed it");
}

#[test]
fn fumigate_counts_creatures_destroyed_even_if_exiled_instead() {
    cr!("608.2c", "614.1a", "701.8a");
    ruling!(
        "Fumigate",
        "Creatures destroyed this way count toward the life gained even if they're put into a zone other than a graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    bears(&mut t, P0, 1);
    bears(&mut t, P1, 2);
    cast_resolve(&mut t, P0, "Fumigate", &[]);
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn aerial_assault_and_depose_illegal_target_versus_no_effect() {
    cr!("608.2b", "701.8b", "701.26a");
    ruling!(
        "Aerial Assault",
        "If the target creature is an illegal target by the time Aerial Assault tries to resolve, the spell doesn't resolve. You won't gain life. If the target is legal but is not destroyed (most likely because it has indestructible), you will gain life."
    );
    ruling!(
        "Depose // Deploy",
        "If the target creature is an illegal target by the time Depose tries to resolve, the spell doesn't resolve. You don't draw a card. If the target is legal but not tapped (most likely because it was already tapped), you do draw a card."
    );
    // Aerial Assault: the target untaps in response -> illegal, no life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    let bear = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bear);
    cast_only(&mut t, P0, "Aerial Assault", &[obj(bear)]);
    t.g.untap(bear);
    t.resolve_all();
    assert!(t.on_battlefield(bear));
    assert_eq!(t.life(P0), 20);
    // ... an indestructible target isn't destroyed, but the life is gained.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    let myr = t.battlefield(P1, "Darksteel Myr");
    tap(&mut t, myr);
    cast_resolve(&mut t, P0, "Aerial Assault", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 21);

    // Depose: the target leaves -> no card; an already tapped target -> a card.
    supported("Depose // Deploy");
    for (leaves, draws) in [(true, 0), (false, 1)] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        tap(&mut t, bear);
        mana_all(&mut t, P0, 2);
        let card = t.hand(P0, "Depose // Deploy");
        let hand = t.hand_size(P0);
        t.answer_targets(P0, &[obj(bear)]);
        t.cast(P0, card).method(CastMethod::Half(0)).go();
        if leaves {
            kill(&mut t, bear);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand - 1 + draws, "leaves={leaves}");
    }
}

#[test]
fn benevolent_offering_counts_spirits_that_will_die_to_sbas() {
    cr!("704.3", "704.5f", "608.2c");
    ruling!(
        "Benevolent Offering",
        "Because state-based actions aren't performed in the middle of a spell's resolution, you'll gain 2 life for each of those Spirit tokens."
    );
    supported("Benevolent Offering");
    supported("Night of Souls' Betrayal");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Night of Souls' Betrayal");
    t.battlefield(P0, "Hill Giant");
    mana_all(&mut t, P0, 4);
    let s = t.hand(P0, "Benevolent Offering");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 28, "the Giant and three 0/0 Spirits");
    assert_eq!(t.life(P1), 26, "three 0/0 Spirits");
    assert!(tokens_of(&t, P0, "Spirit").is_empty());
    assert!(tokens_of(&t, P1, "Spirit").is_empty());
}

#[test]
fn bounding_felidar_gains_life_even_if_no_counters_can_be_put() {
    cr!("608.2c", "122.1");
    ruling!(
        "Bounding Felidar",
        "In the rare case where +1/+1 counters can’t be put on one or more creatures you control, you still gain 1 life for each creature you control when Bounding Felidar’s first ability resolves."
    );
    supported("Bounding Felidar");
    supported("Solemnity");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Solemnity");
    let felidar = t.battlefield(P0, "Bounding Felidar");
    let b = bears(&mut t, P0, 2);
    t.answer_choose(P0, &[obj(b[0])]);
    t.activate(P0, felidar, 0, &[]).unwrap();
    t.resolve_all();
    crate::r_s01_common::attack_with(&mut t, &[(felidar, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(b[1], counters::PLUS1), 0);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn protect_the_negotiators_counts_its_own_soldier() {
    cr!("608.2c", "702.33d");
    ruling!(
        "Protect the Negotiators",
        "If it was kicked, the Soldier creature token will be on the battlefield when the amount of mana the target spell's controller needs to pay is determined."
    );
    supported("Protect the Negotiators");
    // P0 controls one creature; P1 has one untapped land left to pay with.
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let spell = cast_only(&mut t, P1, "Hill Giant", &[]);
        t.lands(P1, "Forest", 1);
        mana_all(&mut t, P0, 1);
        let ptn = t.hand(P0, "Protect the Negotiators");
        t.answer_yes(P1, true);
        t.cast(P0, ptn).kicked(kicked).target(obj(spell)).go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Hill Giant").is_empty(),
            kicked,
            "kicked={kicked}"
        );
    }
}

#[test]
fn choices_made_as_the_ability_resolves() {
    cr!("608.2d", "608.2h");
    ruling!(
        "Pol Jamaar, Illusionist",
        "The choice of creature type is made as Pol Jamaar's last ability resolves. Players can't take any actions between the time the choice is made and the time you draw cards."
    );
    ruling!(
        "Manifestation Sage",
        "X is number of cards in your hand as Manifestation Sage’s triggered ability resolves."
    );
    // Pol Jamaar (a Human Illusion Wizard): Wizard, with another Wizard arriving in
    // response, draws two.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    enter(&mut t, P0, "Pol Jamaar, Illusionist");
    assert!(crate::r_s35_common::creature_type_options(&t, P0, from).is_empty());
    add(&mut t, P0, "Prodigal Sorcerer");
    choose_creature_type(&mut t, P0, "Wizard");
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));

    // Manifestation Sage: two cards in hand when it triggers, three as it resolves.
    supported("Manifestation Sage");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    enter(&mut t, P0, "Manifestation Sage");
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    let fractal = tokens_of(&t, P0, "Fractal");
    assert_eq!(fractal.len(), 1);
    assert_eq!(t.counters(fractal[0], counters::PLUS1), 3);
}

#[test]
fn aphemia_can_exile_itself_if_it_died_in_response() {
    cr!("608.2c", "603.10");
    ruling!(
        "Aphemia, the Cacophony",
        "If Aphemia dies in response to its own triggered ability, it could be the card you exile."
    );
    supported("Aphemia, the Cacophony");
    let mut t = TestGame::new(2);
    let aphemia = t.battlefield(P0, "Aphemia, the Cacophony");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, aphemia);
    let card = t.g.current(aphemia);
    assert!(t.in_graveyard(P0, "Aphemia, the Cacophony"));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.resolve_all();
    assert!(t.in_exile("Aphemia, the Cacophony"));
    assert_eq!(tokens_of(&t, P0, "Zombie").len(), 1);
}
