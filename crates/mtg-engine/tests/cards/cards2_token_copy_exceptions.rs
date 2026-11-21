//! Token copies and copy effects with exceptions (CR 707.9b, 707.9e, 306.5b, 205.1a):
//! "except it enters with an additional +1/+1 counter on it" (Littjara Mirrorlake),
//! "except it's 1/1 and has toxic 1" (Kinzu), "except it's not legendary and its starting
//! loyalty is 1" (Jace, Mirror Mage), "except it's a legendary Alien named Prisoner Zero"
//! (The Eleventh Hour), "except it's a Vehicle artifact with crew 3 and it loses all other
//! card types" (Imposter Mech), a token copy and a token with the card's power and
//! toughness (Soul Separator), a token copy entering attacking (Echoing Assault), a copy
//! until end of turn with exceptions (Cephalid Facetaker).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn token_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.obj_now(*o).is_token())
        .collect()
}

#[test]
fn littjara_mirrorlake_token_enters_with_an_additional_counter() {
    cr!("707.9e", "122.6");
    ruling!(
        "Littjara Mirrorlake",
        "The token copies exactly what is printed on the creature and nothing else"
    );
    supported("Littjara Mirrorlake");
    let mut t = TestGame::new(2);
    let lake = t.battlefield(P0, "Littjara Mirrorlake");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(obj(giant), "+1/+1", 2, None);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 3);
    t.activate(P0, lake, 1, &[obj(giant)]).unwrap();
    t.resolve_all();
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.counters(toks[0], "+1/+1"), 1);
    assert_eq!(t.pt(toks[0]), (4, 4));
}

#[test]
fn kinzu_token_copy_is_1_1_with_toxic() {
    cr!("707.9b", "702.164a");
    ruling!(
        "Kinzu of the Bleak Coven",
        "If something becomes a copy of the token, the copy also has base power and toughness 1/1 and has toxic 1."
    );
    supported("Kinzu of the Bleak Coven");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kinzu of the Bleak Coven");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(giant).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(t.obj_now(toks[0]).has_keyword(KeywordKind::Toxic));
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn jace_mirror_mage_token_is_not_legendary_with_starting_loyalty_1() {
    cr!("707.9b", "306.5b");
    ruling!(
        "Jace, Mirror Mage",
        "The token created by Jace's triggered ability doesn't have the legendary supertype and has a starting loyalty of 1 rather than 4."
    );
    supported("Jace, Mirror Mage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let j = t.hand(P0, "Jace, Mirror Mage");
    t.cast(P0, j).kicked(true).go();
    t.resolve_all();
    let jaces = t.named_on_battlefield("Jace, Mirror Mage");
    assert_eq!(jaces.len(), 2);
    let tok = jaces
        .iter()
        .copied()
        .find(|o| t.obj_now(*o).is_token())
        .unwrap();
    assert_eq!(t.counters(tok, "loyalty"), 1);
    assert!(!t.obj_now(tok).chars.is_legendary());
    let card_jace = jaces.iter().copied().find(|o| *o != tok).unwrap();
    assert_eq!(t.counters(card_jace, "loyalty"), 4);
}

#[test]
fn the_eleventh_hour_makes_prisoner_zero() {
    cr!("707.9b", "714.2b");
    ruling!(
        "The Eleventh Hour",
        "The token will still have any card types it normally has, such as creature or artifact, but it will lose any non-Alien subtypes it has"
    );
    supported("The Eleventh Hour");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let saga = t.battlefield(P0, "The Eleventh Hour");
    t.g.add_counters(obj(saga), "lore", 2, None);
    t.answer_targets(P0, &[obj(bears)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let pz = token_named(&t, "Prisoner Zero");
    assert_eq!(pz.len(), 1);
    let c = &t.obj_now(pz[0]).chars;
    assert!(c.is_legendary());
    assert!(c.has_subtype("Alien") && !c.has_subtype("Bear"));
    assert!(c.is(CardType::Creature));
    assert_eq!(t.pt(pz[0]), (2, 2));
}

#[test]
fn imposter_mech_is_a_vehicle_with_crew_3() {
    cr!("707.9b", "205.1a", "702.122a");
    ruling!(
        "Imposter Mech",
        "Imposter Mech does not copy any subtypes related to a card type that it lost."
    );
    supported("Imposter Mech");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(theirs)]);
    let mech = t.enter(P0, "Imposter Mech");
    t.resolve_all();
    let c = &t.obj_now(mech).chars;
    assert_eq!(c.name, "Hill Giant");
    assert!(c.is(CardType::Artifact) && !c.is(CardType::Creature));
    assert!(c.has_subtype("Vehicle") && !c.has_subtype("Giant"));
    assert!(t.obj_now(mech).has_keyword(KeywordKind::Crew));
}

#[test]
fn soul_separator_makes_a_1_1_spirit_copy_and_a_zombie_with_its_power() {
    cr!("707.9b", "111.3");
    ruling!(
        "Soul Separator",
        "The power and toughness of the Zombie token are set as it\u{2019}s created based on the card as it last existed in your graveyard."
    );
    supported("Soul Separator");
    let mut t = TestGame::new(2);
    let sep = t.battlefield(P0, "Soul Separator");
    let dead = t.graveyard(P0, "Craw Wurm");
    t.lands(P0, "Wastes", 5);
    t.activate(P0, sep, 0, &[obj(dead)]).unwrap();
    t.resolve_all();
    let spirit = token_named(&t, "Craw Wurm");
    assert_eq!(spirit.len(), 1);
    assert_eq!(t.pt(spirit[0]), (1, 1));
    assert!(t.obj_now(spirit[0]).chars.has_subtype("Spirit"));
    assert!(t.obj_now(spirit[0]).has_keyword(KeywordKind::Flying));
    let z = t.named_on_battlefield("Zombie Token");
    assert_eq!(z.len(), 1);
    assert_eq!(t.pt(z[0]), (6, 4));
}

#[test]
fn echoing_assault_token_copy_enters_attacking_that_player() {
    cr!("508.4", "707.9b");
    supported("Echoing Assault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoing Assault");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(t.obj_now(toks[0]).tapped);
    let combat = t.g.combat.as_ref().unwrap();
    assert_eq!(combat.attack_target(toks[0]), Some(Entity::Player(P1)));
}

#[test]
fn cephalid_facetaker_becomes_a_1_4_unblockable_copy_until_end_of_turn() {
    cr!("707.9b", "611.2a");
    supported("Cephalid Facetaker");
    let mut t = TestGame::new(2);
    let ceph = t.battlefield(P0, "Cephalid Facetaker");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wurm)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(ceph).chars.name, "Craw Wurm");
    assert_eq!(t.pt(ceph), (1, 4));
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(ceph).chars.name, "Cephalid Facetaker");
}

#[test]
fn shredder_attacks_each_other_opponent_with_nonlegendary_copies() {
    cr!("508.4", "707.9b", "603.7");
    supported("Shredder, Shadow Master");
    let mut t = TestGame::new(3);
    let sh = t.battlefield(P0, "Shredder, Shadow Master");
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(sh, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let toks = token_named(&t, "Shredder, Shadow Master");
    assert_eq!(toks.len(), 1);
    assert!(!t.obj_now(toks[0]).chars.is_legendary());
    assert!(t.obj_now(toks[0]).tapped);
    let combat = t.g.combat.as_ref().unwrap();
    assert_eq!(combat.attack_target(toks[0]), Some(Entity::Player(P2)));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(token_named(&t, "Shredder, Shadow Master").is_empty());
}

#[test]
fn mardu_siegebreaker_attacks_each_opponent_with_copies_of_the_exiled_card() {
    cr!("508.4", "707.2", "603.7");
    supported("Mardu Siegebreaker");
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let ms = t.enter(P0, "Mardu Siegebreaker");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(ms, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let toks = token_named(&t, "Hill Giant");
    assert_eq!(toks.len(), 2);
    let combat = t.g.combat.as_ref().unwrap();
    let mut defenders: Vec<Option<Entity>> =
        toks.iter().map(|o| combat.attack_target(*o)).collect();
    defenders.sort();
    assert_eq!(
        defenders,
        vec![Some(Entity::Player(P1)), Some(Entity::Player(P2))]
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(token_named(&t, "Hill Giant").is_empty());
}

#[test]
fn orvar_copies_one_of_the_targeted_permanents_you_control() {
    cr!("115.9b", "603.4", "707.2");
    supported("Orvar, the All-Form");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orvar, the All-Form");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let g = t.hand(P0, "Giant Growth");
    t.cast(P0, g).target(giant).go();
    t.resolve_all();
    assert_eq!(token_named(&t, "Hill Giant").len(), 1);
    // Targeting an opponent's creature: no copy.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let g = t.hand(P0, "Giant Growth");
    t.cast(P0, g).target(theirs).go();
    t.resolve_all();
    assert!(token_named(&t, "Grizzly Bears").is_empty());
    assert_eq!(token_named(&t, "Hill Giant").len(), 1);
}

#[test]
fn ultron_token_copy_of_a_noncreature_artifact_becomes_a_2_2_robot() {
    cr!("707.2", "205.1b");
    supported("Ultron, Artificial Malevolence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ultron, Artificial Malevolence");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.enter(P0, "Sol Ring");
    t.resolve_all();
    let toks = token_named(&t, "Sol Ring");
    assert_eq!(toks.len(), 1);
    let c = &t.obj_now(toks[0]).chars;
    assert!(c.is(CardType::Creature) && c.is(CardType::Artifact));
    assert!(c.has_subtype("Robot"));
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn mirror_match_blocks_each_attacker_with_a_copy_of_it() {
    cr!("509.4", "707.2", "603.7");
    ruling!(
        "Mirror Match",
        "The delayed triggered ability that exiles the tokens will trigger at end of combat no matter what happens to the original creature."
    );
    supported("Mirror Match");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Island", 6);
    let mm = t.hand(P0, "Mirror Match");
    t.set_step(P1, Step::PrecombatMain);
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![
            (bears, Entity::Player(P0)),
            (giant, Entity::Player(P0)),
        ]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.g.turn.priority = Some(P0);
    t.cast(P0, mm).go();
    t.resolve();
    assert_eq!(token_named(&t, "Grizzly Bears").len(), 1);
    assert_eq!(token_named(&t, "Ornithopter").len(), 1);
    t.advance_to(P1, Step::PostcombatMain);
    // The Bears traded with their copy; the Ornithopter copy survived combat and was
    // exiled at end of combat.
    assert!(!t.on_battlefield(bears) && t.on_battlefield(giant));
    assert_eq!(t.life(P0), 20);
    assert!(token_named(&t, "Grizzly Bears").is_empty());
    assert!(token_named(&t, "Ornithopter").is_empty());
}

#[test]
fn mirage_phalanx_token_copies_have_haste_and_lose_soulbond() {
    cr!("707.9b", "702.95a");
    supported("Mirage Phalanx");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ph = t.enter(P0, "Mirage Phalanx");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(
        mtg_engine::kw::soulbond::partner(&t.g, t.g.current(ph)),
        Some(t.g.current(bears))
    );
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    let phalanxes = t.named_on_battlefield("Mirage Phalanx");
    assert_eq!(phalanxes.len(), 2);
    let tok = phalanxes
        .iter()
        .copied()
        .find(|o| t.obj_now(*o).is_token())
        .unwrap();
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(tok).has_keyword(KeywordKind::Soulbond));
    assert!(t.obj_now(ph).has_keyword(KeywordKind::Soulbond));
    // Each paired creature has the ability: a Bears token too.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    t.advance_to_step(Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mirage Phalanx").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn benthic_anomaly_copies_one_chosen_creature_with_their_total_power() {
    cr!("707.9b", "707.9d", "603.2");
    ruling!(
        "Benthic Anomaly",
        "The token is an Eldrazi creature instead of its other types and subtypes and is colorless instead of retaining any of its colors."
    );
    supported("Benthic Anomaly");
    let mut t = TestGame::new(3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let angel = t.battlefield(P2, "Serra Angel");
    t.lands(P0, "Island", 7);
    let b = t.hand(P0, "Benthic Anomaly");
    t.cast(P0, b).go();
    // The creature each opponent controls, then the one to copy.
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer_choose(P0, &[Entity::Object(angel)]);
    t.answer_choose(P0, &[Entity::Object(angel)]);
    t.resolve_all();
    let angels = t.named_on_battlefield("Serra Angel");
    assert_eq!(angels.len(), 2);
    let tok = angels
        .iter()
        .copied()
        .find(|o| t.obj_now(*o).is_token())
        .unwrap();
    // 6/4 Wurm + 4/4 Angel.
    assert_eq!(t.pt(tok), (10, 8));
    let o = t.obj_now(tok);
    assert!(o.chars.colors.is_colorless());
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Eldrazi"));
    assert!(!o.chars.subtypes.iter().any(|s| s.as_str() == "Angel"));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert!(o.chars.card_types.contains(CardType::Creature));
}

#[test]
fn arna_kennerud_copies_each_nontoken_attachment_attached_to_the_attacker() {
    cr!("707.2", "303.4f", "301.5e", "701.10e");
    ruling!(
        "Arna Kennerüd, Skycaptain",
        "To double the number of each kind of counter on a permanent, put another counter on it for each counter it already has."
    );
    supported("Arna Kennerüd, Skycaptain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arna Kennerüd, Skycaptain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let eq = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(eq, Entity::Object(bears)));
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    let strengths = t.named_on_battlefield("Holy Strength");
    let splitters = t.named_on_battlefield("Bonesplitter");
    assert_eq!(strengths.len(), 2);
    assert_eq!(splitters.len(), 2);
    for o in strengths.iter().chain(splitters.iter()) {
        assert_eq!(t.obj_now(*o).attached_to, Some(Entity::Object(bears)));
    }
    // 2/2, two +1/+1 counters, Holy Strength twice (+1/+2 each), Bonesplitter twice.
    assert_eq!(t.pt(bears), (10, 8));
    assert_eq!(t.life(P1), 10);
}

#[test]
fn stangg_twin_gets_copies_of_stanggs_auras_and_equipment() {
    cr!("707.2", "303.4f", "301.5e", "603.7");
    supported("Stangg, Echo Warrior");
    let mut t = TestGame::new(2);
    let stangg = t.battlefield(P0, "Stangg, Echo Warrior");
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(stangg)));
    let eq = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(eq, Entity::Object(stangg)));
    t.attack(&[(stangg, Entity::Player(P1))], &[]);
    let twin = t.named_on_battlefield("Stangg Twin");
    assert_eq!(twin.len(), 1);
    let twin = twin[0];
    for name in ["Holy Strength", "Bonesplitter"] {
        let copies: Vec<_> = t
            .named_on_battlefield(name)
            .into_iter()
            .filter(|o| t.obj_now(*o).is_token())
            .collect();
        assert_eq!(copies.len(), 1, "{name}");
        assert_eq!(t.obj_now(copies[0]).attached_to, Some(Entity::Object(twin)));
    }
    assert_eq!(t.pt(twin), (6, 6));
    assert_eq!(t.life(P1), 8);
    // All tokens created this way are sacrificed at the beginning of the next end step.
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Stangg Twin").is_empty());
    assert_eq!(t.named_on_battlefield("Holy Strength").len(), 1);
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
}

#[test]
fn three_dog_copies_the_sacrificed_aura_onto_each_other_attacker() {
    cr!("603.12", "303.4f", "707.2");
    ruling!(
        "Three Dog, Galaxy News DJ",
        "Since Three Dog's reflexive triggered ability doesn't target, Aura tokens created by that ability will be attached to attacking creatures you control with shroud."
    );
    supported("Three Dog, Galaxy News DJ");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Three Dog, Galaxy News DJ");
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(dog)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Plains", 2);
    t.answer_yes(P0, true);
    t.attack(
        &[
            (dog, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
        &[],
    );
    let strengths = t.named_on_battlefield("Holy Strength");
    assert_eq!(strengths.len(), 2);
    assert!(strengths.iter().all(|o| t.obj_now(*o).is_token()));
    let on: Vec<_> = strengths
        .iter()
        .map(|o| t.obj_now(*o).attached_to)
        .collect();
    assert!(on.contains(&Some(Entity::Object(bears))));
    assert!(on.contains(&Some(Entity::Object(elves))));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    // Dog 1, Bears 3, Elves 2.
    assert_eq!(t.life(P1), 14);
}
