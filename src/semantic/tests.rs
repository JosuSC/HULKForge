use super::checker::{check_program, SemanticError};
use crate::lexer::lexer::TokenStream;
use crate::parser::Parser;

fn semantic_errors(source: &str) -> Vec<SemanticError> {
    let stream = TokenStream::new(source);
    let mut parser = Parser::new(stream);
    let program = parser
        .parse_program()
        .expect("the test source should parse successfully");
    check_program(&program)
}

fn assert_has_error(errors: &[SemanticError], expected_fragment: &str) {
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains(expected_fragment)),
        "expected an error containing '{}', got: {:?}",
        expected_fragment,
        errors
    );
}

#[test]
fn equality_rejects_number_and_string() {
    let errors = semantic_errors(r#"
        1 == "hola";
    "#);

    assert!(
        errors.iter().any(|error| error.message.contains("equality operator")),
        "expected an equality-type mismatch error, got: {:?}",
        errors
    );
}

#[test]
fn equality_rejects_boolean_and_number() {
    let errors = semantic_errors(r#"
        true == 42;
    "#);

    assert!(
        errors.iter().any(|error| error.message.contains("equality operator")),
        "expected an equality-type mismatch error, got: {:?}",
        errors
    );
}

#[test]
fn relational_rejects_string_and_number() {
    let errors = semantic_errors(r#"
        "hola" > 2;
    "#);

    assert!(
        errors.iter().any(|error| error.message.contains("relational operator requires Number")),
        "expected a relational-type mismatch error, got: {:?}",
        errors
    );
}

#[test]
fn relational_rejects_boolean_and_number() {
    let errors = semantic_errors(r#"
        false <= 10;
    "#);

    assert!(
        errors.iter().any(|error| error.message.contains("relational operator requires Number")),
        "expected a relational-type mismatch error, got: {:?}",
        errors
    );
}

#[test]
fn equality_rejects_mixed_types_inside_let_binding() {
    let errors = semantic_errors(r#"
        let left: String = "hola", right: Number = 42 in left != right;
    "#);

    assert!(
        errors.iter().any(|error| error.message.contains("equality operator")),
        "expected an equality-type mismatch error, got: {:?}",
        errors
    );
}

#[test]
fn relational_rejects_mixed_types_inside_let_binding() {
    let errors = semantic_errors(r#"
        let left: Boolean = true, right: Number = 2 in left >= right;
    "#);

    assert!(
        errors.iter().any(|error| error.message.contains("relational operator requires Number")),
        "expected a relational-type mismatch error, got: {:?}",
        errors
    );
}

#[test]
fn reports_call_to_nonexistent_function() {
    let errors = semantic_errors(r#"
        desconocida(1, 2);
    "#);

    assert_has_error(&errors, "function 'desconocida' not defined");
}

#[test]
fn reports_invalid_arity_for_user_function() {
    let errors = semantic_errors(r#"
        function suma(a, b) => a + b;
        suma(1);
    "#);

    assert_has_error(&errors, "call to 'suma' with invalid arity");
}

#[test]
fn reports_invalid_arity_for_builtin_function() {
    let errors = semantic_errors(r#"
        sin();
    "#);

    assert_has_error(&errors, "call to 'sin' with invalid arity");
}

#[test]
fn reports_invalid_argument_types_for_user_function() {
    let errors = semantic_errors(r#"
        function mezclar(texto: String, cantidad: Number) => texto;
        mezclar(10, "hola");
    "#);

    assert_has_error(&errors, "call to 'mezclar' argument 1 expects String, found Number");
}

#[test]
fn reports_invalid_argument_types_for_method_call_on_self() {
    let errors = semantic_errors(r#"
        type A {
            m(texto: String, cantidad: Number) {
                0
            }

            n() {
                self.m(10, "hola");
                0
            }
        }
        0;
    "#);

    assert_has_error(&errors, "method 'm' argument 1 expects String, found Number");
}

#[test]
fn reports_invalid_argument_types_for_builtin_function() {
    let errors = semantic_errors(r#"
        sin("hola");
    "#);

    assert_has_error(&errors, "call to 'sin' argument 1 expects Number, found String");
}

#[test]
fn while_requires_boolean_condition_for_number() {
    let errors = semantic_errors(r#"
        while (1) 0;
    "#);

    assert_has_error(&errors, "while condition must be Boolean");
}

#[test]
fn while_requires_boolean_condition_for_string() {
    let errors = semantic_errors(r#"
        while ("hola") 0;
    "#);

    assert_has_error(&errors, "while condition must be Boolean");
}

#[test]
fn while_with_assignment_to_undefined_variable_reports_error() {
    let errors = semantic_errors(r#"
        while (true) {
            x := 1;
            0
        };
    "#);

    assert_has_error(&errors, "assignment to undefined variable 'x'");
}

#[test]
fn nested_while_reports_nonexistent_function_inside_body() {
    let errors = semantic_errors(r#"
        while (true) {
            while (true) inexistente(10);
            0
        };
    "#);

    assert_has_error(&errors, "function 'inexistente' not defined");
}

#[test]
fn type_inheritance_reports_undefined_parent_type() {
    let errors = semantic_errors(r#"
        type Hijo inherits Fantasma {
            valor = 1;
        }
        0;
    "#);

    assert_has_error(&errors, "parent type 'Fantasma' not defined");
}

#[test]
fn type_inheritance_reports_wrong_parent_arity() {
    let errors = semantic_errors(r#"
        type Padre(a, b) {
            valor = 1;
        }

        type Hijo inherits Padre(1) {
            otro = 2;
        }
        0;
    "#);

    assert_has_error(&errors, "parent type 'Padre' requires 2 arguments");
}

#[test]
fn function_parameter_reports_undefined_type_annotation() {
    let errors = semantic_errors(r#"
        function f(x: TipoNoDefinido) => x;
        0;
    "#);

    assert_has_error(&errors, "type 'TipoNoDefinido' not defined");
}

#[test]
fn let_binding_reports_undefined_type_annotation() {
    let errors = semantic_errors(r#"
        let x: TipoFantasma = 1 in x;
    "#);

    assert_has_error(&errors, "type 'TipoFantasma' not defined");
}

#[test]
fn protocol_extends_reports_undefined_parent_protocol() {
    let errors = semantic_errors(r#"
        protocol P extends Q {
            m(x: Number): Number;
        }
        0;
    "#);

    assert_has_error(&errors, "parent protocol 'Q' not defined");
}

#[test]
fn protocol_method_signature_reports_undefined_return_type() {
    let errors = semantic_errors(r#"
        protocol Serializable {
            serialize(x: Number): TipoRaro;
        }
        0;
    "#);

    assert_has_error(&errors, "type 'TipoRaro' not defined");
}

#[test]
fn duplicate_function_parameters_report_error() {
    let errors = semantic_errors(r#"
        function repetir(x, x) => x;
        0;
    "#);

    assert_has_error(&errors, "duplicate parameter 'x'");
}

#[test]
fn self_outside_method_reports_error() {
    let errors = semantic_errors(r#"
        self;
    "#);

    assert_has_error(&errors, "use of self outside of a method");
}

#[test]
fn base_outside_method_reports_error() {
    let errors = semantic_errors(r#"
        base(1);
    "#);

    assert_has_error(&errors, "use of base outside of a method");
}

#[test]
fn method_call_on_self_reports_missing_method() {
    let errors = semantic_errors(r#"
        type A {
            m() {
                self.no_existe();
                0
            }
        }
        0;
    "#);

    assert_has_error(&errors, "method 'no_existe' with arity 0 not defined on current type");
}

#[test]
fn field_access_on_self_reports_missing_attribute() {
    let errors = semantic_errors(r#"
        type A {
            m() {
                self.no_existe;
                0
            }
        }
        0;
    "#);

    assert_has_error(&errors, "attribute 'no_existe' not defined on current type");
}

#[test]
fn reports_invalid_argument_types_for_method_call_on_variable() {
    let errors = semantic_errors(r#"
        type A {
            m(texto: String, cantidad: Number) {
                0
            }
        }

        let a = new A() in {
            a.m(10, "hola");
            0
        };
    "#);

    assert_has_error(&errors, "method 'm' argument 1 expects String, found Number");
}

#[test]
fn logical_and_requires_boolean_operands_number_left() {
    let errors = semantic_errors(r#"
        true and 1;
    "#);
        let errors = semantic_errors(r#"
            true & 1;
        "#);

        assert_has_error(&errors, "logical operator requires Boolean");
}

#[test]
fn logical_or_requires_boolean_operands_number_left() {
    let errors = semantic_errors(r#"
            1 | false;
        "#);

        assert_has_error(&errors, "logical operator requires Boolean");
}

#[test]
fn function_call_in_and_reports_nonboolean() {
    let errors = semantic_errors(r#"
            function factorial(n: Number, j: String): Number => n;

            if (factorial(1, "x") & true) { 0 } else { 0 };
        "#);

        assert_has_error(&errors, "logical operator requires Boolean");
}
