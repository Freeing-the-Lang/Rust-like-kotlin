use crate::semantic::*;
use crate::parser::TypeName;

use std::collections::HashMap;

pub struct Codegen {
    label_count: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self { label_count: 0 }
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.label_count += 1;
        format!("{}_{}", prefix, self.label_count)
    }

    pub fn gen_program(&mut self, program: &IRProgram) -> String {
        let mut out = String::new();

        out.push_str("global _start\n");
        out.push_str("section .text\n\n");

        // find main function
        if !program.funcs.iter().any(|f| f.name == "main") {
            panic!("No main() function found");
        }

        // entrypoint → call main → exit
        out.push_str("_start:\n");
        out.push_str("    call main\n");
        out.push_str("    mov rax, 60\n");
        out.push_str("    xor rdi, rdi\n");
        out.push_str("    syscall\n\n");

        // generate all functions
        for f in &program.funcs {
            out.push_str(&self.gen_function(f));
        }

        // data section
        out.push_str("section .data\n");
        out.push_str("__print_newline: db 10\n");

        // embed string literals
        out.push_str("\nsection .rodata\n");

        // already collected inside gen_function
        out
    }

    fn gen_function(&mut self, f: &IRFunction) -> String {
        let mut out = String::new();

        out.push_str(&format!("{}:\n", f.name));

        out.push_str("    push rbp\n");
        out.push_str("    mov rbp, rsp\n");

        // allocate stack
        // gather all vars from body
        let mut locals: HashMap<String, i64> = HashMap::new();
        let mut offset: i64 = -8;

        // walk IR to find all vars
        collect_locals(&f.body, &mut locals, &mut offset);

        if offset != -8 {
            let size = (-offset - 8);
            out.push_str(&format!("    sub rsp, {}\n", size));
        }

        // generate body
        for ins in &f.body {
            out.push_str(&self.gen_ir(ins, &locals));
        }

        // ensure end with return if missing
        out.push_str("    mov rsp, rbp\n");
        out.push_str("    pop rbp\n");
        out.push_str("    ret\n\n");

        out
    }

    fn gen_ir(&mut self, ir: &IR, locals: &HashMap<String, i64>) -> String {
        match ir {
            IR::StoreVar(name, expr) => {
                let val = self.gen_expr(expr, locals);
                format!(
                    "{}    mov [rbp{}], rax\n",
                    val,
                    locals_offset(name, locals)
                )
            }

            IR::Return(expr) => {
                let val = self.gen_expr(expr, locals);
                format!(
                    "{}    mov rsp, rbp\n    pop rbp\n    ret\n",
                    val
                )
            }

            IR::If(cond, then_ir, else_ir) => {
                let cond_code = self.gen_expr(cond, locals);

                let l_else = self.new_label("else");
                let l_end = self.new_label("endif");

                let mut out = String::new();
                out.push_str(&cond_code);

                out.push_str("    cmp rax, 0\n");
                out.push_str(&format!("    je {}\n", l_else));

                for x in then_ir {
                    out.push_str(&self.gen_ir(x, locals));
                }
                out.push_str(&format!("    jmp {}\n", l_end));

                out.push_str(&format!("{}:\n", l_else));
                for x in else_ir {
                    out.push_str(&self.gen_ir(x, locals));
                }
                out.push_str(&format!("{}:\n", l_end));
                out
            }

            IR::LoadVar(name) => {
                format!("    mov rax, [rbp{}]\n", locals_offset(name, locals))
            }

            _ => {
                "// unsupported IR\n".to_string()
            }
        }
    }

    fn gen_expr(&mut self, expr: &IRExpr, locals: &HashMap<String, i64>) -> String {
        match expr {
            IRExpr::Int(v) => format!("    mov rax, {}\n", v),
            IRExpr::Str(s) => self.gen_string_literal(s),
            IRExpr::Var(name) => format!("    mov rax, [rbp{}]\n", locals_offset(name, locals)),

            IRExpr::Binary(a, op, b) => {
                let mut out = String::new();
                out.push_str(&self.gen_expr(a, locals));
                out.push_str("    push rax\n");
                out.push_str(&self.gen_expr(b, locals));
                out.push_str("    pop rcx\n"); // left in rcx, right in rax

                match op.as_str() {
                    "+" => out.push_str("    add rax, rcx\n"),
                    "-" => out.push_str("    sub rcx, rax\n    mov rax, rcx\n"),
                    "*" => out.push_str("    imul rax, rcx\n"),
                    "/" => {
                        out.push_str("    mov rdx, 0\n");
                        out.push_str("    mov rbx, rax\n");
                        out.push_str("    mov rax, rcx\n");
                        out.push_str("    div rbx\n");
                    }
                    ">" => {
                        out.push_str("    cmp rcx, rax\n    setg al\n    movzx rax, al\n");
                    }
                    "<" => {
                        out.push_str("    cmp rcx, rax\n    setl al\n    movzx rax, al\n");
                    }
                    "==" => {
                        out.push_str("    cmp rcx, rax\n    sete al\n    movzx rax, al\n");
                    }
                    "!=" => {
                        out.push_str("    cmp rcx, rax\n    setne al\n    movzx rax, al\n");
                    }
                    _ => panic!("Unsupported op {}", op),
                }

                out
            }

            IRExpr::Call(name, args) => {
                if name == "print" {
                    return self.gen_print(args, locals);
                }

                let mut out = String::new();
                for arg in args {
                    out.push_str(&self.gen_expr(arg, locals));
                    out.push_str("    push rax\n");
                }

                // pop args in reverse order into registers
                let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for i in (0..args.len()).rev() {
                    out.push_str(&format!("    pop {}\n", regs[i]));
                }

                out.push_str(&format!("    call {}\n", name));
                out
            }
        }
    }

    fn gen_string_literal(&mut self, s: &str) -> String {
        let label = self.new_label("str");
        format!(
            "section .rodata\n{}: db \"{}\", 0\nsection .text\n    lea rax, [{}]\n",
            label, s, label
        )
    }

    fn gen_print(&mut self, args: &Vec<IRExpr>, locals: &HashMap<String, i64>) -> String {
        let mut out = String::new();

        if args.len() != 1 {
            panic!("print() requires 1 argument");
        }

        let arg = &args[0];
        let label = self.new_label("print_string");

        match arg {
            IRExpr::Str(s) => {
                out.push_str(&format!(
                    "section .rodata\n{}: db \"{}\", 10, 0\nsection .text\n",
                    label, s
                ));
                out.push_str(&format!("    lea rsi, [{}]\n", label));
                out.push_str("    mov rdi, 1\n");
                out.push_str("    mov rdx, 64\n");
                out.push_str("    mov rax, 1\n");
                out.push_str("    syscall\n");
                out
            }
            other => {
                out.push_str(&self.gen_expr(other, locals));
                out.push_str("    // TODO: int printing\n");
                out
            }
        }
    }
}

fn collect_locals(body: &Vec<IR>, locals: &mut HashMap<String, i64>, offset: &mut i64) {
    for ins in body {
        match ins {
            IR::StoreVar(name, _) => {
                if !locals.contains_key(name) {
                    locals.insert(name.clone(), *offset);
                    *offset -= 8;
                }
            }
            IR::If(_, then_ir, else_ir) => {
                collect_locals(then_ir, locals, offset);
                collect_locals(else_ir, locals, offset);
            }
            _ => {}
        }
    }
}

fn locals_offset(name: &str, locals: &HashMap<String, i64>) -> String {
    if let Some(v) = locals.get(name) {
        format!("{}", v)
    } else {
        panic!("Unknown local variable {}", name);
    }
}
