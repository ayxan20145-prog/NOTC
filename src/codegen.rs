use std::collections::HashMap;

use crate::{
    ast::{Expression, Program, Statement},
    token::{Token, Type, Value},
};

pub fn compile(program: &Program) -> String {
    let mut assembly = String::new();
    let mut variables: HashMap<String, (i32, Type)> = HashMap::new();
    let mut offset = 0;

    collect_variables(&program.statements, &mut variables, &mut offset);

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

    let mut label_id: usize = 0;
    for statement in &program.statements {
        compile_statement(
            &mut assembly,
            statement,
            &variables,
            &strings,
            &mut label_id,
        );
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

fn compile_statement(
    assembly: &mut String,
    statement: &Statement,
    variables: &HashMap<String, (i32, Type)>,
    strings: &[Vec<u8>],
    label_id: &mut usize,
) {
    match statement {
        Statement::Decleration { typee, name, value } => {
            compile_expression(assembly, value, &variables, &strings);
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
        } => {
            let id = *label_id;
            *label_id += 1;

            compile_expression(assembly, condition, variables, strings);

            assembly.push_str("     cmp rax, 0\n");
            assembly.push_str(&format!("     je .__else_{}\n", id));

            for stmt in body {
                compile_statement(assembly, stmt, variables, strings, label_id);
            }

            assembly.push_str(&format!("    jmp .__end_{}\n", id));

            assembly.push_str(&format!("    .__else_{}:\n", id));

            if let Some(else_body) = else_body {
                for stmt in else_body {
                    compile_statement(assembly, stmt, variables, strings, label_id);
                }
            }

            assembly.push_str(&format!("    .__end_{}:\n", id));
        }
        Statement::Print { value } => {
            compile_expression(assembly, value, &variables, &strings);
            assembly.push_str("    mov rsi, rax\n");
            assembly.push_str("    call print\n");
        }
        Statement::Push { value } => {
            compile_expression(assembly, value, &variables, &strings);
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
        Statement::While { condition, body } => {
            let id = *label_id;
            *label_id += 1;

            assembly.push_str(&format!("    .__while_start_{}:\n", id));

            compile_expression(assembly, condition, variables, strings);

            assembly.push_str("    cmp rax, 0\n");
            assembly.push_str(&format!("    je .__while_end_{}\n", id));

            for stmt in body {
                compile_statement(assembly, stmt, variables, strings, label_id);
            }

            assembly.push_str(&format!("    jmp .__while_start_{}\n", id));
            assembly.push_str(&format!("    .__while_end_{}:\n", id));
        }
    }
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

fn collect_variables(
    statements: &[Statement],
    variables: &mut HashMap<String, (i32, Type)>,
    offset: &mut i32,
) {
    for statement in statements {
        match statement {
            Statement::Decleration { typee, name, .. } | Statement::Pop { typee, name } => {
                *offset += 8;
                variables.insert(name.clone(), (*offset, typee.clone()));
            }
            Statement::If {
                body, else_body, ..
            } => {
                collect_variables(body, variables, offset);
                if let Some(else_body) = else_body {
                    collect_variables(else_body, variables, offset);
                }
            }
            Statement::While { body, .. } => {
                collect_variables(body, variables, offset);
            }
            _ => {}
        }
    }
}

fn collect_strings(program: &Program) -> Vec<Vec<u8>> {
    let mut strings = Vec::new();
    actually_collect_strings(&program.statements, &mut strings);
    strings
}

fn actually_collect_strings(statements: &[Statement], strings: &mut Vec<Vec<u8>>) {
    for statement in statements {
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
            Statement::If {
                body, else_body, ..
            } => {
                actually_collect_strings(body, strings);
                if let Some(else_body) = else_body {
                    actually_collect_strings(else_body, strings);
                }
            }
            Statement::While { body, .. } => {
                actually_collect_strings(body, strings);
            }
            _ => {}
        }
    }
}
