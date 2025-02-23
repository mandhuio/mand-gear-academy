#![allow(warnings)]

mod utils;

use gtest::Log;
use session_io::{Action, Event, GameOverStatus, GameStatus, State};
use utils::*;

const GAME_NOT_PLAYABLE: &str = "Game is not available to play";
const INVALID_WORD_CASE: &str = "Word must be lowercased";
const INVALID_WORD_LEN: &str = "Word must be 5 character long";
const MAX_ATTEMPTS: u32 = 5;

#[test]
fn check_word_should_work_on_wrong_answer() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: A game is in progress
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: User guess the word incorrectly
    let result = proxy_program.send(
        USER,
        Action::CheckWord {
            word: WRONG_ANSWER.into(),
        },
    );
    system.run_next_block();
    println!("result: {:?}", result);
    // Then:
    //  - WordChecked event is emitted
    //  - User game info is updated properly
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload(word_checked_on_wrong_answer_event());
    println!("log: {:?}", log);
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);
    assert_eq!(info.attempts_count, 1);
}

#[test]
fn check_word_should_fail_when_not_playing() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: A game session is over
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    system.run_to_block(200);
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: User guess the word
    let result = proxy_program.send(
        USER,
        Action::CheckWord {
            word: CORRECT_ANSWER.into(),
        },
    );
    system.run_next_block();
    println!("result: {:?}", result);
    // Then: Program reverts since the game is not playable
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload_bytes(final_panic_message(GAME_NOT_PLAYABLE));
    println!("log: {:?}", log);
}

#[test]
fn check_word_should_fail_when_invalid_length() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: A game is in progress
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    let info = players.get(&USER.into()).expect("Failed to read state");
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: User submits invalid length word
    let result = proxy_program.send(
        USER,
        Action::CheckWord {
            word: "honk".to_owned(),
        },
    );
    system.run_next_block();
    println!("result: {:?}", result);
    // Then: Program reverts with invalid length message
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload_bytes(final_panic_message(INVALID_WORD_LEN));
    println!("log: {:?}", log);
}

#[test]
fn check_word_should_fail_when_not_lowercased() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: A game is in progress
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: User submits five-character-long but not lowercased
    let result = proxy_program.send(
        USER,
        Action::CheckWord {
            word: "HAPPY".to_owned(),
        },
    );
    system.run_next_block();
    println!("result: {:?}", result);
    // Then: Program reverts with invalid letter case
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload_bytes(final_panic_message(INVALID_WORD_CASE));
    println!("log: {:?}", log);
}

#[test]
fn check_word_should_end_game_when_guessed() {
    let system = init_system();
    let ProgramPair { proxy_program, .. } = init_programs(&system);

    // Given: Game is in progress
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    let info = players.get(&USER.into()).expect("Failed to read state");
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: User enters the correct word
    let result = proxy_program.send(
        USER,
        Action::CheckWord {
            word: CORRECT_ANSWER.into(),
        },
    );
    system.run_next_block();
    // Then: GameOver event is emitted and user's session info is updated
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload(Event::GameOver(GameOverStatus::Win));
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    let info = players.get(&USER.into()).expect("Failed to read state");
    assert_eq!(info.attempts_count, 1);
    assert_eq!(info.game_status, GameStatus::InProgress);
}

#[test]
fn check_word_should_end_game_when_all_attempts_used_up() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: Game is in progress and the user keeps answering wrong word
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    for _ in 0..MAX_ATTEMPTS - 1 {
        proxy_program.send(
            USER,
            Action::CheckWord {
                word: WRONG_ANSWER.into(),
            },
        );
        system.run_next_block();
    }

    // When: Send wrong answer for the last chance
    let result = proxy_program.send(
        USER,
        Action::CheckWord {
            word: WRONG_ANSWER.into(),
        },
    );
    system.run_next_block();
    println!("result: {:?}", result);
    // Then: GameOver event is emitted and user's session info is updated
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload(Event::GameOver(GameOverStatus::Lose));
    println!("log: {:?}", log);
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    let info = players.get(&USER.into()).expect("Failed to read state");
    assert_eq!(info.attempts_count, MAX_ATTEMPTS);
    assert_eq!(
        info.game_status,
        GameStatus::Completed(GameOverStatus::Lose)
    );
}
