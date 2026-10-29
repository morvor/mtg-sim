# Decisions and policies

How the engine fills in choices that the rules leave to a player when the player's agent gives
no answer (`Answer::Default`) or an invalid one, and what agents can see and do. One entry per
kind of decision. Every `game.ask` caller validates the answer and falls back to these
defaults; an answer of the wrong shape counts as `Default`. Format and kinds:
`docs/agents/items/interpretation-register.md`; index: `docs/INTERPRETATIONS.md`.

## default-priority-pass: With priority, a default or invalid answer passes; repeated illegal actions pass too
- Kind: policy
- Status: decided
- Rules: 117.3d "If a player has priority and chooses not to take any actions, that player passes. ..." / 733.1 "If a player takes an illegal action or starts to take an action but can’t legally complete it, the entire action is reversed and any payments already made are canceled. ..." / 733.2 "When reversing illegal spells and abilities, the player who had priority retains it and may take another action or pass. ..."
- Question: What does a player with priority do when their agent gives no action, or keeps proposing actions that turn out to be illegal?
- Decision: `Answer::Default` or an answer that isn't an action passes priority. An action that turns out to be illegal is reversed and the player keeps priority (733.1, 733.2); after three illegal attempts in a row, the player passes. During a shortcut, a choice the player may not make is replaced by the first one they may.
- Alternatives: Re-ask indefinitely after illegal actions (733.2 read strictly): an agent that always proposes the same illegal action would stall the game.
- Why: Passing is the choice "not to take any actions" of 117.3d; the attempt limit keeps games finite.
- Affects: every priority decision.
- Code: `crates/mtg-engine/src/shortcuts.rs` (`priority_decision`); `crates/mtg-engine/src/turn.rs` (`take_action`, `MAX_ILLEGAL_ATTEMPTS`).
- Tests: `tests/cr/interp_decision_defaults.rs` (`a_default_priority_answer_passes`, `an_invalid_answer_is_treated_as_the_default`); `tests/cr/r733_002_repeated_illegal_actions.rs` (`a_player_who_keeps_attempting_illegal_actions_passes_priority`).

## default-mulligan-keep: A default mulligan decision keeps; the default bottom cards are the first ones offered
- Kind: policy
- Status: decided
- Rules: 103.5 "... A player who is dissatisfied with their initial hand may take a mulligan. ... then puts a number of those cards equal to the number of times that player has taken a mulligan on the bottom of their library in any order. Once a player chooses not to take a mulligan, the remaining cards become that player’s opening hand, ..."
- Question: Keep or mulligan, and which cards go to the bottom, when the agent doesn't say?
- Decision: Keep. When cards must go to the bottom, the first `n` cards of the hand as offered (hand order) go, in that order.
- Alternatives: Mulligan by a heuristic (no lands, all lands); choose the bottom cards by a heuristic. Both are strategy, which belongs in agents.
- Why: Keeping is the choice that ends the procedure; the engine makes no strategic choices.
- Affects: every game's opening hands.
- Code: `crates/mtg-engine/src/mulligan.rs` (the mulligan loop, `PutOnBottom`).
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_mulligan_answers_keep_and_bottom_the_first_cards`).

## default-yes-no: A default yes/no answer does what "you may" offers but doesn't pay optional costs
- Kind: policy
- Status: decided
- Rules: 603.5 "Some triggered abilities’ effects are optional (they contain “may,” as in “At the beginning of your upkeep, you may draw a card”). ... The choice is made when the ability resolves. ..." / 118.12 "... “[A player] may [do something]. If [that player] [does, doesn’t, or can’t], [effect].” The action [do something] is a cost, paid when the spell or ability resolves. ..."
- Question: What does a player choose for a "you may" or "unless [a player] pays" question when the agent doesn't say?
- Decision: Each call site sets the default, following one rule: yes for doing what an optional instruction or ability offers at no further cost ("you may draw", "you may search", optional replacement effects, casting a card for free, revealing a miracle); no for paying an optional cost or accepting a drawback ("unless that player pays {1}", "pay N life?", exert, tribute, intentional draws, choosing new targets for a copy).
- Alternatives: Always no: every "may" left undone. Always yes: agents would pay every tax and life cost.
- Why: The defaults stand in for an uninterested player: take free value, don't spend resources.
- Affects: every yes/no decision (about 50 call sites).
- Code: `crates/mtg-engine/src/game.rs` (`ask_yes_no`, its `default` argument at each call site); `crates/mtg-engine/src/resolve.rs` (`Effect::May`: yes; `Effect::PayOptional`: no).
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_yes_no_does_what_you_may_but_doesnt_pay_optional_costs`).

## default-first-listed-choice: Default choices take the first listed options: objects, options, targets, modes, casting method, replacement effect
- Kind: policy
- Status: decided
- Rules: 601.2b "If the spell is modal, the player announces the mode choice (see rule 700.2). ... If the spell has alternative or additional costs that will be paid as it’s being cast ..., the player announces their intentions to pay any or all of those costs ..." / 601.2c "The player announces their choice of an appropriate object or player for each target the spell requires. ..." / 700.2a "The controller of a modal spell or activated ability chooses the mode(s) as part of casting that spell or activating that ability. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can’t be chosen. ..." / 616.1 "If two or more replacement and/or prevention effects are attempting to modify the way an event affects an object or player, the affected object’s controller (or its owner if it has no controller) or the affected player chooses one to apply, ..."
- Question: Which objects, option, targets, modes, casting method or replacement effect does a player choose when the agent doesn't say?
- Decision: The first legal ones in the order the engine lists them: `ChooseEntities` takes the first `min` candidates; `ChooseOption` takes option 0; `ChooseTargets` takes the first `min` candidates (at least one when targets are required); `ChooseModes` takes the first available modes up to the minimum (at least one, skipping modes that can't be combined; when the same mode may be chosen more than once, the available modes again, in order, until the minimum is reached: Eldrazi Confluence with no nonland permanent to target takes its token mode three times. This only fills a gap: before, such a default came out short of the minimum and the cast failed, although the player could legally cast the spell); `ChooseCastingMethod` takes the first way to cast that could be begun; `ChooseReplacement` applies the first listed effect. A few choices have defaults of their own, documented where they're made: costs paid with permanents or cards (`default-cost-helpers`); friend or foe (the controller and their teammates are friends, `kw/friend_or_foe.rs`); a party (a largest one, `game_terms.rs`); "search for any number of cards with that name" (all of them, `kw/search_same_name.rs`); a coin flip one may keep or reflip (keep a winning flip, `dice.rs`); "you may choose not to untap" while a "remains tapped" effect applies (stay tapped, `untap_choice.rs`); the starting player's chooser (their own seat or team, `start.rs`).
- Alternatives: Random legal choices (games would vary with the seed rather than with the agent), or heuristics (strategy, which belongs in agents).
- Why: Deterministic, always legal, and with "the minimum" the least commitment the rules allow.
- Affects: every such decision.
- Code: `crates/mtg-engine/src/game.rs` (`ask_entities`, `ask_option`); `crates/mtg-engine/src/stack.rs` (modes, targets); `crates/mtg-engine/src/casting.rs` (`ChooseCastingMethod`); `crates/mtg-engine/src/replacement.rs` (`ChooseReplacement`).
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_choices_take_the_first_listed_candidates`, `default_targets_are_the_first_legal_ones`, `default_modes_are_the_first_available_ones`, `the_default_way_to_cast_is_the_first_that_can_be_begun`, `the_default_replacement_effect_is_the_first_listed`); `tests/cr/r700_002a_modal_castability.rs` (`default_modes_repeat_a_mode_that_may_be_chosen_more_than_once`, `fiery_confluence_default_modes_repeat_the_available_ones`).

## default-cost-helpers: Costs paid with permanents or cards (convoke, delve, improvise, crew, ...) use them only as needed by default
- Kind: policy
- Status: decided
- Rules: 702.51a "Convoke is a static ability that functions while the spell with convoke is on the stack. “Convoke” means “For each colored mana in this spell’s total cost, you may tap an untapped creature of that color you control rather than pay that mana. For each generic mana in this spell’s total cost, you may tap an untapped creature you control rather than pay that mana.”" / 702.66a "Delve is a static ability that functions while the spell with delve is on the stack. “Delve” means “For each generic mana in this spell’s total cost, you may exile a card from your graveyard rather than pay that mana.”" / 702.126a "Improvise is a static ability that functions while the spell with improvise is on the stack. “Improvise” means “For each generic mana in this spell’s total cost, you may tap an untapped artifact you control rather than pay that mana.”"
- Question: Which creatures, artifacts or graveyard cards does a player use for convoke, improvise, delve, crew ("tap any number of creatures with total power N or greater"), emerge, offering, harmonize, waterbending or assist when the agent doesn't say?
- Decision: As little as possible: none if the mana can be paid otherwise; else the fewest that make the rest payable (convoke and improvise prefer creatures and artifacts without mana abilities; delve takes the cards at the bottom of the graveyard; crew taps the most powerful creatures until the total is reached; emerge, offering and harmonize use the permanent that reduces the cost most; assist asks a teammate who could pay, who pays as much as they can, and an opponent pays nothing).
- Alternatives: Use the alternative resources first (keep mana open): strategy.
- Why: The least commitment that pays the cost, like `automatic-mana-payment`.
- Affects: spells and abilities with those keywords.
- Code: `crates/mtg-engine/src/kw/convoke.rs`, `kw/delve.rs`, `kw/improvise.rs`, `kw/crew.rs` (`pay_total_power`), `kw/emerge.rs`, `kw/offering.rs`, `kw/harmonize.rs`, `kw/assist.rs`, `kwa/bending.rs` (each `default_choice` or equivalent).
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_convoke_taps_creatures_only_for_what_mana_cant_pay`).

## default-optional-costs-declined: Optional additional costs aren't paid by default
- Kind: policy
- Status: decided
- Rules: 601.2b (see `default-first-listed-choice`) / 702.33a "Kicker is a static ability that functions while the spell with kicker is on the stack. “Kicker [cost]” means “You may pay an additional [cost] as you cast this spell.” ..."
- Question: Does a player pay kicker, buyback, multikicker, gift or other optional additional costs when the agent doesn't say?
- Decision: No (multikicker: zero times).
- Alternatives: Pay whenever affordable: spends resources the player may want for something else.
- Why: Same rule as `default-yes-no`: don't spend resources by default.
- Affects: every optional additional cost.
- Code: `crates/mtg-engine/src/casting.rs` (`OptionalCost`); `crates/mtg-engine/src/kw/offered_costs.rs`, `crates/mtg-engine/src/kw/gift.rs`, `crates/mtg-engine/src/splice.rs`, `crates/mtg-engine/src/cost_choices.rs`.
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_optional_costs_are_declined_and_the_first_castable_way_is_used`).

## default-numbers: A default X for a spell is the greatest affordable; other numbers default to the least allowed
- Kind: policy
- Status: decided
- Rules: 107.3 "Many objects use the letter X as a placeholder for a number that needs to be determined. Some objects have abilities that define the value of X; the rest let their controller choose the value of X." / 601.2b "... If the spell has a variable cost that will be paid as it’s being cast (such as an {X} in its mana cost; see rule 107.3), the player announces the value of that variable. ..."
- Question: Which number does a player choose when the agent doesn't say?
- Decision: X in a spell's cost: the greatest legal value whose total cost could be paid. Every other number (X of an activated ability, "choose a number", join forces, morph and other X costs): the least allowed, usually 0 (suspend's X: the least payable value of at least 1).
- Alternatives: Always the least (an X spell cast for 0 usually does nothing); always the greatest (abilities would spend everything).
- Why: Casting an X spell signals intent to use it; for the other numbers, spend least.
- Affects: every X and number decision.
- Code: `crates/mtg-engine/src/casting.rs` (`ChooseX` for spells and for abilities); `crates/mtg-engine/src/game.rs` (`ask_number`); `crates/mtg-engine/src/kw/` (join_forces, morph_face_up, optional_cost_x, spell_cost_grammar, suspend); `crates/mtg-engine/src/mana_abilities.rs`.
- Tests: `tests/cr/interp_decision_defaults.rs` (`a_default_x_is_the_greatest_affordable_and_a_default_number_the_least`).

## default-order-as-listed: Default orderings keep the listed order (triggers: the first listed goes on the stack first)
- Kind: policy
- Status: decided
- Rules: 603.3b "If multiple abilities have triggered since the last time a player received priority, the abilities are placed on the stack in a two-part process. First, each player, in APNAP order, puts each triggered ability they control with a trigger condition that isn’t another ability triggering on the stack in any order they choose. ..." / 101.4 "If multiple players would make choices and/or take actions at the same time, the active player (the player whose turn it is) makes any choices required, then the next player in turn order ... makes any choices required, followed by the remaining nonactive players in turn order. ..."
- Question: In which order does a player put simultaneous triggered abilities on the stack (or order cards, copies and the like) when the agent doesn't say?
- Decision: The order offered (the identity permutation). For triggered abilities that's the order they were detected in, the first one going on the stack first, so it resolves last. Players still order in APNAP order (101.4), which the engine follows as a rule.
- Alternatives: The reverse order (the first detected resolves first); a heuristic.
- Why: Deterministic and neutral.
- Affects: every ordering decision.
- Code: `crates/mtg-engine/src/game.rs` (`ask_order`); `crates/mtg-engine/src/triggers.rs` (stacking triggers).
- Tests: `tests/cr/interp_decision_defaults.rs` (`a_default_order_keeps_the_listed_order`).

## default-combat-declarations: A default or illegal declaration of attackers or blockers is replaced by the least legal one
- Kind: policy
- Status: decided
- Rules: 508.1d "The active player checks each creature they control to see whether it’s affected by any requirements ... If the number of requirements that are being obeyed is fewer than the maximum possible number of requirements that could be obeyed without disobeying any restrictions, the declaration of attackers is illegal. ..." / 509.1a "The defending player chooses which creatures they control, if any, will block. ..."
- Question: What does the engine declare when the agent gives no declaration, or an illegal one?
- Decision: The declaration that obeys the most requirements without any costs and without anything else: only creatures that must attack (or block) do. An illegal declaration (or one whose costs can't be paid) is undone and replaced by that one. When another player chooses the blocks (Odric, Master Tactician; Master Warcraft) and the blocking player declines the costs, new blocks are proposed up to three times.
- Alternatives: Re-ask until legal (can stall); declare nothing even when requirements exist (illegal).
- Why: The least commitment that is legal.
- Affects: every combat.
- Code: `crates/mtg-engine/src/combat.rs` (declaring attackers and blockers, `best_blocks`); `crates/mtg-engine/src/attack_choice.rs`; `crates/mtg-engine/src/block_choice.rs`.
- Tests: `tests/cr/interp_decision_defaults.rs` (`a_default_attack_declaration_obeys_requirements_and_nothing_more`, `a_default_block_declaration_blocks_with_nothing`).

## default-damage-assignment: Default combat damage is lethal to each blocker in order, then the rest to what it's attacking (with trample)
- Kind: policy
- Status: decided
- Rules: 510.1c "... If two or more creatures are blocking it, it assigns its combat damage to those creatures divided as its controller chooses among them." / 702.19b "The controller of an attacking creature with trample first assigns damage to the creature(s) blocking it. Once all those blocking creatures are assigned lethal damage, any excess damage is assigned as its controller chooses among those blocking creatures and the player, planeswalker, or battle the creature is attacking. ..."
- Question: How does a creature's controller divide its combat damage when the agent doesn't say?
- Decision: Lethal damage to each blocker in the order listed, then the rest to the player, planeswalker or battle it's attacking if it has trample (with trample over planeswalkers: the planeswalker first, then its controller), otherwise the rest to the last blocker. When the defending player assigns the damage (banding, 702.22j), all of it stays on the blockers.
- Alternatives: All damage to the first blocker; spread evenly.
- Why: Kills the most blockers and pushes the most damage through, which is what dividing damage is usually for.
- Affects: every combat with several blockers or trample.
- Code: `crates/mtg-engine/src/combat.rs` (`TrampleShape::default_assignment`, `AssignCombatDamage`).
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_combat_damage_assignment_is_lethal_in_order_then_the_rest_through`).

## default-scry-surveil: Default scry and surveil keep every card on top in its order
- Kind: policy
- Status: decided
- Rules: 701.22a "To “scry N” means to look at the top N cards of your library, then put any number of them on the bottom of your library in any order and the rest on top of your library in any order." / 701.25a "To “surveil N” means to look at the top N cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order."
- Question: Where do the looked-at cards go when the agent doesn't say?
- Decision: All of them back on top, in the order they were.
- Alternatives: All to the bottom (or the graveyard): a strategic choice.
- Why: Changes nothing: the neutral choice.
- Affects: scry, surveil, fateseal and the like.
- Code: `crates/mtg-engine/src/scry_rules.rs`; `crates/mtg-engine/src/kwa/fateseal_clash.rs`.
- Tests: `tests/cr/interp_decision_defaults.rs` (`default_scry_keeps_every_card_on_top`).

## default-division: A default division splits evenly, the remainder to the first recipients
- Kind: policy
- Status: decided
- Rules: 601.2d "If the spell requires the player to divide or distribute an effect (such as damage or counters) among one or more targets, the player announces the division. Each of these targets must receive at least one of whatever is being divided."
- Question: How is damage (or counters) divided among targets when the agent doesn't say?
- Decision: As evenly as possible, the remainder one each to the first targets.
- Alternatives: All but one each to the first target.
- Why: Always legal (there are never more targets than the amount) and neutral.
- Affects: spells and abilities that divide (Forked Bolt, Arc Lightning, ...).
- Code: `crates/mtg-engine/src/stack.rs` (`choose_divisions`).
- Tests: `tests/cr/interp_decision_defaults.rs` (`a_default_division_splits_evenly_with_the_remainder_first`).

## default-card-name: A default or invalid card name names nothing
- Kind: deviation
- Status: decided
- Rules: 201.4 "If an effect instructs a player to choose a card name, the player must choose the name of a card in the Oracle card reference. (See rule 108.1.) A player may not choose the name of a token unless it’s also the name of a card." / 607.5a "... If an ability refers to an undefined choice, that part of the ability won’t do anything."
- Question: 201.4 requires a real card name. What is chosen when the agent gives none, or a name that isn't a card's (or doesn't fit the effect's restriction)?
- Decision: Nothing is named: the choice is left undefined and what refers to it does nothing (like 607.5a's undefined choice).
- Alternatives: Name some real card by a fixed rule (the first card of an opponent's deck, or of the Oracle card reference): satisfies 201.4 but is an arbitrary strategic choice.
- Why: Accepted: naming nothing never helps the player who failed to name a card, and the engine makes no strategic choices.
- Affects: "choose a card name" cards such as Pithing Needle, and hidden agenda, only when the agent gives no valid name.
- Code: `crates/mtg-engine/src/choices.rs` (`ChoiceKind::CardName`, `valid_card_name`); `crates/mtg-engine/src/kw/hidden_agenda.rs`.
- Tests: `tests/cr/interp_decision_defaults.rs` (`a_default_card_name_names_nothing`).

## test-harness-defaults: TestGame's scripted agents pass, keep, and attack and block with nothing
- Kind: policy
- Status: decided
- Rules: 117.3d (see `default-priority-pass`)
- Question: What do tests' agents do when a test queues no answer for a decision?
- Decision: `testing::ScriptedAgent` uses queued answers first (matched by kind of decision, so unrelated prompts don't consume them); with nothing queued it passes priority, keeps its hand, declares no attackers and no blockers, and answers everything else with `Answer::Default` (the engine defaults in this file).
- Alternatives: Fail the test on an unscripted decision: stricter, but most tests rely on the defaults for incidental decisions.
- Why: Tests should say only what matters to them; a test that relies on a default for the behavior it checks should cite that default's entry.
- Affects: every TestGame test. "No attackers" is an explicit declaration, so with attack requirements the engine replaces it (`default-combat-declarations`).
- Code: `crates/mtg-engine/src/testing.rs` (`ScriptedAgent::decide`).
- Tests: `tests/cr/interp_decision_defaults.rs` (`the_test_harness_answers_unscripted_decisions_with_fixed_defaults`).

## external-agent-invalid-answers: External agents get three more tries after an invalid answer, then the engine's default
- Kind: policy
- Status: decided
- Rules: none directly (the agent interface); its parallel in the rules is 733.2 "When reversing illegal spells and abilities, the player who had priority retains it and may take another action or pass. ..."
- Question: What happens when an agent outside the engine answers a decision invalidly, or stops answering?
- Decision: `ProtocolAgent` replies to an invalid answer with an error and asks again, up to `max_retries` times (default 3); then the decision gets `Answer::Default`. After the transport closes, every later decision gets `Answer::Default` without asking. With `auto_pass` on (off by default), priority is passed without asking when passing, conceding and mana abilities are the only options.
- Alternatives: Fail the game; or concede for the agent.
- Why: The game always finishes with defined behavior, and agents get feedback to correct themselves.
- Affects: games with external agents (mtg-api).
- Code: `crates/mtg-api/src/agent.rs` (`ProtocolAgent::decide`, `ProtocolOptions`).
- Tests: `crates/mtg-api/tests/protocol.rs` (`invalid_answers_get_an_error_and_are_asked_again`, `too_many_invalid_answers_fall_back_to_the_default`).

## hidden-information: In-process agents see everything; mtg-api views follow the hidden-information rules
- Kind: policy
- Status: decided
- Rules: 400.2 "... Hidden zones are zones in which not all players can be expected to see the cards’ faces. Library and hand are hidden zones, even if all the cards in one such zone happen to be revealed." / 401.2 "Each library must be kept in a single face-down pile. Players can’t look at or change the order of cards in a library." / 402.3 "... A player can’t look at the cards in another player’s hand but may count those cards at any time." / 106.4a "If any mana remains in a player’s mana pool after mana is spent to pay a cost, that player announces what mana is still there." / 106.4b "If a player passes priority (see rule 117) while there is mana in their mana pool, that player announces what mana is there." / 712.6 "Players who are allowed to look at a double-faced card may look at both sides of that card."
- Question: The rules say what each player may see; announcements (106.4) and handling the cards are left to the players. What does the engine give agents?
- Decision: The in-process `Agent` trait receives the whole `Game`, hidden information included: agents that shouldn't cheat must restrict themselves. mtg-api's observations, requests and event feed show each player only what the rules let them see: other players' hands and libraries only as counts, face-down objects' identities only to those allowed to look, and a controlled player's view shared with their controller (723). Mana pools are public, standing in for 106.4's announcements. A player allowed to look at a double-faced card sees both faces.
- Alternatives: Filter the in-process interface too: safer, but the engine's own helpers and tests read the full state through it.
- Why: Hidden information is enforced at the boundary where untrusted agents live (mtg-api); in-process agents are part of the program.
- Affects: all agents.
- Code: `crates/mtg-engine/src/decision.rs` (`Agent`); `crates/mtg-api/src/view.rs` (`observe`); `crates/mtg-api/src/events.rs`; `crates/mtg-api/src/describe.rs`.
- Tests: `crates/mtg-api/tests/hidden_info.rs` (`opponents_hand_and_libraries_are_hidden`, `face_down_permanents_are_seen_only_by_their_controller`, `requests_and_events_dont_leak_hidden_cards`, `every_players_mana_pool_is_public`).

## seeded-randomness: Random results come from a pseudorandom generator seeded per game
- Kind: policy
- Status: decided
- Rules: 701.24a "To shuffle a library or a face-down pile of cards, randomize the cards within it so that no player knows their order." / 705.1 "Some cards refer to flipping a coin. A coin used in a flip must be a two-sided object with easily distinguished sides and equal likelihood that either side lands face up. ..."
- Question: Shuffles, coin flips, die rolls and random choices must be random. How does the engine randomize?
- Decision: Every game has one ChaCha8 generator seeded from `GameConfig::seed`; all random results (shuffles, flips, rolls, "at random" choices, the starting player) come from it, so a game replays exactly from its seed and its agents' answers. No player (agent) can predict results without reading the generator's state, which in-process agents could do (see `hidden-information`).
- Alternatives: An unseeded (system) random source: no replays or reproducible fuzz failures.
- Why: Reproducibility: mtg-sim prints a repro command with the seed for every failure.
- Affects: every random event.
- Code: `crates/mtg-engine/src/game.rs` (`Game::rng`, `shuffle_library`, `GameConfig::seed`); `crates/mtg-engine/src/dice.rs`.
- Tests: `tests/cr/interp_decision_defaults.rs` (`random_results_come_from_the_games_seed`).

## ability-ids-depend-on-the-process: Ability ids in requests come from a process-wide counter; answers name options by their place
- Kind: policy
- Status: decided
- Rules: none directly (the agent interface). Abilities have identities because of rules such as 602.5c "If an object acquires an activated ability with a restriction on its use from another object, that restriction applies only to that ability as acquired from that object. It doesn’t apply to other, identically worded abilities." and 607.1 "An object may have two abilities printed on it such that one of them causes actions to be taken or objects or players to be affected and the other one directly refers to those actions, objects, or players. If so, these two abilities are linked: ..."
- Question: The rules don't number abilities; the engine does, to tell them apart (activation limits, linked abilities, removing one ability). Must the numbers be the same every time a game is played?
- Decision: No. Ability ids come from one counter the whole process shares (`ability::next_ability_uid`) and are handed out as abilities are first needed: compiled card abilities (cached for every game), abilities effects give, delayed triggers, tokens' abilities. So the ids in agent-protocol requests (`action.ability`) differ between processes, and even between two plays of one game in one process, unless everything before them happened the same way. Answers refer to options by `index`, which doesn't depend on the ids, so a game replays from its answers; replays and the property checks compare ids up to a consistent renaming. The links of abilities effects give, which are also numbers, are numbered per game (`ability_grants::FreshLinks`).
- Alternatives: Allocate ability ids per game: the same requests in every process, but the caches of compiled and derived abilities, shared by all games, would have to be reworked to renumber per game.
- Why: Agents never need to send an id back, so stable ids would buy only easier comparison of requests across processes, which renaming already gives.
- Affects: games with external agents (mtg-api requests and transcripts), replays, the determinism and snapshot properties.
- Code: `crates/mtg-engine/src/ability.rs` (`next_ability_uid`); `crates/mtg-api/src/replay.rs` (`AbilityRenaming`); `crates/mtg-engine/src/properties/agent.rs` (`rename_uids`).
- Tests: `tests/cr/interp_ability_ids.rs` (`the_same_game_numbers_a_gained_ability_differently_but_lists_it_in_the_same_place`).

## unbounded-number-ceiling: "Choose a number" with no upper limit goes up to 1,000,000
- Kind: deviation
- Status: decided
- Rules: 107.1 "The only numbers the Magic game uses are integers." / 107.1a "You can’t choose a fractional number, deal fractional damage, gain fractional life, and so on. ..." / Menacing Ogre ruling: "You have to choose zero or a positive number. You can’t choose a negative number."
- Question: "Choose a number" (Void, Menacing Ogre), "a number 0 or greater" (Wheel of Misfortune), "a number greater than 0" (Mindblaze), "hides at least one item" (Goblin Game) set no upper limit. What range does the number decision offer?
- Decision: From the least number the text allows to `choices::MAX_CHOSEN_NUMBER` (1,000,000), shown to agents as the decision's `max` (mtg-api `number` spec). An answer outside the range is invalid and the least number is chosen (`default-numbers`). The same ceiling bounds life bids (`bidding.rs`) and "any number of counters" moved.
- Alternatives: No maximum at all (the protocol can express it: a `number` spec without `max`), which needs every arithmetic path that a chosen number reaches (life totals, damage, counters, all 32-bit) to saturate; the former ceiling of 1000, which a player could want to exceed (losing more life than 1000 is never needed, but numbers are also compared and matched against mana values and card counts).
- Why: A million is beyond every life total, library, card count or mana value of a real game (outside the un-cards not supported), so no choice a player could want is excluded, while the engine's 32-bit game arithmetic stays exact.
- Affects: Void, Menacing Ogre, Wheel of Misfortune, Mindblaze, Goblin Game, "As ~ enters, choose a number", bidding (Illicit Auction, Pain's Reward, ...), "move any number of counters".
- Code: `crates/mtg-engine/src/choices.rs` (`MAX_CHOSEN_NUMBER`); `crates/mtg-engine/src/oracle/patterns/choice_grammar_secret_numbers.rs`, `choice_grammar_multi.rs`, `etb_choices.rs`, `r122_move_counters.rs`; `crates/mtg-engine/src/bidding.rs`.
- Tests: `tests/cards/choices_number_ceiling.rs`.

## deck-order-ids-before-shuffle: Until the decks are shuffled, cards' ids follow the order the decks were given in
- Kind: policy
- Status: decided
- Rules: 103.3 "Each player shuffles their deck so that the cards are in a random order. Each player may then shuffle or cut their opponents’ decks. The players’ decks become their libraries." / 400.2 "... Library and hand are hidden zones, even if all the cards in one such zone happen to be revealed." / 401.2 "Each library must be kept in a single face-down pile. Players can’t look at or change the order of cards in a library."
- Question: Object ids are visible to agents (an object in a hidden zone is shown with its id). Cards are made into objects from the deck lists as the game is set up, so their ids follow the lists' order. Does that tell anyone which hidden card is which?
- Decision: Not once the decks are shuffled. Library objects keep their ids but aren't shown; cards drawn, moved or put anywhere become new objects with new ids (CR 400.7); the cards of face-down decks kept in the command zone (Attraction, planar and scheme decks) become new objects in their shuffled order (`zones::renumber_shuffled_deck`); conspiracies put into the command zone become new objects in random order. Before the decks are shuffled (the decisions of the game's setup, such as choosing who goes first or which conspiracies to put into the command zone), the ids of a deck's face-down cards in the command zone follow the order the deck was given in, which the shuffle then makes meaningless.
- Alternatives: Make every card a new object, in random order, before any decision is asked; or shuffle the deck lists before making their objects (which would change every seeded game).
- Why: Before the shuffle, the order of a deck is the player's own arrangement and says nothing about the game's hidden information once play begins; the hidden information property compares what players see from the opening hands on.
- Affects: decisions during the game's setup (choosing the starting player, conspiracies, companions) in games with Attraction, planar or scheme decks.
- Code: `crates/mtg-engine/src/game.rs` (`Game::new`); `crates/mtg-engine/src/zones.rs` (`renumber_shuffled_deck`); `crates/mtg-engine/src/start.rs` (`conspiracies_to_command_zone`); `crates/mtg-api/src/hidden.rs` (`check_start`).
- Tests: `crates/mtg-api/tests/hidden_information.rs` (`object_ids_dont_tell_hidden_cards_apart`, `the_ids_of_face_down_decks_in_the_command_zone_dont_tell_their_cards`).

## draft-three-colors-default: Colors chosen as a card is drafted default to three different colors at random
- Kind: policy
- Status: decided
- Rules: 905.2 "Some cards have abilities that function during the draft. ..." / 905.2b "... Players may also be instructed to note information. ..."
- Question: "Reveal [this card] as you draft it. The player to your right chooses a color, you choose another color, then the player to your left chooses a third color." (Paliano, the High City; Regicide): the draft has no agents asking the three players; which colors are noted?
- Decision: The program running the draft sets them in advance with `Draft::preset_colors(player, [right, own, left])` (three different colors); otherwise the three colors are different colors picked at random with the draft's seeded random generator.
- Alternatives: Ask agents during the draft (the draft API has no agents); pick a fixed order (WUB), which would make every such draft the same.
- Why: The rulings require three different colors; random choices keep simulated drafts varied and reproducible from the seed.
- Affects: Paliano, the High City and Regicide (and any card with the same draft ability).
- Code: `crates/mtg-engine/src/draft.rs` (`Draft::three_colors`, `Draft::preset_colors`).
- Tests: `tests/cards/draft_4.rs` (`paliano_and_regicide_use_the_colors_chosen_as_they_were_drafted`).

## opponents-choice-untargeted: "A land of an opponent's choice" is chosen by an opponent the controller picks
- Kind: interpretation
- Status: decided
- Rules: 601.7a "If there is more than one opponent who could make such a choice, the spell’s controller decides which of those opponents will make the choice." / 701.21a "To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. ..."
- Question: Demonic Hordes: "tap ~ and sacrifice a land of an opponent's choice." 601.7a is about targets chosen by an opponent; this choice is made on resolution and isn't a target. Which opponent chooses in multiplayer?
- Decision: As the ability resolves, its controller chooses an opponent (`ChoiceKind::Opponent`), who then chooses which land the controller sacrifices among the lands the controller controls.
- Alternatives: (A) Each opponent in turn order picks one (more lands than the card says). (B) The next opponent in turn order chooses automatically.
- Why: 601.7a gives the controller that decision for the same wording about targets; nothing suggests a different procedure on resolution.
- Affects: Demonic Hordes.
- Code: `crates/mtg-engine/src/oracle/patterns/paper_choices.rs` (`sacrifice_of_an_opponents_choice`).
- Tests: `tests/cards/paper_choosers.rs` (`demonic_hordes_an_opponent_chooses_the_land_you_sacrifice`).

## player-destroys-chosen-permanents: "[Player] destroys N lands they control": chosen by that player among those that can be destroyed
- Kind: interpretation
- Status: decided
- Rules: 701.8a "To destroy a permanent, move it from the battlefield to its owner’s graveyard." / 702.12b "A permanent with indestructible can’t be destroyed. ..." / Burning of Xinye's rulings: "That player chooses which of their lands to destroy. Neither of you can choose to destroy a land with indestructible." and "A land with a regeneration shield on it will regenerate from Burning of Xinye’s destruction effect."
- Question: With a player as the subject of "destroy", how are the permanents chosen?
- Decision: As the instruction is performed, that player chooses that many permanents of the kind they control, excluding ones with indestructible (as the ruling says), and they're destroyed at once; a regeneration shield replaces the destruction. With fewer such permanents, all of them.
- Alternatives: Let the player choose indestructible permanents (which then survive): contradicts the ruling.
- Why: The ruling, generalized to the same grammar on any card.
- Affects: Burning of Xinye.
- Code: `crates/mtg-engine/src/oracle/patterns/paper_players.rs` (`player_destroys`).
- Tests: `tests/cards/paper_players.rs` (`burning_of_xinye_each_player_chooses_their_own_lands_to_destroy`).

## mill-until-stops-when-nothing-moves: A "mill until" process ends when the library is empty or milling moves nothing
- Kind: policy
- Status: decided
- Rules: 701.17a "For a player to mill a number of cards, that player puts that many cards from the top of their library into their graveyard." / 104.4b (loops) / Helm of Obedience's ruling: "If an effect like that of Leyline of the Void prevents cards from being put into your opponent’s graveyard, the process described in the first sentence of Helm of Obedience’s effect will never stop. Your opponent’s entire library will be exiled, even if X is 1."
- Question: Helm of Obedience's process counts only cards put into the graveyard. If milled cards go elsewhere, when does it stop; and what if a replacement keeps cards from leaving the library at all?
- Decision: One card at a time; only cards put into that graveyard count toward X and only a creature card put there stops it. The process also ends when the library is empty or a mill moved no card (nothing would ever change: a loop the rules leave open).
- Alternatives: Count every milled card toward X (contradicts the ruling).
- Why: The ruling; ending a process that can't change anything follows the loop rules' intent.
- Affects: Helm of Obedience.
- Code: `crates/mtg-engine/src/kw/mill_until.rs`.
- Tests: `tests/cards/paper_loops.rs` (`helm_of_obedience_mills_the_whole_library_against_leyline_of_the_void`, `helm_of_obedience_stops_after_x_cards`).
