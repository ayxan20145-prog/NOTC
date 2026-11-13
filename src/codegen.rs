use std::collections::HashMap;

use crate::{
    ast::{Expression, Program, Statement},
    token::{Token, Type, Value},
};

pub fn compile(program: &Program) -> String {
    let mut assembly = String::new();
    let mut variables: HashMap<String, (i32, Type)> = HashMap::new();
    let mut offset = 0;

    for statement in &program.statements {
        match statement {
            Statement::Decleration { typee, name, .. } | Statement::Pop { typee, name } => {
                offset += 8;
                variables.insert(name.clone(), (offset, typee.clone()));
            }
            _ => {}
        }
    }

    let stack_size = offset;

    let strings = collect_strings(program);

    assembly.push_str("global _start\n");
    assembly.push_str("section .data\n");
    for (i, s) in strings.iter().enumerate() {
        let mut parts: Vec<String> = s.iter().map(|b| b.to_string()).collect();
        parts.push("0".to_string()); // null terminator
        assembly.push_str(&format!("str_{}:\n    db {}\n", i, parts.join(", ")));
    }

    assembly.push_str("section .text\n");
    assembly.push_str("_start:\n");
    assembly.push_str("    push rbp\n");
    assembly.push_str("    mov rbp, rsp\n");

    assembly.push_str(&format!("    sub rsp, {}\n", stack_size));

    for statement in &program.statements {
        match statement {
            Statement::Decleration { typee, name, value } => {
                compile_expression(&mut assembly, value, &variables, &strings);
                let (off, _) = &variables[name];
                match typee {
                    Type::Int => assembly.push_str(&format!("    mov [rbp - {}], rax\n", off)),
                    Type::Char => assembly.push_str(&format!("    mov [rbp - {}], al\n", off)),
                    Type::Str => assembly.push_str(&format!("    mov [rbp - {}], rax\n", off)),
                    Type::Bool => assembly.push_str(&format!("    mov [rbp - {}], al\n", off)),
                }
            }
            Statement::If {
                condition,
                body,
                else_body,
            } => todo!(),
            Statement::Print { value } => {
                compile_expression(&mut assembly, value, &variables, &strings);
                assembly.push_str("    mov rsi, rax\n");
                assembly.push_str("    call print\n");
            }
            Statement::Push { value } => {
                compile_expression(&mut assembly, value, &variables, &strings);
                assembly.push_str("    push rax\n");
            }
            Statement::Pop { typee, name } => {
                assembly.push_str("    pop rax\n");
                let (off, _) = &variables[name];
                match typee {
                    Type::Int => assembly.push_str(&format!("    mov [rbp - {}], rax\n", off)),
                    Type::Char => assembly.push_str(&format!("    mov [rbp - {}], al\n", off)),
                    Type::Str => assembly.push_str(&format!("    mov [rbp - {}], rax\n", off)),
                    Type::Bool => assembly.push_str(&format!("    mov [rbp - {}], al\n", off)),
                }
            }
        }
    }

    assembly.push_str("    mov rsp, rbp\n");
    assembly.push_str("    pop rbp\n");
    assembly.push_str("    mov rax, 60\n");
    assembly.push_str("    xor rdi, rdi\n");
    assembly.push_str("    syscall\n");

    assembly.push_str("print:\n");
    assembly.push_str(".loop:\n");
    assembly.push_str("    cmp byte [rsi], 0\n");
    assembly.push_str("    je .done\n");
    assembly.push_str("    mov rdx, 1\n");
    assembly.push_str("    mov rax, 1\n");
    assembly.push_str("    mov rdi, 1\n");
    assembly.push_str("    syscall\n");
    assembly.push_str("    inc rsi\n");
    assembly.push_str("    jmp .loop\n");
    assembly.push_str(".done:\n");
    assembly.push_str("    ret\n");

    assembly
}

fn compile_expression(
    assembly: &mut String,
    expr: &Expression,
    variables: &HashMap<String, (i32, Type)>,
    strings: &[Vec<u8>],
) {
    match expr {
        Expression::Value(Value::Str(bytes)) => {
            let i = strings.iter().position(|s| s == bytes).unwrap();
            assembly.push_str(&format!("    lea rax, [rel str_{}]\n", i));
        }
        Expression::Value(Value::Bool(b)) => {
            assembly.push_str(&format!("    mov rax, {}\n", if *b { 1 } else { 0 }));
        }
        Expression::Value(v) => {
            assembly.push_str(&format!("    mov rax, {}\n", v));
        }

        Expression::Variable(name) => {
            let (offset, typee) = variables
                .get(name)
                .unwrap_or_else(|| panic!("undefined variable: {}", name));
            match typee {
                Type::Int => assembly.push_str(&format!("    mov rax, [rbp - {}]\n", offset)),
                Type::Char => {
                    assembly.push_str(&format!("    movzx rax, byte [rbp - {}]\n", offset))
                }
                Type::Str => assembly.push_str(&format!("    mov rax, [rbp - {}]\n", offset)),
                Type::Bool => {
                    assembly.push_str(&format!("    movzx rax, byte [rbp - {}]\n", offset))
                }
            }
        }

        Expression::Operation(left, op, right) => {
            compile_expression(assembly, left, variables, strings);
            assembly.push_str("    push rax\n");
            compile_expression(assembly, right, variables, strings);
            assembly.push_str("    mov rcx, rax\n");
            assembly.push_str("    pop rax\n");

            match op {
                Token::Plus => assembly.push_str("    add rax, rcx\n"),
                Token::Minus => assembly.push_str("    sub rax, rcx\n"),
                Token::Star => assembly.push_str("    imul rax, rcx\n"),
                Token::Slash => {
                    assembly.push_str("    cqo\n");
                    assembly.push_str("    idiv rcx\n");
                }

                Token::EqualEqual => {
                    assembly.push_str("    cmp rax, rcx\n");
                    assembly.push_str("    sete al\n");
                    assembly.push_str("    movzx rax, al\n");
                }
                Token::NotEqual => {
                    assembly.push_str("    cmp rax, rcx\n");
                    assembly.push_str("    setne al\n");
                    assembly.push_str("    movzx rax, al\n");
                }
                Token::Less => {
                    assembly.push_str("    cmp rax, rcx\n");
                    assembly.push_str("    setl al\n");
                    assembly.push_str("    movzx rax, al\n");
                }
                Token::LessEqual => {
                    assembly.push_str("    cmp rax, rcx\n");
                    assembly.push_str("    setle al\n");
                    assembly.push_str("    movzx rax, al\n");
                }
                Token::Greater => {
                    assembly.push_str("    cmp rax, rcx\n");
                    assembly.push_str("    setg al\n");
                    assembly.push_str("    movzx rax, al\n");
                }
                Token::GreaterEqual => {
                    assembly.push_str("    cmp rax, rcx\n");
                    assembly.push_str("    setge al\n");
                    assembly.push_str("    movzx rax, al\n");
                }

                _ => unreachable!("non-operator token"),
            }
        }
    }
}

fn collect_strings(program: &Program) -> Vec<Vec<u8>> {
    let mut strings = Vec::new();

    for statement in &program.statements {
        match statement {
            Statement::Decleration {
                value: Expression::Value(Value::Str(bytes)),
                ..
            }
            | Statement::Print {
                value: Expression::Value(Value::Str(bytes)),
            }
            | Statement::Push {
                value: Expression::Value(Value::Str(bytes)),
            } => {
                if !strings.contains(bytes) {
                    strings.push(bytes.clone());
                }
            }
            _ => {}
        }
    }

    strings
}
