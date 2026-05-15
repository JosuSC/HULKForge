#![allow(dead_code)]

mod lexer;
mod parser;
mod evaluator;

use crate::lexer::lexer::TokenStream;
use crate::lexer::lexer::Span;
use crate::parser::{Parser, Expr};
use crate::parser::{Decl, FuncBody, FuncDecl, Param, Program, TypeExpr};

fn print_type_expr(ty: &TypeExpr, indent: usize) {
	let pad = " ".repeat(indent);
	match ty {
		TypeExpr::Named(name) => println!("{}Type::Named({})", pad, name),
		TypeExpr::Iterable(inner) => {
			println!("{}Type::Iterable", pad);
			print_type_expr(inner, indent + 2);
		}
		TypeExpr::Vector(inner) => {
			println!("{}Type::Vector", pad);
			print_type_expr(inner, indent + 2);
		}
		TypeExpr::Functor { params, returns } => {
			println!("{}Type::Functor", pad);
			println!("{}  params", pad);
			for param in params {
				print_type_expr(param, indent + 4);
			}
			println!("{}  returns", pad);
			print_type_expr(returns, indent + 4);
		}
	}
}

fn print_span(span: Span, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}span: {}", pad, span);
}

fn print_type_decl(ty: &crate::parser::TypeDecl, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}TypeDecl", pad);
	println!("{}  name: {}", pad, ty.name);
	if !ty.type_params.is_empty() {
		println!("{}  type_params:", pad);
		for p in &ty.type_params {
			print_param(p, indent + 4);
		}
	} else {
		println!("{}  type_params: []", pad);
	}
	match &ty.inherits {
		Some(ic) => {
			println!("{}  inherits:", pad);
			println!("{}    parent: {}", pad, ic.parent);
			if !ic.args.is_empty() {
				println!("{}    args:", pad);
				for a in &ic.args {
					print_expr(a, indent + 6);
				}
			} else {
				println!("{}    args: []", pad);
			}
			print_span(ic.span, indent + 4);
		}
		None => println!("{}  inherits: None", pad),
	}
	println!("{}  members:", pad);
	for m in &ty.members {
		match m {
			crate::parser::TypeMember::Attribute(a) => {
				println!("{}    Attribute", pad);
				println!("{}      name: {}", pad, a.name);
				match &a.ty {
					Some(t) => {
						println!("{}      ty:", pad);
						print_type_expr(t, indent + 8);
					}
					None => println!("{}      ty: None", pad),
				}
				println!("{}      init:", pad);
				print_expr(&a.init, indent + 8);
				print_span(a.span, indent + 6);
			}
			crate::parser::TypeMember::Method(mdef) => {
				println!("{}    Method", pad);
				print_method_def(mdef, indent + 6);
			}
		}
	}
	print_span(ty.span, indent + 2);
}

fn print_method_def(m: &crate::parser::MethodDef, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}MethodDef", pad);
	println!("{}  name: {}", pad, m.name);
	println!("{}  params:", pad);
	for p in &m.params {
		print_param(p, indent + 4);
	}
	match &m.return_type {
		Some(t) => {
			println!("{}  return_type:", pad);
			print_type_expr(t, indent + 4);
		}
		None => println!("{}  return_type: None", pad),
	}
	println!("{}  body:", pad);
	print_func_body(&m.body, indent + 4);
	print_span(m.span, indent + 2);
}

fn print_protocol_decl(p: &crate::parser::ProtocolDecl, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}ProtocolDecl", pad);
	println!("{}  name: {}", pad, p.name);
	println!("{}  extends: {}", pad, p.extends.as_deref().unwrap_or("None"));
	println!("{}  methods:", pad);
	for m in &p.methods {
		println!("{}    MethodSig", pad);
		println!("{}      name: {}", pad, m.name);
		println!("{}      params:", pad);
		for sp in &m.params {
			println!("{}        SigParam", pad);
			println!("{}          name: {}", pad, sp.name);
			match &sp.ty {
				Some(t) => {
					println!("{}          ty:", pad);
					print_type_expr(t, indent + 12);
				}
				None => println!("{}          ty: None", pad),
			}
			print_span(sp.span, indent + 10);
		}
		println!("{}      return_type:", pad);
		print_type_expr(&m.return_type, indent + 6);
		print_span(m.span, indent + 4);
	}
	print_span(p.span, indent + 2);
}

fn print_macro_decl(m: &crate::parser::MacroDecl, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}MacroDecl", pad);
	println!("{}  name: {}", pad, m.name);
	println!("{}  params:", pad);
	for mp in &m.params {
		match mp {
			crate::parser::MacroParam::Regular(p) => {
				println!("{}    Regular", pad);
				print_param(p, indent + 6);
			}
			crate::parser::MacroParam::Block { name, ty, span } => {
				println!("{}    Block name: {}", pad, name);
				println!("{}      ty:", pad);
				print_type_expr(ty, indent + 8);
				print_span(*span, indent + 6);
			}
			crate::parser::MacroParam::Symbolic { name, ty, span } => {
				println!("{}    Symbolic name: {}", pad, name);
				println!("{}      ty:", pad);
				print_type_expr(ty, indent + 8);
				print_span(*span, indent + 6);
			}
			crate::parser::MacroParam::Placeholder { name, ty, span } => {
				println!("{}    Placeholder name: {}", pad, name);
				println!("{}      ty:", pad);
				print_type_expr(ty, indent + 8);
				print_span(*span, indent + 6);
			}
		}
	}
	println!("{}  body:", pad);
	print_func_body(&m.body, indent + 4);
	print_span(m.span, indent + 2);
}

fn print_param(param: &Param, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}Param", pad);
	println!("{}  name: {}", pad, param.name);
	match &param.ty {
		Some(ty) => {
			println!("{}  ty:", pad);
			print_type_expr(ty, indent + 2);
		}
		None => println!("{}  ty: None", pad),
	}
	println!("{}  span: {}", pad, param.span);
}

fn print_func_body(body: &FuncBody, indent: usize) {
	let pad = " ".repeat(indent);
	match body {
		FuncBody::Inline(expr) => {
			println!("{}FuncBody::Inline", pad);
			print_expr(expr, indent + 2);
		}
		FuncBody::Block(expr) => {
			println!("{}FuncBody::Block", pad);
			print_expr(expr, indent + 2);
		}
	}
}

fn print_func_decl(func: &FuncDecl, indent: usize) {
	let pad = " ".repeat(indent);
	println!("{}FuncDecl", pad);
	println!("{}  name: {}", pad, func.name);
	println!("{}  params:", pad);
	for param in &func.params {
		print_param(param, indent + 4);
	}
	match &func.return_type {
		Some(ty) => {
			println!("{}  return_type:", pad);
			print_type_expr(ty, indent + 4);
		}
		None => println!("{}  return_type: None", pad),
	}
	println!("{}  body:", pad);
	print_func_body(&func.body, indent + 4);
	println!("{}  span: {}", pad, func.span);
}

fn print_decl(decl: &Decl, indent: usize) {
	let pad = " ".repeat(indent);
	match decl {
		Decl::Function(func) => {
			println!("{}Decl::Function", pad);
			print_func_decl(func, indent + 2);
		}
		Decl::Type(t) => {
			println!("{}Decl::Type", pad);
			print_type_decl(t, indent + 2);
		}
		Decl::Protocol(p) => {
			println!("{}Decl::Protocol", pad);
			print_protocol_decl(p, indent + 2);
		}
		Decl::Macro(m) => {
			println!("{}Decl::Macro", pad);
			print_macro_decl(m, indent + 2);
		}
	}
}

fn print_program(program: &Program) {
	println!("Program");
	println!("  decls:");
	for decl in &program.decls {
		print_decl(decl, 4);
	}
	println!("  expr:");
	print_expr(&program.expr, 4);
	println!("  span: {}", program.span);
}

fn print_expr(expr: &Expr, indent: usize) {
	let pad = " ".repeat(indent);
	match expr {
		Expr::Number { value, span } => { println!("{}Number({})", pad, value); print_span(*span, indent + 2); }
		Expr::StringLit { value, span } => { println!("{}String(\"{}\")", pad, value); print_span(*span, indent + 2); }
		Expr::Bool { value, span } => { println!("{}Bool({})", pad, value); print_span(*span, indent + 2); }
		Expr::Ident { name, span } => { println!("{}Ident({})", pad, name); print_span(*span, indent + 2); }
		Expr::Call { callee, args, span } => {
			println!("{}Call", pad);
			print_span(*span, indent + 2);
			print_expr(callee, indent + 2);
			for arg in args { print_expr(arg, indent + 2); }
		}
		Expr::MethodCall { object, method, args, span } => {
			println!("{}MethodCall {}(...)", pad, method);
			print_span(*span, indent + 2);
			print_expr(object, indent + 2);
			for arg in args { print_expr(arg, indent + 2); }
		}
		Expr::FieldAccess { object, field, span } => {
			println!("{}FieldAccess {}", pad, field);
			print_span(*span, indent + 2);
			print_expr(object, indent + 2);
		}
		Expr::New { type_name, args, span } => {
			println!("{}New {}(...)", pad, type_name);
			print_span(*span, indent + 2);
			for arg in args { print_expr(arg, indent + 2); }
		}
		Expr::SelfRef { span } => { println!("{}SelfRef", pad); print_span(*span, indent + 2); }
		Expr::Base { args, span } => {
			println!("{}Base", pad);
			print_span(*span, indent + 2);
			for arg in args { print_expr(arg, indent + 2); }
		}
		Expr::BinaryOp { op, left, right, span } => {
			println!("{}BinaryOp({:?})", pad, op);
			print_span(*span, indent + 2);
			print_expr(left, indent + 2);
			print_expr(right, indent + 2);
		}
		Expr::UnaryOp { op, operand, span } => {
			println!("{}UnaryOp({:?})", pad, op);
			print_span(*span, indent + 2);
			print_expr(operand, indent + 2);
		}
		Expr::IsType { expr: e, ty, span } => {
			println!("{}IsType", pad);
			print_span(*span, indent + 2);
			println!("{}  ty:", pad);
			print_type_expr(ty, indent + 4);
			println!("{}  expr:", pad);
			print_expr(e, indent + 4);
		}
		Expr::AsType { expr: e, ty, span } => {
			println!("{}AsType", pad);
			print_span(*span, indent + 2);
			println!("{}  ty:", pad);
			print_type_expr(ty, indent + 4);
			println!("{}  expr:", pad);
			print_expr(e, indent + 4);
		}
		Expr::If { condition, then_expr, elif_branches, else_expr, span } => {
			println!("{}If", pad);
			print_span(*span, indent + 2);
			println!("{}  condition:", pad);
			print_expr(condition, indent + 4);
			println!("{}  then:", pad);
			print_expr(then_expr, indent + 4);
			if !elif_branches.is_empty() {
				println!("{}  elif_branches:", pad);
				for eb in elif_branches { print_expr(&eb.condition, indent + 4); print_expr(&eb.body, indent + 4); print_span(eb.span, indent + 4); }
			}
			println!("{}  else:", pad);
			print_expr(else_expr, indent + 4);
		}
		Expr::While { condition, body, span } => {
			println!("{}While", pad);
			print_span(*span, indent + 2);
			println!("{}  condition:", pad);
			print_expr(condition, indent + 4);
			println!("{}  body:", pad);
			print_expr(body, indent + 4);
		}
		Expr::For { var, iterable, body, span } => {
			println!("{}For var={}", pad, var);
			print_span(*span, indent + 2);
			println!("{}  iterable:", pad);
			print_expr(iterable, indent + 4);
			println!("{}  body:", pad);
			print_expr(body, indent + 4);
		}
		Expr::Let { bindings, body, span } => {
			println!("{}Let", pad);
			print_span(*span, indent + 2);
			println!("{}  bindings:", pad);
			for b in bindings {
				println!("{}    LetBinding name: {}", pad, b.name);
				match &b.ty { Some(t) => { println!("{}      ty:", pad); print_type_expr(t, indent + 8); } None => println!("{}      ty: None", pad) }
				println!("{}      init:", pad);
				print_expr(&b.init, indent + 8);
				print_span(b.span, indent + 6);
			}
			println!("{}  body:", pad);
			print_expr(body, indent + 4);
		}
		Expr::Assign { target, value, span } => {
			println!("{}Assign", pad);
			print_span(*span, indent + 2);
			println!("{}  target:", pad);
			print_expr(target, indent + 4);
			println!("{}  value:", pad);
			print_expr(value, indent + 4);
		}
		Expr::Block { exprs, span } => {
			println!("{}Block", pad);
			print_span(*span, indent + 2);
			for e in exprs { print_expr(e, indent + 2); }
		}
		Expr::VectorLit { elements, span } => {
			println!("{}VectorLit", pad);
			print_span(*span, indent + 2);
			for e in elements { print_expr(e, indent + 2); }
		}
		Expr::VectorGen { element, var, iterable, span } => {
			println!("{}VectorGen var={}", pad, var);
			print_span(*span, indent + 2);
			println!("{}  element:", pad);
			print_expr(element, indent + 4);
			println!("{}  iterable:", pad);
			print_expr(iterable, indent + 4);
		}
		Expr::Index { object, index, span } => {
			println!("{}Index", pad);
			print_span(*span, indent + 2);
			println!("{}  object:", pad);
			print_expr(object, indent + 4);
			println!("{}  index:", pad);
			print_expr(index, indent + 4);
		}
		Expr::Lambda { params, return_type, body, span } => {
			println!("{}Lambda", pad);
			print_span(*span, indent + 2);
			println!("{}  params:", pad);
			for p in params { print_param(p, indent + 4); }
			match return_type { Some(ty) => { println!("{}  return_type:", pad); print_type_expr(ty, indent + 4); } None => println!("{}  return_type: None", pad) }
			println!("{}  body:", pad);
			print_func_body(body, indent + 4);
		}
	}
}

fn test_expression(src: &str) {
	println!("\n=== Test: Expression ===");
	println!("Source: {}", src);

	// Tokenization
	let (tokens, lex_errors) = TokenStream::tokenize_all(src);
	println!("\nTokens:");
	for t in &tokens {
		println!("  {:?} -> {}", t.token, t.span);
	}
	if !lex_errors.is_empty() {
		println!("\nLexer Errors:");
		for e in &lex_errors {
			println!("  {}", e);
		}
	}

	// Parse
	let ts = TokenStream::new(src);
	let mut parser = Parser::new(ts);
	match parser.parse_expr() {
		Some(expr) => {
			println!("\nAST:");
			print_expr(&expr, 0);
		}
		None => {
			println!("\nParser Error:");
			for e in parser.errors {
				println!("  {}", e);
			}
		}
	}
}

fn test_program(src: &str) {
	println!("\n=== Test: Program ===");
	println!("Source: {}", src);

	let (tokens, lex_errors) = TokenStream::tokenize_all(src);
	println!("\nTokens:");
	for t in &tokens {
		println!("  {:?} -> {}", t.token, t.span);
	}
	if !lex_errors.is_empty() {
		println!("\nLexer Errors:");
		for e in &lex_errors {
			println!("  {}", e);
		}
	}

	let ts = TokenStream::new(src);
	let mut parser = Parser::new(ts);
	match parser.parse_program() {
		Some(program) => {
			println!("\nAST:");
			print_program(&program);
		}
		None => {
			println!("\nParser Error:");
			for e in parser.errors {
				println!("  {}", e);
			}
		}
	}
}

fn main() {
    
	test_program(r#"
        type A {
            value: Number = 10;
            getValue() => self.value;
            inc() => {
                self.value := self.value + 1;
                self.value
            };
        }

        type Person(name: String, age: Number) inherits A {
            name: String = name;
            age: Number = age;

            getName() => self.name;

            birthday() => {
                self.age := self.age + 1;
                self.age
            };

            isAdult() => self.age >= 18;
        }

        function greet(p: Person) => {
            print("Hola " @@ p.getName());
            p.getValue()
        };

        function makePeople(n: Number) => {
            let result: Person[] = [] in
            for (i in range(0, n)) {
                let p = new Person("User" @@ i, i + 10) in {
                    result := result @@ [p];
                };
            };
            result
        };

        protocol Printable {
            printSelf(x): String;
        }

        type Box(value: Number) {
            value: Number = value;

            double() => self.value * 2;
        }
	
        let xs = [1,2,3,4,5] in
        let ys = [x*2 | x in xs] in
        let p = new Person("Jery", 25) in
        let b = new Box(99) in
        {
            print("Adult? " @@ (if (p.isAdult()) "yes" else "no"));

            let i = 0 in
            while (i < 3) {
                print("Loop: " @@ i);
                i := i + 1;
            };

            print("People list:");
            let ps = makePeople(5) in
            for (q in ps) {
                print(q.getName() @@ " age=" @@ q.age);
            };

            print("Box double: " @@ b.double());

            if (b is Box) {
                let bb: Box = b as Box in {
                    print("Downcast ok: " @@ bb.value);
                }
            } else {
    			"No box"		
			};

            let f = (x: Number, y: Number) -> Number => x + y in
            print("Functor sum: " @@ f(10, 20));

        }
    "#);

    // test_program(r#"
    //     function sum_until(max : Number): Number => {
    //         let result = 0, i = 0 in
    //         while (i < max) {
    //             result := result + i;
    //             i := i + 1;
    //         };
    //         result
    //     };
    // "#);

    // test_program(r#"
    //     function sum_vec(v): Number => {
    //         let total = 0 in
    //         for (i in v) {
    //             if (i < 0) {
    //                 total := total + (0 - i);
    //             } elif (i == 0) {
    //                 total := total + 0;
    //             } else {
    //                 total := total + i;
    //             };
    //         };
    //         total
    //     };
    // "#);

    // test_program(r#"
    //     function factorial(n: Number): Number => {
    //         let result = 1, i = 1 in {
    //             while (i <= n) {
    //                 result := result * i;
    //                 i := i + 1;
    //             };
    //             result
    //         }
    //     };
    // "#);

    // test_program(r#"
    //     let evens = [ x * 2 | x in [1, 2, 3, 4, 5] ];
    //     evens;
    // "#);

    // test_program(r#"
    //     if (true) {
    //         1
    //     } elif (false) {
    //         2
    //     } else {
    //         3
    //     };
    // "#);

    // test_program(r#"
    //     let a = 10 in {
    //         let b = 20 in {
    //             a := a + b;
    //             a
    //         }
    //     };
    // "#);

    // test_program(r#"
    //     function make_adder(n): Function => {
    //         function (x): Number => { x + n }
    //     };
    //     make_adder(5)(3);
    // "#);

    // test_program(r#"
    //     let v = [1, 2, 3, 4] in v[2];
    // "#);

    // test_program(r#"
    //     function f(a, b): Number => { if (a > b) { a } else { b } };
    //     function g(): Number => {
    //         let r = f(10, 20) in
    //         r
    //     };
    //     g();
    // "#);

    // test_program(r#"
    //     { let x = 1 in { x := x + 1; x } };
    // "#);

    // test_program(r#"
    //     let s = "hello" in {
    //         s
    //     };
    // "#);

    // test_program(r#"
    //     function nested(a) : Number => {
    //         let sum = 0 in
    //         for (i in a) {
    //             for (j in i) {
    //                 if (j % 2 == 0) { sum := sum + j } else { sum := sum + 0 };
    //             };
    //         };
    //         sum
    //     };
	// "#);
}

