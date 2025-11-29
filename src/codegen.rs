use crate::semantic::*;
use std::fmt::Write;

pub struct Codegen;

#[cfg(target_os = "windows")]
const ENTRY: &str = "main";

#[cfg(not(target_os = "windows"))]
const ENTRY: &str = "_main";

impl Codegen {
    pub fn generate(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // -------------------------
        // .data (문자열 리터럴)
        // -------------------------
        writeln!(&mut out, "section .data").unwrap();

        let mut strs = Vec::new();
        for f in &ir.funcs {
            for stmt in &f.body {
                self.collect_str(stmt, &mut strs);
            }
        }

        for (i, s) in strs.iter().enumerate() {
            writeln!(&mut out, "str_{}: db \"{}\", 0", i, s).unwrap();
            writeln!(&mut out, "str_{}_len: equ $ - str_{}", i, i).unwrap();
        }

        // -------------------------
        // .text
        // -------------------------
        writeln!(&mut out, "section .text").unwrap();
        writeln!(&mut out, "global {}", ENTRY).unwrap();

        for f in &ir.funcs {
            writeln!(&mut out, "global {}_func", f.name).unwrap();
            writeln!(&mut out, "global {}_func_end", f.name).unwrap();
        }

        for f in &ir.funcs {
            self.gen_function(&mut out, f, &strs);
        }

        writeln!(&mut out, "{}:", ENTRY).unwrap();
        writeln!(&mut out, "    call main_func").unwrap();
        writeln!(&mut out, "    ret").unwrap();

        out
    }

    fn collect_str(&self, stmt: &IR, out: &mut Vec<String>) {
        match stmt {
            IR::Println(IRExpr::Str(s)) => out.push(s.clone()),
            _ => {}
        }
    }

    fn gen_function(&self, out: &mut String, f: &IRFunction, strs: &Vec<String>) {
        writeln!(out, "{}_func:", f.name).unwrap();

        for stmt in &f.body {
            self.gen_stmt(out, stmt, strs);
        }

        writeln!(out, "{}_func_end:", f.name).unwrap();
        writeln!(out, "    ret").unwrap();
    }

    fn gen_stmt(&self, out: &mut String, stmt: &IR, strs: &Vec<String>) {
        match stmt {
            IR::Return(expr) => {
                self.gen_expr(out, expr, strs);
                writeln!(out, "    ret").unwrap();
            }

            IR::Println(expr) => self.gen_print(out, expr, strs),

            IR::StoreVar(_, expr) => {
                self.gen_expr(out, expr, strs);
            }

            _ => {}
        }
    }

    fn gen_expr(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        match expr {
            IRExpr::Int(n) => writeln!(out, "    mov rax, {}", n).unwrap(),

            IRExpr::Str(s) => {
                let idx = strs.iter().position(|x| x == s).unwrap();
                writeln!(out, "    lea rax, [rel str_{}]", idx).unwrap();
            }

            _ => {}
        }
    }

    // ------------------------------------------------------------
    // ★ println 구현 (Windows / Linux / macOS)
    // ------------------------------------------------------------
    fn gen_print(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        let idx = if let IRExpr::Str(s) = expr {
            strs.iter().position(|x| x == s).unwrap()
        } else {
            panic!("println only supports string literal");
        };

        #[cfg(target_os = "windows")]
        {
            writeln!(out, "    sub rsp, 32").unwrap();
            writeln!(out, "    mov rcx, -11").unwrap();
            writeln!(out, "    call GetStdHandle").unwrap();
            writeln!(out, "    mov rcx, rax").unwrap();
            writeln!(out, "    lea rdx, [rel str_{}]", idx).unwrap();
            writeln!(out, "    mov r8, str_{}_len", idx).unwrap();
            writeln!(out, "    xor r9d, r9d").unwrap();
            writeln!(out, "    call WriteConsoleA").unwrap();
            writeln!(out, "    add rsp, 32").unwrap();
        }

        #[cfg(target_os = "linux")]
        {
            writeln!(out, "    mov rax, 1").unwrap();
            writeln!(out, "    mov rdi, 1").unwrap();
            writeln!(out, "    lea rsi, [rel str_{}]", idx).unwrap();
            writeln!(out, "    mov rdx, str_{}_len", idx).unwrap();
            writeln!(out, "    syscall").unwrap();
        }

        #[cfg(target_os = "macos")]
        {
            writeln!(out, "    mov rax, 0x2000004").unwrap();
            writeln!(out, "    mov rdi, 1").unwrap();
            writeln!(out, "    lea rsi, [rel str_{}]", idx).unwrap();
            writeln!(out, "    mov rdx, str_{}_len", idx).unwrap();
            writeln!(out, "    syscall").unwrap();
        }
    }
}
