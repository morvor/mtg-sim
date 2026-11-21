//! Token creation grammar (`src/oracle/patterns/token_copy_grammar.rs`): more of the same
//! tokens "instead", tokens whose P/T a later sentence gives, choices between tokens,
//! counters on the tokens just created, "that many", Aura tokens created attached, named
//! tokens, delayed triggers about the tokens, tokens other players create, tokens created
//! attacking a given player or blocking, incubating repeatedly, and "tokens created with
//! ~".

use mtg_engine::object::{CastMethod, ObjKind};
use mtg_engine::types::CardType;
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
fn token_creation_wordings_compile() {
    assert_compiles(&[
        // "If ..., [instead] create N of those tokens [instead]."
        "From Under the Floorboards",
        "The Final Days",
        "Adipose Offspring",
        "Andúril, Flame of the West",
        "Runo Stromkirk // Krothuss, Lord of the Deep",
        // P/T given by the next sentence.
        "Ajani Goldmane",
        "Elephant Resurgence",
        "Ritual of the Returned",
        "Kalitas, Bloodchief of Ghet",
        "Daxos the Returned",
        "Gemini Engine",
        // Choices, several kinds, counters, "that many".
        "Tireless Provisioner",
        "Ant-Man's Army",
        "The Third Doctor",
        "Transmutation Font",
        "Farmer Cotton",
        "Fractal Anomaly",
        "Berta, Wise Extrapolator",
        "Edgar, Charmed Groom // Edgar Markov's Coffin",
        "Hellion Eruption",
        "Mutable Explorer",
        // Aura tokens, named tokens, delayed triggers.
        "Estrid, the Masked",
        "The Rani",
        "Stangg",
        "Mysterio, Master of Illusion",
        // Other players creating tokens.
        "Death by Dragons",
        "Gor Muldrak, Amphinologist",
        "Eiganjo Uprising",
        "Acorn Catapult",
        // Combat.
        "Soaring Lightbringer",
        "Combat Calligrapher",
        "Ellie, Brick Master",
        "Brimaz, King of Oreskos",
        "Flash Foliage",
        // Incubate.
        "Glissa, Herald of Predation",
        "Sunder the Gateway",
        "Glistening Dawn",
        "Progenitor Exarch",
        "Elesh Norn // The Argent Etchings",
        // Tokens created with ~.
        "Tombstone Stairwell",
        "Dual Nature",
        "Faerie Artisans",
    ]);
}

fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| {
            let o = t.g.obj(*o);
            o.kind == ObjKind::Token && o.controller == p && o.chars.has_subtype(subtype)
        })
        .collect()
}

fn activate_containing(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    use mtg_engine::ability::AbilityKind;
    t.g.recompute();
    let s = t.g.current(source);
    let uid = t
        .g
        .obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such activated ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, s, uid).expect("activation failed");
    t.g.flush_events();
}

#[test]
fn the_final_days_cast_from_a_graveyard_creates_x_tokens_instead() {
    cr!("608.2c", "111.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let fd = t.hand(P0, "The Final Days");
    t.cast(P0, fd).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Horror").len(), 2);
    // From the graveyard (flashback): X tokens, X the creature cards in your graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let fd = t.graveyard(P0, "The Final Days");
    t.cast(P0, fd).method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback)).go();
    t.resolve_all();
    let horrors = tokens(&t, P0, "Horror");
    assert_eq!(horrors.len(), 3, "{}", t.dump_log());
    assert!(horrors.iter().all(|h| t.g.obj(*h).tapped));
}

#[test]
fn ajani_goldmanes_avatar_has_power_and_toughness_equal_to_your_life() {
    cr!("111.3", "604.3");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    t.g.add_counters(Entity::Object(ajani), "loyalty", 6, None);
    activate_containing(&mut t, P0, ajani, "Avatar");
    t.resolve_all();
    let avatar = tokens(&t, P0, "Avatar");
    assert_eq!(avatar.len(), 1);
    assert_eq!(t.pt(avatar[0]), (20, 20));
    t.g.lose_life(P0, 5);
    t.g.recompute();
    assert_eq!(t.pt(avatar[0]), (15, 15));
}

#[test]
fn ritual_of_the_returned_zombie_copies_the_exiled_cards_power_and_toughness() {
    cr!("111.3", "608.2h");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let giant = t.graveyard(P0, "Hill Giant");
    let r = t.hand(P0, "Ritual of the Returned");
    t.cast(P0, r).target(giant).go();
    t.resolve_all();
    let z = tokens(&t, P0, "Zombie");
    assert_eq!(z.len(), 1);
    assert_eq!(t.pt(z[0]), (3, 3));
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn elephant_resurgence_gives_each_player_an_elephant_counting_their_graveyard() {
    cr!("111.2", "111.3", "604.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P1, "Grizzly Bears");
    let er = t.hand(P0, "Elephant Resurgence");
    t.cast(P0, er).go();
    t.resolve_all();
    let mine = tokens(&t, P0, "Elephant");
    let theirs = tokens(&t, P1, "Elephant");
    assert_eq!((mine.len(), theirs.len()), (1, 1));
    // Elephant Resurgence itself is in P0's graveyard now (not a creature card).
    assert_eq!(t.pt(mine[0]), (2, 2));
    assert_eq!(t.pt(theirs[0]), (1, 1));
}

#[test]
fn tireless_provisioner_creates_the_token_its_controller_chooses() {
    cr!("111.10a", "111.10b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tireless Provisioner");
    let forest = t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Treasure").len(), 1);
    assert_eq!(tokens(&t, P0, "Food").len(), 0);
}

#[test]
fn transmutation_font_creates_a_blood_clue_or_food_token() {
    cr!("111.10b", "111.10f", "111.10g");
    let mut t = TestGame::new(2);
    let font = t.battlefield(P0, "Transmutation Font");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    activate_containing(&mut t, P0, font, "your choice");
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Food").len(), 1);
}

#[test]
fn mutable_explorer_creates_a_tapped_mutavault_token() {
    cr!("111.11");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let me = t.hand(P0, "Mutable Explorer");
    t.cast(P0, me).go();
    t.resolve_all();
    let m = t.named_on_battlefield("Mutavault");
    assert_eq!(m.len(), 1);
    let o = t.obj_now(m[0]);
    assert!(o.kind == ObjKind::Token && o.tapped && o.chars.is(CardType::Land));
}

#[test]
fn farmer_cotton_creates_x_halflings_and_x_food() {
    cr!("107.3m", "111.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 2);
    let fc = t.hand(P0, "Farmer Cotton");
    t.cast(P0, fc).x(2).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Halfling").len(), 2);
    assert_eq!(tokens(&t, P0, "Food").len(), 2);
}

#[test]
fn fractal_anomaly_puts_counters_on_the_fractal_it_creates() {
    cr!("111.2", "122.1");
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 2);
    t.lands(P0, "Island", 1);
    let fa = t.hand(P0, "Fractal Anomaly");
    t.cast(P0, fa).go();
    t.resolve_all();
    let f = tokens(&t, P0, "Fractal");
    assert_eq!(f.len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(f[0], "+1/+1"), 2);
    assert_eq!(t.pt(f[0]), (2, 2));
}

#[test]
fn hellion_eruption_creates_a_hellion_for_each_creature_sacrificed() {
    cr!("701.21a", "111.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Llanowar Elves");
    let he = t.hand(P0, "Hellion Eruption");
    t.cast(P0, he).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(tokens(&t, P0, "Hellion").len(), 3);
}

#[test]
fn edgar_markovs_coffin_creates_a_vampire_and_puts_a_bloodline_counter_on_itself() {
    cr!("111.2", "122.1");
    let mut t = TestGame::new(2);
    let edgar = t.battlefield(P0, "Edgar, Charmed Groom // Edgar Markov's Coffin");
    mtg_engine::dfc::transform(&mut t.g, edgar);
    let coffin = t.g.current(edgar);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Vampire").len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(coffin, "bloodline"), 1);
}

#[test]
fn estrid_creates_a_mask_aura_token_attached_to_another_permanent() {
    cr!("303.4f", "702.5a", "111.3");
    let mut t = TestGame::new(2);
    let estrid = t.battlefield(P0, "Estrid, the Masked");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, estrid, "Mask");
    t.resolve_all();
    let mask = t.named_on_battlefield("Mask");
    assert_eq!(mask.len(), 1, "{}", t.dump_log());
    let m = t.obj_now(mask[0]);
    assert_eq!(m.attached_to, Some(Entity::Object(bears)));
    assert!(m.chars.has_subtype("Aura"));
}

#[test]
fn the_rani_mark_makes_the_enchanted_creature_bigger_and_goaded() {
    cr!("303.4f", "701.15a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "The Rani");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mark of the Rani").len(), 1);
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.g.goaders(bears), vec![P0]);
}

#[test]
fn stangg_and_stangg_twin_leave_together() {
    cr!("603.7a", "603.7c", "111.4");
    // Stangg leaves: the twin is exiled.
    let mut t = TestGame::new(2);
    let stangg = t.enter(P0, "Stangg");
    t.resolve_all();
    let twin = t.named_on_battlefield("Stangg Twin");
    assert_eq!(twin.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(twin[0]).chars.is_legendary());
    t.g.destroy(stangg, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Stangg Twin").is_empty());
    // The twin leaves: Stangg is sacrificed.
    let mut t = TestGame::new(2);
    let stangg = t.enter(P0, "Stangg");
    t.resolve_all();
    let twin = t.named_on_battlefield("Stangg Twin")[0];
    t.g.destroy(twin, None);
    t.resolve_all();
    assert!(!t.on_battlefield(stangg));
    assert!(t.in_graveyard(P0, "Stangg"));
}

#[test]
fn mysterio_exiles_its_illusions_when_it_leaves() {
    cr!("603.7a", "603.7c");
    let mut t = TestGame::new(2);
    let m = t.enter(P0, "Mysterio, Master of Illusion");
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Illusion").len(), 1);
    t.g.destroy(m, None);
    t.resolve_all();
    assert!(tokens(&t, P0, "Illusion").is_empty());
}

#[test]
fn death_by_dragons_gives_a_dragon_to_each_player_but_the_target() {
    cr!("111.2");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 6);
    let d = t.hand(P0, "Death by Dragons");
    t.cast(P0, d).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Dragon").len(), 1);
    assert_eq!(tokens(&t, P1, "Dragon").len(), 0);
    assert_eq!(tokens(&t, P2, "Dragon").len(), 1);
}

#[test]
fn gor_muldrak_gives_a_salamander_to_each_player_with_the_fewest_creatures() {
    cr!("111.2");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gor Muldrak, Amphinologist");
    t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // P2 controls no creatures: the fewest.
    assert_eq!(tokens(&t, P2, "Salamander").len(), 1, "{}", t.dump_log());
    assert_eq!(tokens(&t, P1, "Salamander").len(), 0);
    assert_eq!(tokens(&t, P0, "Salamander").len(), 0);
}

#[test]
fn eiganjo_uprising_gives_opponents_one_fewer_samurai() {
    cr!("107.1b", "111.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 3);
    let e = t.hand(P0, "Eiganjo Uprising");
    t.cast(P0, e).x(3).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Samurai").len(), 3);
    assert_eq!(tokens(&t, P1, "Samurai").len(), 2);
}

#[test]
fn acorn_catapult_gives_the_player_or_permanents_controller_a_squirrel() {
    cr!("111.2");
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Acorn Catapult");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, cat, "Squirrel");
    t.resolve_all();
    assert_eq!(tokens(&t, P1, "Squirrel").len(), 1);
    assert_eq!(tokens(&t, P0, "Squirrel").len(), 0);
}

#[test]
fn soaring_lightbringer_glimmer_attacks_the_player_you_attacked() {
    cr!("508.4", "508.4b");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Soaring Lightbringer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P2))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let g = tokens(&t, P0, "Glimmer");
    assert_eq!(g.len(), 1, "{}", t.dump_log());
    assert!(t.g.obj(g[0]).tapped);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(g[0]), Some(Entity::Player(P2)));
}

#[test]
fn combat_calligrapher_gives_the_attacking_player_an_inkling_attacking_that_opponent() {
    cr!("508.4", "111.2");
    let mut t = TestGame::new(3);
    t.battlefield(P2, "Combat Calligrapher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let ink = tokens(&t, P0, "Inkling");
    assert_eq!(ink.len(), 1, "{}", t.dump_log());
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(ink[0]), Some(Entity::Player(P1)));
}

#[test]
fn brimaz_blocking_creates_a_cat_blocking_that_creature() {
    cr!("509.4", "509.4a");
    let mut t = TestGame::new(2);
    let brimaz = t.battlefield(P0, "Brimaz, King of Oreskos");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(brimaz, giant)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.resolve_all();
    let cats = tokens(&t, P0, "Cat");
    assert_eq!(cats.len(), 1, "{}", t.dump_log());
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.blockers_of(giant).contains(&cats[0]));
}

#[test]
fn flash_foliage_saproling_blocks_the_target_attacker() {
    cr!("509.4", "509.4a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 3);
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    let ff = t.hand(P0, "Flash Foliage");
    t.cast(P0, ff).target(giant).go();
    t.resolve();
    let s = tokens(&t, P0, "Saproling");
    assert_eq!(s.len(), 1, "{}", t.dump_log());
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.blockers_of(giant).contains(&s[0]));
}

#[test]
fn glistening_dawn_incubates_twice() {
    cr!("701.53a", "111.10i");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let gd = t.hand(P0, "Glistening Dawn");
    t.cast(P0, gd).go();
    t.resolve_all();
    let inc = tokens(&t, P0, "Incubator");
    assert_eq!(inc.len(), 2);
    for i in inc {
        assert_eq!(t.counters(i, "+1/+1"), 4);
    }
}

#[test]
fn sunder_the_gateway_incubates_then_transforms_an_incubator() {
    cr!("701.53a", "701.27a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Sunder the Gateway");
    t.cast(P0, s).modes(&[1]).go();
    t.resolve_all();
    let p = tokens(&t, P0, "Phyrexian");
    assert_eq!(p.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(p[0]), (2, 2));
}

#[test]
fn tombstone_stairwell_destroys_the_tokens_it_created() {
    cr!("607.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tombstone Stairwell");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    // P1's upkeep (no cumulative upkeep for P0's enchantment): each player creates a
    // Tombspawn for each creature card in their graveyard.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Zombie").len(), 1, "{}", t.dump_log());
    assert_eq!(tokens(&t, P1, "Zombie").len(), 2);
    assert_eq!(t.named_on_battlefield("Tombspawn").len(), 3);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Tombspawn").is_empty(), "{}", t.dump_log());
}
