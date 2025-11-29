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
        // 데이터 영역
        // -------------------------
        writeln!(&mut out, "section .data").unwrap();

        // 문자열 출력용 format
        writeln!(&mut out, "fmt_str: db \"%s\", 0").unwrap();

        // 문자열 리터럴 수집
        let mut strs = Vec::new();
        for f in &ir.funcs {
            for stmt in &f.body {
                self.collect_str(stmt, &mut strs);
            }
        }

        for (i, s) in strs.iter().enumerate() {
            writeln!(&mut out, "str_{}: db \"{}\", 0", i, s).unwrap();
        }

        // -------------------------
        // TEXT 영역
        // -------------------------
        writeln!(&mut out, "section .text").unwrap();
        writeln!(&mut out, "global {}", ENTRY).unwrap();

        // CRT printf 사용
        writeln!(&mut out, "extern printf").unwrap();

        // every function
        for f in &ir.funcs {
            writeln!(&mut out, "global {}_func", f.name).unwrap();
            writeln!(&mut out, "global {}_func_end", f.name).unwrap();
        }

        // 함수 본문 생성
        for f in &ir.funcs {
            self.gen_function(&mut out, f, &strs);
        }

        // ENTRY (main wrapper)
        writeln!(&mut out, "{}:", ENTRY).unwrap();
        writeln!(&mut out, "    call main_func").unwrap();
        writeln!(&mut out, "    ret").unwrap();

        out
    }

    fn collect_str(&self, stmt: &IR, out: &mut Vec<String>) {
        if let IR::Println(IRExpr::Str(s)) = stmt {
            out.push(s.clone());
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
    // ★ printf 기반 println
    // ------------------------------------------------------------
    fn gen_print(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        let idx = if let IRExpr::Str(s) = expr {
            strs.iter().position(|x| x == s).unwrap()
        } else {
            panic!("println only supports string literal")
        };

        // 호출 규약: Windows/Linux/macOS 모두 printf는 C ABI 사용
        writeln!(out, "    lea rcx, [rel fmt_str]").unwrap();    // format
        writeln!(out, "    lea rdx, [rel str_{}]", idx).unwrap(); // argument
        writeln!(out, "    sub rsp, 32").unwrap();                // shadow space (Windows)
        writeln!(out, "    call printf").unwrap();
        writeln!(out, "    add rsp, 32").unwrap();
    }
}
