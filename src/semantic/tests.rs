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
fn uppercase_call_reports_missing_type_instead_of_missing_function() {
    let errors = semantic_errors(r#"
        Fantasma(1);
    "#);

    assert_has_error(&errors, "type 'Fantasma' not defined");
}

#[test]
fn uppercase_call_reports_type_arity_errors() {
    let errors = semantic_errors(r#"
        type Person(name, age) {
            name: String = name;
            age: Number = age;
        }

        new Person("Ana");
    "#);

    assert_has_error(&errors, "type 'Person' requires 2 arguments");
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
fn reports_all_invalid_argument_types_for_user_function() {
    let errors = semantic_errors(r#"
        function nested(a: Number, b: String) : Number {
            let sum = 0 in {
                for (i in a) {
                    for (j in i) {
                        if (j % 2 == 0) { sum := sum + j } else { sum := sum + 0 };
                    };
                };
                sum
            }
        }
        nested(true, true)
    "#);

    assert_has_error(&errors, "call to 'nested' argument 1 expects Number, found Boolean");
    assert_has_error(&errors, "call to 'nested' argument 2 expects String, found Boolean");
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
fn type_inheritance_inherits_parent_constructor_arity_by_default() {
    let errors = semantic_errors(r#"
        type Person(name, age) {
            name = name;
            age = age;
        }

        type Knight inherits Person {
            title = "Sir";
        }

        let p = new Knight("Phil", "Collins") in p;
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn type_inheritance_reports_cyclic_inheritance() {
    let errors = semantic_errors(r#"
        type A inherits B {
            value = 1;
        }

        type B inherits A {
            value = 2;
        }

        0;
    "#);

    assert_has_error(&errors, "type 'A' has cyclic inheritance");
    assert_has_error(&errors, "type 'B' has cyclic inheritance");
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
fn let_binding_reports_type_mismatch_for_initializer() {
    let errors = semantic_errors(r#"
        let b = 4 * 2 in
            let a: Boolean = b + 4 in {
                print(a);
            };
    "#);

    assert_has_error(&errors, "let binding 'a' expects Boolean, found Number");
}

#[test]
fn let_binding_accepts_subtype_initializer() {
    let errors = semantic_errors(r#"
        type A {}
        type B inherits A{}
        type C inherits B {}

        let x: A = new C() in x;
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn let_binding_reports_inconsistent_function_return_note() {
    let errors = semantic_errors(r#"
        function g(a): Boolean => a + 5;

        let b = 4 * 2 in
            let a: String = g(5) in {
                print(a);
            };
    "#);

    assert_has_error(
        &errors,
        "function 'g' has an inconsistent return type: it declares Boolean, but its body returns Number",
    );
    assert_has_error(&errors, "let binding 'a' expected a String, but found a value of another type; note: function 'g' has an inconsistent return type: it declares Boolean, but its body returns Number");
}

#[test]
fn arithmetic_operator_reports_source_binding_inconsistency_note() {
    let errors = semantic_errors(r#"
        function g(a): Number => a + 5;

        let b: String = 4 * 2 in
            let a: Number = g(5) + b in {
                print(a);
            };
    "#);

    assert_has_error(&errors, "let binding 'b' expects String, found Number");
    assert_has_error(
        &errors,
        "arithmetic operator requires Number (right side: String); note: let binding 'b' expects String, found Number",
    );
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
fn protocol_extends_reports_error_when_parent_is_a_type() {
    let errors = semantic_errors(r#"
        type Base {}

        protocol P extends Base {
            m(x: Number): Number;
        }
        0;
    "#);

    assert_has_error(&errors, "parent type 'Base' cannot be extended by a protocol");
}

#[test]
fn protocol_extends_reports_cyclic_inheritance() {
    let errors = semantic_errors(r#"
        protocol A extends B {
            m(): Number;
        }

        protocol B extends A {
            n(): Number;
        }

        0;
    "#);

    assert_has_error(&errors, "protocol 'A' has cyclic inheritance");
    assert_has_error(&errors, "protocol 'B' has cyclic inheritance");
}

#[test]
fn type_inherits_reports_error_when_parent_is_a_protocol() {
    let errors = semantic_errors(r#"
        protocol Greetable {
            greet(): String;
        }

        type Person(name) inherits Greetable {
            name: String = name;

            greet(): String => "Hello, I am " @ self.name;
        }

        0;
    "#);

    assert_has_error(&errors, "type 'Person' cannot inherit from protocol 'Greetable'");
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
fn protocol_is_implemented_implicitly_by_matching_methods() {
    let errors = semantic_errors(r#"
        protocol Printable {
            printSelf(): String;
        }

        type Box {
            value = 10;
            printSelf() => "Box(" @ self.value @ ")";
        }

        let p: Printable = new Box() in print(p.printSelf());
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn protocol_extends_and_is_implemented_implicitly_by_matching_methods() {
    let errors = semantic_errors(r#"
        protocol MyProtocol {
            greet(): String;
            alwaysTrue(): Boolean;
        }

        protocol Printable extends MyProtocol {
            printSelf(): String;
            printValue(): Number;
        }

        type Box {
            value = 10;

            printSelf(): String => "Box(" @ self.value @ ")";
            printValue(): Number => self.value;
            alwaysTrue(): Boolean => true;
            greet(): String => "Hello, I am a box!";
        }

        let p: Printable = new Box() in print(p.greet());
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn protocol_implementation_reports_assignment_to_self_inside_method() {
    let errors = semantic_errors(r#"
        protocol Greetable {
            greet(): String;
        }

        type Person(name) {
            name: String = name;

            reset() => self := new Person("Hilko");
            greet(): String => "Hello, I am " @ self.name;
        }

        let p: Greetable = new Person("Alice") in print(p.greet());
    "#);

    assert_has_error(&errors, "cannot assign to 'self'");
}

#[test]
fn protocol_accepts_covariant_and_contravariant_overrides() {
    let errors = semantic_errors(r#"
        type Animal {}
        type Dog inherits Animal {}

        protocol Maker {
            make(input: Dog): Animal;
        }

        type AnyMaker {
            make(input: Object) => new Dog();
        }

        let maker: Maker = new AnyMaker() in maker.make(new Dog());
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn protocol_let_binding_reports_why_concrete_type_does_not_match() {
    let errors = semantic_errors(r#"
        protocol Printable {
            printSelf(): String;
        }

        type Box {
            value = 10;
        }

        let p: Printable = new Box() in print(p.printSelf());
    "#);

    assert_has_error(
        &errors,
        "let binding 'p' expects Printable, found Box; Box does not satisfy the requirements of Printable",
    );
}

#[test]
fn protocol_methods_must_be_fully_typed() {
    let errors = semantic_errors(r#"
        protocol Broken {
            hash(): Number;
            equals(other): Boolean;
        }
        0;
    "#);

    assert_has_error(&errors, "protocol method 'equals' parameter 'other' must be typed");
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
fn inherited_method_is_found_on_subtype_instances() {
    let errors = semantic_errors(r#"
        type A {
            c = 0;

            get_c() => self.c;
        }

        type Person(name, age) inherits A {
            name: String = name;
            age: Number = age;
        }

        let jery = new Person("Jery", 21) in
            print(jery.get_c());
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn inherited_method_override_with_same_signature_is_allowed() {
    let errors = semantic_errors(r#"
        type A {
            m(x: Number): Number => x + 1;
        }

        type B inherits A {
            m(x: Number): Number => x + 2;
        }

        print(0);
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn inherited_method_override_with_different_signature_reports_error() {
    let errors = semantic_errors(r#"
        type A {
            m(x: Number): Number => x + 1;
        }

        type B inherits A {
            m(x: String): String => x;
        }

        print(0);
    "#);

    assert_has_error(&errors, "must match inherited signature");
}

#[test]
fn inherited_type_reports_three_override_signature_mismatches() {
    let errors = semantic_errors(r#"
        type Parent {
            m1(x: Number): Number => x;
            m2(text: String): String => text;
            m3(flag: Boolean): Boolean => flag;
        }

        type Child inherits Parent {
            m1(x: String): Number => 0;
            m2(text: String): Number => 0;
            m3(flag: Number): Boolean => true;
        }

        0;
    "#);

    let mismatch_count = errors
        .iter()
        .filter(|error| error.message.contains("must match inherited signature"))
        .count();

    assert_eq!(
        mismatch_count, 3,
        "expected 3 inherited-signature mismatch errors, got: {:?}",
        errors
    );
}

#[test]
fn inherited_transitive_method_is_found_on_subtype_instances() {
    let errors = semantic_errors(r#"
        type B {
            d = 0;

            get_d() => self.d;
        }
        type A inherits B {
            c = 0;

            get_c() => self.c;
        }
        type Person(name, age) inherits A {
            name: String = name;
            age: Number = age;

            greet() => print("Hola, soy " @ self.name @ " y tengo " @ self.age @ " años");
            get_age() => self.age;
        }

        {
            let jery = new Person("Jery", 21) in
                print(jery.get_d());
        }
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn inferred_method_return_type_is_used_in_later_method_calls() {
    let errors = semantic_errors(r#"
        type A {
            value() => 1;

            twice() {
                self.value() + self.value()
            }
        }

        0;
    "#);
    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn method_return_type_mismatch_is_reported() {
    let errors = semantic_errors(r#"
        type A {
            m(): Number => "hola";
        }
        0;
    "#);

    assert_has_error(&errors, "method 'm' return type expects Number, found String");
}

#[test]
fn attribute_type_mismatch_is_reported() {
    let errors = semantic_errors(r#"
        type A {
        }

        type Person(name: String, age: Number) inherits A {
            name: String = age;
            age: Number = age;
        }

        0;
    "#);

    assert_has_error(&errors, "attribute 'name' expects String, found Number");
}

#[test]
fn factorial_example_return_type_mismatch_is_reported() {
    let errors = semantic_errors(r#"
        function factorial(n: Number, j: String): String {
            let result = 1, i = 1 in {
                while (i <= n) {
                    result := result * i;
                    i := i + 1;
                };
                result
            }
        }
        0;
    "#);

    assert_has_error(&errors, "function 'factorial' return type expects String, found Number");
}

#[test]
fn logical_and_requires_boolean_operands_number_left() {
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

// test removed: vector_comprehension_reports_multiple_semantic_errors

#[test]
fn sum_until_reports_multiple_semantic_errors() {
    let errors = semantic_errors(r#"
        function sum_until(max: Number): Number {
            let result = 0, i = "0" in (
                while (i < max) {
                    result := result + true;
                    i := i + 1;
                };
                result
            )
        }
        sum_until("10");
    "#);

    assert_has_error(&errors, "call to 'sum_until' argument 1 expects Number, found String");
    assert_has_error(&errors, "relational operator requires Number (left side: String)");
    assert_has_error(&errors, "arithmetic operator requires Number (right side: Boolean)");
    assert_has_error(&errors, "arithmetic operator requires Number (left side: String)");
}


#[test]
fn factorial_reports_multiple_semantic_errors() {
    let errors = semantic_errors(r#"
        function factorial(n: Number, j: String): Number {
            let result = 1, i = 1 in {
                while (i <= n) {
                    i := i + true;
                };
                result := result + "x";
                result
            }
        }
        if (factorial(1, 2) > 2 & 1) {
            print("Factorial of 1 is 1");
        } else {
            print("Error in factorial function");
        };
    "#);

    assert_has_error(&errors, "call to 'factorial' argument 2 expects String, found Number");
    assert_has_error(&errors, "arithmetic operator requires Number (right side: String)");
    assert_has_error(&errors, "arithmetic operator requires Number (right side: Boolean)");
    assert_has_error(&errors, "logical operator requires Boolean (right side: Number)");
}

#[test]
fn nested_condition_reports_multiple_semantic_errors() {
    let errors = semantic_errors(r#"
        let x: Number = 2, y: Number = 4 in (
            let b: String = "text", h: Boolean = true in
            if (false & 1) {
                1
            } elif (true | x) {
                2
            } elif (y > "10") {
                3
            } else {
                4
            };
        )
    "#);

    assert_has_error(&errors, "logical operator requires Boolean (right side: Number)");
    assert_has_error(&errors, "logical operator requires Boolean (right side: Number)");
    assert_has_error(&errors, "relational operator requires Number (right side: String)");
}


#[test]
fn self_valid_attribute_reference() {
    let errors = semantic_errors(r#"
    type Counter(n) {
        n = n;
        get() => self.n;
    }
    new Counter(0).get()
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn self_valid_method_call() {
    let errors = semantic_errors(r#"
    type Counter(n) {
        n = n;
        inc() => self.n + 1;
        double() => self.inc() * 2;
    }
    new Counter(3).double()
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn self_valid_as_argument_of_function() {
    let errors = semantic_errors(r#"
    function getId(c) => 42;
    type Counter(n) {
        n = n;
        id() => getId(self);
    }
    new Counter(0).id()
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn self_shadowed_by_parameter_of_method() {
    let errors = semantic_errors(r#"
    type Counter(n) {
        n = n;
        add(self) => self + 1;
    }
    new Counter(0).add(5)
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn self_shadowed_by_let_inside_method() {
    let errors = semantic_errors(r#"
    type Counter(n) {
        n = n;
        compute() => let self = 42 in self * 2;
    }
    new Counter(0).compute()
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn base_valid_calls_parent_method() {
    let errors = semantic_errors(r#"
    type Animal {
        name() => "Animal";
    }
    type Dog inherits Animal {
        name() => base() @ " Dog";
    }
    new Dog().name()
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn base_uses_parent_method_not_parent_constructor_arity() {
    let errors = semantic_errors(r#"
    type Person(firstname, lastname) {
        firstname = firstname;
        lastname = lastname;

        name() => self.firstname @@ self.lastname;
    }

    type Knight inherits Person {
        name() => "Sir" @@ base();
    }

    let p = new Knight("Phil", "Collins") in p.name();
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn base_valid_in_overridden_method_with_parameters() {
    let errors = semantic_errors(r#"
    type Person(firstname, lastname) {
        firstname = firstname;
        lastname = lastname;

        name(a: String, b: Number): String => self.firstname @@ self.lastname;
    }

    type Knight inherits Person {
        name(a: String, b: Number): String => "Sir" @@ base();
    }

    let p = new Knight("Phil", "Collins") in p.name("x", 1);
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn base_valid_in_overridden_method_with_diff_parameters() {
    let errors = semantic_errors(r#"
    type Person(firstname, lastname) {
        firstname = firstname;
        lastname = lastname;

        name(a: String, b: Number): String => self.firstname @@ self.lastname;
    }

    type Knight inherits Person {
        name(a: String, b: Boolean): String => "Sir" @@ base();
    }

    let p = new Knight("Phil", "Collins") in p.name("x", 1);
    "#);

    assert_has_error(&errors, "method 'name' in type 'Knight' must match inherited signature from 'Person'");
    assert_has_error(&errors, "method 'name' argument 2 expects Boolean, found Number");
}

#[test]
fn base_reports_signature_mismatch_on_overridden_method() {
    let errors = semantic_errors(r#"
    type Person(firstname, lastname) {
        firstname = firstname;
        lastname = lastname;

        name(a: String, b: Number): String => self.firstname @@ self.lastname;
    }

    type Knight inherits Person {
        name(b: Boolean): String => "Sir" @@ base();
    }

    let p = new Knight("Phil", "Collins") in p.name(true);
    "#);

    assert_has_error(
        &errors,
        "base method 'name' with arity 1 not defined on parent type 'Person' because overridden methods must keep the same signature",
    );
}

#[test]
fn protocols_and_colored_shapes_example() {
    let errors = semantic_errors(r#"
        protocol Shape {
            area() : Number;
            perimeter() : Number;
            describe() : String;
        }

        protocol ColoredShape extends Shape {
            color() : String;
        }

        type Rectangle(x, y) {
            width: Number = x;
            height: Number = y;

            area(): Number => self.width * self.height;
            perimeter(): Number => 2 * (self.width + self.height);
            describe(): String => "Rectángulo de " @ self.width @ " x " @ self.height;
        }

        type Square(side) {
            side: Number = side;

            area(): Number => self.side * self.side;
            perimeter(): Number => 4 * self.side;
            describe(): String => "Cuadrado de lado " @ self.side;
        }

        type Rhombus(side, d1, d2) {
            side: Number = side;
            d1: Number = d1;
            d2: Number = d2;

            area(): Number => (self.d1 * self.d2) / 2;
            perimeter(): Number => 4 * self.side;
            describe(): String => "Rombo de lado " @ self.side @ " y diagonales " @ self.d1 @ " y " @ self.d2;
        }

        type ColoredRectangle(width, height, c) {
            width: Number = width;
            height: Number = height;
            c: String = c;

            area(): Number => self.width * self.height;
            perimeter(): Number => 2 * (self.width + self.height);
            describe(): String => "Rectángulo de color " @ self.c;
            color(): String => self.c;
        }

        {
            let s1 : Shape = new Rectangle(3, 4) in {
                print(s1.describe() @ " | área = " @ s1.area() @ " | perímetro = " @ s1.perimeter());
            };

            let s2 : Shape = new Square(5) in {
                print(s2.describe() @ " | área = " @ s2.area() @ " | perímetro = " @ s2.perimeter());
            };

            let s3 : Shape = new Rhombus(4, 6, 8) in {
                print(s3.describe() @ " | área = " @ s3.area() @ " | perímetro = " @ s3.perimeter());
            };

            let cs : ColoredShape = new ColoredRectangle(2, 7, "azul") in {
                print(cs.describe() @ " | color = " @ cs.color());
            };
        }
    "#);

    assert!(errors.is_empty(), "expected no semantic errors, got: {:?}", errors);
}

#[test]
fn colored_shape_assignment_reports_missing_protocol_methods() {
    let errors = semantic_errors(r#"
        protocol Shape {
            area() : Number;
            perimeter() : Number;
            describe() : String;
        }

        protocol ColoredShape extends Shape {
            color() : String;
        }

        type ColoredRectangle(width, height, c) {
            width: Number = width;
            height: Number = height;
            c: String = c;

            area(): Number => self.width * self.height;
            describe(): String => "Rectángulo de color " @ self.c;
            perimeter() : => 2 * (self.width + self.height);
        }

        let cs : ColoredShape = new ColoredRectangle(2, 7, "azul") in {
            print(cs.describe() @ " | color = " @ cs.color());
        };
    "#);

    assert_has_error(
        &errors,
        "let binding 'cs' expects ColoredShape, found ColoredRectangle; ColoredRectangle does not satisfy the requirements of ColoredShape",
    );
}

#[test]
fn added_test_inherited_transitive_person_typed_constructor() {
    let errors = semantic_errors(r#"
        type B {
            d = 0;

            get_d() => self.d;
        }
        type A inherits B {
            c = 0;

            get_c() => self.c;
        }
        type Person(name: String, age: Number) inherits A {
            name: String = name;
            age: Number = age;

            greet() => print("Hola, soy " @ self.name @ " y tengo " @ self.age @ " años");
            get_age() => self.age;
        }

        {
            let jery = new Person("Jery", 21) in 
                print(jery.get_d());
        }
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_protocol_chain_is_valid() {
    let errors = semantic_errors(r#"
        protocol C {
            greet() : String;
        }

        protocol A extends C {
            hey() : String;
        }

        protocol B extends A {
            hello() : String;
        }
        print(42);
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_person_inherits_a_and_base_call_is_valid() {
    let errors = semantic_errors(r#"
        type A {
            x = 0;

            get_x() => self.x;
        }

        type Person(firstname, lastname) inherits A {
            firstname = firstname;
            lastname = lastname;

            num(a: Number): Number => a+1;
            hole() => "This is a hole in the Person type";
            name(a: String, b: Number): String => self.firstname @@ self.lastname;
        }

        type Knight inherits Person {
            name(a: String, b: Number): String => "Sir" @@ base();
        }

        let p : Person = new Knight("Phil", "Collins") in
            print(p.get_x());
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_greetable_protocol_assignment_is_valid() {
    let errors = semantic_errors(r#"
        protocol Greetable {
            greet() : String;
        }

        type Person(name) {
            name: String = name;

            greet(): String => "Hello, I am " @ self.name;
        }

        let p : Greetable = new Person("Alice") in print(p.greet());
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_function_with_let_binding_is_valid() {
    let errors = semantic_errors(r#"
        function g(a): Number => a+5;

        let b: Number = 4*2 in
            let a: Number = g(5) + b in {
                print(a);
            };
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_nested_let_if_elif_block_is_valid() {
    let errors = semantic_errors(r#"
        {
            let a = 42, let mod = a % 3, let b: Boolean = true in
                print(
                    if (mod == 0 & b) "Magic"
                    elif (mod % 3 == 1) "Woke"
                    else "Dumb"
                );

            let a: Number = 42, mod = a % 3, b = true in
                print(
                    if (mod == 0 & b) "Magic"
                    elif (mod % 3 == 1) "Woke"
                    else "Dumb"
                );


            let a = 42 in 
                let mod: Number = a % 3 in
                    let b = true in
                        print(
                            if (mod == 0 & b) "Magic"
                            elif (mod % 3 == 1) "Woke"
                            else "Dumb"
                        );
            
            let a = (let b = 6 in b * 7) in print(a);
        };
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_person_age_inheritance_is_valid() {
    let errors = semantic_errors(r#"
        type B {
            d = 0;

            get_d() => self.d;
        }
        type A inherits B {
            c = 0;

            get_c() => self.c;
        }
        type Person(name: String, age: Number) inherits A {
            name: String = name;
            age: Number = age;

            greet() => print("Hola, soy " @ self.name @ " y tengo " @ self.age @ " años");
            get_age() => self.age;
        }

        {
            let jery = new Person("Jery", 21) in 
                print(jery.get_d());
        }
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}


#[test]
fn main_sum_until_is_valid() {
    let errors = semantic_errors(r#"
        function sum_until(max : Number): Number {
            let result = 0, i = 0 in (
                while (i < max) {
                    result := result + i;
                    i := i + 1;
                };
                result
            )
        }
        print(sum_until(10));
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}


#[test]
fn main_factorial_reports_undefined_results() {
    let errors = semantic_errors(r#"
        function factorial(n: Number, j: String): Number {
            let result = 1, i = 1 in {
                while (i <= n) {
                    result := result * i;
                    i := i + 1;
                };
                results
            }
        }
        if (factorial (1, "testing_param") > 2 & true) {
            print("Factorial of 1 is 1");
        } else {
            print("Error in factorial function");
        };
    "#);

    assert_has_error(&errors, "identifier 'results' not defined");
}

#[test]
fn main_assignment_and_indexing_is_valid() {
    let errors = semantic_errors(r#"
        {
            let a = 10, c = 0 in {
                let b = 20 in {
                    a := a + b + c;
                    a
                }
            };
        }
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn main_function_chain_is_valid() {
    let errors = semantic_errors(r#"
        function f(a, b): Number { if (a > b) { a } else { b } }
        
        function g(): Number {
            let r = f(10, 20) in
            r
        }
        g();
    "#);

    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

// ============================================================================
// COMPREHENSIVE SEMANTIC ERROR TESTS
// Pruebas astutos que cubren múltiples errores semánticos a la vez
// ============================================================================

#[test]
fn redefinition_errors_function_and_type() {
    let errors = semantic_errors(r#"
        type MyType { value = 1; }
        type MyType { value = 2; }
        
        function helper(x): Number => x + 1;
        function helper(x, y): Number => x + y;
        
        print(1);
    "#);

    assert_has_error(&errors, "type or protocol 'MyType' already defined");
    assert_has_error(&errors, "function 'helper' already defined");
}

#[test]
fn redefinition_with_builtin_conflicts() {
    let errors = semantic_errors(r#"
        type Number { value = 1; }
        type Boolean inherits String { value = false; }
        
        function print(x) => x;
        function sin(a, b) => a + b;
        
        0;
    "#);

    assert_has_error(&errors, "builtin type 'Number' cannot be redefined");
    assert_has_error(&errors, "builtin type 'Boolean' cannot be redefined");
    assert_has_error(&errors, "builtin function 'print' cannot be redefined");
    assert_has_error(&errors, "builtin function 'sin' cannot be redefined");
}

#[test]
fn inheritance_chain_with_multiple_errors() {
    let errors = semantic_errors(r#"
        type A inherits B { value = 1; }
        type B inherits C { value = 2; }
        type C inherits A { value = 3; }
        
        type D inherits Fantasma { value = 4; }
        
        0;
    "#);

    assert_has_error(&errors, "type 'A' has cyclic inheritance");
    assert_has_error(&errors, "type 'B' has cyclic inheritance");
    assert_has_error(&errors, "type 'C' has cyclic inheritance");
    assert_has_error(&errors, "parent type 'Fantasma' not defined");
}

#[test]
fn type_method_override_parameter_type_mismatch() {
    let errors = semantic_errors(r#"
        type A {
            m(x: Number): Number => x + 1;
        }

        type B inherits A {
            m(x: String): Number => 42;
        }

        0;
    "#);

    assert_has_error(&errors, "must match inherited signature");
}

#[test]
fn type_method_override_return_type_mismatch() {
    let errors = semantic_errors(r#"
        type Vehicle {
            getSpeed(): Number => 100;
            getColor(): String => "red";
        }

        type Car inherits Vehicle {
            getSpeed(): String => "fast";
            getColor(): Boolean => true;
        }

        0;
    "#);

    assert_has_error(&errors, "must match inherited signature");
    assert_has_error(&errors, "must match inherited signature");
}

#[test]
fn protocol_hierarchy_with_multiple_extends() {
    let errors = semantic_errors(r#"
        protocol A extends B { m(): Number; }
        protocol B extends C { m(): Number; }
        protocol C extends A { m(): Number; }
        
        protocol D extends Undefined { n(): String; }
        
        0;
    "#);

    assert_has_error(&errors, "protocol 'A' has cyclic inheritance");
    assert_has_error(&errors, "protocol 'B' has cyclic inheritance");
    assert_has_error(&errors, "protocol 'C' has cyclic inheritance");
    assert_has_error(&errors, "parent protocol 'Undefined' not defined");
}

#[test]
fn complex_type_and_protocol_mixing_errors() {
    let errors = semantic_errors(r#"
        type Drawable {
            draw(): String => "drawing";
        }

        type Shape inherits Drawable {
            area(): Number => 0;
        }

        protocol Printable extends Drawable {
            print(): String;
        }

        0;
    "#);

    assert_has_error(&errors, "type 'Drawable' cannot be extended by a protocol");
}

#[test]
fn function_with_cascading_type_errors() {
    let errors = semantic_errors(r#"
        function compute(a: UndefinedType, b: AnotherUndefined): BadReturn {
            let x: NoType = a + b in
            let y: MissingType = "text" in
            x + y
        }

        compute(1, 2);
    "#);

    assert_has_error(&errors, "type 'UndefinedType' not defined");
    assert_has_error(&errors, "type 'AnotherUndefined' not defined");
    assert_has_error(&errors, "type 'BadReturn' not defined");
    assert_has_error(&errors, "type 'NoType' not defined");
    assert_has_error(&errors, "type 'MissingType' not defined");
}

#[test]
fn arithmetic_and_logical_mixed_type_errors() {
    let errors = semantic_errors(r#"
        {
            let a: Number = 5, b: String = "10" in {
                let c = a + b in {};
                let d = false & b in {};
                let e = "x" + true in {};
                let f = false | 42 in {};
                0
            };
        }
    "#);

    assert_has_error(&errors, "arithmetic operator requires Number (right side: String)");
    assert_has_error(&errors, "logical operator requires Boolean (right side: String)");
    assert_has_error(&errors, "arithmetic operator requires Number (right side: Boolean)");
    assert_has_error(&errors, "logical operator requires Boolean (right side: Number)");
}

#[test]
fn comparison_operators_with_mixed_types() {
    let errors = semantic_errors(r#"
        {
            let num: Number = 5 in {
                if (num > "10") { 1 } else { 2 };
                if ("hello" < num) { 1 } else { 2 };
                if (true >= 3) { 1 } else { 2 };
                if (false <= "text") { 1 } else { 2 };
                0
            };
        }
    "#);

    assert_has_error(&errors, "relational operator requires Number (right side: String)");
    assert_has_error(&errors, "relational operator requires Number (left side: String)");
    assert_has_error(&errors, "relational operator requires Number (left side: Boolean)");
    assert_has_error(&errors, "relational operator requires Number (right side: String)");
}

#[test]
fn equality_operators_with_different_types() {
    let errors = semantic_errors(r#"
        {
            let n = 10, s = "hi", b = true in {
                if (n == s) { 1 } else { 2 };
                if (s == b) { 1 } else { 2 };
                if (b != n) { 1 } else { 2 };
                0
            };
        }
    "#);

    assert_has_error(&errors, "equality operator requires operands of the same type (Number vs String)");
    assert_has_error(&errors, "equality operator requires operands of the same type (String vs Boolean)");
    assert_has_error(&errors, "equality operator requires operands of the same type (Boolean vs Number)");
}

#[test]
fn function_call_arity_errors_with_overloads() {
    let errors = semantic_errors(r#"
        function f(a: Number): Number => a + 1;
        function f(a: Number, b: Number): Number => a + b;
        
        {
            f(1, 2, 3);
            f("x", "y");
            0
        }
    "#);

    assert_has_error(&errors, "call to 'f' with invalid arity (3)");
    assert_has_error(&errors, "call to 'f' argument 1 expects Number, found String");
    assert_has_error(&errors, "call to 'f' argument 2 expects Number, found String");
}

#[test]
fn function_argument_type_mismatches_multiple_args() {
    let errors = semantic_errors(r#"
        function process(id: Number, name: String, active: Boolean): String {
            id @ name @ active
        }

        {
            process("invalid", 123, "wrong");
            process(1, 2, 3);
            process(true, false, null);
            0
        }
    "#);

    assert_has_error(&errors, "call to 'process' argument 1 expects Number, found String");
    assert_has_error(&errors, "call to 'process' argument 2 expects String, found Number");
    assert_has_error(&errors, "call to 'process' argument 3 expects Boolean, found String");
    assert_has_error(&errors, "call to 'process' argument 1 expects Number, found Boolean");
    assert_has_error(&errors, "call to 'process' argument 2 expects String, found Boolean");
    assert_has_error(&errors, "call to 'process' argument 3 expects Boolean, found Number");
}

#[test]
fn method_call_errors_in_type_hierarchy() {
    let errors = semantic_errors(r#"
        type Base {
            m1(x: Number): Number => x;
            m2(x: String): String => x;
        }

        type Child inherits Base {
            m1(x: Number): Number => x + 1;
            m3(y: Boolean): Boolean => y;
        }

        {
            let c = new Child() in {
                c.m1("wrong");
                c.m2(123);
                c.m3(456);
                c.undefined();
                0
            };
        }
    "#);

    assert_has_error(&errors, "method 'm1' argument 1 expects Number, found String");
    assert_has_error(&errors, "method 'm2' argument 1 expects String, found Number");
    assert_has_error(&errors, "method 'm3' argument 1 expects Boolean, found Number");
    assert_has_error(&errors, "method 'undefined' with arity 0 not defined");
}

#[test]
fn conditional_chain_type_errors() {
    let errors = semantic_errors(r#"
        {
            if (1) { "true" }
            elif ("text") { 2 }
            elif (3 > "x") { true }
            else { "false" };
            
            0
        }
    "#);

    assert_has_error(&errors, "if condition must be Boolean (found Number)");
    assert_has_error(&errors, "elif condition must be Boolean (found String)");
    assert_has_error(&errors, "relational operator requires Number (right side: String)");
}

#[test]
fn while_loop_condition_and_body_errors() {
    let errors = semantic_errors(r#"
        {
            let x: Number = 10, y: String = "hi" in {
                while (x) {
                    while ("text") {
                        x := x + y;
                        y := y + true;
                    };
                };
                0
            };
        }
    "#);

    assert_has_error(&errors, "while condition must be Boolean (found Number)");
    assert_has_error(&errors, "while condition must be Boolean (found String)");
    assert_has_error(&errors, "arithmetic operator requires Number (right side: String)");
    assert_has_error(&errors, "arithmetic operator requires Number (right side: Boolean)");
}

#[test]
fn for_loop_with_undefined_variable() {
    let errors = semantic_errors(r#"
        {
            for (item in undefined_var) {
                item := item + 1;
            };
            
            0
        }
    "#);

    assert_has_error(&errors, "identifier 'undefined_var' not defined");
}

#[test]
fn let_binding_cascading_errors() {
    let errors = semantic_errors(r#"
        {
            let a: BadType = someFunc(1, "x") in
            let b: String = a + 10 in
            let c: Number = b @ "text" in
            c;
        }
    "#);

    assert_has_error(&errors, "type 'BadType' not defined");
    assert_has_error(&errors, "function 'someFunc' not defined");
}

#[test]
fn self_and_base_misuse_outside_methods() {
    let errors = semantic_errors(r#"
        {
            let x = self in {};
            let y = base(1) in {};
            self.field;
            base();
            0
        }
    "#);

    assert_has_error(&errors, "use of self outside of a method");
    assert_has_error(&errors, "use of base outside of a method");
    assert_has_error(&errors, "use of self outside of a method");
    assert_has_error(&errors, "use of base outside of a method");
}

#[test]
fn base_call_errors_in_methods() {
    let errors = semantic_errors(r#"
        type A {
            m(x: Number): Number => x;
        }

        type B inherits A {
            m(x: Number): Number => base();
        }

        type C {
            n() => base(1, 2);
        }

        0;
    "#);

    assert_has_error(&errors, "base requires inheritance");
}

#[test]
fn assignment_to_undefined_and_self() {
    let errors = semantic_errors(r#"
        {
            let x = 5 in {
                undefined := 10;
                self := 20;
                x := x + 1;
            };
        }
    "#);

    assert_has_error(&errors, "assignment to undefined variable 'undefined'");
    assert_has_error(&errors, "cannot assign to 'self'");
}

#[test]
fn duplicate_parameters_in_functions_and_methods() {
    let errors = semantic_errors(r#"
        function f(a, a, a) => a;
        
        type T {
            m(x, x, y, y) => x + y;
        }

        protocol P {
            sig(a, b, a): Number;
        }

        0;
    "#);

    assert_has_error(&errors, "duplicate parameter 'a'");
    assert_has_error(&errors, "duplicate parameter 'x'");
    assert_has_error(&errors, "duplicate parameter 'y'");
    assert_has_error(&errors, "duplicate parameter 'a'");
}

#[test]
fn type_constructor_arity_errors() {
    let errors = semantic_errors(r#"
        type Person(name: String, age: Number) {
            name = name;
            age = age;
        }

        {
            new Person();
            new Person("Alice");
            new Person("Bob", 30, "extra");
            0
        }
    "#);

    assert_has_error(&errors, "type 'Person' requires 2 arguments");
    assert_has_error(&errors, "type 'Person' requires 2 arguments");
    assert_has_error(&errors, "type 'Person' requires 2 arguments");
}

#[test]
fn unary_operator_type_errors() {
    let errors = semantic_errors(r#"
        {
            let a = -"text" in {};
            let b = !"number" in {};
            let c = -(true & false) in {};
            let d = !("hello" @ "world") in {};
            0
        }
    "#);

    assert_has_error(&errors, "unary operator '-' requires Number");
    assert_has_error(&errors, "unary operator '!' requires Boolean");
}

#[test]
fn string_concatenation_type_errors() {
    let errors = semantic_errors(r#"
        {
            let a = true @ false in {};
            let b = 10 @@ 20 in {};
            0
        }
    "#);

    assert_has_error(&errors, "concatenation operator requires String");
    assert_has_error(&errors, "concatenation operator requires String or vectors");
}

#[test]
fn complex_nested_scope_errors() {
    let errors = semantic_errors(r#"
        {
            let x: Number = 5 in {
                let y: String = x in {
                    let z: Boolean = y @ "text" in {
                        z & undefined;
                    };
                };
            };
        }
    "#);

    assert_has_error(&errors, "let binding 'y' expects String, found Number");
    assert_has_error(&errors, "identifier 'undefined' not defined");
}

#[test]
fn protocol_method_signature_parameter_type_errors() {
    let errors = semantic_errors(r#"
        protocol Handler {
            handle(evt: BadEventType): String;
            process(data: UndefinedData): Result;
        }

        0;
    "#);

    assert_has_error(&errors, "type 'BadEventType' not defined");
    assert_has_error(&errors, "type 'UndefinedData' not defined");
    assert_has_error(&errors, "type 'Result' not defined");
}

#[test]
fn attribute_type_mismatch_in_inheritance() {
    let errors = semantic_errors(r#"
        type Vehicle {
            speed: Number = "slow";
            name: String = 100;
        }

        0;
    "#);

    assert_has_error(&errors, "attribute 'speed' expects Number, found String");
    assert_has_error(&errors, "attribute 'name' expects String, found Number");
}

#[test]
fn multiple_errors_in_let_bindings() {
    let errors = semantic_errors(r#"
        {
            let a: Number = "string", b = true in {
                let c: Boolean = 42 in {
                    c
                };
            };
        }
    "#);

    assert_has_error(&errors, "let binding 'a' expects Number, found String");
    assert_has_error(&errors, "let binding 'c' expects Boolean, found Number");
}

#[test]
fn empty_block_errors() {
    let errors = semantic_errors(r#"
        {
            {};
            0
        }
    "#);

    assert_has_error(&errors, "empty block");
}

#[test]
fn combined_field_and_method_access_errors() {
    let errors = semantic_errors(r#"
        type Data {
            value: Number = 10;
            getValue(): Number => self.value;
        }

        type Container {
            data: Data = new Data();
            
            test() => {
                self.notExists;
                self.noMethod();
                0
            };
        }

        0;
    "#);

    assert_has_error(&errors, "attribute 'notExists' not defined on current type");
    assert_has_error(&errors, "method 'noMethod' with arity 0 not defined on current type");
}

#[test]
fn protocol_unimplemented_methods_detection() {
    let errors = semantic_errors(r#"
        protocol Drawable {
            draw(): String;
            hide(): Boolean;
        }

        type Shape {
            draw(): String => "drawing";
        }

        {
            let s: Drawable = new Shape() in s.draw();
        }
    "#);

    assert_has_error(
        &errors,
        "let binding 's' expects Drawable, found Shape; Shape does not satisfy the requirements of Drawable",
    );
}

#[test]
fn arithmetic_with_untyped_variables() {
    let errors = semantic_errors(r#"
        {
            let a = 5, b = 10 in {
                let c = a + b in {
                    let d: String = c in {
                        let e = d + a in e;
                    };
                };
            };
        }
    "#);

    assert_has_error(&errors, "let binding 'd' expects String, found Number");
}

#[test]
fn parent_type_requires_correct_argument_count() {
    let errors = semantic_errors(r#"
        type Base(x, y, z) {
            x = x; y = y; z = z;
        }

        type Child1 inherits Base(1) {
            w = 0;
        }

        type Child2 inherits Base(1, 2, 3, 4) {
            w = 0;
        }

        0;
    "#);

    assert_has_error(&errors, "parent type 'Base' requires 3 arguments");
    assert_has_error(&errors, "parent type 'Base' requires 3 arguments");
}


