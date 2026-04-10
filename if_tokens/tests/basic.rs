use if_tokens::if_tokens;

#[test]
fn chooses_first_matching_if_clause() {
    let value = if_tokens! {
        if [a b c] == [x y z] { 10 }
        if [alpha (beta)] == [alpha (beta)] { 20 }
        else { 30 }
    };

    assert_eq!(value, 20);
}

#[test]
fn supports_and_or_not_and_parentheses() {
    let value = if_tokens! {
        if [a] == [a] && !([x] == [x]) { 1 }
        if ([a] == [b]) || ([left] == [left] && [k] != [q]) { 2 }
        else { 3 }
    };

    assert_eq!(value, 2);
}

#[test]
fn falls_back_to_else_clause() {
    let value = if_tokens! {
        if [foo] == [bar] { 1 }
        else { 9 }
    };

    assert_eq!(value, 9);
}

#[test]
fn expands_to_nothing_without_match_or_else() {
    let x = 7;
    if_tokens! {
        if [a] == [b] { let _unused = 99; }
    }
    assert_eq!(x, 7);
}
