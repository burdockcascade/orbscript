use orbscript::run;

#[test]
fn hello_world() {
    assert_eq!(run(include_str!("scripts/hello_world.orb"), None, None).unwrap(), None);
}


// Variables

#[test]
fn var_boolean() {
    assert_eq!(run(include_str!("scripts/var_boolean.orb"), None, None).unwrap(), None);
}

#[test]
fn var_float() {
    assert_eq!(run(include_str!("scripts/var_float.orb"), None, None).unwrap(), None);
}

#[test]
fn var_integer() {
    assert_eq!(run(include_str!("scripts/var_integers.orb"), None, None).unwrap(), None);
}

#[test]
fn var_string() {
    assert_eq!(run(include_str!("scripts/var_string.orb"), None, None).unwrap(), None);
}

#[test]
fn var_array() {
    assert_eq!(run(include_str!("scripts/var_array.orb"), None, None).unwrap(), None);
}

#[test]
fn var_dictionary() {
    assert_eq!(run(include_str!("scripts/var_dictionary.orb"), None, None).unwrap(), None);
}

#[test]
fn var_multi_value() {
    assert_eq!(run(include_str!("scripts/var_multi_value.orb"), None, None).unwrap(), None);
}

#[test]
fn var_lambda() {
    assert_eq!(run(include_str!("scripts/var_lambda.orb"), None, None).unwrap(), None);
}

// Functions

#[test]
fn function_no_parameters() {
    assert_eq!(run(include_str!("scripts/function_no_params.orb"), None, None).unwrap(), None);
}

#[test]
fn function_with_parameters() {
    assert_eq!(run(include_str!("scripts/function_no_params.orb"), None, None).unwrap(), None);
}

#[test]
fn function_with_unused_function() {
    assert_eq!(run(include_str!("scripts/function_with_unused_function.orb"), None, None).unwrap(), None);
}

// LOOPS

#[test]
fn loop_for_i_to() {
    assert_eq!(run(include_str!("scripts/loop_for_i_to.orb"), None, None).unwrap(), None);
}

#[test]
fn loop_for_in_array() {
    assert_eq!(run(include_str!("scripts/loop_for_in_array.orb"), None, None).unwrap(), None);
}

#[test]
fn loop_for_in_dict() {
    assert_eq!(run(include_str!("scripts/loop_for_in_dict.orb"), None, None).unwrap(), None);
}

#[test]
fn loop_while() {
    assert_eq!(run(include_str!("scripts/loop_while.orb"), None, None).unwrap(), None);
}

// IFS

#[test]
fn if_statement() {
    assert_eq!(run(include_str!("scripts/if_statement.orb"), None, None).unwrap(), None);
}

#[test]
fn if_false() {
    assert_eq!(run(include_str!("scripts/if_false.orb"), None, None).unwrap(), None);
}

#[test]
fn if_else() {
    assert_eq!(run(include_str!("scripts/if_else.orb"), None, None).unwrap(), None);
}

#[test]
fn if_else_false() {
    assert_eq!(run(include_str!("scripts/if_else_false.orb"), None, None).unwrap(), None);
}


// PROGRAMS

#[test]
fn fib_quick() {
    assert_eq!(run(include_str!("scripts/program_fib.orb"), None, Some(String::from("fib_quick"))).unwrap(), None);
}

#[test]
fn fib_long() {
    assert_eq!(run(include_str!("scripts/program_fib.orb"), None, Some(String::from("fib_long"))).unwrap(), None);
}